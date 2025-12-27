# Scavenger 系统架构 (System Architecture)

本文档详细描述了 Scavenger MEV 机器人的技术架构、核心逻辑映射及数据流向。

## 1. 架构概览 (Overview)

Scavenger 采用 **Rust 异步架构 (Tokio)**，设计目标为毫秒级响应 Solana 链上事件。
系统架构已从早期的“被动全网监听”演进为 **“基于库存的主动监听 (Inventory-Driven Monitoring)”** 模式。

系统分为三个核心层级：

1.  **数据层 (Data Layer)**: 负责构建全网代币索引 (Inventory)，识别 Raydium/Orca 共有的套利对，并生成监听白名单。
2.  **侦察层 (Scout Layer)**: 基于白名单，通过 Geyser/WebSocket 精准监听特定账户 (Account Updates) 的余额与数据变动。
3.  **策略与执行层 (Strategy & Execution Layer)**: 接收变动事件，进行双向比价 (Pricing)，构建原子交易 (Bundle)，并发送至 Jito Block Engine。

## 2. 核心逻辑映射 (Core Logic Map)

### 🗄️ 数据与索引 (Data & Inventory)

| 逻辑模块 | 关键功能 | 文件路径 | 备注 |
| :--- | :--- | :--- | :--- |
| **Inventory** | 全网代币索引 | `scavenger/src/state.rs` | 核心组件。启动时并发拉取 API，构建 `DashMap<TokenMint, ArbitragePair>`，找出 DEX 间的共有市场。 |
| **API Fetcher** | 数据预加载 | `scavenger/src/scout/api.rs` | 封装 REST API (Raydium/Orca)，用于冷启动数据获取。 |

### 🔍 侦察系统 (Scout System)

| 逻辑模块 | 关键功能 | 文件路径 | 备注 |
| :--- | :--- | :--- | :--- |
| **Monitor** | 精准监听 | `scavenger/src/scout/monitor.rs` | 使用 `SubscribeUpdateAccount` 监听 Inventory 中的 Pool Address。 |
| **Decoder** | 协议解析 | `scavenger/src/scout/{protocol}.rs` | 直接解析 Account Data (Reserves/SqrtPrice)，而非仅仅依赖 Logs。 |

### 🧠 策略引擎 (Strategy Engine)

| 逻辑模块 | 关键功能 | 文件路径 | 备注 |
| :--- | :--- | :--- | :--- |
| **Engine** | 双向比价主控 | `scavenger/src/strategy/engine.rs` | 单边变动 -> 查对手盘价格 -> 计算价差 -> 触发。 |
| **Pricing** | 本地定价 | `scavenger/src/amm/` | 实现 CPMM (Raydium) 和 CLMM (Orca) 的数学模型，不依赖 RPC 模拟。 |

### ⚙️ 执行与基础设施 (Infrastructure)

| 逻辑模块 | 关键功能 | 文件路径 | 备注 |
| :--- | :--- | :--- | :--- |
| **Swap Builder** | 指令构建 | `scavenger/src/strategy/swap.rs` | 构建 Raydium/Orca Swap IX。 |
| **Jito Client** | Bundle 发送 | `scavenger/src/strategy/jito.rs` | HTTP JSON-RPC 连接 Block Engine，支持 Bundle 模拟与发送。 |

---

## 3. 数据流向 (Data Flow)

```mermaid
graph TD
    Init[系统启动 Warm-up] --> A[API Fetcher]
    A -->|拉取 Raydium Pools| B[数据清洗 & 匹配]
    A -->|拉取 Orca Whirlpools| B
    B -->|Intersection| C[Inventory (Shared Pairs)]
    
    C -->|生成监听列表 (Watchlist)| D[Scout: Monitor]
    
    D -->|Subscribe Account Updates| E[Solana Chain]
    E -->|Pool 余额/价格变动| F[Scout: Decoder]
    
    F -->|解析最新状态| G{Strategy Engine}
    
    G -->|1. 获取变动源价格| H[Pricing Engine]
    G -->|2. 查询对手盘价格 (Cache/RPC)| H
    
    H -->|Spread > Threshold| I[Transaction Builder]
    H -->|No Spread| X[Discard]
    
    I -->|3. 构建 Atomic Bundle| J[Jito Block Engine]
    J -->|Send| K[Solana Validators]
```


## 4. 技术栈选型 (Tech Stack)

*   **Language**: Rust (性能与安全性)
*   **Async Runtime**: Tokio (高并发处理)
*   **RPC**: Solana Client (Non-blocking), Jito Geyser (Planned)
*   **Serialization**: Borsh (Solana 标准), Serde
*   **Math**: `uint` (U256 高精度计算), `rust_decimal`

## 5. 关键算法与模型

1.  **Constant Product Market Maker (CPMM)**: 用于 Raydium V4/V5。
    *   公式: $(x_{old} + x_{in}) \cdot (y_{old} - y_{out}) = k$
2.  **Concentrated Liquidity Market Maker (CLMM)**: 用于 Orca Whirlpool / Raydium CLMM。
    *   需要实时维护 Tick Bitmap 和 Tick Arrays。
    *   价格计算涉及跨 Tick 的流动性聚合。
3.  **Jito Bundle**:
    *   特性: 原子性 (All-or-Nothing)，抗 MEV (不会被三明治攻击)，无 Revert 成本 (模拟失败不扣费)。