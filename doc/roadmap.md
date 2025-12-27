# Scavenger 开发路线图 (Project Roadmap)

本文档记录了 Scavenger 项目的当前状态、历史进度以及基于最新市场认知的战略转型规划。

## 📅 当前状态 (Status)

*   **当前阶段**: Phase 2.5 (战略转型与数据层构建)
*   **核心能力**: 
    *   ✅ 多路监听 (Raydium + Orca)
    *   ✅ 基础交易解析
    *   ✅ 基础设施连接 (RPC/Jito)
    *   ⚠️ 缺失核心套利数据链 (Inventory & Pricing)

---

## 🗺️ 战略转型规划 (Strategic Pivot)

基于实战分析，原定的“监听 Raydium 新池 -> 去 Orca 套利”策略成功率极低（因为新币在 Orca 通常无流动性）。我们调整为**“存量套利 + 新池狙击”**双轨策略。

### Phase 2.5: 数据孤岛打通 (Data Connectivity)
> **目标**: 建立全网代币索引，解决“听到消息但不知道去哪套利”的问题。

- [x] **构建全网代币索引 (In-Memory Inventory)**
    - [x] 冷启动：拉取 Orca 所有 Whirlpool，构建 `HashMap<TokenMint, Vec<PoolAddress>>`。(已实现架构，受限于公共RPC无法全量拉取，但逻辑已就绪)
    - [x] 增量更新：监听 `InitializePool` 实时更新内存表。
    - [x] 纳秒级查询：实现 `has_liquidity(token_mint)` 快速判断。
- [ ] **实现 Orca 本地定价 (Pricing)**
    - [x] **数据结构定义**: 完整定义 `Whirlpool` 账户的 Borsh 布局 (Liquidity, SqrtPrice, TickCurrentIndex)。
    - [x] **数学库实现**: 实现 CLMM 价格计算 (SqrtPrice -> Price) 和 Tick Math。
    - [x] **价格预言**: 在侦察到事件时，实时输出 Token 在 Orca 的当前报价 (Quote)。
- [ ] **升级策略引擎**
    - [ ] 从单边触发改为双边联动 (Event -> Cache Lookup -> Spread Calc)。

### Phase 3: 交易执行闭环 (Execution Loop)
> **目标**: 发送第一笔盈利的原子交易。

- [ ] **交易构建器 (Transaction Builder)**
    - [ ] 实现 Raydium `swap_v2` 指令构建。
    - [ ] 实现 Orca `swap` 指令构建。
    - [ ] 账户关联：自动查找 `open_orders`, `tick_arrays` 等辅助账户。
- [ ] **Jito 集成修复**
    - [ ] 解决 SDK 版本冲突。
    - [ ] 实现 Bundle 模拟与发送。
- [ ] **实盘测试 (First Blood)**
    - [ ] 选取特定“僵尸币”进行小额（0.1 SOL）套利测试。

### Phase 4: 性能与扩展 (Performance & Expansion)
> **目标**: 工业级稳定运行。

- [ ] **Geyser gRPC 升级**: 替换 WebSocket，降低 200ms+ 延迟。
- [ ] **Jito Backrun**: 监听 Mempool 实现无风险尾随套利。
- [ ] **多策略并行**: 同时运行“搬砖”和“狙击”策略。

---

## ✅ 历史进度追踪 (History)

### Phase 2: 侦察系统 (Scout) - *Completed*
- [x] **多路监听器**: 同时监听 Raydium AMM V4 和 Orca Whirlpool 日志。
- [x] **深度解析器**: 提取交易中的 Token Mint 和 Pool ID。
- [x] **基础设施**: 完成 RPC 连接、钱包管理和配置系统。

### Phase 1: 基础设施 (Infra) - *Completed*
- [x] 项目初始化 (Rust + Tokio)。
- [x] 钱包生成与鉴权系统。
- [x] Jito 客户端初步连接 (gRPC)。

---

## 📝 版本日志
*   **2025-12-25**: 确立战略转型，新增 Phase 2.5，聚焦内存索引与 CLMM 定价。
*   **2025-12-24**: 完成双路监听系统，实现 Scout 模块。
