### Rooch v0.11.0 版本发布啦！🎉

我们很高兴地宣布 Rooch v0.11.0 现已发布！本次更新带来了重大的 DID（去中心化身份）系统增强和支付基础设施改进，特别感谢所有贡献者的努力！

**主要亮点：**

*   **DID 验证器系统全面升级：** 引入了全新的 DID 验证器架构，支持 `did:bitcoin` 作为控制器，实现了 Bitcoin 地址对 DID 文档的直接控制能力。
*   **支付收入分配系统：** 实现了支付收入分配机制，支持自动检测 DID 地址和钱包地址，为收入共享和分配提供了完整的基础设施。
*   **Gas 支付中心：** 增强了 Gas 支付系统，支持从支付中心支付 Gas 费用，改进了交易 Gas 处理机制。
*   **WebAuthn 支持：** 新增 WebAuthn 信封支持，提供更安全的身份验证方式，并与会话密钥系统完全兼容。

**技术改进：**

*   **状态修剪器优化：** 启用并优化了状态修剪器，支持压缩操作，有效降低存储占用。
*   **测试套件增强：** 改进了测试框架，提供更好的错误报告和调试能力。
*   **数据库工具：** 新增 dump-state 和 import-state 命令，方便状态数据的导出和导入。
*   **运维工具：** 添加了测试网维护 Pod 和脚本，提升了网络运维效率。

**其他更新：**

*   重构了认证验证器错误码，提供更好的调试体验。
*   优化了 DID 虚拟机和会话密钥交互机制。
*   改进了 Bitcoin 消息处理和信封处理流程。
*   增强了会话签名信封机制。
*   完善了支付通道和收入文档。

**特别感谢：**

感谢所有为这个版本做出贡献的社区成员！

**了解更多：**

- 完整发布说明请参阅 GitHub Release 页面：
  - [https://github.com/rooch-network/rooch/releases/tag/v0.11.0](https://github.com/rooch-network/rooch/releases/tag/v0.11.0)
- 完整变更列表（Changelog）：
  - [https://github.com/rooch-network/rooch/compare/v0.10.0...v0.11.0](https://github.com/rooch-network/rooch/compare/v0.10.0...v0.11.0)

我们鼓励您升级到最新版本并体验新的 DID 系统和支付基础设施。如果您有任何问题或反馈，请随时通过我们的社区渠道与我们联系。

让我们一起共建更好的 Rooch！🚀

---

### Rooch v0.11.0 is Live! 🎉

We are excited to release Rooch v0.11.0! This version delivers major enhancements to the DID (Decentralized Identity) system and payment infrastructure improvements. Thank you to all contributors!

**Key Highlights:**

*   **Comprehensive DID Validator System Upgrade:** Introduced a new DID validator architecture with support for `did:bitcoin` as controller, enabling Bitcoin addresses to directly control DID documents.
*   **Payment Revenue Distribution System:** Implemented payment revenue distribution mechanism with auto-detection of DID addresses and wallet addresses, providing complete infrastructure for revenue sharing and distribution.
*   **Gas Payment Hub:** Enhanced gas payment system with support for paying gas fees from payment hub, improving transaction gas handling mechanisms.
*   **WebAuthn Support:** Added WebAuthn envelope support for more secure authentication, fully compatible with the session key system.

**Technical Improvements:**

*   **State Pruner Optimization:** Enabled and optimized the state pruner with compaction support, effectively reducing storage usage.
*   **Enhanced Test Suite:** Improved testing framework with better error reporting and debugging capabilities.
*   **Database Tools:** Added dump-state and import-state commands for convenient state data export and import.
*   **Operations Tools:** Added testnet maintenance pod and scripts, improving network operations efficiency.

**Other Updates:**

*   Refactored authentication validator error codes for better debugging experience.
*   Optimized DID virtual machine and session key interaction mechanisms.
*   Improved Bitcoin message handling and envelope processing.
*   Enhanced session signing envelope mechanism.
*   Improved payment channel and revenue documentation.

**Special Thanks:**

Huge thanks to all community contributors!

**Learn More:**

- Full Release: [https://github.com/rooch-network/rooch/releases/tag/v0.11.0](https://github.com/rooch-network/rooch/releases/tag/v0.11.0)
- Full Changelog: [https://github.com/rooch-network/rooch/compare/v0.10.0...v0.11.0](https://github.com/rooch-network/rooch/compare/v0.10.0...v0.11.0)

We encourage you to upgrade and try out the new DID system and payment infrastructure. If you have questions or feedback, please reach out via our community channels.

Let's build a better Rooch together! 🚀


