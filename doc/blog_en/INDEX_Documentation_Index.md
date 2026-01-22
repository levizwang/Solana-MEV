# Solana-MEV Documentation Index

This directory is intended for "Technical Blog Sharing" scenarios. It breaks down the key modules of Scavenger (Solana MEV Bot) by system layers, and provides runnable minimal examples, key implementation details, and cross-references in each article.

## Reading Path

1.  **Read the Overview first** to establish a common context for MEV and the Solana transaction pipeline.
    -   [SolanaMEV_Technical_Analysis.md](./SolanaMEV_Technical_Analysis.md)
2.  **Then read the Module Breakdowns**, understanding the closed loop from "Data → Monitoring → Pricing → Strategy → Execution → Risk Control".
    -   [Project_Module_Breakdown_Overview.md](./Project_Module_Breakdown_Overview.md)
    -   [ControlPlane_Strategy_Scheduling_and_Configuration.md](./ControlPlane_Strategy_Scheduling_and_Configuration.md)
    -   [Inventory_Network_Wide_Token_Index.md](./Inventory_Network_Wide_Token_Index.md)
    -   [Scout_Transaction_Monitoring_and_Parsing.md](./Scout_Transaction_Monitoring_and_Parsing.md)
    -   [AMM_Pricing_and_Mathematical_Model.md](./AMM_Pricing_and_Mathematical_Model.md)
    -   [StrategyArb_Cross_DEX_Arbitrage_Strategy.md](./StrategyArb_Cross_DEX_Arbitrage_Strategy.md)
    -   [Execution_Atomic_Transaction_and_JitoBundle.md](./Execution_Atomic_Transaction_and_JitoBundle.md)
    -   [Risk_Risk_Control_and_Safety_Checks.md](./Risk_Risk_Control_and_Safety_Checks.md)

## Correspondence with Source Code (Recommended to read together)

-   **Architecture/User Manual (Original Project Documentation)**
    -   `doc/ARCHITECTURE.md`
    -   `doc/USER_MANUAL.md`
-   **Rust Data Plane**
    -   Inventory: `scavenger/src/state.rs`
    -   Scout: `scavenger/src/scout/*`
    -   AMM/Pricing: `scavenger/src/amm/*`, `scavenger/src/core/quote.rs`
    -   Strategy: `scavenger/src/strategies/*`
    -   Execution/Jito: `scavenger/src/core/{swap,arbitrage,jito_http}.rs`
    -   Risk: `scavenger/src/core/risk.rs`
-   **Python Control Plane**
    -   `commander/main.py`

## Cross-Reference Conventions

-   **Closed-loop path from "Upstream → Downstream":** Inventory → Scout → AMM/Pricing → Strategy → Execution → Risk
-   Each document contains "Next/Related Article" links at the end, connected according to this path.
