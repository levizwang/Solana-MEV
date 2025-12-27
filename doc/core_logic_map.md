# 核心逻辑映射表 (Core Logic Map)

> 本文档用于记录系统核心逻辑的代码位置，便于后续维护与性能优化。

## 🔍 侦察系统 (Scout System)

| 逻辑模块 | 关键功能 | 文件路径 | 代码位置/结构 | 备注 |
| :--- | :--- | :--- | :--- | :--- |
| **Data Source** | Jito gRPC 连接 | `scavenger/src/scout/mod.rs` | `Scout::new` | 目前处于无鉴权模式，用于发送 Bundle 或尝试订阅 |
| **Monitor** | RPC WebSocket 监听 | `scavenger/src/scout/monitor.rs` | `start_monitoring` | 使用 `PubsubClient` 监听 `RAYDIUM_AMM_V4` 日志 |
| **Decoder** | Raydium 指令解析 | `scavenger/src/scout/raydium.rs` | `parse_log_for_new_pool` | 初步筛选 `Initialize2` 交易签名 |

## 🧠 策略引擎 (Strategy Engine)

| 逻辑模块 | 关键功能 | 文件路径 | 代码位置/结构 | 备注 |
| :--- | :--- | :--- | :--- | :--- |
| **Risk Filter** | 蜜罐/权限检测 | `scavenger/src/strategy/risk.rs` | `check_token_risk` | 检查 Mint/Freeze Authority |
| **Swap (Raydium)** | Raydium Swap 指令构建 | `scavenger/src/strategy/swap.rs` | `swap` | 构建 V4 Swap Instruction |
| **Swap (Orca)** | Orca Swap 指令构建 | `scavenger/src/strategy/orca.rs` | `swap` | 模拟 Whirlpool Swap Instruction |
| **Arbitrage** | 原子套利组装 | `scavenger/src/strategy/arbitrage.rs` | `execute_arbitrage` | 组装 `[Orca, Raydium, Jito]` 原子交易 |
| **Engine** | 策略主控流程 | `scavenger/src/strategy/engine.rs` | `process_new_pool` | 串联 Scout -> Risk -> Execution |

## ⚙️ 基础设施 (Infrastructure)

| 逻辑模块 | 关键功能 | 文件路径 | 代码位置/结构 | 备注 |
| :--- | :--- | :--- | :--- | :--- |
| **Config** | 配置加载 | `scavenger/src/config.rs` | `AppConfig::load` | 支持 TOML |
| **Wallet** | 密钥管理 | `scavenger/src/main.rs` | `read_keypair_from_file` | |

---

## 📝 优化记录 (Optimization Log)

*   **2025-12-24**: 
    *   解决了 `tonic` 依赖冲突 (0.9 -> 0.8) 以适配 Jito SDK。
    *   在 `Scout::new` 中实现了手动 gRPC Endpoint 连接逻辑，绕过了 SDK helper 的鉴权限制。
