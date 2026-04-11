# Mainnet Replay Selective-CF Redesign

## Summary

当前 `db state-prune replay` 的主要问题不是 replay 增量本身，而是输出库构建方式：

1. 先对 live DB 做整库 RocksDB checkpoint
2. 清空 output 的 `state_node`
3. 再把 snapshot 的 `state_node` 全量导入
4. 再 replay changesets

这条路径会把 `state_node` 这类本来准备被 snapshot 替换掉的数据也先复制一遍，导致输出空间需求接近 live DB 量级，不适合当前主网。

本方案建议把 replay 改成：

- **fresh output DB**
- **selective CF copy**
- **snapshot state import**
- **changeset replay**

也就是不再 checkpoint 整个 live DB，而是只复制续跑真正需要的 column families。

## Why Current Replay Is Too Heavy

当前实现见：

- [replay.rs](../../crates/rooch/src/commands/db/commands/state_prune/replay.rs)
- [incremental_replayer.rs](../../crates/rooch-pruner/src/state_prune/incremental_replayer.rs)

关键路径：

1. `prepare_output_store(output_dir)`
2. `clear_state_nodes(output_store)`
3. `import_snapshot_nodes(snapshot_store, output_store, ...)`
4. `replay_changesets_batched(...)`

其中最重的是：

- [incremental_replayer.rs](../../crates/rooch-pruner/src/state_prune/incremental_replayer.rs)

```rust
let checkpoint = Checkpoint::new(rocks_db)?;
checkpoint.create_checkpoint(output_dir)?;
```

对于主网当前数据，这一步会先复制接近完整 live DB 的输出库，然后才删除旧 `state_node`。

## Mainnet Size Reference

基于 `rooch db rocksdb-stats --db-path /data/.rooch/main/roochdb/store` 的主网统计：

- `state_node`: `~10.3T`
- `transaction_acc_node`: `~1.18T`
- `state_change_set`: `~687G`
- `transaction`: `~332G`
- `event`: `~167G`
- `transaction_execution_info`: `~51G`
- `tx_sequence_info_mapping`: `~17G`

当前 reset 后 snapshot：

- node count: `940,198,963`
- snapshot dir: `~724G`

这说明当前 replay 最大的纯浪费是：

- 先复制 `~10.3T state_node`
- 紧接着再删掉它
- 再导入 `~724G` 的 snapshot state

## Redesign Goal

把 replay 从“整库 checkpoint 再替换 state”改成：

1. 创建 fresh output DB
2. 只复制续跑需要的 CF
3. 导入 snapshot 的 `state_node`
4. replay changesets
5. 重写 `startup_info` / `sequencer_info`
6. 可选做 history prune

目标：

- 避免复制 `state_node`
- 避免复制 `state_node_recycle`
- 明显降低临时空间需求
- 保持 replay 到最新后可继续运行

## Proposed Output Construction

### Step 1. Create Fresh DB

不再调用 `Checkpoint::create_checkpoint(output_dir)`，改成：

- 创建一个空的 output RocksDB
- 初始化需要的 column families

可复用当前 `load_output_stores()` 的列族集合逻辑，但输出目录应从空目录开始构建。

### Step 2. Copy Required CFs Only

当前仓库已有整 CF 复制工具：

- [cp_cf.rs](../../crates/rooch/src/commands/db/commands/cp_cf.rs)

新 replay 不需要整库 checkpoint，但可以复用“按 CF 迭代复制”的思路。

建议把 CF 分为三类。

#### A. 必须复制

这些是“续跑最小集”：

- `transaction_acc_node`
- `transaction`
- `transaction_execution_info`
- `tx_sequence_info_mapping`
- `config_genesis`
- `da_last_block_number`
- `da_block_submit_state`
- `proposer_last_block`

说明：

- `transaction_acc_node` 是 sequencer continuation 的硬依赖
- `transaction` / `tx_sequence_info_mapping` / `transaction_execution_info` 是按 `to_order` 重建 `sequencer_info` 和后续正常读取所需的最小交易面
- `config_genesis`、DA、proposer 元数据体量小，但启动需要

