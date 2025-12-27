# 📘 Scavenger 系统使用手册

> **版本**: 0.1.0 (Phase 2 & 3 Alpha)
> **最后更新**: 2025-12-25

## 1. 系统简介
**Scavenger** 是一个高性能的 Solana MEV 机器人框架，专为捕捉链上套利机会而设计。
当前版本已完成 **Phase 3 准备阶段**，具备以下核心能力：
*   **全网监听**: 通过 WebSocket 实时监听 Solana 主网的所有交易日志。
*   **新池发现**: 毫秒级识别 Raydium AMM (V4) 的新流动性池创建事件 (`Initialize2`)。
*   **深度解析**: 自动抓取并解析交易数据，提取 **Pool ID** (池子地址)、**Token Mint A** (代币A)、**Token Mint B** (代币B)。
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
trade_amount_sol = 0.1
static_tip_sol = 0.001
dynamic_tip_ratio = 0.5
max_tip_sol = 0.1

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
    ✅ WebSocket 连接成功，开始订阅日志...
    👀 正在监听 Raydium AMM v4 日志...
    ```

3.  **发现新池子**:
    当链上有人创建新池子时，你会立即看到：
    ```text
    ✨ 发现潜在新池子! Tx: https://solscan.io/tx/5...
    ```

4.  **解析详情 (Phase 3 核心)**:
    系统会自动去抓取该交易的详情。如果 RPC 响应及时，你会看到：
    ```text
    🎉 成功解析池子详情: Pool: 7X..., TokenA: So11..., TokenB: EPj...
    ```
    *如果没有看到这条，或者看到重试日志，通常是因为 Public RPC 尚未索引到该交易。这是正常现象，更换为付费 RPC 可解决。*

---

## 5. 系统架构说明

### 核心模块
*   **Scout (`src/scout`)**: 系统的“眼睛”。
    *   `monitor.rs`: 维护 WebSocket 长连接，过滤 `Initialize2` 日志。
    *   `raydium.rs`: 交易解析器。它包含智能重试逻辑，能够处理 RPC 的 `Transaction not found` 错误，从原始二进制数据中提取账户信息。

### 为什么需要异步抓取？
Solana 的 `logsSubscribe` 推送非常快（毫秒级），但它只包含“发生了什么”，不包含“具体参数”（如 Token 地址）。
因此，我们采取 **“推拉结合”** 的策略：
1.  **推 (Push)**: WebSocket 收到信号 -> 立即锁定目标 Tx。
2.  **拉 (Pull)**: 异步任务立即向 RPC 请求 `getTransaction`。

---

## 6. 常见问题 (FAQ)

**Q: 为什么我只看到 "发现潜在新池子"，却看不到 "成功解析池子详情"？**
A: 这通常是因为您使用的是公共 RPC节点。公共节点在交易上链后，可能有 10-30 秒的索引延迟，甚至找不到交易。我们的系统会重试 5 次（约 2.5 秒），如果 RPC 依然返回空，就会放弃。**解决方案**: 使用 Helius 或 QuickNode 的付费服务。

**Q: Jito 连接是必须的吗？**
A: 在目前的侦察阶段，Jito 连接用于建立通道，不是必须的（您可以忽略相关日志）。但在未来的 **Phase 4 (交易执行)** 阶段，必须通过 Jito 发送 Bundle 才能实现防夹子 (Sandwich Protection) 和 失败不付费 (Revert Protection)。

**Q: 如何停止程序？**
A: 按 `Ctrl + C` 即可安全退出。
