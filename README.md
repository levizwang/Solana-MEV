# Scavenger (Solana MEV Bot)

![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)
![Solana](https://img.shields.io/badge/Solana-Mainnet-green.svg)
![Status](https://img.shields.io/badge/Status-Development-blue.svg)

Scavenger 是一个高性能的 Solana 链上套利与新池狙击机器人 (MEV Bot)。

## 📚 文档

- **[📖 系统使用手册 (User Manual)](doc/USER_MANUAL.md)**: 包含详细的安装、配置和运行指南。
- **[🗺 核心逻辑映射 (Core Logic)](doc/core_logic_map.md)**: 代码结构与业务逻辑的映射表。
- **[🚧 开发进度 (Progress)](doc/progress.md)**: 当前开发状态与路线图。

## 🚀 快速开始

1. **配置**: 编辑 `scavenger/config.toml`。
2. **运行**:
   ```bash
   cd scavenger
   cargo run --bin scavenger
   ```

## ✨ 核心特性

- **极速侦察**: 基于 WebSocket 的毫秒级新池监听 (Raydium V4)。
- **智能解析**: 自动抓取并解析交易数据，提取 Token Mint 和 Pool ID。
- **Jito 集成**: 内置 Jito Block Engine 客户端，支持 Bundle 发送 (开发中)。
- **Rust 原生**: 内存安全，低延迟。

## ⚠️ 免责声明

本项目仅供教育和研究使用。在主网使用可能涉及资金风险，请务必在充分理解代码的前提下操作。
