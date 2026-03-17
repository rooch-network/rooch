### Rooch v0.13.x 版本发布

Rooch v0.13.x 当前包含三个版本：
- `v0.13.0`（2026-01-27）
- `v0.13.1`（2026-01-29）
- `v0.13.2`（2026-02-16）

对比区间 `v0.12.2...v0.13.2` 共 66 个提交，重点围绕状态修剪（state-prune）与回放（replay）稳定性、Bitcoin header-only 能力、以及工程基础设施升级展开。

**主要亮点**

- **State Prune / Snapshot / Replay 全链路增强**
  - 引入 snapshot + changeset replayer 主流程。
  - Snapshot 改为流式构建并采用 RocksDB 后端，降低 OOM 风险。
  - 支持 snapshot 任务中断恢复（resume/restart）。
  - Replay 增加严格的 final state root 验证，避免“回放成功但状态不一致”。
  - 修复 changeset range 加载与 tx order 相关问题，补齐关键回归测试。

- **Bitcoin 能力升级**
  - `bitcoin-move` 支持 header-only 导入与 Merkle proof 验证路径。
  - 针对 UTXO 加载限流场景，加入指数退避重试机制。
  - 修复 PSBT 费率估算逻辑，并增强 `verify-psbt` 费率/体积显示。

- **工程与运维改进**
  - Rust 工具链升级到 `1.91.1`。
  - CI 从 self-hosted 向 GitHub-hosted 迁移并优化成本/效率。
  - 继续完善依赖、测试与构建流程稳定性。

**v0.13.1 / v0.13.2 补丁重点**

- 修复 replay 过程中 changeset range 读取不完整问题并增加回归测试。
- 临时关闭 BTC -> RGAS 购买入口，为新验证方案重构预留切换窗口。
- 恢复 Bitcoin header-only 下“确认窗口后 finalize”的机制，并补齐 reorg/边界测试。
- 新增 `db estimate-state-nodes` 分层采样/精确统计能力，帮助评估状态规模。
- 新增 snapshot 速度调优参数（如 `--skip-dedup` / `--skip-final-compact` 等）。

**升级建议**

- 建议运行 state-prune/replay 的节点优先升级到 `v0.13.2`。
- 依赖 BTC -> RGAS 购买流程的用户请关注后续验证机制恢复公告。
- 流量参数优先使用 `--requests-per-second`。
- 截至 2026-02-16，`origin` 上尚无 `v0.13.3` tag。

**参考链接**

- v0.13.0 Release: https://github.com/rooch-network/rooch/releases/tag/v0.13.0
- v0.13.1 Release: https://github.com/rooch-network/rooch/releases/tag/v0.13.1
- v0.13.2 Release: https://github.com/rooch-network/rooch/releases/tag/v0.13.2
- Compare `v0.12.2...v0.13.0`: https://github.com/rooch-network/rooch/compare/v0.12.2...v0.13.0
- Compare `v0.13.0...v0.13.1`: https://github.com/rooch-network/rooch/compare/v0.13.0...v0.13.1
- Compare `v0.13.1...v0.13.2`: https://github.com/rooch-network/rooch/compare/v0.13.1...v0.13.2

---

### Rooch v0.13.x Is Live

Rooch v0.13.x currently includes:
- `v0.13.0` (January 27, 2026)
- `v0.13.1` (January 29, 2026)
- `v0.13.2` (February 16, 2026)

Across `v0.12.2...v0.13.2`, this cycle delivered 66 commits focused on reliability and scalability in state pruning/replay, Bitcoin header-only support, and infrastructure upgrades.

**Key Highlights**

- **State Prune / Snapshot / Replay Improvements**
  - Introduced the snapshot + changeset replayer flow.
  - Switched snapshot building to streaming with RocksDB backend to reduce OOM risk.
  - Added resume/restart support for long-running snapshot jobs.
  - Enforced strict final state-root verification during replay.
  - Fixed changeset range and tx-order related issues with regression coverage.

- **Bitcoin Upgrades**
  - Added header-only import mode and Merkle proof verification path in `bitcoin-move`.
  - Added exponential backoff retry for UTXO loading under rate limits.
  - Improved PSBT fee-rate estimation and `verify-psbt` output.

- **Engineering and Operations**
  - Upgraded Rust toolchain to `1.91.1`.
  - Migrated CI from self-hosted to GitHub-hosted runners and optimized cost/perf.
  - Continued reliability improvements across build/test/dependency workflows.

**v0.13.1 / v0.13.2 Patch Focus**

- Fixed incomplete changeset-range loading during replay and added regression tests.
- Temporarily disabled BTC -> RGAS swap entry while migrating to the new verification path.
- Restored confirmation-window finalize behavior in Bitcoin header-only flow.
- Added `db estimate-state-nodes` (stratified sampling + exact counting modes).
- Added snapshot speed knobs (`--skip-dedup`, `--skip-final-compact`, etc.).

**Upgrade Guidance**

- Strongly recommend `v0.13.2` for nodes running snapshot/replay workflows.
- If you rely on BTC -> RGAS purchase, watch upcoming announcements for the re-enabled flow.
- Prefer `--requests-per-second` for traffic rate configuration.
- As of February 16, 2026, there is no `v0.13.3` tag on `origin`.

**Links**

- v0.13.0 Release: https://github.com/rooch-network/rooch/releases/tag/v0.13.0
- v0.13.1 Release: https://github.com/rooch-network/rooch/releases/tag/v0.13.1
- v0.13.2 Release: https://github.com/rooch-network/rooch/releases/tag/v0.13.2
- Compare `v0.12.2...v0.13.0`: https://github.com/rooch-network/rooch/compare/v0.12.2...v0.13.0
- Compare `v0.13.0...v0.13.1`: https://github.com/rooch-network/rooch/compare/v0.13.0...v0.13.1
- Compare `v0.13.1...v0.13.2`: https://github.com/rooch-network/rooch/compare/v0.13.1...v0.13.2
