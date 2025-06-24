# Rooch StateDB Pruning Specification

## 1  Sparse-Merkle-Tree (SMT) – Detailed Design

### 1.1 Core node types

| Variant | Tag | Stored fields | Role |
|---------|-----|---------------|------|
| **NullNode** (`Node::Null`) | `0x00` | – | 256-bit placeholder; hash is the compile-time constant `SPARSE_MERKLE_PLACEHOLDER_HASH` |
| **InternalNode** (`Node::Internal`) | `0x01` | `Children<u8, Child>` (16 slots) | 4-bit branch; each **Child** = `{ hash: H256, is_leaf: bool }` |
| **LeafNode** (`Node::Leaf`) | `0x02` | `key_hash` (32 B), `value_hash` (32 B), `value` (Vec<u8>) | terminal (key,value) pair |

Hash derivation

```
leaf_hash     = Keccak256(0x00 │ key_hash │ value_hash)
internal_hash = Keccak256(0x01 │ child₀ │ … │ child₁₅)
```

### 1.2 Insertion path

```
SMTree::puts(prev_root, UpdateSet)
   for each (k, maybe_v) in UpdateSet
       • hash key  → path
       • descend from prev_root
       • create Leaf or Null (delete) at divergence
       • rebuild ancestors (copy-on-write)
→ TreeChangeSet { state_root, node_batch }
```

`node_batch : BTreeMap<NodeHash, Vec<u8>>` is written to RocksDB (`cf_smt_nodes`) in one atomic batch.

### 1.3 Query path

*Single key* — `get_with_proof()` walks from root, obtains value + sibling list.  
*Range* — `SMTIterator` (DFS) yields each Leaf in **hash order**.

### 1.4 Update / Delete

Identical algorithm; `Option<V>::None` means delete (leaf → Null).  
Only nodes on the modified path receive new hashes.

### 1.5 Multi-version storage

*   Old and new state-roots **share every byte-identical subtree**.
*   Storage growth per commit ≈ `touched_keys × (path_height+1)` nodes.
*   Historical proofs stay valid because their hashes never disappear until a disk-GC job removes unreachable nodes.

---

## 2  Object System & Hierarchical Model

### 2.1 Core structures

```rust
struct ObjectMeta {
    id:          ObjectID,
    owner:       AccountAddress,
    state_root:  H256,   // field-SMT root
    size:        u64,
    flags:       u8,     // shared, frozen, embedded …
    created_at:  u64,
    updated_at:  u64,
}

struct ObjectState {
    metadata: ObjectMeta,
    value:    Vec<u8>,   // BCS-encoded Move value T
}
```

`ObjectID.child_id(FieldKey)` deterministically derives a field-object id.

### 2.2 Two-layer storage layout

```
GLOBAL-SMT  (root stored in block header)
 └─ Leaf  = ObjectState<GlobalObj>
        └─ metadata.state_root ──► FIELD-SMT of that object
               └─ Leaf = ObjectState<FieldObj>
```

First layer = global objects;  
second layer = per-object field trees (same SMT code).

### 2.3  RuntimeObject (MoveVM)

`runtime_object.rs` keeps two `GlobalValue`s (`value`, `pointer`) plus metadata and child map, enforcing:

* borrow / move rules
* size & timestamp accounting
* `ObjectChange` generation for the executor.

---

## 3  Identifying Orphaned Nodes

```
Reachable = { every NodeHash reachable from ANY state-root with
              commit_ts ≥ now − 30 days }

Orphan    = NodeHash reachable ONLY through roots with
              commit_ts <  now − 30 days
```

Because NodeHash is the content hash, presence in `Reachable` is sufficient liveness proof; no explicit ref-counters required.

---

## 4  Building the Global Reachable-StateRoot Set

### 4.1 Live-root enumeration

```
cutoff = now − 30d
live_roots = cf_state_roots
             .filter(ts ≥ cutoff)
             .map(value → H256)
```

### 4.2 Algorithm (two layers)

```
Reachable = ∅                // HashSet<NodeHash>
Q_field   = ∅                // VecDeque<H256>

for g_root in live_roots {           // 1️⃣ layer-1
    dfs(g_root):
        Reachable += node_hash
        if Node::Leaf => Q_field += obj_state.metadata.state_root
}

while let Some(f_root) = Q_field.pop_front() {   // 2️⃣ layer-2
    dfs(f_root):
        Reachable += node_hash
}
```

`dfs(root)`

```
stack = [root]
while let Some(h) = stack.pop() {
    raw = node_store.raw_node_bytes(h)?
    Reachable.insert(h)
    for child in node_store.children_hashes(&raw)? {
        stack.push(child)
    }
}
```

*   Check-point cursor & queue index every **SCAN_BATCH (10 000)** nodes in `cf_prune_meta` → restart resumes instantly.
*   RAM cap: first 4 M hashes (≈128 MB) kept in memory, overflow spills to `cf_reach_spill` column family.

