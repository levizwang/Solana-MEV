# PRD: Scavenger 策略转型 - 从“新池狙击”转向“波动套利”

## 1. 背景与目标 (Context & Objective)

*   **现状**：当前系统监听 Raydium `Initialize` 事件并查询 Orca。由于 Raydium 新池绝大多数为全网首发，Orca 上不存在对应池子，导致套利机会为零。
*   **目标**：将核心策略从 **“新池监听 (New Pool Sniping)”** 转型为 **“重叠资产波动套利 (Overlap Volatility Arbitrage)”**。
*   **核心逻辑**：
    1.  在系统启动时，识别 Raydium 和 Orca (未来包含 Meteora) 上**共有**的代币对（建立白名单）。
    2.  利用 Geyser gRPC 精准监听这些白名单池子的**储备量/价格变动**。
    3.  一旦发现单边价格波动产生价差，立即触发 Jito 原子套利。

---

## 2. 核心模块变更 (Module Requirements)

### 2.1 状态管理模块 (`src/state.rs`)

**变更需求**：
不再被动等待新池加入，而是主动预加载“共有资产”。

*   **新增结构体 `ArbitragePair`**：
    *   存储同一 Token Mint 在不同 DEX 的池子地址。
    *   `token_mint`: Pubkey
    *   `raydium_pool`: Pubkey
    *   `orca_pool`: Pubkey (Optional)
    *   `meteora_pool`: Pubkey (Optional)
*   **新增 `Inventory` 逻辑**：
    *   启动时 (Warm-up)：并发请求 Raydium API 和 Orca API (REST)。
    *   数据清洗：找出两个列表的交集（Intersection on Token Mint）。
    *   存储：将几百个共有对存入 `DashMap<Pubkey, ArbitragePair>`。
    *   **输出**：生成一个包含所有相关 Pool Address 的 `Vec<Pubkey>`，用于 Geyser 订阅。

### 2.2 侦察模块 (`src/scout/`)

**变更需求**：
从“广撒网监听 Logs”改为“精准监听 Account Data”。

*   **`monitor.rs` 改造**：
    *   **移除**：对 Raydium `Initialize2` 和 Orca `InitializePool` 的全网日志监听（或者降低优先级）。
    *   **新增**：使用 Geyser 的 `SubscribeUpdateAccountFilter`。
    *   **订阅列表**：传入 Inventory 生成的 `Vec<Pubkey>` (即那些重叠的池子地址)。
    *   **触发条件**：当订阅的 Account Data 发生变化（Lamports 或 Data 改变，意味着有人交易了），推送事件到 Engine。

*   **`orca.rs` / `raydium.rs` 解析器增强**：
    *   需要实现从 Account Data (`Vec<u8>`) 中直接解析出储备量 (Reserves) 或 `sqrt_price` 的能力，而不仅仅是解析 Log。

### 2.3 策略引擎 (`src/strategy/engine.rs`)

**变更需求**：
实现“单边驱动，双边比价”逻辑。

*   **事件处理循环**：
    1.  接收 `PoolUpdateEvent` (来源: Scout)。
    2.  识别更新的是哪个 DEX 的哪个池子 (e.g., Raydium SOL/USDC)。
    3.  从 `Inventory` 查找该 Token 在对手盘 DEX (Orca) 的池子地址。
    4.  **读取价格**：
        *   *Raydium*: 直接从 Update 数据中计算新价格。
        *   *Orca*: 尝试读取缓存价格，或者发起一个极速 RPC 查询（因为 Orca 变动频率低，可容忍少量按需查询）。
    5.  **价差计算**：`Spread = |Price_A - Price_B| / Price_A`。
    6.  **触发**：若 `Spread > Threshold (e.g. 1%)`，构建 Jito Bundle。

---

## 3. 数据流架构图 (Data Flow)

