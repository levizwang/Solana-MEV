# Solana-MEV 文档索引

本目录面向“技术博客分享”场景，按系统分层拆解 Scavenger（Solana MEV Bot）的关键模块，并在每篇文档中提供可运行的最小示例、关键实现细节与交叉引用。

## 阅读路线

1. 先读总览，建立 MEV 与 Solana 交易管线的共同语境  
   - [SolanaMEV_技术解析.md](./SolanaMEV_技术解析.md)
2. 再读模块拆解，从“数据→监听→定价→策略→执行→风控”理解闭环  
   - [Project_模块拆分总览.md](./Project_模块拆分总览.md)  
   - [ControlPlane_策略调度与配置.md](./ControlPlane_策略调度与配置.md)  
   - [Inventory_全网代币索引.md](./Inventory_全网代币索引.md)  
   - [Scout_交易监听与解析.md](./Scout_交易监听与解析.md)  
   - [AMM_定价与数学模型.md](./AMM_定价与数学模型.md)  
   - [StrategyArb_跨DEX套利策略.md](./StrategyArb_跨DEX套利策略.md)  
   - [Execution_原子交易与JitoBundle.md](./Execution_原子交易与JitoBundle.md)  
   - [Risk_风控与安全检查.md](./Risk_风控与安全检查.md)

## 与源码的对应关系（建议对照阅读）

- 架构/使用说明（原项目文档）
  - `doc/ARCHITECTURE.md`
  - `doc/USER_MANUAL.md`
- Rust 数据平面（Data Plane）
  - Inventory：`scavenger/src/state.rs`
  - Scout：`scavenger/src/scout/*`
  - AMM/Pricing：`scavenger/src/amm/*`、`scavenger/src/core/quote.rs`
  - 策略：`scavenger/src/strategies/*`
  - 执行/Jito：`scavenger/src/core/{swap,arbitrage,jito_http}.rs`
  - 风控：`scavenger/src/core/risk.rs`
- Python 控制平面（Control Plane）
  - `commander/main.py`

## 交叉引用约定

- “上游→下游”的闭环路径：Inventory → Scout → AMM/Pricing → Strategy → Execution → Risk
- 每篇文档末尾包含“下一篇/相关篇”链接，按该路径串联。
