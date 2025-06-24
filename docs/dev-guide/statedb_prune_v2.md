# Rooch StateDB Pruning – **v2 设计提案**

> **目的**：在不破坏 30 天可验证窗口的前提下，将磁盘与内存占用降到最低，并充分复用 `SMTree` / `StateDBStore` 已产生的增量日志，避免全量 DFS。

*作者*: Rooch DevRel · 2025-06-24

---

## 0  设计目标

| 编号 | 目标 | 说明 |
|------|------|------|
| G1 | ≥ 30 天内任意 `state_root` 可完整验证 | 业务需求，与旧设计一致 |
| G2 | 无停机、可断点续扫 | Pruner 崩溃/重启不影响节点服务 |
| G3 | **O(#增量节点)** 开销 | 摒弃旧 DFS=O( #全体节点 ) 方案 |
| G4 | 最终删除 **cf_smt_nodes** 与 LRU-cache 项 | 真正腾出磁盘/内存 |

---

## 1  关键观察

### 1.1  `TreeUpdateBatch` 已产出 *StaleNodeIndex*

```rust
pub struct TreeUpdateBatch<K,V> {
    pub node_batch: BTreeMap<NodeKey, Node<K,V>>,        // 新节点
    pub stale_node_index_batch: BTreeSet<StaleNodeIndex> // 被覆盖的旧节点
}
```

*StateDBStore* 目前仅持久化 `node_batch` → RocksDB。**改进点**：同步落盘 `stale_node_index_batch`，即天然的 *增量 GC 日志*。

### 1.2  `StateDBStore::cache` = `quick_cache::Cache<(root,key),ObjectState>`

* 删除一条 `(root,key)` 即可回收内存。 
* 不需要额外索引 —— root 与 key 已在缓存键中。

---

## 2  数据布局调整

| CF / 表        | 键                         | 值                                              | 说明 |
|----------------|---------------------------|-------------------------------------------------|------|
| **cf_smt_nodes**     | `node_hash : [u8;32]`         | `NodeBytes`                                     | 与旧版一致 |
| **cf_smt_stale** (NEW) | `(stale_since_root, node_hash)` | *空字节* 或变长标记                             | 记录「何时被覆盖」 |
| **cf_state_roots**   | `timestamp_be`              | `root_hash`                                     | 全局 root 时间轴 |
| **cf_prune_meta**    | `meta_key`                  | `value`                                         | 进度 checkpoint |

> `cf_smt_stale` 使用 *prefix seek*：可按 `stale_since_root` 范围批量扫描。

---

## 3  Pruning 算法

### 3.1 Live-Window & 数学定义

```
cutoff_root = first_root_with_ts( now – WINDOW )   // 等价于"最老活跃 root"
```

> 记 **活跃窗口** Roots = 所有 ≥ `cutoff_root` 的 root。

### 3.2 条目可删除条件

给定 **StaleNodeIndex**︰ `(stale_since_root, node_hash)`

```
If   stale_since_root  <  cutoff_root        // ① 老版本产生的 stale 节点
and  node_hash NOT visited by任何活跃 root   // ② 无活跃引用
then delete node_hash (cf_smt_nodes) & 相关 cache 键
```

**关键：② 的判定不再 DFS！**

* 我们在 *写入阶段* 增量维护 `node_refcount`：
  * 新节点写入时 `refcount = 1`。
  * 若相同 `node_hash` 已存在 ➜ `refcount += 1`。
  * 对于每个 `stale_node_hash` ➜ `refcount -= 1`。
* 存储位置：
  * 轻量方案 —— 将 `u32 refcount` 放入 `cf_smt_nodes` 的 value 前缀（8 字节）; 不影响哈希 (hash 取原始 NodeBytes)。
  * 或独立 CF `cf_node_rc`。

如今 ② 转化成 `refcount==0` 即无活跃引用，O(1) 判定。

### 3.3 流程 (Mark → Sweep)

```text
1. Mark Phase (增量完成于写逻辑)
   • 每块提交时处理 TreeUpdateBatch：
     - node_batch   : new_node.ref++
     - stale_batch  : old_node.ref--

2. Sweep Phase (后台线程)
   • Scan cf_smt_stale by prefix < cutoff_root
   • 对每条:
       if refcount(node_hash)==0 {
           rocksdb.delete(cf_smt_nodes, node_hash)
           rocksdb.delete(cf_smt_stale, (stale_root,node_hash))
       }
   • 同时清除 quick_cache 前缀匹配 node_hash
```

**复杂度**
* Mark 已并入正常写路径，Δ成本 ≈ `touch_nodes`。
* Sweep 只遍历 *待删除日志*，上限 `touch_nodes × WINDOW_DAYS`，远小于全量 DFS。

---

## 4  与 `StateDBStore` 的集成修改

### 4.1 写入路径

`StateDBStore::update_nodes` 当前仅 `write_nodes(nodes)`。

```rust
// after tree_change_set generated
node_store.write_nodes(nodes)?;                           // ✔ 旧逻辑
node_store.write_stale_indices(stale_indices)?;           // ✔ 新增
```

*`stale_indices` 来自 `tree_change_set.stale_node_index_batch`*

### 4.2 NodeDBStore 接口草案

```rust
impl NodeDBStore {
    pub fn write_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> { … }

    pub fn write_stale_indices(&self, indices: BTreeSet<StaleNodeIndex>) -> Result<()> {
        let mut batch = WriteBatch::default();
        for idx in indices {
            let key = idx.encode();        // (stale_root | node_hash)
            batch.put_cf(cf_smt_stale, key, EMPTY);
            self.dec_refcount(&idx.node_hash, &mut batch)?;
        }
        self.db.write(batch)
    }

    fn inc_refcount(&self, h: &H256, b: &mut WriteBatch) -> Result<()> { … }
    fn dec_refcount(&self, h: &H256, b: &mut WriteBatch) -> Result<()> { … }
}
```

### 4.3 Pruner 线程简述

```rust
loop every PRUNE_INTERVAL {
    let cutoff = calc_cutoff_root();
    let prefix = upper_bound(cutoff);
    for (k,_) in cf_smt_stale.prefix_iter(..prefix).take(BATCH) {
        let (_, node_hash) = decode(k);
        if refcount(node_hash)==0 {
            db.delete(cf_smt_nodes, node_hash);
            db.delete(cf_smt_stale, k);
            cache.remove_prefix(node_hash);
        }
    }
    checkpoint();
}
```

*确保删 node 后 `refcount` 条目自动 drop (无 key)*

---

## 5  Checkpoint & 容错

* `cf_prune_meta` 保存：
  * `last_processed_stale_key`
  * `cutoff_root_at_checkpoint`
* 重启后按 `last_processed_stale_key` 续扫；若 `cutoff_root` 发生变化，重新计算 prefix 边界。

---

## 6  优势与对比

| 指标          | 旧 DFS 方案 | **v2 增量方案** |
|---------------|------------|-----------------|
| Scan IO       | O(#所有节点) | **O(#stale_nodes)** |
| 额外内存      | 4 M hash spill | `refcount` 嵌入节点 (固定) |
| 删除正确性    | 需 DFS + 可达集 | 由 `refcount==0` 保证 |
| 实施难度      | 中等 | 略高（需要改写 NodeStore） |

---

## 7  兼容性与迁移

1. **线上迁移**：
   * 部署 v2 节点，以 *只读* 模式重放区块，补齐 `refcount`；
   * 切换主节点后开始正常写入。
2. **碾压式**：
   * 直接在 RocksDB 新 CF 写 `refcount`，旧节点默认 1，写入增量逐渐校正。

---

## 8  总结

* 该方案充分利用 `SMTree` 已经生成的 *stale-node* 日志，将 GC 复杂度从 "全网 DFS" 降为 "增量日志清理"。 
* 通过轻量 `refcount` 达到 *一遍扫描 + O(1) 判定* 删除条件，既保证 30 天窗口，又真正减少 10 TB/日的磁盘消耗。
* 需要修改：
  * `NodeDBStore` 增加引用计数 & stale index CF。  
  * `StateDBStore::update_nodes` 落盘 `stale_node_index_batch`。
  * 后台 `Pruner` 实现 Mark(count) & Sweep(delete)。

如接受该提案，可在后续版本实现并替换现有 `statedb_prune_specification.md`。 

## 9  开发工作量估算与冷启动策略

### 9.1 人力 / 时间预算

| 模块 | 主要改动 | 代码量估计 | 预计投入 |
|------|----------|-----------|-----------|
| RocksDB CF 调整 | 建立 `cf_smt_stale`、可选 `refcount` 前缀 | <50 行 | 0.5 d |
| `NodeDBStore` | `write_stale_indices`、`inc_/dec_refcount`、查询接口 | ~300 行 | 3–4 d |
| `StateDBStore` | 调用新增接口、透传 batch | ~80 行 | 1 d |
| Pruner 线程 | 扫描前缀、批删、断点续扫 | ~400 行 | 4–5 d |
| 测试 & Bench | 单元 / 集成 / 压力测试 | ~400 行 | 5 d |
| 文档 & 运维脚本 | 配置、启动参数、监控 | ~50 行 | 1 d |

**合计**：约 1k–1.3k 行 Rust，合 10–15 人/日（3–4 人·周含测试）。

### 9.2 不重放区块的初始化方案

v2 依赖 *正确的* `refcount`。若直接上线旧数据，所有节点计数缺失→Pruner 可能误删。需要一次性**冷启动索引**：

1. **活跃窗口扫描（推荐）**
   * 遍历 30 天内全部 `state_root`，DFS 标记可达节点；
   * 对每个命中的 `node_hash` 写 `refcount += 1`；
   * 扫描成本与旧 DFS 方案相当，但只需一次。
2. **默认 refcount=1**
   * 为所有历史节点批量写 `refcount = 1`；
   * 之后只做 `--` 操作；
   * 结果：旧节点永远 ≥1，几乎无法被回收 → **不建议**。

冷启动可分批后台执行（10 M 节点/批次）以免阻塞服务，完成后再开启 `Sweep` 阶段。 

---

## 10  Ethereum 方案与 Rooch 的对比

### 10.1 Ethereum (Geth) 状态裁剪概览

1. **Trie 类型**：Merkle Patricia Trie (MPT)。
2. **窗口**：默认仅保留最近 ~128 区块的可回滚状态；`--gcmode=archive` 关闭裁剪。
3. **引用计数 + Journal**
   * 区块执行生成 *TrieDifference* (`new_nodes`, `old_nodes`)。
   * `new_nodes` → `ref++` ，`old_nodes` → `ref--`；变更写入 *journal 文件*。
   * 仅当 journal 条目超过保护窗口且 `ref==0` 时才真正删除 LevelDB 中的节点键。
4. **后台 GC**：批量 `DeleteRange`；同时维护 snapshot（压缩扁平表）。

> 归纳：Geth 采用 *增量 refcount* 与 *短窗口*，与本文 v2 的方向一致。

### 10.2 三方案对比

| 维度 | Ethereum (Geth 默认) | Rooch **v1** (DFS) | Rooch **v2** (本提案) |
|------|----------------------|--------------------|-----------------------|
| 数据结构 | MPT | Jellyfish SMT | Jellyfish SMT |
| 保留窗口 | 128 区块 (~2h) | 30 天 | 30 天 (可调) |
| 可删判定 | `ref==0` & 超窗 | DFS + Reachable | `ref==0` & `stale_since_root < cutoff` |
| GC 复杂度 | O(#delta) | **O(#全部节点)** | O(#stale_nodes) |
| 初始化 | 计数随链生成 | 无需 | 需一次窗口扫描写 `ref` |
| archive 支持 | `--gcmode=archive` | 关闭 pruner | 同上，可停 Sweep |
| 实现成熟度 | 已多年生产验证 | 已实现草稿 | 待开发 (本文件) |

**结论**：v2 方案保留了 Ethereum 成熟经验中的「增量计数 + 回滚窗口」思想，同时兼顾 Rooch 的 30 天可验证需求，扫描复杂度显著优于 v1。 