# Mainnet Snapshot Refactor Handoff (2026-03-12)

## 1. Purpose

This note is for the next engineer who needs to rethink or refactor the mainnet snapshot path.

The current snapshot job has been running for over a month and is still not complete. We have already migrated it from a dedicated prune disk to the main node data directory to reduce cost, but the core problem remains: traversal throughput is too low and completion time is not predictable enough for operations.

## 2. Current Runtime State

As of `2026-03-12T12:23:10Z`:

- Host: `jolestar@34.146.31.215`
- Main node container: `rooch-mainnet`
- Snapshot container: `rooch-snapshot-main`
- Snapshot command:

```bash
/rooch/rooch db state-prune snapshot \
  --chain-id main \
  --tx-order 240930877 \
  --output /root/snapshot-work/snapshots/snapshot-optimized \
  --batch-size 50000 \
  --skip-confirm \
  --skip-dedup \
  --skip-final-compact \
  --data-dir /root/.rooch
```

- Current progress:
  - `nodes_written=1,323,750,000`
  - `nodes_visited=1,323,760,008`
  - `worklist_len=88`
  - `worklist_position=0`
  - `worklist_remaining=88`
  - `bytes_processed=563,399,956,989`
  - output size: `568G`
- Process state:
  - `container_running=yes`
  - `process_running=yes`
  - `done=no`
  - `failed=no`

Recent progress lines:

```text
2026-03-12T12:14:22Z ... 1323550000 nodes written
2026-03-12T12:16:41Z ... 1323600000 nodes written
2026-03-12T12:18:40Z ... 1323650000 nodes written
2026-03-12T12:20:38Z ... 1323700000 nodes written
2026-03-12T12:23:10Z ... 1323750000 nodes written
```

The most recent interval was `+50,000 nodes / 151s`, roughly `1.19M nodes/hour`.

## 3. Operational History

Earlier execution and migration details are in:

- `docs/dev-guide/mainnet_snapshot_resume_plan.md`
- `docs/dev-guide/mainnet_snapshot_ops_log_20260227.md`

Important completed operations:

1. Snapshot originally ran against `/data-prune/.rooch` in container `rooch-prune-snapshot`.
2. We stopped that job, made a full backup of the existing snapshot output, and resumed from the main node data directory `/data/.rooch`.
3. The dedicated prune disk was decommissioned to reduce cost:
   - unmounted from `/data-prune`
   - detached from the VM
   - deleted in GCP
4. The current job resumes successfully and continues from the same snapshot output directory.

## 4. What We Know

### 4.1 Resume Semantics Work

The current implementation can resume from prior progress safely enough for operations.

Observed resume signals:

- `Loaded valid progress`
- `Found resumable progress`
- `Restoring snapshot writer with ... previously written nodes`
- `Resuming from previous snapshot operation`
- `Resuming traversal`

This means restart/resume is not the primary problem.

### 4.2 The Problem Is Throughput and Predictability

The job is not obviously stuck. It keeps making forward progress, but too slowly.

Observed long-run throughput has usually fallen in this range:

- short windows: roughly `1.1M ~ 2.4M nodes/hour`
- many recent windows: roughly `1.5M ~ 1.9M nodes/hour`

From the migration point (`~762.75M written` on 2026-02-27) to now (`1,323.75M written` on 2026-03-12), the job has added about `560.95M` nodes over about `313.7` hours, which is roughly:

- `1.79M nodes/hour` average over the migration period

This average is operationally too slow.

### 4.3 Worklist Is Not a Reliable ETA Signal

We spent a lot of time monitoring `worklist_len`.

What we observed:

- `worklist_len` does decrease at times
- but it also rebounds significantly
- it is not monotonic near the tail
- therefore it cannot be used as a stable ETA estimator

Examples observed during the run:

- it dropped into the `50s`
- later rebounded back into the `90s` and `100s`
- current value is still `88`

Conclusion:

- a small-ish worklist does **not** imply the job is close to completion
- any estimator based only on current worklist depth is likely misleading

### 4.4 Output Size Grows Slowly Relative to Runtime

The snapshot output grew from about `369G` at migration time to about `568G` now.

That means:

- traversal time is not explained by output size alone
- the dominant cost is likely repeated RocksDB reads / tree walking / random-access traversal overhead
- this is consistent with a read-heavy bottleneck rather than a pure write bandwidth bottleneck

### 4.5 CPU Is Not the Primary Bottleneck

Earlier inspection showed the job was not CPU-saturated. Increasing container CPU was not expected to change outcomes materially.

Practical implication:

