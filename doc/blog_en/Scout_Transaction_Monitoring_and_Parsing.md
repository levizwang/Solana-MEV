# Sub-millisecond Vision: Scout Monitoring and Extreme Parsing in Solana MEV

If the Inventory module is the bot's "memory", then the Scout module is its "eyes". In the turbulence of Solana generating tens of thousands of state changes per second, Scout's task is to filter, filter again, and decode signals that are truly meaningful for arbitrage strategies at extreme speeds.

In the world of MEV, **speed isn't everything, but without speed, there is nothing**. This article will delve into how to build a low-latency, high-concurrency transaction monitoring and parsing system.

---

## 1. Monitoring Philosophy: Scalpel vs. Dragnet

On Solana, we typically face two distinct monitoring needs, corresponding to different technical paths:

### 1.1 `accountSubscribe`: The Precise Scalpel (Arb Mode)
For cross-protocol arbitrage (Arbitrage), we have already locked onto specific pools through Inventory. At this point, we don't need to observe the entire network, we just need to stare at the changes in the **Data field** of these pool accounts.
*   **Mechanism:** Once the token balance or price in the pool changes, the RPC node will immediately push the latest account data.
*   **Advantage:** The signal is extremely direct, skipping tedious transaction parsing, making it the fastest path for high-frequency arbitrage.

### 1.2 `logsSubscribe`: The Network-Wide Dragnet (Sniper Mode)
For sniping new pools (Sniping), we cannot foresee the pool address, so we can only capture signals of "new pool creation" or "initial liquidity injection" instructions by listening to **Program Logs** of specific protocols (such as Raydium or Orca).
*   **Mechanism:** Scan for specific keywords (like `initialize2`) in logs.
*   **Challenge:** The noise is huge, and after a hit, a "slow path" process (such as requesting `getTransaction`) is usually needed to supplement the parsing of pool token information.

---

## 2. Core Architecture: Stream Multiplexing

In a mature system, you may need to subscribe to updates for hundreds of pools simultaneously. If you open a thread for each subscription, system overhead will explode instantly.

### 2.1 Asynchronous Stream Merging (Select All)
We use Rust's asynchronous ecosystem (Tokio + Futures), utilizing `select_all` to merge hundreds or thousands of WebSocket subscription streams into a single event stream. This is like aggregating feeds from hundreds of surveillance cameras onto a single video wall, uniformly distributed and processed by a core loop (Event Loop).

### 2.2 Thread Model and "Slow Path" Detachment
The response speed of the monitoring main loop determines the system's latency upper limit.
*   **Fast Path (Hot Path):** Receive data -> Memory decode -> Trigger calculation.
*   **Slow Path (Long Path):** If extra RPC requests are needed to complete information (like in Sniper mode), use `tokio::spawn` to immediately detach it to a background task execution, strictly prohibiting blocking the monitoring main loop.

---

## 3. Extreme Parsing: Skipping Useless Information

Solana's Account Data is usually a binary Buffer. An inefficient approach is to deserialize it into a complete object, while the extreme approach is **"On-Demand Parsing"**.

### 3.1 Zero-Copy and Offset Positioning
For example, when monitoring Orca Whirlpool, we might only need `sqrt_price` and `tick_current_index`.
*   We don't need to parse the entire pool state (hundreds of bytes), just directly read the 16 bytes at a specific Offset in the data stream.
*   In Rust, by cooperating with `bytemuck` or simple pointer offsets, key pricing parameter extraction can be completed in microseconds.

### 3.2 The Art of Filters
In the `logsSubscribe` stage, utilizing the `mentions` filter provided by RPC can filter out 90% of irrelevant logs on the node side, greatly reducing network IO pressure on the Searcher end.

---

## 4. Performance Optimization Points: Milliseconds from Engineering Implementation

1.  **Sharding:** Addressing connection limits of public RPC nodes, Scout automatically shards whitelist pools, receiving concurrently via multiple WebSocket connections to avoid Backpressure on a single connection.
2.  **Noise Reduction Mechanism:** For high-frequency changing pools, implement simple packet loss or coalescing logic. If multiple updates occur for the same pool within 1ms, only process the last state to save computing resources in the strategy layer.
3.  **Pre-reading Index:** When parsing logs, pre-load Decimals information of common tokens to avoid secondary requests when calculating spreads.

---

## 5. Technical Demo: Multi-Stream Event Merging Logic (Python Simulation)

Although the high-performance core is in Rust, its "many-to-one" merging and distribution logic can be perfectly expressed using asyncio:

```python
import asyncio
import random

async def pool_monitor(pool_id: str):
    """Simulate a subscription stream for an independent account"""
    while True:
        await asyncio.sleep(random.uniform(0.01, 0.1)) # Simulate random push
        yield {"pool": pool_id, "data": random.random()}

async def main_scout_loop():
    # Simulate monitoring list obtained from Inventory
    watchlist = ["Pool_A", "Pool_B", "Pool_C"]
    
    # Aggregate all streams into a queue
    queue = asyncio.Queue()

    async def producer(pool_id):
        async for update in pool_monitor(pool_id):
            await queue.put(update)

    # Start all producer tasks
    for p in watchlist:
        asyncio.create_task(producer(p))

    print("[*] Scout engine started, listening to multi-channel signals...")
    
    # Core consumption loop: Strategy distribution processing
    while True:
        event = await queue.get()
        # Immediately trigger asynchronous calculation in strategy layer
        asyncio.create_task(execute_strategy(event))

async def execute_strategy(event):
    print(f"⚡️ Signal captured: {event['pool']} -> Triggering pricing model calculation")

if __name__ == "__main__":
    asyncio.run(main_scout_loop())
```

---

## 6. Summary: The Sharpest Radar

The design level of the Scout module directly determines the bot's "starting speed". An excellent Scout should be:
*   **Broad enough:** Capable of capturing new opportunities via logs.
*   **Accurate enough:** Capable of locking onto price fluctuations via account subscriptions.
*   **Fast enough:** Adopting asynchronous architecture and binary parsing to suppress latency to the microsecond level.

## What's Next

Captured the signal, got the raw data, what next? We need to convert binary data into real asset prices. In the next article, we will enter the **AMM Module**, revealing how **Raydium's Constant Product Formula** and **Orca's Concentrated Liquidity Mathematical Model** run at extreme speeds in memory.

---
*Written by Levi.eth, dedicated to sharing extreme engineering art in the Solana MEV field.*
