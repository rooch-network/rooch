## Mainnet Tail Replay Tool Design

### Summary

当前已经验证了两件关键事实：

1. `db state-prune replay` 可以基于 reset 后 snapshot 构建一份可启动的主网候选库。
2. 这份 replay 输出库可以启动节点，但不能通过 `sendRawTransaction` 重新提交 canonical raw tx 来追平尾部交易。

第二点已经在主网候选库上验证过：

- 基线 replay 输出库：
  - `last_order = 241186740`
  - `state_root = 0xc77da37b34b4340250e2b23e018bc58c643d4f09f0b451d451f65cff0a3bd119`
- 直接把 canonical `tx_order = 241186741` 的 raw tx 发到候选库执行后：
  - `tx_hash` 相同
  - 但 resulting `state_root` 与主网 canonical `execution_info.state_root` 不一致

结论很明确：

- **不能**依赖 `sendRawTransaction` 做 tail catch-up
- 应该新增一个真正的 **tail replay / delta changeset replay** 工具

目标是：

- 基于**已有 replay 输出库**
- 只追平 `last_order + 1 .. target_order`
- 不重新导入 snapshot
- 不重新复制整批 required CF
- 不重新跑整轮重 replay

### Why A New Tool Is Needed

当前可用命令：

- `rooch db state-prune replay`
- `rooch db state-prune finalize-replay-output`

它们解决的是：

- 从 snapshot 构建 slim output DB
- 对中途失败但主体完成的 output 做 finalize

但它们都不适合处理如下场景：

- replay 输出库已经构建完成
- 只落后主网几千笔交易
- 希望把这几千笔 canonical ledger tx 正确追平

当前代码里虽然已有 changeset 回放能力：

- `load_changesets_range(...)`
- `replay_changesets_batched(...)`
- `apply_changeset_batch(...)`

但它们都内嵌在 `IncrementalReplayer::replay_changesets(...)` 里，没有独立 CLI 用于：

- 对**现有 output 库**
- 直接应用一个 delta order range

### Scope

本工具第一阶段只解决：

- 对已有 replay 输出库做 canonical tail catch-up
- 使用源库中的原始 canonical history
- 通过 changeset 和 sequencer metadata 把 output 库推进到新 `to_order`

第一阶段**不**解决：

- 从空库构建 full replay 输出
- 重新导入 snapshot
- 历史裁剪策略重设计
- event/event_handle 的历史补齐

### Command Proposal

新增一个命令：

```bash
rooch db state-prune tail-replay
```

建议参数：

```bash
rooch db state-prune tail-replay \
  --data-dir <source_live_data_dir> \
  -n <chain> \
  --output <existing_replay_output_base_dir> \
  --to-order <target_order> \
  [--from-order <explicit_from_order>] \
  [--batch-size 1000] \
  [--verify-root true]
```

语义：

- `--output` 指向**已有 replay 输出库的 base dir**
- `from_order` 默认取：
  - `existing_output.sequencer_info.last_order + 1`
- `to_order` 为目标追平上界
- 命令只对 `from_order..=to_order` 做增量处理

### Preconditions

在进入 tail replay 前，输出库必须已经满足：

1. `startup_info` 存在
2. `sequencer_info` 存在
3. 两者一致
4. output 库能至少以只读模式启动

也就是当前 `finalize-replay-output` 之后的 replay output 状态。

### Why Not Resubmit Canonical Raw Transactions

虽然区块链执行是确定的，但 **RPC 提交路径** 并不等于“同步 canonical 历史”。

当前已经验证：

- 把 canonical raw tx 通过 `rooch_sendRawTransaction` 发到 replay 输出节点
- 得到的 `tx_hash` 与 canonical 相同
- 但 resulting `state_root` 与 canonical 不同

这说明：

- `sendRawTransaction` 走的是“本地当前节点重新接收/排序/执行”的路径
- 它不是“按原历史 order 与原系统上下文同步”的路径

因此 tail catch-up 必须基于：

- canonical `LedgerTransaction`
- canonical `TransactionExecutionInfo`
- canonical `StateChangeSetExt`

而不是 RPC 重提交。

### Data To Read From Source

对 delta range `from_order..=to_order`，从源库读取：

- `transaction`
- `tx_sequence_info_mapping`
- `transaction_execution_info`
- `state_change_set`

并额外读取：

- `da_last_block_number`
- `da_block_submit_state`
- `proposer_last_block`

说明：

- `transaction` / `tx_sequence_info_mapping` / `transaction_execution_info` / `state_change_set`
  是 canonical tail 的最小历史面
- `da_*` / `proposer_*` 需要在 tail 完成后刷新到 `to_order`

### Data To Write To Output

输出库需要更新：

- `state_node`
- `transaction`
- `tx_sequence_info_mapping`
- `transaction_execution_info`
- `state_change_set`
- `transaction_acc_node`
- `meta_sequencer_info`
- `config_startup_info`
- `da_*`
- `proposer_*`

### Core Design

#### Step 1. Open Existing Output Store

不创建 fresh DB。

直接打开：

- existing output `MoveOSStore`
- existing output `RoochStore`

并读取当前锚点：

- `base_order = sequencer_info.last_order`
- `base_accumulator_info = sequencer_info.last_accumulator_info`
- `base_state_root = startup_info.state_root`
- `base_global_size = startup_info.size`

如果用户显式传了 `from_order`，校验：

- `from_order == base_order + 1`

否则直接推导：

- `from_order = base_order + 1`

#### Step 2. Load Canonical Delta Range

复用现有：

- `load_changesets_range(from_order, to_order, ...)`

同时新增一个轻量历史复制阶段：

- 只复制 `from_order..=to_order` 的
  - `transaction`
  - `tx_sequence_info_mapping`
  - `transaction_execution_info`