---


### 4.3  APIs we must use

#### 4.3.1 `NodeReader` (already auto-impl for `NodeDBStore`)

```rust
trait NodeReader {
    fn get(&self, hash: &H256) -> Result<Option<Vec<u8>>>;   // raw bytes

    // ↑ is auto-lifted to
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node<K,V>>> {
        self.get(&(*node_key).into())?
            .map(|bytes| Node::<K,V>::decode(&bytes))
            .transpose()
    }
}
```

#### 4.3.2 `InternalNode::children()`

```rust
impl<K,V> InternalNode<K,V> {
    pub fn children(&self) -> impl Iterator<Item = Child> + '_
}
Child { hash: NodeKey /*H256*/, is_leaf: bool }
```

#### 4.3.3 `SMTIterator` (for enumerating leaves & obtaining `ObjectState`)

```rust
let iter = SMTree::<FieldKey,ObjectState,_>::iter(root_hash, None)?;
for (_k, obj_state) in iter { … }
```

---

### 4.4  DFS routine that **really** walks an SMT

Below is exactly what runs inside the background task; it shows where `NodeReader`
and `children()` are used.

```rust
/// depth-first walk that pushes EVERY visited hash into `reachable`.
///
/// • `collect_field_roots` – if true we are scanning the global SMT
///                           and must extract each object’s field-SMT root
///                           from the leaf payload.
///   otherwise we are in a field-SMT and do *not* recurse further.
fn dfs_collect(
    root: H256,
    collect_field_roots: bool,
    node_reader: &impl NodeReader,
    reachable: &mut HashSet<H256>,
    field_queue: &mut VecDeque<H256>,      // out-param
) -> Result<()> {
    // classic stack DFS – avoids recursion depth
    let mut stack = vec![root];

    while let Some(hash) = stack.pop() {
        if !reachable.insert(hash) {
            continue;                      // already seen via another path
        }

        // ---- 1. fetch & decode the node via NodeReader ----
        let raw = node_reader
            .get(&hash)?
            .ok_or_else(|| anyhow::format_err!("missing node {hash:?}"))?;
        let node: Node<FieldKey, ObjectState> = Node::decode(&raw)?;

        match node {
            Node::Internal(inode) => {
                // ---- 2. iterate children() ----
                for child in inode.children() {
                    if !child.is_placeholder() {
                        stack.push(child.hash);       // NodeKey == H256 alias
                    }
                }
            }

            Node::Leaf(leaf) => {
                // ---- 3. process leaf ----
                if collect_field_roots {
                    let field_root = leaf.value().metadata.state_root();
                    field_queue.push_back(field_root);
                }
            }

            Node::Null => { /* nothing */ }
        }
    }
    Ok(())
}
```

Key points
* `NodeReader::get()` delivers *raw* bytes; we decode **once** per node.
* `children()` gives us every real child **hash**; placeholders are skipped.
* Every visited hash (leaf + internal) goes into `reachable`.

---

### 4.5  Building the full set – complete algorithm

```rust
fn build_reachable(
    live_roots: &[H256],             // 30-day window
    node_reader: &impl NodeReader,
) -> Result<HashSet<H256>> {
    let mut reachable = HashSet::<H256>::new();
    let mut field_queue = VecDeque::<H256>::new();

    // ---------- layer-1  (global SMT) ----------
    for &g_root in live_roots {
        dfs_collect(
            g_root,
            true,                    // collect field roots
            node_reader,
            &mut reachable,
            &mut field_queue,
        )?;
    }

    // ---------- layer-2  (field SMTs) ----------
    while let Some(f_root) = field_queue.pop_front() {
        dfs_collect(
            f_root,
            false,                   // no deeper layer today
            node_reader,
            &mut reachable,
            &mut VecDeque::new(),    // unused
        )?;
    }
    Ok(reachable)
}
```

*Time complexity* – `O(#distinct nodes in 30 day window)`.  
*Memory* – 32 bytes per unique hash, capped by RAM spill strategy (e.g. 4 M entries).

---


## 5  State Pruning Procedure

### 5.1 Build –and maintain– the **prune set**

For each **expired** global root (`ts < cutoff`):

```
prune_pairs = ∅
Q_field     = ∅

dfs_prune(global_root):
    if Leaf.hash ∉ Reachable
         prune_pairs += (global_root, Leaf.hash)
    Q_field += obj_state.metadata.state_root

for field_root in Q_field
    dfs_prune(field_root) same rule
```

Each `(state_root, leaf_hash)` identifies one cache entry.

### 5.2 Execute prune

```rust
for (root, hash) in prune_pairs {
    state_store.cache
        .remove(&(root, FieldKey::from_hash(hash)));
}
```



