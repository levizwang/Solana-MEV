# Execution：原子交易与 Jito Bundle

执行层负责把“策略决策”转换为“可上链的可执行载荷”，核心目标是：

- 原子性：套利闭环要么全部成功要么全部失败，避免腿断损失。
- 可包含性：尽量提高交易被 leader 打包的概率（tip/priority fee、bundle 通道）。

## 1. 模块功能说明

- 指令构建：
  - Raydium swap 指令：`core/swap.rs`
  - Orca swap 指令（需要 tick arrays）：`core/swap.rs` / `core/orca.rs`
- 原子交易：指令列表 + tip 指令组装为一笔交易：`core/arbitrage.rs`
- Bundle 发送：HTTP JSON-RPC 调用 `sendBundle`：`core/jito_http.rs`

```mermaid
flowchart LR
  STRAT[Strategy] --> IX[Instructions]
  IX --> TX[Signed Transaction]
  TX --> B58[Base58 serialize]
  B58 --> RPC[Jito sendBundle]
```

对应源码：

- `../../scavenger/src/core/swap.rs`
- `../../scavenger/src/core/arbitrage.rs`
- `../../scavenger/src/core/jito_http.rs`

## 2. 技术实现细节

### 2.1 原子交易（Transaction）与 Bundle 的关系

在 Solana 上：

- 一笔 Transaction 可以包含多条指令（多个 swap + transfer tip）。
- Bundle 是“多笔交易”作为一个集合提交给 block engine，通常提供更强的原子与排序语义（实现依赖具体引擎）。

本项目当前策略层通常把“swap(s) + tip”放入同一笔交易，再作为 bundle 提交（bundle 中只有 1 笔 tx）。

### 2.2 Jito sendBundle（HTTP JSON-RPC）

`core/jito_http.rs` 会把交易序列化为 base58 字符串数组，构造 payload：

- `jsonrpc: "2.0"`
- `method: "sendBundle"`
- `params: [ [tx_base58_1, tx_base58_2, ...] ]`

然后 POST 到 `https://mainnet.block-engine.jito.wtf/api/v1/bundles`。

## 3. 关键算法和数据结构

- 指令序列化：Raydium swap 指令 data 为 `[instruction_id|amount_in|min_out]` 的字节布局
- 指令账户表：`Vec<AccountMeta>` 的顺序必须与 on-chain program 期待一致
- 原子交易 builder：聚合 `Vec<Instruction>` 后签名生成 `Transaction`

## 4. 性能优化点

- 预计算账户与 PDA：如 ATA、AMM authority、Serum vault signer、Orca tick arrays 等，避免在机会窗口内做过多 RPC。
- 动态 tip 与熔断：结合 `core/pricing.rs` 的利润模型，动态调整 tip，同时不超过 `max_tip_sol`。

## 5. 可运行示例（构造 sendBundle JSON-RPC payload）

该示例不发送网络请求，仅构造 payload 并打印，便于在本地验证结构：

```python
import json
from dataclasses import dataclass
from typing import List

@dataclass(frozen=True)
class SendBundleRequest:
    txs_base58: List[str]

    def to_jsonrpc(self) -> dict:
        # 对应 sendBundle 的最简 JSON-RPC 载荷结构
        return {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [self.txs_base58],
        }

if __name__ == "__main__":
    req = SendBundleRequest(txs_base58=["3Bxs...mockTx1", "4Cys...mockTx2"])
    print(json.dumps(req.to_jsonrpc(), indent=2))
```

## 6. 相关篇

- 上游（策略如何产生指令）：[StrategyArb_跨DEX套利策略.md](./StrategyArb_跨DEX套利策略.md)
- 风控（是否允许执行）：[Risk_风控与安全检查.md](./Risk_风控与安全检查.md)