#### B. 由 replay/build 重写，不复制旧值

- `config_startup_info`
- `meta_sequencer_info`

说明：

- `startup_info` 应由新的 final state root 重写
- `sequencer_info` 应由 `to_order` 对应 tx 的 accumulator info 重写

#### C. 第一版先不复制

- `state_node`
- `state_node_recycle`
- `state_change_set`
- `event`
- `event_handle`

说明：

- `state_node` 由 snapshot 导入，不应复制旧值
- `state_change_set` / `event` / `event_handle` 不是继续跑的最小硬依赖
- 这部分可作为第二阶段“历史兼容集”再加

## Expected Size After Redesign

如果按最小续跑集构建 output，大致量级是：

- 新 `state_node`: `~0.7T ~ 1.0T`
- `transaction_acc_node`: `~1.18T`
- `transaction`: `~0.33T`
- `transaction_execution_info`: `~0.05T`
- `tx_sequence_info_mapping`: `~0.02T`
- 其他元数据：可忽略

所以结果库本体大约：

- `~2.3T ~ 2.6T`

再加 RocksDB 临时写放大，临时空间需求保守按：

- `~3T ~ 3.5T`

这比当前“接近整库 checkpoint”的 TB 级更重方案明显更合理，也更接近当前 `5T` 盘可承受范围。

## Expected Runtime

当前 replay 失败前在 `store.tmp` 上的增长很快，说明 checkpoint/import 阶段是主要时间消耗。

如果不再复制 `state_node ~10.3T`，只复制最小续跑集并导入 snapshot state，预期总工作量会下降到：

- 复制 `~1.5T ~ 1.7T`
- 写入 `~0.7T ~ 1.0T` snapshot state
- replay 增量 changesets

粗估窗口：

- `12 ~ 24 小时`
- 保守按 `1 ~ 2 天`

## Required Code Changes

### 1. Replace `prepare_output_store()`

当前：

- 从 live DB 做整库 checkpoint

改为：

- `prepare_fresh_output_store()`
- 初始化空 RocksDB + all required CFs

### 2. Add selective CF copy stage

新增一个阶段，例如：

- `copy_required_cfs(&self, output_store_dir: &Path, required_cfs: &[&str])`

要求：

- 只复制配置中列出的 CF
- 支持进度日志
- 支持 dry-run / size estimate

### 3. Keep `clear_state_nodes()` only for copied DB variants

在 fresh DB 路径里：

- `state_node` 默认是空的
- 可以跳过 `clear_state_nodes()`

如果后面保留兼容旧模式，可根据构建模式判断是否调用。

### 4. Preserve current snapshot import path

`import_snapshot_nodes()` 本身仍可复用。

### 5. Keep `trim_output_store()` but clarify semantics

当前 `trim_output_store()` 只会删除 `to_order` 之后的数据。

如果 `to_order == latest_order`：

- 不会裁掉旧历史

因此后续如果想进一步减小结果库，应叠加：

- `history_prune`

但这不是本方案第一阶段的阻塞项。

## Rollout Plan

### Phase 1

- fresh DB + minimal CF copy
- import snapshot state
- replay to latest
- 验证库能启动并继续运行

### Phase 2

- 视需求追加 `event` / `event_handle`
- 评估是否保留 `state_change_set`
- 与 `history_prune` 结合进一步压缩历史

## Non-Goals

本方案第一阶段不解决：

- 完整历史查询兼容
- `event` / `event_handle` 的完整保留
- 更深度的 archive 能力保真

第一目标只有一个：

- **在合理空间内跑通 replay，并产出可继续运行的更小主库**

## Recommendation

如果继续走 replay 路线，优先级最高的改造应是：

- **去掉整库 checkpoint**
- **改成 selective CF copy**

这比先加更大临时盘更值得做，因为它直接解决了当前实现最明显的空间浪费。
