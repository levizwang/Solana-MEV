# Scavenger (Solana MEV Bot)

![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)
![Python](https://img.shields.io/badge/Python-3.8+-blue.svg)
![Solana](https://img.shields.io/badge/Solana-Mainnet-green.svg)
![Status](https://img.shields.io/badge/Status-Development-blue.svg)

Scavenger 是一个高性能的 Solana 链上套利与新池狙击机器人 (MEV Bot)，采用 **Python (Control Plane)** + **Rust (Data Plane)** 的混合架构。

## 📚 文档

- **[📖 系统使用手册 (User Manual)](doc/USER_MANUAL.md)**: 包含详细的安装、配置和运行指南。
- **[🏗 系统架构 (Architecture)](doc/ARCHITECTURE.md)**: 详细的系统分层架构、数据流向图与核心逻辑映射。
- **[🛣 开发路线图 (Roadmap)](doc/ROADMAP.md)**: 当前开发状态、战略转型规划与详细任务清单。

## 📂 项目结构

```text
/Users/yqg/Documents/Solana-MEV/
 ├── commander/          <-- Python 控制平面 (Control Plane)
 │   ├── main.py         <-- 统一启动脚本
 │   └── configs/        <-- 策略配置文件 (YAML)
 │       ├── arb.yaml    <-- 套利策略配置
 │       └── sniper.yaml <-- 狙击策略配置
 ├── scavenger/          <-- Rust 数据平面 (Data Plane)
 │   ├── Cargo.toml
 │   ├── auth_key.json   <-- Jito 私钥 (挂载/本地)
 │   ├── scavenger.json  <-- 交易私钥 (挂载/本地)
 │   └── src/
 │       ├── main.rs     <-- 命令行入口
 │       ├── strategies/ <-- 策略实现模块
 │       │   ├── arb.rs  <-- 波动套利策略
 │       │   └── ...
 │       ├── core/       <-- 核心组件 (Pricing, Swap, Risk)
 │       └── scout/      <-- 链上侦察兵
 └── doc/                <-- 文档库
```

## 🚀 快速开始

### 方式一：使用 Python Commander (推荐)

通过 Python 脚本灵活调度不同的策略：

```bash
# 运行套利策略 (默认)
python3 commander/main.py --strategy arb

# 运行狙击策略 (需配置 sniper.yaml)
python3 commander/main.py --strategy sniper
```

### 方式二：使用 Docker

```bash
docker-compose up -d --build
docker-compose logs -f
```

### 方式三：Rust 原生运行

```bash
cd scavenger
cargo run --release --bin scavenger -- --strategy arb --config ../commander/configs/arb.yaml
```

## ✨ 核心特性

- **混合架构**: Python 负责配置与调度，Rust 负责高性能计算与链上交互。
- **策略解耦**: 支持多种策略 (Arbitrage, Sniper) 独立运行，互不干扰。
- **极速侦察**: 基于 WebSocket 的毫秒级新池监听 (Raydium V4 & Orca Whirlpool)。
- **智能索引**: 全网代币与流动性池内存索引 (Inventory)。
- **Jito 集成**: 内置 Jito Block Engine 客户端架构。

## ⚠️ 免责声明

本项目仅供教育和研究使用。在主网使用可能涉及资金风险，请务必在充分理解代码的前提下操作。
