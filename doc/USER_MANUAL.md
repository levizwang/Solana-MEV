# 📘 Scavenger 系统使用手册

> **版本**: 0.1.0 (Phase 2 & 3 Alpha)
> **最后更新**: 2025-12-25

## 1. 系统简介
**Scavenger** 是一个高性能的 Solana MEV 机器人框架，专为捕捉链上套利机会而设计。
当前版本已完成 **Phase 3 准备阶段**，具备以下核心能力：
*   **全网监听**: 通过 WebSocket 实时监听 Solana 主网的所有交易日志。
*   **多路侦察**: 同时监控 **Raydium AMM V4** (新池) 和 **Orca Whirlpool** (价格变动)。
*   **深度解析**: 自动抓取并解析交易数据，提取 **Pool ID** (池子地址)、**Token Mint A** (代币A)、**Token Mint B** (代币B)。
*   **轻量级 AMM**: 内置 Rust 原生实现的 Raydium AMM 状态解析与 Swap 算法 (Constant Product)，零重型 SDK 依赖。
*   **策略引擎 (Alpha)**: 
    *   内置 **Raydium** 和 **Orca** (Whirlpool) 的 Swap 指令构建器。
    *   支持 **原子套利 (Atomic Arbitrage)**: 将 "买入 -> 卖出 -> Jito贿赂" 打包为单笔交易，实现无风险套利（模拟失败即不上链）。
*   **Jito 集成**: 已集成 Jito Block Engine 连接，为未来的 MEV 交易打包做好了准备。

---

## 2. 环境准备

在使用本系统前，请确保您的环境满足以下要求：

### 2.1 基础软件
*   **操作系统**: macOS (推荐), Linux (Ubuntu 20.04+), 或 Windows (需 WSL2)。
*   **Rust 工具链**:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
*   **Solana CLI** (可选，用于生成钱包):
    ```bash
    sh -c "$(curl -sSfL https://release.solana.com/v1.14.17/install)"
    ```

### 2.2 节点服务 (RPC)
本系统严重依赖 RPC 节点的性能。
*   **Public RPC (免费)**: `https://api.mainnet-beta.solana.com`。**仅限测试**，存在严重的速率限制 (Rate Limit) 和延迟，会导致交易抓取失败 (`Transaction not found`)。
*   **Private RPC (付费)**: 推荐使用 [Helius](https://helius.xyz/), [QuickNode](https://www.quicknode.com/), 或 [Triton](https://triton.one/)。你需要一个 **HTTPS** 地址和一个 **WSS** 地址。

---

## 3. 快速开始 (Quick Start)

### 步骤 1: 准备钱包
系统需要两个钱包文件（JSON 格式 Keypair）：
1.  **Auth Wallet (`auth_key.json`)**: 用于 Jito Block Engine 身份验证（目前系统支持无鉴权模式，但建议准备）。
2.  **Trade Wallet (`scavenger.json`)**: 用于发送交易和支付 Gas。

如果你没有钱包，可以使用 Solana CLI 生成：
```bash
# 生成交易钱包
solana-keygen new -o scavenger.json --no-bip39-passphrase

# 生成 Jito 鉴权钱包 (可以是同一个)
cp scavenger.json auth_key.json
```
> ⚠️ **注意**: 请确保 `scavenger.json` 中有少量的 SOL (例如 0.01 SOL) 用于测试连接，尽管目前阶段主要进行只读操作。

### 步骤 2: 配置文件
在 `scavenger/` 目录下找到 `config.toml` 文件。如果不存在，请新建一个。

**配置示例 (`scavenger/config.toml`):**
```toml
[network]
# 替换为你自己的 RPC 地址
rpc_url = "https://api.mainnet-beta.solana.com"
ws_url = "wss://api.mainnet-beta.solana.com"
# Jito Block Engine (通常无需更改)
grpc_url = "https://amsterdam.mainnet.block-engine.jito.wtf"

[jito]
block_engine_url = "https://amsterdam.mainnet.block-engine.jito.wtf"
auth_keypair_path = "./auth_key.json"

[strategy]
wallet_path = "./scavenger.json"
trade_amount_sol = 0.001
static_tip_sol = 0.001
dynamic_tip_ratio = 0.5
max_tip_sol = 0.002  # 最大允许小费，超过此值会熔断

[log]
level = "info"
```

### 步骤 3: 运行系统
进入项目目录并运行：
```bash
cd scavenger
cargo run --bin scavenger
```

---

## 4. 运行状态解读

当您看到以下日志时，说明系统运行正常：

1.  **启动成功**:
    ```text
    🚀 Scavenger (拾荒者) MEV Bot 正在启动...
    ✅ RPC 连接成功...
    ✅ Jito Searcher Client 连接成功...
    ```

2.  **开始监听**:
    ```text
    🔌 连接 WebSocket: wss://...
    ✅ WebSocket 连接成功，开始多路订阅 (Raydium & Orca)...
    👀 正在监听 Raydium V4 和 Orca Whirlpool 日志...
    ```

3.  **发现新池子**:
    当链上有人创建新池子时，你会立即看到：
    ```text
    ✨ [Raydium] 发现潜在活动! Tx: https://solscan.io/tx/5...
    ```

4.  **跨市场套利触发**:
    当检测到 Orca 上的大额变动时：
    ```text
    🌊 [Orca] 成功解析池子详情...
    ⚙️ 策略引擎 (Orca): 检测到活动 Pool ...
    🧮 链上计算: Pool=..., In=1000000000, Out=...
    ```

---

## 5. 系统架构说明

### 核心模块
*   **Scout (`src/scout`)**: 系统的“眼睛”。
    *   `monitor.rs`: 维护 WebSocket 长连接，分流 Raydium/Orca 日志。
    *   `raydium.rs` / `orca.rs`: 专用的协议解析器。
*   **AMM Engine (`src/amm`)**: 
    *   `raydium_v4.rs`: 手写实现的 Raydium V4 状态反序列化 (Borsh)。
    *   `math.rs`: 基于 `U256` 的 Constant Product Swap 算法，确保计算精度与链上一致。
*   **Strategy (`src/strategy`)**:
    *   `quote.rs`: 集成 RPC 数据与本地 AMM 算法，实现真实的链上询价。
    *   `arbitrage.rs`: 原子交易组装器，负责打包 Tx + Jito Tip。

### 为什么需要异步抓取？
Solana 的 `logsSubscribe` 推送非常快（毫秒级），但它只包含“发生了什么”，不包含“具体参数”（如 Token 地址）。
因此，我们采取 **“推拉结合”** 的策略：
1.  **推 (Push)**: WebSocket 收到信号 -> 立即锁定目标 Tx。
2.  **拉 (Pull)**: 异步任务立即向 RPC 请求 `getTransaction`。

---

## 6. 常见问题 (FAQ)

**Q: 为什么日志中显示 "Jito Client 暂时禁用"？**
A: 目前我们专注于通过 RPC 进行数据抓取和模拟计算。Jito SDK 的 gRPC 接口在某些版本上存在兼容性问题，因此暂时使用 HTTP RPC 替代。这不影响侦察和策略逻辑的验证。

**Q: 如何停止程序？**
A: 按 `Ctrl + C` 即可安全退出。
