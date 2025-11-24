### Rooch v0.12.0 版本发布啦！🎉

我们很高兴地宣布 Rooch v0.12.0 现已发布！本次版本在支付通道、DID 资源控制以及状态修剪器（Pruner）可靠性方面带来了关键增强，同时全面升级了依赖与 CI/构建流程，提升了网络稳定性与运维体验。完整更新列表可参考 GitHub Release 页面和 Changelog（见文末链接）。[`Rooch v0.12.0` GitHub Release](https://github.com/rooch-network/rooch/releases/tag/v.12.0)

**主要亮点：**

- **X402 支付通道与收入事件增强**  
  - 引入 **X402 支付通道** 功能（`X402 payment channel`），为基于 Bitcoin 的支付场景提供更丰富的通道能力。  
  - 为支付 Hub 增加 **按 Hub 维度的收入事件**，并通过自定义事件句柄（custom event handles）统一管理 DID 与支付通道相关事件，方便链上监控和对账。  
  - 为支付 Hub 提现增加 **锁定单元预留（locked unit reserve）** 机制，进一步提升资金安全与结算过程的稳健性。

- **DID 资源与验证器体系增强**  
  - 在 DID 与支付通道模块中引入 **资源上限控制（resource limits）**，防止异常状态或滥用导致的资源膨胀。  
  - 增加更多 **DID 验证器测试用例**，覆盖关键验证路径，强化 DID 相关逻辑的稳定性。  
  - 推出新的 **DID 技术博客与文档更新**，帮助开发者快速理解新版 DID 架构与使用方式。

- **状态修剪器（Pruner）与存储可靠性改进**  
  - 对 Pruner 的压缩（compact）行为进行排查和修复，解决在特定场景下的压缩问题。  
  - 为 RocksDB 增加 **检查点（checkpoint）生成功能**，便于长周期运行时的状态备份与回滚。  
  - 通过配置优化和效果验证，**改进 Pruner 机制与默认参数**，使长时间运行下的状态修剪更稳定、更可预期。  
  - 新增 **Pruner 端到端（e2e）测试**，从整体视角验证修剪与存储在真实负载下的表现。

**技术改进：**

- **CI 与构建流程优化**  
  - 修复 Windows 平台 Release 流程问题，确保多平台发布一致性。  
  - 优化 CI 工作流，包含取消任务流程与 Docker 构建流程修复，使得构建与发布过程更加稳定、可观测、可恢复。  

- **SDK 与测试基础设施**  
  - 改善 Web SDK 的测试端口管理与进程清理逻辑，减少测试过程中端口冲突和遗留进程问题，提升开发者本地与 CI 环境下的测试体验。

- **依赖升级与性能提升**  
  - 系统性升级了包括 `serde`、`tokio`、`tower-http`、`multibase`、`bip32`、`bytes`、`pprof`、`thiserror`、`lru`、`petgraph` 等在内的核心依赖。  
  - 更新 `regex`、`csv`、`toml`、`strum`、`strum_macros`、`serde-reflection` 等库版本，获得更好的性能、兼容性与安全性。  
  - 这些升级为未来功能演进与性能调优打下了更稳固的基础。

**其他更新：**

- 修复了大量注释中的拼写问题，统一和改进代码注释质量。  
- 持续更新文档与 Release Notes，确保版本变更透明清晰。  

**特别感谢：**

感谢所有为 v0.12.0 做出贡献的社区成员！

**了解更多：**

- 完整发布说明请参阅 GitHub Release 页面：  
  - [https://github.com/rooch-network/rooch/releases/tag/v.12.0](https://github.com/rooch-network/rooch/releases/tag/v.12.0)  
- 完整变更列表（Changelog）：  
  - [https://github.com/rooch-network/rooch/compare/v0.11.0...v.12.0](https://github.com/rooch-network/rooch/compare/v0.11.0...v.12.0)

我们鼓励您升级到 Rooch v0.12.0，体验更强大的支付通道能力、更安全的 DID 资源控制以及更可靠的状态修剪机制。如果您有任何问题或反馈，欢迎通过社区渠道与我们联系，一起共建更好的 Rooch！🚀


---

### Rooch v0.12.0 is Live! 🎉

We are excited to release Rooch v0.12.0! This version brings key enhancements to payment channels, DID resource control, and the state pruner’s reliability, along with comprehensive dependency and CI/build workflow upgrades. For full details, please refer to the GitHub Release and Changelog. See the [`Rooch v0.12.0` GitHub Release](https://github.com/rooch-network/rooch/releases/tag/v.12.0).

**Key Highlights:**

- **X402 Payment Channel and Revenue Events**  
  - Introduces the **X402 payment channel** to enrich Bitcoin-based payment use cases on Rooch.  
  - Adds **per-hub payment revenue events** with custom event handles shared across DID and payment channel modules, enabling better on-chain monitoring and accounting.  
  - Adds a **locked unit reserve** mechanism for payment hub withdrawals to enhance fund safety and settlement robustness.

- **Stronger DID Resources and Validator System**  
  - Adds **resource limits** to DID and payment channel modules to prevent resource bloat and abusive patterns.  
  - Extends **DID validator test coverage** with more test cases on critical validation paths.  
  - Ships new **DID blog content and documentation updates** to help developers quickly understand and adopt the updated DID architecture.

- **State Pruner and Storage Reliability Improvements**  
  - Troubleshoots and fixes issues related to pruner **compaction behavior** in specific scenarios.  
  - Adds **RocksDB checkpoint generation**, making it easier to back up and restore long-running networks.  
  - **Improves the pruner mechanism and default configurations**, validating effectiveness to ensure more stable and predictable pruning over time.  
  - Introduces **end-to-end pruner tests**, validating pruning and storage behavior under realistic workloads.

**Technical Improvements:**

- **CI and Build Workflow Optimization**  
  - Fixes Windows release workflows to restore consistent multi-platform releases.  
  - Optimizes CI workflows, including cancel workflows and Docker build fixes, making the build and release process more stable, observable, and recoverable.

- **SDK and Testing Infrastructure**  
  - Improves Web SDK test port management and process cleanup, reducing port conflicts and orphaned processes in both local and CI environments.

- **Dependency Upgrades and Performance**  
  - Systematically upgrades core dependencies such as `serde`, `tokio`, `tower-http`, `multibase`, `bip32`, `bytes`, `pprof`, `thiserror`, `lru`, `petgraph`, and more.  
  - Updates `regex`, `csv`, `toml`, `strum`, `strum_macros`, `serde-reflection`, and others to benefit from performance, compatibility, and security improvements.  
  - These upgrades provide a stronger foundation for future features and optimizations.

**Other Updates:**

- Fixes a large number of spelling issues in comments, improving overall code readability and consistency.  
- Keeps documentation and release notes aligned with the latest changes for better transparency.

**Special Thanks:**

Huge thanks to all community contributors for v0.12.0!

**Learn More:**

- Full Release:  
  - [https://github.com/rooch-network/rooch/releases/tag/v.12.0](https://github.com/rooch-network/rooch/releases/tag/v.12.0)  
- Full Changelog:  
  - [https://github.com/rooch-network/rooch/compare/v0.11.0...v.12.0](https://github.com/rooch-network/rooch/compare/v0.11.0...v.12.0)

We encourage you to upgrade to Rooch v0.12.0 and explore the enhanced payment channels, stronger DID resource controls, and more reliable pruning. If you have questions or feedback, please reach out through our community channels. Let’s keep building a better Rooch together! 