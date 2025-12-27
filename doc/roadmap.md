# Scavenger 项目后续规划 (Roadmap)

本文档描述了 Scavenger MEV 机器人在完成 Phase 3 基础功能后的演进路线。

## 🎯 总体目标
从一个“被动侦察兵”进化为“主动出击的猎人”，实现**毫秒级响应**、**零风险套利**和**高并发执行**。

---

## 🛠 Phase 4: Jito Bundle 实战集成 (Execution)
> **目标**: 打通最后的“发送”环节，实现真正的链上获利。

- [ ] **Jito gRPC 修复**
    - [ ] 解决 `jito-searcher-client` 与 `tonic`/`prost` 的版本冲突。
    - [ ] 实现稳定的 Block Engine 连接池。
- [ ] **Bundle 组装与发送**
    - [ ] 封装 `SearcherServiceClient::send_bundle`。
    - [ ] 实现 Tip 账户的动态轮询（获取随机 Tip Account 以防拥堵）。
- [ ] **防夹子与防失败 (Protection)**
    - [ ] 确保 Bundle 只有在模拟成功时才发送。
    - [ ] 利用 Jito 的 `Backrun` 特性实现无风险套利。

## ⚡ Phase 5: 性能极致优化 (Performance)
> **目标**: 将从“发现”到“发送”的端到端延迟压缩至 50ms 以内。

- [ ] **Geyser gRPC 接入**
    - [ ] 替代 WebSocket (PubSub)，改用 Helius/Triton 的 Geyser 插件订阅 Account Update。
    - [ ] 优势: 延迟降低 100-300ms，且包含更丰富的 Slot 信息。
- [ ] **本地状态缓存 (Local State Cache)**
    - [ ] 维护一个内存中的 Raydium/Orca Pool 镜像。
    - [ ] 收到 Update 时直接更新内存，而非每次都去 RPC `getAccount`。
- [ ] **FPGA/GPU 加速 (远期)**
    - [ ] 将 Swap 路径搜索算法迁移至 CUDA 或 FPGA。

## 🧠 Phase 6: 策略多样化 (Strategy)
> **目标**: 不仅仅是新池狙击，拓展更多获利模式。

- [ ] **三明治攻击 (Sandwiching)**
    - [ ] 监听 Mempool (需 Jito ShredStream)，识别大额 Pending 交易。
    - [ ] 插入 Buy -> Victim -> Sell。
- [ ] **CEX-DEX 套利**
    - [ ] 接入 Binance/OKX API。
    - [ ] 捕捉链上与中心化交易所的价差。
- [ ] **多路径环形套利 (Graph Search)**
    - [ ] 实现 Bellman-Ford 或 SPFA 算法，寻找 `SOL -> USDC -> RAY -> SOL` 的复杂获利路径。

---

## 📊 监控与运维 (DevOps)

- [ ] **Grafana + Prometheus**
    - [ ] 可视化展示：发现数/秒、成功率、利润曲线、Gas 消耗。
- [ ] **Telegram/Discord Bot**
    - [ ] 实时推送获利通知和余额预警。
