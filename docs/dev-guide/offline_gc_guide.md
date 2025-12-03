# Rooch Offline GC Guide

The runtime pruner has been removed. All storage reclamation now happens through a stop-the-world Mark–Sweep command that runs while the Rooch node is offline. This guide consolidates the previously scattered plans (`pruner_shutdown_gc_plan.md`, `state_gc_design.md`, `gc_system_comprehensive_test_analysis.md`, etc.) into a single authoritative reference.

## When to Run Offline GC

- Before bringing a validator back online after prolonged testing or fuzzing.
- After large-scale Move upgrades that leave behind orphaned table nodes.
- Whenever disk consumption approaches the recycle-bin safety threshold.
- As part of quarterly maintenance to prove the state tree is still well-formed.

## Safety Checklist

1. **Stop all writers**  
   - Shut down `rooch server` (or pause the ingress tier).  
   - Wait for RocksDB locks to disappear and run `rooch db safety-check` if available.
2. **Record the protected roots**  
   - Capture the latest `state_root`, `latest_order`, and desired retention window.  
   - Store the values in the maintenance log so the GC report can be audited later.
3. **Back up the recycle bin metadata**  
   - `rooch db recycle-export -n <net> -o pre_gc.json --format json`
4. **Ensure disk headroom**  
   - GC creates temporary column families for the marker and can double disk usage for a short period.
5. **Decide on dry-run vs execute**  
   - Always start with `--dry-run` when operating on a new dataset.

GC refuses to start if it detects the RocksDB lock file is still held. Use `--skip-confirm` only in CI or in scripted E2E tests where you already killed the server.

## Command Reference

```bash
rooch db gc \
  --chain-id local \
  --data-dir ~/.rooch/local/data \
  --dry-run \                # optional, skips deletion
  --root 0x<state_root> \    # optional, defaults to latest snapshot/startup info
  --roots-file roots.json \  # optional list of roots to protect
  --batch-size 10000 \
  --workers 4 \
  --marker-strategy auto \   # auto | memory | persistent
  --recycle-bin \            # enable recycle bin writes
  --compact \                # run RocksDB compaction at the end
  --json                     # emit structured report
```

### Key Options

| Flag | Purpose | Notes |
|------|---------|-------|
| `--dry-run` | Run the mark phase only | Still produces statistics so you can size the maintenance window. |
| `--root` / `--roots-file` | Override the protected root set | Supply multiple historical roots if you need a wider safety window. |
| `--marker-strategy` | Force a marker backend | `auto` uses RocksDB `approximate_num_keys` to pick between an in-memory `HashSet` (<10M nodes) and a persistent CF. |
| `--batch-size` | Sweep delete batch | Tune for RocksDB write amplification; 10k is a safe default. |
| `--workers` | DFS worker pool | Only affects the Mark phase; keep <= number of physical cores. |
| `--recycle-bin` | Enable recycle bin writes | Strongly recommended in production. |
| `--skip-confirm` | Bypass “are you sure the node is stopped?” prompt | Only for automated tests—never for humans. |
| `--json` | Emit structured `GCJsonReport` | Parsing helpers live in `sdk/typescript/rooch-pruner-e2e/src/utils/gc-utils.ts`. |

## Execution Playbook

1. **Preparation**
   - Stop the node, wait 3–5 seconds for locks to flush (tests use `stopServerForGC()` for this reason).
   - Capture `du -sh` for the data directory so you can compare reclaimed space later.
2. **Mark phase**
   - DFS starts from the protected roots, follows nested `child_root`s and internal node edges.
   - Progress is logged every 100k nodes: `Marked <n> nodes...`.
   - `MarkStats` records `markedCount`, `durationMs`, and the marker strategy.
3. **Sweep phase**
   - Iterates every key in the `state_node` CF, deletes items that are not marked, optionally dumps them into the recycle bin.
   - Logs include `sweep scanned/kept/deleted` counters and recycle-bin insertions.
4. **Compaction & restart**
   - If `--compact` is set, run RocksDB aggressive compaction after deletions to reclaim disk space immediately.
   - Start the server, monitor health (`rooch_status`, `account list`) and re-run lightweight read/write smoke tests.

### Expected Timing (SSD, 100 M nodes, 4 workers)

| Phase | Typical Duration |
|-------|------------------|
| Mark | 3–4 minutes (20 M reachable nodes @ 100k random reads/sec) |
| Sweep | 5–10 minutes (100 M sequential reads + 80 M deletes) |
| Compaction | 5–10 minutes depending on disk load |

## Observability & Reporting

- **Standard log markers** (imported from the old `state_gc_design.md`): `Starting GC`, `=== Mark Phase ===`, `=== Sweep Phase ===`, `=== Compaction ===`, `GC completed`.
- **JSON report** (see `GCJsonReport` in `gc-utils.ts`):
  - `executionMode`: `dry-run` or `execute`
  - `protectedRoots`: list + count
  - `markStats`: `markedCount`, `durationMs`, `memoryStrategy`
  - `sweepStats`: `scannedCount`, `deletedCount`, `recycleBinEntries`, `durationMs`
  - `spaceReclaimed`: estimated bytes reclaimed (node count × average node size)
