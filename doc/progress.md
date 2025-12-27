# 开发进度追踪 (Development Progress)

## 🌳 进度树

- [x] **Phase 1: 基础设施搭建 (Infrastructure)**
    - [x] 初始化 Rust 项目结构 (`scavenger`)
    - [x] 配置依赖 (`Cargo.toml`: solana-sdk, jito-searcher-client, tokio)
    - [x] 配置文件系统 (`config.toml` & `src/config.rs`)
    - [x] 钱包与鉴权模块脚手架 (`src/main.rs`)
    - [ ] Jito 客户端实际连接测试 (等待用户填入 Keypair)

- [ ] **Phase 2: 侦察系统 (Scout)**
    - [ ] Geyser gRPC 连接配置
    - [ ] Raydium 新池监听逻辑
    - [ ] 日志解析与代币过滤

- [ ] **Phase 3: 策略执行 (Execution)**
    - [ ] Swap 指令构建 (Orca/Raydium)
    - [ ] Jito Bundle 构建与 Tip 策略
    - [ ] 模拟交易与上链

## 📝 最近更新
- **2025-12-24**: 完成项目初始化，创建可配置的 Rust 架构。引入 `config.toml` 实现配置解耦。