- a small CPU multiplier is unlikely to solve the problem
- algorithmic or IO-pattern changes matter much more than container sizing

## 5. Things We Tried or Considered

### 5.1 Estimation Tools

We explored several estimation approaches:

- random or layered sampling from the trie
- approximate node counting under a state root
- exact counting

Outcome:

- shallow estimators undercounted
- deeper estimators became too slow to be decision-useful
- exact counting was also too slow relative to the actual export job

Conclusion:

- estimation alone did not solve the operational problem
- the real need is a faster export path or a different workflow

### 5.2 Exact Count Tool

We implemented and ran exact counting attempts, including release builds.

Outcome:

- exact count throughput was still poor
- it did not provide a practical shortcut to completion planning

### 5.3 Parallel Read Discussion

We discussed adding parallel readers/workers.

Preliminary conclusion at the time:

- parallel reads might improve throughput somewhat
- but likely by a factor such as `~1.8x` to `~2.5x`, not enough to fundamentally change the operational picture if the baseline remains this slow

This remains worth validating properly in code, but it is not obviously sufficient as the only fix.

## 6. Important Open Questions for Refactor

### 6.1 Why Is the Export Path So Slow?

This is the main question.

Candidates to investigate:

- excessive point-lookups into RocksDB
- poor locality when traversing state nodes
- repeated reads of the same structures
- hidden deduplication or duplicate checks despite `--skip-dedup`
- serialization / buffering inefficiency
- save-progress / checkpoint overhead
- write-path stalls inside snapshot RocksDB

### 6.2 Is `--skip-dedup` Actually Honored End-to-End?

There was an observed inconsistency during earlier runs: the command line used `--skip-dedup`, but logs at one point mentioned RocksDB deduplication.

That needs code-level verification:

- is this only a misleading log line?
- or is some dedup-related structure still being opened or consulted?

If duplicate tracking is still active in any meaningful way, it may be contributing to cost.

### 6.3 Can Traversal Be Reworked Into a More Sequential Access Pattern?

A likely high-value refactor direction is reducing random RocksDB access.

Questions:

- can node export be reorganized by SST-friendly ordering?
- can child fetches be prefetched or batched better?
- can traversal be split into producer/consumer stages?
- can branch expansion be parallelized without breaking resume semantics or correctness?

### 6.4 Can Snapshot Be Built From a Different Intermediate Representation?

If exact trie traversal is the bottleneck, another possibility is changing the workflow rather than micro-optimizing the current command.

For example:

- build from a more compact state dump
- precompute a node index
- export subtrees in partitions
- checkpoint partition progress separately

These are larger design changes, but may be the only way to reduce runtime by an order of magnitude.

## 7. Constraints the Refactor Must Respect

The next design should preserve:

- resumability after interruption
- safety against corrupting the partial snapshot output
- operational visibility
- bounded cost on mainnet hardware

At minimum, the implementation should continue to expose:

- current `nodes_written`
- current `nodes_visited`
- worklist size or equivalent frontier metric
- recent throughput
- clear completion / failure events

## 8. Recommended Starting Points for the Refactorer

1. Trace the exact hot path in `state-prune snapshot`:
   - where RocksDB reads happen
   - where duplicate checks happen
   - where writes happen
   - where time is spent between progress logs

2. Verify the true runtime behavior of `--skip-dedup` in code, not just CLI wiring.

3. Quantify IO pattern:
   - random vs sequential reads
   - repeated reads for the same node/hash
   - per-batch RocksDB operation count

4. Prototype one higher-leverage change instead of small tuning:
   - parallel subtree walkers
   - batched child fetches
   - partitioned snapshot export
   - alternate intermediate format

5. Keep the current job and logs as a baseline for before/after comparison.

## 9. Key Runtime Paths

- Output directory: `/data/snapshot-work/snapshots/snapshot-optimized`
- Snapshot log: `/data/snapshot-work/logs/snapshot-mainnet-resume.log`
- Snapshot status file: `/data/snapshot-work/logs/snapshot-mainnet-status.txt`
- Watchdog log: `/data/snapshot-work/logs/snapshot-mainnet-watchdog.log`
- Main data directory mounted into the snapshot container: `/data/.rooch`

## 10. Bottom Line

The current snapshot pipeline is functionally alive but operationally inadequate.

What has been proven:

- resume works
- migration to main DB works
- old prune disk can be removed safely
- the job keeps moving forward

What has **not** been solved:

- export throughput
- ETA predictability
- a credible way to finish within a normal operational window

The next engineer should treat this as a performance/architecture problem, not an operations-only problem.