```mermaid
graph TD
    Start[系统启动] --> A[API Fetcher]
    A -->|拉取 Raydium List| B[数据清洗]
    A -->|拉取 Orca List| B
    B -->|取交集| C[Inventory (WhiteList)]
    
    C -->|生成池子地址列表| D[Geyser Monitor]
    
    D -->|订阅 Account Update| E[Solana Chain]
    E -->|池子余额变动| F[Scout Layer]
    
    F -->|解析新储备量| G[Strategy Engine]
    
    G -->|查询对手盘缓存/RPC| H[Pricer]
    H -->|计算价差| I{Spread > 1%?}
    
    I -->|Yes| J[构建 Swap 指令]
    J -->|打包| K[Jito Client]
    I -->|No| L[等待下一次变动]
```

---

## 4. Cursor 迭代任务清单 (Prompt Guide)

请按顺序执行以下 Prompt，指挥 AI 修改代码。

### Task 1: 实现 API 预加载与白名单构建
**目标文件**: `src/state.rs`, `src/scout/mod.rs`

> **Prompt**:
> 我需要重构 `Inventory`。请引入 `reqwest` 库。
> 1. 在 `src/scout` 下实现两个辅助函数：`fetch_raydium_pools` 和 `fetch_orca_pools`，通过官方 REST API 获取所有池子列表。
> 2. 在 `Inventory` 结构体中，新增一个 `common_pairs` 字段，类型为 `DashMap<Pubkey, ArbitragePair>`。
> 3. 实现 `Inventory::load_from_api` 方法：在启动时调用 API，找出 Raydium 和 Orca 共有的 Token Mint，并将对应的两个 Pool Address 存入 `common_pairs`。
> 4. 提供一个方法 `get_watch_list() -> Vec<Pubkey>`，返回所有需要监听的 Pool Address 列表。

### Task 2: 改造 Monitor 监听逻辑
**目标文件**: `src/scout/monitor.rs`

> **Prompt**:
> 修改 `monitor.rs` 的监听逻辑。
> 1. 在 `start_monitoring` 启动前，先调用 `inventory.load_from_api()` 初始化白名单。
> 2. 修改 Geyser 的订阅配置。不再订阅全网的 Program Logs。
> 3. 使用 `SubscribeUpdateAccountFilter`，将 `inventory.get_watch_list()` 返回的地址列表作为 `accounts` 参数传入。
> 4. 当收到 Account Update 时，根据 Owner 是 Raydium 还是 Orca，调用不同的解析器更新价格。

### Task 3: 实现 Orca Whirlpool 价格解析
**目标文件**: `src/amm/orca_whirlpool.rs`

> **Prompt**:
> 我需要解析 Orca Whirlpool 的 Account Data。
> 1. 定义 Whirlpool 的 Account Layout 结构体 (参考 Orca SDK 的 Rust 定义，使用 BorshDeserialize)。重点包含 `sqrt_price`, `tick_current_index`, `liquidity`。
> 2. 实现一个方法 `decode_price_from_data(data: &[u8]) -> f64`。
> 3. 利用 `sqrt_price` 计算出当前 Token A 对 Token B 的价格。暂时忽略跨 Tick 的复杂计算，只计算当前 Tick 的价格。

### Task 4: 升级 Engine 比价逻辑
**目标文件**: `src/strategy/engine.rs`

> **Prompt**:
> 重构 `process_event` 逻辑。
> 1. 当收到一个池子的 Update 事件时，从 `Inventory` 中找到它在另一个 DEX 的配对池子。
> 2. 如果是 Raydium 变动，去获取 Orca 的最新价格（可以先尝试从缓存读，读不到则打印日志跳过，后续再加 RPC 查）。
> 3. 计算价差。如果价差超过配置的阈值（如 0.5%），打印 "Arbitrage Opportunity Found!" 日志，并输出买卖方向。

---

## 5. 验收标准 (Acceptance Criteria)

1.  **启动日志**：系统启动时，应打印 "Loaded X common pairs from Raydium/Orca" (X 应该在 100-1000 之间)。
2.  **监听日志**：不再疯狂刷屏 Raydium Initialize，而是打印 "Price Update: [Token] Raydium: $100, Orca: $101"。
3.  **错误处理**：Orca API 请求失败时不应导致程序崩溃（重试机制）。
4.  **性能**：从收到 Geyser 推送到触发 Engine 逻辑，延迟应在毫秒级。