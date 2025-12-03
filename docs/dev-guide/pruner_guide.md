# Rooch Pruner Design & Background

> Status: Runtime pruning is archived, but the design rationale still matters whenever we evaluate future online GC work. Storage reclamation in production is performed with the offline Mark–Sweep flow documented in `offline_gc_guide.md`.

## Goals the Pruner Tried to Meet

| ID | Objective | Notes |
|----|-----------|-------|
| G1 | Retain ≥ N days of verifiable state roots | Business requirement for audits and fast sync. |
| G2 | Reclaim historical disk space | V1 DFS approach. |
| G3 | Keep day‑to‑day growth bounded | V2 incremental counting approach. |
| G4 | Run without downtime | Needed to coexist with the sequencer. |
| G5 | Remove data from both RocksDB and caches | Otherwise reclaiming space is just theoretical. |

These goals led to two complementary designs that we attempted before moving to the offline GC baseline.

## Legacy Runtime Architecture (What Shipped)

1. **Boot cleanup (V1)** – Single-shot DFS from genesis roots to remove obviously unreachable nodes.
2. **SweepExpired (V1.5)** – Batch process that walked historical roots, used a Bloom filter to skip reachable hashes, and deleted the rest.
3. **IncrementalSweep (V2)** – Continuous loop that read the stale index, relied on refcounts, and deleted nodes in small batches.

The three phases (`BuildReach → SweepExpired → Incremental`) exported Prometheus metrics such as `pruner_current_phase`, `pruner_sweep_nodes_deleted`, and `pruner_reachable_nodes_scanned`, and exposed many tuning knobs (`scan_batch`, `bloom_bits`, `protection_orders`, etc.).

### Pain Points Observed in Production

| Issue | Details |
|-------|---------|
| Concurrency risk | The pruner shared the same column families as live writes, so any stale/refcount bug could delete data still referenced by the sequencer. |
| Operational opacity | Operators had to interpret three asynchronous phases plus Bloom stats to know whether deletion was safe. |
| Complex bug surface | Investigations (for example the 2025 missing-node incident) showed that stale indices, timestamp cutoffs, and Bloom reuse interacted in surprising ways. |
| Recoverability gaps | Even with the recycle bin, restoring a mis-deleted node meant replaying WALs and hoping the state machine could be repaired without downtime. |
| Costly proof obligations | We never had a clean, auditable statement like “mark set = reachable set” because the daemon ran concurrently with writes. |

## StateDB Pruning Proposals (What We Designed Next)

To make pruning sustainable without replacing the SMT, we drafted two complementary designs.

### Part I – DFS 2.0 (Historical Cleanup)

- Keep all state roots in `cf_state_roots`, split them into “live window” and “expired”.
- Run a parallel DFS over all live roots to build a reachable set (with Bloom deduplication and optional spill-to-CF).
- Sweep expired roots: for each node reached from an expired root, delete it if it is not in the reachable set.
- Maintain checkpoints in `cf_prune_meta` (`phase`, `dfs_cursor`, optional Bloom snapshot) to resume after crashes.

This pass is ideal for bootstrapping or when disk has already ballooned, but it is O(all nodes) and still needs downtime for the heaviest workloads.

### Part II – Incremental Counting (Ongoing Cleanup)

- Persist the stale index produced by `TreeUpdateBatch` so every “node overwritten” event is recorded.
- Extend the write path to keep refcounts (either as a prefix in `cf_smt_nodes` or via a dedicated CF).
- Background sweep scans stale entries whose `tx_order` is older than the protection window and deletes those whose refcount reached zero.
- Deletes also purge cache entries keyed by `(root, key)` to free memory.

This mode keeps daily growth bounded but depends on perfect refcount accounting. Any missed increment or decrement leads to corruption, which is exactly what we experienced.

### Combined View

| Aspect | DFS 2.0 | Incremental Counting | Combined Strategy |
|--------|---------|----------------------|------------------|
| Purpose | Clean historical bulk | Keep growth low | Run DFS once, rely on incremental afterward |
| Schema changes | None | + stale CF + refcount | Same as incremental |
| Runtime cost | O(all nodes) | O(#stale in window) | One-time heavy cost + cheap steady state |
| Downtime | Optional | None (in theory) | Depends on how safe IncrementalSweep is |

## Why We Paused Runtime Pruning and Shipped Offline GC First

Despite the refined designs, every attempt to run the daemon in production ran into the same safety wall:

1. **Hash reuse is inevitable.** As long as we share `cf_smt_nodes` across versions, a node hash can be resurrected after it was marked stale. Without a strictly serialized delete window, the daemon has to prove that the hash is not referenced by any current or future root — an impossible task without locking writes.
2. **Refcounts cannot be trusted under races.** We tried missing-refcount guards, timestamp cutoffs, and Bloom cross-checks. None of them provided a deterministic proof that “refcount == 0” really meant “safe to delete” when new transactions could land mid-sweep.
3. **Operator experience matters.** Diagnosing issues required scraping multiple metrics, reading RocksDB CFs, and correlating logs. When a deletion bug happened, rollback required stopping the node anyway.
4. **Maintenance windows are acceptable.** The ops team confirmed that a controlled shutdown for GC is easier to schedule than constantly babysitting a background daemon whose behavior is hard to explain.

Because of these reasons we pivoted to a “safety-first” baseline: implement a stop-the-world Mark–Sweep (see `offline_gc_guide.md`), prove it works end to end, and only then revisit online pruning with the benefit of simpler rollback semantics.

## Current Status

1. **Runtime pruner: archived.** The code path is disabled by default and kept only for historical reference. This document supersedes the older design notes.
2. **Offline GC: supported.** Follow `offline_gc_guide.md` to mark from a locked root set and delete everything else inside a maintenance window. Use `pruner_e2e_testing_guide.md` to run the GC suites.
3. **Recycle bin: mandatory.** All deletions go through the recycle-bin CLI described in `recycle_bin_user_guide.md`, so manual recovery is always possible if a maintenance run goes wrong.

## Moving Forward

If we ever revisit online pruning, we already know what must change:

- Enforce a write-free window (or gated transactions) around delete batches.
- Replace ad hoc refcounts with verifiable lifecycle records or versioned keys.
- Keep protection windows explicit and auditable.

Until those prerequisites are met, the offline GC implementation remains the only supported way to reclaim disk safely.