这里不需要再复制整段历史，只要复制 delta 即可。

#### Step 3. Rebuild Accumulator Tail Locally

这是 tail replay 设计里的关键点。

不建议再次整表复制 `transaction_acc_node`。

更合理的方式是：

1. 用 `base_accumulator_info` 和 output 库已有的 `transaction_acc_node` 构造：
   - `MerkleAccumulator::new_with_info(...)`
2. 对 delta range 内每一笔 canonical `LedgerTransaction`：
   - 取其 `tx_hash`
   - 依 canonical `tx_order` 顺序 append
3. 每 append 一批后：
   - `pop_unsaved_nodes()`
   - 把新增 `AccumulatorNode` 写入 output `transaction_acc_node`

现有 sequencer 就是这么构造 canonical accumulator 的：

- `tx_accumulator.append(&[tx_hash])`
- `tx_accumulator.pop_unsaved_nodes()`
- `tx_accumulator.get_info()`

所以 tail replay 可以直接复用同一套语义。

这样做的好处：

- 不需要再整 CF 复制 `transaction_acc_node`
- 只写 delta 对应的新 accumulator nodes
- output 库最终的 `SequencerInfo.last_accumulator_info` 与 canonical `to_order` 保持一致

#### Step 4. Apply Delta Changesets To Existing State

复用现有：

- `replay_changesets_batched(...)`

但起点改成：

- `base_state_root = existing output startup_info.state_root`
- `base_global_size = existing output startup_info.size`

也就是：

- **不导入 snapshot**
- 直接在已有 replay 输出库的 state tree 上继续推进

#### Step 5. Save Delta Transaction Metadata

对每一笔 delta tx：

- 写 `transaction`
- 写 `tx_sequence_info_mapping`
- 写 `transaction_execution_info`
- 写 `state_change_set`

这里建议新增一个专门的写路径，而不是散落成多个独立调用：

- `save_tail_replayed_tx(...)`

建议语义：

1. 先保存 `LedgerTransaction`、`SequencerInfo`、delta accumulator nodes
   - 可复用 `RoochStore::save_sequenced_tx(...)`
2. 再保存：
   - `transaction_execution_info`
   - `state_change_set`

注意：

- `save_sequenced_tx(...)` 已经能处理：
  - `transaction`
  - `tx_sequence_info_mapping`
  - `sequencer_info`
  - `transaction_acc_node`
- 但 `transaction_execution_info` 与 `state_change_set` 还需要单独补写

#### Step 6. Refresh Output Metadata

在 tail replay 结束后，刷新：

- `startup_info`
- `sequencer_info`
- `da_last_block_number`
- `da_block_submit_state`
- `proposer_last_block`

这部分应直接复用现有 replay/finalize 里已经成熟的：

- `refresh_output_metadata(...)`
- `verify_startup_sequencer_consistency(...)`

#### Step 7. Final Verification

验证：

1. `startup_info.state_root == expected_state_root`
2. `sequencer_info.last_order == to_order`
3. `sequencer_info.last_accumulator_info == canonical tx(to_order).sequence_info.tx_accumulator_info()`
4. `DA/proposer` 修复通过

如果开启 `--verify-root`：

- 必须对 canonical `to_order` 的 `execution_info.state_root` 做最终校验

### Implementation Plan

建议在 `IncrementalReplayer` 里新增一个独立入口：

```rust
pub async fn tail_replay_existing_output(
    &self,
    output_dir: &Path,
    from_order: Option<u64>,
    to_order: u64,
) -> Result<ReplayReport>
```

核心结构：

1. `load_output_stores(output_dir)`
2. `load_output_anchor()`
3. `copy_delta_history_range(from_order, to_order)`
4. `rebuild_accumulator_tail(...)`
5. `load_changesets_range(from_order, to_order, ...)`
6. `replay_changesets_batched(...)`
7. `refresh_output_metadata(...)`
8. `verify_startup_sequencer_consistency(...)`

### Report Design

沿用现有 `ReplayReport`，但新增几个字段更有帮助：

- `mode = "tail_replay_existing_output"`
- `base_order`
- `to_order`
- `delta_orders`
- `delta_transactions_copied`
- `delta_accumulator_nodes_written`

### Safety Properties

tail replay 必须保证：

1. 输出库的 `from_order` 必须连续衔接现有 `last_order + 1`
2. 只接受 canonical source range
3. 不允许通过 RPC 重新提交原 raw tx 作为追平方式
4. 最终 `state_root` 必须与 canonical `to_order` 对齐
5. 最终 `sequencer_info` 必须与 canonical `to_order` 对齐

### Test Plan

#### Unit / component

- base output `last_order` 推导 `from_order`
- delta accumulator append 后的 `AccumulatorInfo` 与 canonical 一致
- `save_tail_replayed_tx(...)` 能同时写入 tx / sequence / accumulator / execution / changeset

#### Integration

1. 先构造一份 replay output 到 `N`
2. 再对 `N+1..M` 跑 tail replay
3. 校验：
   - final `state_root`
   - final `sequencer_info`
   - output 库可启动

#### Mainnet rehearsal

对已有主网 replay output：

- 当前 `last_order = 241186740`
- 只追少量 delta range
- 验证最终 root 与主网一致

### Recommended Rollout

1. 先实现 `tail-replay` CLI
2. 在已有 replay output 上对一个很小的 delta range 演练
3. 验证 root 一致
4. 再决定是否把 replay output 正式追到切换窗口的最终 `to_order`

### Non-Goals

本工具不做：

- 从 raw tx 重新提交来补历史
- 重新导入 snapshot
- 重新复制全量 required CF
- 完整 archive 历史恢复
