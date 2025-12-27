# 开发进度追踪 (Development Progress)

## 🌳 进度树 (Project Status Tree)

### ✅ Phase 1: 基础设施搭建 (Infrastructure)
> **状态**: 100% 完成
- [x] **项目初始化**
    - [x] Rust 项目结构 (`scavenger`)
    - [x] 依赖管理 (`Cargo.toml`: 解决 `tonic` 与 `console` 版本冲突)
    - [x] 配置文件系统 (`config.toml` & `src/config.rs`)
- [x] **钱包与鉴权**
    - [x] 钱包文件自动加载 (`src/main.rs`)
    - [x] 鉴权密钥对配置
    - [x] 余额自动检测 (已验证: 0.1049 SOL)
- [x] **网络连接**
    - [x] RPC 连接 (Solana Mainnet)
    - [x] Jito Block Engine 连接 (gRPC 建立成功)

### 🚧 Phase 2: 侦察系统 (Scout)
> **状态**: 80% 进行中
- [x] **数据源接入**
    - [x] gRPC 客户端骨架 (`src/scout/mod.rs`)
    - [x] WebSocket 监听器设计 (RPC Logs)
    - [x] 实现 RPC 日志订阅 (`logsSubscribe`)
- [x] **Raydium 监听器**
    - [x] 识别 `Initialize2` (CPMM) 指令
    - [x] 识别 `Initialize` (Standard AMM) 指令
    - [ ] **[TODO]** 解析代币 Mint 地址与池子信息 (需 fetch tx)
    - [ ] **[TODO]** 解析代币 Mint 地址与池子信息
- [ ] **过滤器 (Filter)**
    - [ ] **[TODO]** 基础过滤 (Mint 权限检查)
    - [ ] **[TODO]** 蜜罐检测 (模拟交易预检)

### ⏳ Phase 3: 策略执行 (Execution)
> **状态**: 0% 等待中
- [ ] **交易构建**
    - [ ] **[TODO]** 集成 Jupiter Swap SDK (或手写 Raydium Swap)
    - [ ] **[TODO]** 构建 Jito Bundle (Tx + Tip)
- [ ] **上链执行**
    - [ ] **[TODO]** 策略引擎 (买入 -> 卖出逻辑)
    - [ ] **[TODO]** 利润统计与风险控制

---

## � 迁移指南 (Migration Guide)

如果您更换开发机器，除了 `git clone` 代码仓库外，您**必须手动迁移**以下文件（它们被 `.gitignore` 忽略以保护安全）：

| 文件名 | 路径 | 用途 | 必须性 |
| :--- | :--- | :--- | :--- |
| **scavenger.json** | `Solana-MEV/scavenger.json` | **资金钱包私钥** (支付 Gas/本金) | 🚨 **必须** |
| **auth_key.json** | `Solana-MEV/auth_key.json` | **Jito 鉴权私钥** (与 Block Engine 握手) | 🚨 **必须** |
| **config.toml** | `Solana-MEV/scavenger/config.toml` | **本地配置文件** (RPC URL 等) | ⚠️ 推荐 (避免重新配置) |

> **提示**: 您的 `withdrawal_wallet.json` 如果不再使用可以不迁移，但建议备份。

## 📝 最近更新
- **2025-12-25**:
    - **核心策略模块实现**:
        - 新增 `src/strategy/orca.rs`: 实现 Orca Whirlpool 协议的 Swap 指令构建。
        - 新增 `src/strategy/arbitrage.rs`: 实现原子套利引擎，支持 `Orca -> Raydium -> Jito` 三步原子交易。
    - 更新文档库以反映最新的模块架构。
- **2025-12-24**: 
    - 细化进度树，明确 Phase 2/3 的 TODO 项。
    - 添加迁移指南，列出关键敏感文件。
    - 完成 Phase 1 所有基础设施验证。
