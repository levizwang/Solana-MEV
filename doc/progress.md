# 开发进度追踪 (Development Progress)

## 🌳 进度树

- [x] **Phase 1: 基础设施搭建 (Infrastructure)**
    - [x] 初始化 Rust 项目结构 (`scavenger`)
    - [x] 配置依赖 (`Cargo.toml`: solana-sdk, jito-searcher-client, tokio)
    - [x] 配置文件系统 (`config.toml` & `src/config.rs`)
    - [x] 钱包与鉴权模块脚手架 (`src/main.rs`)
    - [x] RPC 连接与钱包余额自动检测 (已验证: 0.1049 SOL)

- [ ] **Phase 2: 侦察系统 (Scout)**
    - [x] Jito Block Engine gRPC 连接 (解决 Tonic 版本冲突)
    - [ ] Raydium 新池监听逻辑 (Pending)
    - [ ] 日志解析与代币过滤 (Pending)

- [ ] **Phase 3: 策略执行 (Execution)**
    - [ ] Swap 指令构建 (Orca/Raydium)
    - [ ] Jito Bundle 构建与 Tip 策略
    - [ ] 模拟交易与上链

## 📝 最近更新
- **2025-12-24**: 
    - 成功连接 Jito Block Engine gRPC 接口。
    - 解决了 `tonic` (0.8 vs 0.9) 和 `indicatif` (console 依赖) 的多个版本冲突问题。
    - 验证了钱包资金状态，侦察系统骨架已启动。
- **2025-12-24**: 完成项目初始化，创建可配置的 Rust 架构。引入 `config.toml` 实现配置解耦。