### 5.3 Scanning one SMT with **StateDBStore::iter**

```rust
/// Scan every (FieldKey, ObjectState) under `state_root` using
/// `StateDBStore::iter`.  Push prune-candidates into `out` and
/// if `collect_field_roots == true` queue the object’s field-SMT
/// root into `field_q`.
fn scan_with_statedb_iter(
    state_root: H256,
    collect_field_roots: bool,
    state_store: &StateDBStore,
    reachable: &HashSet<H256>,
    out: &mut Vec<(H256,H256)>,          // (state_root, leaf_hash)
    field_q: &mut VecDeque<H256>,        // queue for layer-2
) -> anyhow::Result<()> {

    let mut iter = state_store.iter(state_root, None)?;

    while let Some(res) = iter.next() {
        let (k, v) = res?;                       // k = FieldKey, v = ObjectState

        // --- 1. rebuild merkle hash of key -------------------
        let kh = k.merkle_hash();
        let vh = v.merkle_hash();                // via EncodeToObject impl
        //let leaf_h = leaf_node_hash(kh, vh);

        // --- 2. liveness test ---------------------------
        if !reachable.contains(&kh) {
            out.push((state_root, k));
        }

        // --- 3. enqueue second-layer root ---------------
        if collect_field_roots {
            field_q.push_back(v.metadata.state_root());
        }
    }
    Ok(())
}
```

*`StateDBStore::iter()` already pages through the underlying SMT;
we merely reconstruct the leaf hash locally.*

---

### 5.4 Building the prune-set for **one expired root**

```rust
fn build_prune_for_root(
    expired_root: H256,
    state_store: &StateDBStore,
    reachable: &HashSet<H256>,
    prune_set: &mut Vec<(H256,H256)>,
) -> anyhow::Result<()> {

    let mut field_q = VecDeque::new();

    // ---------- LAYER-1  ----------
    scan_with_statedb_iter(
        expired_root,
        true,                   // collect field roots
        state_store,
        reachable,
        prune_set,
        &mut field_q)?;

    // ---------- LAYER-2  ----------
    while let Some(f_root) = field_q.pop_front() {
        scan_with_statedb_iter(
            f_root,
            false,              // no deeper layer today
            state_store,
            reachable,
            prune_set,
            &mut VecDeque::new())?;
    }
    Ok(())
}
```

---

### 5.5 Full prune-set builder (all expired roots)

```rust
fn build_prune_set(
    expired_roots: Vec<H256>,
    state_store: &StateDBStore,
    reachable: &HashSet<H256>,
) -> anyhow::Result<Vec<(H256,H256)>> {

    let mut prune = Vec::new();
    for r in expired_roots {
        build_prune_for_root(r, state_store, reachable, &mut prune)?;
    }
    Ok(prune)
}
```

---

### 5.6 Evicting entries from quick-cache

```rust
fn evict_quick_cache(
    state_store: &StateDBStore,
    prune_set: &[(H256,H256)],
    batch: usize,
) {
    for chunk in prune_set.chunks(batch) {
        for &(root, field_key) in chunk {
            state_store
                .cache
                .remove(&(root, field_key));
        }
    }
}
```

*Node bytes in `cf_smt_nodes` remain untouched, so historical state-roots and
proofs are unaffected.*

---


#### What is *not* removed

| Layer | Action |
|-------|--------|
| NodeDBStore rows (`hash → NodeBytes`) | **removed** – proofs for old roots still work |
| quick_cache / LRU | **evicted** |
| NodeHash in smt | **remain**, The possible impact is that when traversing from smt, the data corresponding to some node hashes cannot be obtained |

---

## 6. Persistence & Restart Continuation
- Record the state pruning of the most recently executed tx_order (corresponding to stateroot), and continue the original state cleanup from this tx_order after restart


---

## 7  Life-cycle in Code

```rust
// server.rs
StateDBPruner::spawn(db.clone(),
                     node_store.clone(),
                     state_store.clone());

// after each block commit
cf_state_roots.put(timestamp_be, new_global_root);
```

*`StateDBPruner`* (see provided skeleton)

1. **tick()** every `interval_s` (default 1 h)
2. if `phase==BuildReach` → run §4 until checkpoint or finish
3. if `phase==SweepRoot`  → run §5 until checkpoint or finish one root
4. flush `delete_queue` to quick_cache; write meta

Config (toml)

```toml
[statedb_pruner]
enable            = true
age_threshold_s   = 2592000   # 30 days
scan_batch        = 10000
delete_batch      = 5000
interval_s        = 3600
ram_reachable_cap = 4000000   # spill after 4M hashes
```

---

### Result

* 30-day live window fully reachable.
* Quick-cache memory footprint bounded.
* Historical state-roots stay verifiable.
* Pruner can restart at any moment and continue exactly where it stopped.
---------
