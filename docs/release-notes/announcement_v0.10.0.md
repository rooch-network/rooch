### Rooch v0.10.0 版本发布啦！🎉

我们很高兴地宣布 Rooch v0.10.0 现已发布！本次更新带来了重要的网络升级与协议能力增强，特别感谢所有贡献者的努力！

**主要亮点：**

*   **测试网重置并切换到 Bitcoin testnet4：** 本次发布重置了测试网，并将比特币网络切换到 `testnet4`，以对齐上游生态的最新测试网络。
*   **Payment Channel 实现并已部署至测试网：** 新增支付通道能力，支持更高吞吐与近实时结算的应用形态，已在测试网上线，欢迎试用与反馈。

**请注意：** 本次测试网已重置并切换至 Bitcoin testnet4，历史状态已清空。请开发者与用户：

- 重新初始化（或同步）本地测试网数据（若运行节点）。
- 重新申请测试币、重新部署合约与脚本。
- 检查与更新依赖到最新版本的 SDK/工具。

**其他更新：**

*   改进文档与开发指引：包含错误码规范与 AI 配置指南优化。
*   **DID 会话范围（DID session scope）：** 完善 DID 相关能力与边界定义。
*   **状态修剪（Pruner）：** 支持 StateDB 修剪，并可遍历全局与表级状态，降低存储占用。
*   部署脚本增强：支持使用哈希标签加速部署定位。
*   稳定性提升：在回滚时忽略可安全忽略的索引器错误；临时关闭测试网的比特币同步以配合切网。
*   构建与发布：修复 Windows 发布流程；多项依赖升级（tokio、serde_json、diesel、quick_cache 等）。

**特别感谢：**

感谢所有为这个版本做出贡献的社区成员！

**新贡献者：**

- @houpo-bob 在 PR [#3657](https://github.com/rooch-network/rooch/pull/3657) 完成了首次贡献
- @tanhuaan 在 PR [#3663](https://github.com/rooch-network/rooch/pull/3663) 完成了首次贡献

**了解更多：**

- 完整发布说明请参阅 GitHub Release 页面：
  - [https://github.com/rooch-network/rooch/releases/tag/v0.10.0](https://github.com/rooch-network/rooch/releases/tag/v0.10.0)
- 完整变更列表（Changelog）：
  - [https://github.com/rooch-network/rooch/compare/v0.9.7...v0.10.0](https://github.com/rooch-network/rooch/compare/v0.9.7...v0.10.0)

我们鼓励您升级到最新版本并体验新的支付通道与测试网环境。如果您有任何问题或反馈，请随时通过我们的社区渠道与我们联系。

让我们一起共建更好的 Rooch！🚀

---

### Rooch v0.10.0 is Live! 🎉

We are excited to release Rooch v0.10.0! This version delivers a major network transition and key protocol enhancements. Thank you to all contributors!

**Key Highlights:**

*   **Testnet reset and switch to Bitcoin testnet4:** The public testnet has been reset and migrated to `testnet4` to align with the latest upstream Bitcoin test network.
*   **Payment Channel implemented and deployed to testnet:** A new payment channel capability is now available on testnet, enabling higher throughput and near real-time settlement use cases.

**Please note:** The testnet has been reset and switched to Bitcoin testnet4. Previous state has been cleared. Developers and users should:

- Reinitialize (or resync) local testnet data if you run a node.
- Reclaim test tokens and redeploy your contracts and scripts.
- Update SDKs/tools to the latest compatible versions.

**Other Updates:**

*   Docs and developer experience: improved error code conventions and AI configuration/guide.
*   **DID session scope:** Enhancements across DID capability boundaries.
*   **State pruning (Pruner):** Support pruning StateDB and traversing both global and table states to reduce storage.
*   Deployment script: support hash tag for faster pinpoint deployments.
*   Stability: ignore safe-to-ignore indexer errors on rollback; temporarily disable testnet Bitcoin sync during the network switch.
*   Build & release: fix Windows release workflow; multiple dependency upgrades (tokio, serde_json, diesel, quick_cache, etc.).

**Special Thanks:**

Huge thanks to all community contributors!

**New Contributors:**

- @houpo-bob made their first contribution in [#3657](https://github.com/rooch-network/rooch/pull/3657)
- @tanhuaan made their first contribution in [#3663](https://github.com/rooch-network/rooch/pull/3663)

**Learn More:**

- Full Release: [https://github.com/rooch-network/rooch/releases/tag/v0.10.0](https://github.com/rooch-network/rooch/releases/tag/v0.10.0)
- Full Changelog: [https://github.com/rooch-network/rooch/compare/v0.9.7...v0.10.0](https://github.com/rooch-network/rooch/compare/v0.9.7...v0.10.0)

We encourage you to upgrade and try out the new payment channel and the refreshed testnet environment. If you have questions or feedback, please reach out via our community channels.

Let's build a better Rooch together! 🚀


