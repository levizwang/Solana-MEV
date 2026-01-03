# Scavenger (Solana MEV Bot)

![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)
![Python](https://img.shields.io/badge/Python-3.8+-blue.svg)
![Solana](https://img.shields.io/badge/Solana-Mainnet-green.svg)
![License](https://img.shields.io/badge/License-MIT-purple.svg)

**Scavenger** 是一个高性能的 Solana 链上套利与新池狙击机器人 (MEV Bot)，采用 **Python (Control Plane)** + **Rust (Data Plane)** 的混合架构。

本项目旨在提供一个生产级的 MEV 框架，展示如何利用 Rust 的高性能进行链上数据监听与解析，同时利用 Python 的灵活性进行策略调度与配置管理。

> ⚠️ **风险提示**：本项目仅供技术研究与教育目的使用。MEV 竞争极其激烈且涉及资金风险，请勿在未充分测试的情况下在主网投入大量资金。

## ✨ 核心特性

- **混合架构**: Python 负责配置与调度，Rust 负责高性能计算与链上交互。
- **策略解耦**: 支持多种策略 (Arbitrage, Sniper) 独立运行，互不干扰。
- **极速侦察**: 基于 WebSocket 的毫秒级新池监听 (Raydium V4 & Orca Whirlpool)。
- **智能索引**: 全网代币与流动性池内存索引 (Inventory)，实现“基于库存的主动监听”。
- **本地定价**: 内置 CPMM (Raydium) 和 CLMM (Orca) 数学模型，减少 RPC 模拟依赖。
- **Jito 集成**: 内置 Jito Block Engine 客户端架构，支持原子 Bundle 发送。

## 📂 项目结构

```text
Solana-MEV/
 ├── commander/          # Python 控制平面 (Control Plane)
 │   ├── main.py         # 统一启动入口
 │   └── configs/        # 策略配置文件 (YAML)
 ├── scavenger/          # Rust 数据平面 (Data Plane)
 │   ├── src/            # 源代码
 │   │   ├── amm/        # 定价模型 (CPMM/CLMM)
 │   │   ├── core/       # 核心组件 (Swap, Jito, Risk)
 │   │   ├── scout/      # 链上侦察兵 (Monitor, Parser)
 │   │   └── strategies/ # 策略实现 (Arb, Sniper)
 │   ├── config.toml     # Rust 基础配置
 │   └── Cargo.toml      # 依赖管理
 ├── doc/                # 项目文档
 └── requirements.txt    # Python 依赖说明
```

## 🛠 环境准备

1.  **Rust 工具链**:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
2.  **Python 3.8+**: 确保已安装 Python 环境。
3.  **Solana CLI** (可选): 用于生成钱包。
4.  **RPC 节点**: 需要一个支持 WebSocket 的 Solana RPC 节点 (推荐 Helius, QuickNode 等)。

## 🚀 快速开始

### 1. 克隆项目

```bash
git clone https://github.com/levizwang/Solana-MEV.git
cd Solana-MEV
```

### 2. 配置钱包与密钥

在 `scavenger/` 目录下创建钱包文件 (或修改 `scavenger/config.toml` 指向你的钱包路径)。

```bash
cd scavenger
# 生成交易钱包 (仅用于测试，请勿存放大量资金)
solana-keygen new -o scavenger.json --no-bip39-passphrase
# 生成 Jito 鉴权钱包
cp scavenger.json auth_key.json
```

### 3. 修改配置文件

编辑 `scavenger/config.toml` 和 `commander/configs/*.yaml`，填入你的 RPC URL。

```toml
[network]
rpc_url = "https://your-rpc-url.com"
ws_url = "wss://your-ws-url.com"
```

### 4. 运行策略

回到项目根目录，使用 Commander 启动：

```bash
# 方式一：运行套利策略 (Arbitrage)
python3 commander/main.py --strategy arb

# 方式二：运行狙击策略 (Sniper)
python3 commander/main.py --strategy sniper
```

程序会自动编译 Rust 二进制文件并启动。

## 📚 文档资源

- **[📖 系统使用手册](doc/USER_MANUAL.md)**: 详细配置与运行指南。
- **[🏗 系统架构](doc/ARCHITECTURE.md)**: 深度解析系统设计。
- **[🛣 开发路线图](doc/roadmap.md)**: 未来规划。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License
