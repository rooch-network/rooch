docs/dev-guide/statedb_prune_mixed_full.md# Rooch StateDB Pruning – **设计提案**

> **目的**：在不破坏 N 天可验证窗口的前提下，将磁盘与内存占用降到最低，并充分复用 `SMTree` / `StateDBStore` 已产生的增量日志。

*作者*: Rooch Dev · 2025-06-24

---

## 设计目标

| 编号 | 目标                                  | 说明                  |
|----|-------------------------------------|---------------------|
| G1 | ≥ N 天内任意 `state_root` 可完整验证         | 业务需求                |
| G2 | 历史状态数据裁剪                            | DFS 方案              |
| G3 | 增量状态数据裁剪                            | 引用计数方案              |
| G4 | 无停机、可断点续扫                           | Pruner 崩溃/重启不影响节点服务 |
| G5 | 最终删除 **cf_smt_nodes** 与 LRU-cache 项 | 真正腾出磁盘/内存           |

---

## 目录

1. [Part I – v1（DFS方案）](#part-i)
2. [Part II – v2（增量计数方案）](#part-ii)
3. [Part III – 结合与对比](#part-iii)

---

<a name="part-i"></a>
## Part I – v1（DFS 方案）

---

### 1 数据布局

| CF | Key | Value | 备注 |
|----|-----|-------|------|
| `cf_smt_nodes` | `node_hash` | `NodeBytes` | 无 `refcount` 前缀 |
| `cf_state_roots` | `timestamp_be` | `root_hash` | 根时间轴 |
| `cf_prune_meta` | `phase / dfs_cursor / stats` | 进度检查点 |
| （可选）`cf_reach_seen` | `node_hash` | 空 | Bloom 溢写去重 |

### 2 核心算法

#### 2.1 Live-Window 及 cutoff
```text
cutoff_ts  = now − Nd
live_roots = cf_state_roots(ts ≥ cutoff_ts)
expired    = cf_state_roots(ts < cutoff_ts)
```

#### 2.2 两阶段流程
```text
Phase-A BuildReach:
    并行 DFS(live_roots)  → ReachableSet

Phase-B SweepExpired:
    DFS(expired_root):
        node_hash ∉ ReachableSet → delete cf_smt_nodes & cache
```

#### 2.3 DFS 去重优化（Bloom + L0 索引）
```text
stack=[root]
while pop(h):
    if bloom.contains(h) || disk_index.contains(h):
        continue
    bloom.insert(h)
    node=db.get(h)
    if node.internal: push(children)
    else if leaf & collect_field_root: enqueue(field_root)
```

#### 2.4 并发策略
* 将 `live_roots` 切片给 N 个 worker 并行 DFS。  
* 共享只读 `NodeDBStore` 与原子 Bloom，I/O 近线性加速。

### 3 流程与检查点

| 字段 | 说明 |
|------|------|
| `phase` | `"BuildReach" / "SweepExpired"` |
| `dfs_cursor` | `(root_idx, stack_pos)` 扫描位置 |
| `bloom_snapshot` | 可选持久化 Bloom 位图 |

节点崩溃后根据 `phase` 与游标续扫；Bloom 可重建或加载快照。

### 4 复杂度与特性

| 指标 | 传统 DFS | **DFS 2.0** |
|------|-----------|-------------|
| 扫描 I/O | O(全库节点) | **O(活动节点 × logN)** |
| 重复遍历 | 大量 | Bloom 去重≈0 |
| 额外内存 | ReachableSet 全存 | Bloom 512 KB + 溢写 CF |
| Schema 变更 | 无 | 无 |

---

<a name="part-ii"></a>
## Part II – v2（增量计数方案）

---

### 1  关键观察

#### 1.1  `TreeUpdateBatch` 已产出 *StaleNodeIndex*

```rust
pub struct TreeUpdateBatch<K,V> {
    pub node_batch: BTreeMap<NodeKey, Node<K,V>>,        // 新节点
    pub stale_node_index_batch: BTreeSet<StaleNodeIndex> // 被覆盖的旧节点
}
```

*StateDBStore* 目前仅持久化 `node_batch` → RocksDB。**改进点**：同步落盘 `stale_node_index_batch`，即天然的 *增量 GC 日志*。

#### 1.2  `StateDBStore::cache` = `quick_cache::Cache<(root,key),ObjectState>`

* 删除一条 `(root,key)` 即可回收内存。
* 不需要额外索引 —— root 与 key 已在缓存键中。

---

### 2  数据布局调整

| CF / 表        | 键                         | 值                                              | 说明 |
|----------------|---------------------------|-------------------------------------------------|------|
| **cf_smt_nodes**     | `node_hash : [u8;32]`         | `NodeBytes`                                     | 与旧版一致 |
| **cf_smt_stale** (NEW) | `(stale_since_root, node_hash)` | *空字节* 或变长标记                             | 记录「何时被覆盖」 |
| **cf_state_roots**   | `timestamp_be`              | `root_hash`                                     | 全局 root 时间轴 |
| **cf_prune_meta**    | `meta_key`                  | `value`                                         | 进度 checkpoint |

> `cf_smt_stale` 使用 *prefix seek*：可按 `stale_since_root` 范围批量扫描。

---

### 3  Pruning 算法

#### 3.1 Live-Window & 数学定义

```
cutoff_root = first_root_with_ts( now – WINDOW )   // 等价于"最老活跃 root"
```

> 记 **活跃窗口** Roots = 所有 ≥ `cutoff_root` 的 root。

#### 3.2 条目可删除条件

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

#### 3.3 流程 (Mark → Sweep)

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

### 4  与 `StateDBStore` 的集成修改

#### 4.1 写入路径

`StateDBStore::update_nodes` 当前仅 `write_nodes(nodes)`。

```rust
// after tree_change_set generated
node_store.write_nodes(nodes)?;                           // ✔ 旧逻辑
node_store.write_stale_indices(stale_indices)?;           // ✔ 新增
```

*`stale_indices` 来自 `tree_change_set.stale_node_index_batch`*

#### 4.2 NodeDBStore 接口草案

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

#### 4.3 Pruner 线程简述

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

### 5  Checkpoint & 容错

* `cf_prune_meta` 保存：
    * `last_processed_stale_key`
    * `cutoff_root_at_checkpoint`
* 重启后按 `last_processed_stale_key` 续扫；若 `cutoff_root` 发生变化，重新计算 prefix 边界。

---

<a name="part-iii"></a>
## PartPart III  总结与对比

### 1. 整体运行流程

步骤说明  
1. **启动期清理（v1）**  
   * 仅针对 `ts < cutoff(N day)` 的根。  
   * 并行 DFS + Bloom 去重，删除不可达节点。  
   * 完成后写 `boot_cleanup_done=true`，停止 v1 线程。  
2. **在线增量 GC（v2）**  
   * 写路径 `ref++ / ref--`，落盘 `cf_smt_stale`。  
   * 后台定时按前缀扫描，`refcount==0` 即删。  
3. 两阶段共享 `cf_prune_meta`，重启可按 `phase / cursor` 精确续扫。  

### 2. 整体优势

| 维度 | v1-DFS 2.0 | v2-计数 | **混合方案(推荐)** |
|------|-----------|--------|-------------------|
| 历史磁盘立即减负 | ✅ 一次性 | — | **✅** |
| 写后增长成本 | O(活动×logN) | **O(#增量)** | **O(#增量)** |
| N 天验证窗口 | 保留 | 保留 | 保留 |
| RocksDB 变更 | 0 | 2 新 CF | 2 新 CF |
| 实施风险 | 低 | 中 | 分阶段、风险可控 |

### 3. 与 Ethereum-Geth 横向比较

| 指标 | Ethereum (Geth) | Rooch v1-DFS 2.0 | Rooch v2-计数 | **Rooch 混合** |
|------|----------------|------------------|--------------|---------------|
| 数据结构 | MPT | Jellyfish SMT | Jellyfish SMT | Jellyfish SMT |
| GC 机制 | `refcount + journal` | DFS 可达集 | `refcount + stale_index` | 启动 DFS + 持续计数 |
| 窗口 | 128 区块 ≈ 2h | N 天 | N 天 | N 天 |
| 扫描复杂度 | O(delta) | O(活动×logN) | O(stale×窗) | 一次 O(全) + 常态 O(delta) |
| Schema 变更 | journal 文件 | 无 | 2 新 CF | 2 新 CF |
| 部署路径 | 成熟 | 已有代码 | 约 1.3 kLoC | 先 v1 后 v2，平滑 |

---

### 结论

* **v1-DFS 2.0**：在不改 schema 的约束下，通过 Bloom 去重 + 并行 DFS，实现一次性快速清理历史。  
* **v2-增量计数**：引用计数 + `stale_index`，将 GC 复杂度降至 O(#增量)，适合长期在线。  
* **混合方案** 结合两者优点：  
  * 部署后先跑 v1，立即释放旧盘；  
  * 之后 v2 持续维持低增长；  
  * 全程零停机，重启安全，N 天窗口完整保留。  

升级步骤：  
1. 升级二进制（包含 v2 新 CF）；  
2. 冷启动 `refcount`（DFS N 天窗口或批量 1）；  
3. 运行 v1-DFS 2.0 清理；  
4. 自动切换至 v2 Mark-Sweep 进入常态；  
5. 监控 `rooch_pruner_phase`、`deleted_nodes_total`、`refcount_underflow` 等指标。

> **最终结论**：先以 v1-DFS 2.0 清理历史，立即腾出磁盘；随后切换 v2 持续增量 GC，实现零停机、N 天窗口完整可验证且长期低成本运行。