- **Prometheus**: reuse the historic metrics if the node still exports them, but treat them as advisory. Disk delta (`peak - final`) remains the source of truth.

## Recycle Bin Integration

Offline GC always recommends running with `--recycle-bin`. Operators can then use:

- `rooch db recycle-stat -n <net> --detailed` – capacity overview + recommendations.
- `rooch db recycle-clean --dry-run --older-than 30d` – plan a cleanup, then re-run with `--force`.
- `rooch db recycle-export --include-node-data` – archive deleted nodes before purging.
- `rooch db recycle-list --phase GC` – inspect exactly what the most recent GC deleted.

See `docs/dev-guide/recycle_bin_user_guide.md` for complete operational procedures. The redundant `recycle_bin_operational_procedures.md` and `recycle_bin_troubleshooting.md` docs were merged into that guide and removed from the tree.

## Testing & Validation

### TypeScript E2E Suite

Located in `sdk/typescript/rooch-pruner-e2e` (name kept for backwards compatibility).

```bash
cd sdk/typescript/rooch-pruner-e2e
pnpm install
pnpm test:gc                 # full suite (dry-run + execute paths)
pnpm test:gc:dry-run         # only the mark-only scenario
pnpm test:gc:execute         # end-to-end mark+sweep
pnpm test:gc:stress:quick    # 5-minute stress smoke
```

The suite:

- Generates Move workloads, publishes helper packages, and records disk usage.
- Stops the server, runs `rooch db gc --json`, parses `GCJsonReport`, and prints a structured summary.
- Restarts the node and verifies basic RPCs to prove the state tree is still readable.

### Current Coverage Snapshot

(Summarized from the archived `gc_system_comprehensive_test_analysis.md`)

- 80 automated tests total; 71 pass (core functionality + safety), 9 pending improvements (performance scaling + real-data integration).
- Safety verifier tests (lock detection, concurrent health checks) all pass and run >1000 checks/second.
- Stress tests highlighted two gaps:
  1. **Hash collision depth** – extremely adversarial states can still overflow recursion unless we use the iterative DFS. This is tracked as P0.
  2. **Mock store realism** – integration tests fail when no snapshot/startup metadata exists. We need real datasets.

### Test Data & Dataset Plan

The missing metadata is addressed by the dataset blueprint that used to live in the (now archived) `gc_failed_tests_analysis_and_dataset_solution.md`. Action items:

1. **Snapshot fixtures** – provide small/medium/large JSON snapshots (block height, state root, node statistics) plus matching startup info.
2. **Programmatic generator** – `GCTestDataGenerator` to synthesize random accounts, transactions, churn, and GC expectations.
3. **Importer** – optional tool to sample real RocksDB instances into fixtures.

Until those land, always run the TypeScript suite against a real node instead of `MoveOSStore::mock_moveos_store()`.

## Troubleshooting & Known Risks

| Issue | Mitigation |
|-------|------------|
| **Recycle bin disabled** | The CLI now warns loudly. Always pass `--recycle-bin` unless running synthetic stress tests. |
| **Mark results ignored** | The CLI has unit + E2E coverage to ensure the sweep phase consults the same marker. If you modify the GC code, keep the `GCJsonReport` assertions green. |
| **No stop-the-world confirmation** | `--skip-confirm` is only for automation. Production runs should answer the interactive prompt or supply an explicit `--confirm-stopped` flag (future work). |
| **Missing roots** | GC locks onto snapshots (`prune_meta_snapshot`) first, then falls back to startup info. If both are absent, the command fails early—populate at least one before running GC. |
| **Perf regressions** | Monitor `markStats.memoryStrategy` and `sweepStats.scannedCount`. Unexpected spikes indicate that `approximate_num_keys` mis-estimated the dataset size; rerun with `--marker-strategy persistent`. |

## Future Work

- **Pre-marking to shorten downtime** – Reuse the “freeze roots online, sweep offline” ideas from the old (archived) `pruner_gc_style_plan.md`.
- **Dataset-driven CI** – Ship the data generator/importer so we can run GC integration tests without spinning up a full node.
- **Automated recycle-bin quotas** – Convert the operational scripts from `recycle_bin_operational_procedures.md` into built-in alerts.

## References

- `docs/dev-guide/pruner_guide.md` – runtime pruner history (archived).
- `docs/dev-guide/recycle_bin_user_guide.md` – detailed recycle-bin procedures.
- `sdk/typescript/rooch-pruner-e2e/src/case/gc-recycle.spec.ts` – canonical E2E flow.
- `sdk/typescript/rooch-pruner-e2e/src/utils/gc-utils.ts` – JSON report schema & CLI wrapper.

