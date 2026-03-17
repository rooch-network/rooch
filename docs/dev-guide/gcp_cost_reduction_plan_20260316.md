# GCP Cost Reduction Plan For Mainnet Operations (2026-03-16)

> Consolidated execution context: see `docs/dev-guide/mainnet_state_slimming_master_plan_20260316.md`.

## Goal

Reduce monthly infrastructure cost to about `$1000/month`, while keeping:

- a usable public-facing Rooch mainnet service,
- user account / contract / asset state,
- testnet and basic auxiliary services.

This document is based on the live GCP inventory of project `rooch-394000` inspected on `2026-03-16`, plus public GCP pricing pages.

It is intentionally pragmatic: the current mainnet archive/snapshot shape cannot be optimized into `$1000/month` by tuning alone. We need role split and storage strategy changes.

## Current Cost Drivers

### 1. Mainnet archive node dominates the bill

Observed on `rooch-mainnet-sha` in `asia-northeast1-b`:

- machine type: `c3-standard-22`
- boot disk: `100 GiB pd-ssd`
- data disk: `disk-7`, `14336 GiB pd-ssd`
- snapshot work disk: `snapshot-work`, `5120 GiB pd-ssd`

Live usage on host:

- `/data` (`disk-7`): `13T` used / `14T` total
- `/data/snapshot-work`: `995G` used / `5.0T` total

Main DB breakdown:

- `/data/.rooch/main/roochdb/store`: `12T`
- `/data/.rooch/main/roochdb/indexer`: `871G`

There is also an auto-created snapshot from `2026-03-15`:

- `disk-7-asia-northeast1-b-20260315211305-vphtbcom`
- `storageBytes = 12,668,862,730,752`

`disk-7` is also attached to an automatic snapshot policy:

- resource policy: `default-schedule-1`
- region: `asia-northeast1`
- cadence: every `1` day
- retention: `1` day

### 2. Approximate monthly cost of just this one node group

Using official GCP pricing pages:

- `c3-standard-22` in `asia-northeast1`: about `$1.108844088/hour`
- Tokyo/Osaka `pd-ssd`: about `$0.000232877/GiB-hour`
- multi-region standard snapshot storage: about `$0.000113699/GiB-hour`

Approximate monthly cost:

| Resource | Approx monthly |
| --- | ---: |
| `disk-7` `14336 GiB pd-ssd` | `$2437` |
| `snapshot-work` `5120 GiB pd-ssd` | `$870` |
| auto snapshot of `disk-7` (`12.67 TB stored`) | `$979` |
| `rooch-mainnet-sha` compute (`c3-standard-22`) | `$809` |
| boot disk | `$17` |
| **Subtotal** | **`~$5112/month`** |

This means the current mainnet archive/snapshot setup alone is already roughly `5x` the target budget.

### 3. Other visible but secondary costs

These are not the main cause, but they still matter after the archive problem is fixed.

- GKE cluster `cluster-1`
  - `3 x e2-medium`
  - `1 x e2-standard-4`
  - `1 TiB` PVC for `rooch-testnet`
  - `200 GiB` PVC for `bitcoin-testnet4`
  - multiple external LBs / static IPs
- standalone VMs
  - `rooch-testnet` (`e2-standard-2`, `350 GiB pd-balanced`)
  - `rooch-testnet-2` (`e2-medium`, `2 x 100 GiB pd-balanced`)
  - `btc-fullnode-for-rooch-mainnet` (`e2-medium`, `1024 GiB pd-balanced`)
  - `test-workflow` (`e2-medium`)
- terminated but still billed by disk
  - `babylon-integration` (`500 GiB + 50 GiB pd-balanced`)

Roughly, the non-mainnet-archive compute/storage footprint appears to be in the low hundreds to around `$1000/month`, depending on LB and network charges.

## Key Findings

### 1. This is primarily a storage problem, not a CPU problem

The major bill increase is explained by premium GCP persistent SSD storage:

- `14 TiB pd-ssd`
- `5 TiB pd-ssd`
- `12.7 TB` snapshot storage

This is far more important than trimming a few small VMs or idle services.

### 2. The snapshot attempt is actively increasing cost

`rooch-mainnet-sha` currently still has a running process:

- `rooch db state-prune snapshot --chain-id main ...`

And `snapshot-work` currently contains:

- `627G` in `snapshots`
- `369G` in `backup`

So we are paying for a large premium SSD volume while running a snapshot path that has already been identified as too slow.

### 3. Keeping the current archive node online on GCP is incompatible with a `$1000/month` target

Even before counting testnet, GKE, LBs, or BTC dependencies, the current mainnet archive node group is already around `$5.1k/month`.

If the budget target is real, this role must change.

## Recommended Strategy

## Phase 0: Immediate Stop-The-Bleed Actions (same day)

These are the highest-confidence cost cuts.

### A. Stop the current snapshot job

Reason:

- it is not completing fast enough,
- it keeps consuming CPU and writing to `snapshot-work`,
- it does not solve the cost problem on its own.

### B. Delete the `2026-03-15` auto snapshot unless there is a hard restore requirement

Resource:

- `disk-7-asia-northeast1-b-20260315211305-vphtbcom`

Important:

- deleting the current snapshot is not enough by itself,
- `disk-7` must also be detached from the `default-schedule-1` resource policy, otherwise a new large snapshot will be created again on the next cycle.

Approx saving:

- about `$980/month`

This is the fastest single saving after the disks themselves.

### C. Remove or replace the `snapshot-work` disk

Current state:

- provisioned `5 TiB pd-ssd`
- only about `995 GiB` used

Preferred action:

1. copy the needed `backup/` and `snapshots/` artifacts to cheaper storage,
2. delete `snapshot-work`,
3. if temporary workspace is still needed, recreate as:
   - smaller disk, and
   - `pd-balanced` instead of `pd-ssd`, or
   - object storage backed workflow.

Approx saving:

- delete entirely: about `$870/month`
- replace with `1 TiB pd-balanced`: still saves roughly `$770/month`

### D. Delete the terminated `babylon-integration` instance and disks if obsolete

Resource state:

- instance is `TERMINATED`
- disks still exist and still cost money

Approx saving:

- about `$55/month`

### E. Reevaluate `btc-fullnode-for-rooch-mainnet`

Observed:

- `1 TiB pd-balanced`
- `e2-medium`

If mainnet no longer processes Bitcoin transactions and only tracks block headers, this full node may no longer be justified as a permanent online dependency.

Approx saving if removed:

- roughly `$130-$180/month`

## Phase 1: Structural Change Required To Reach Target (1-4 weeks)

The critical change is to stop serving public mainnet from a full-history, archive-heavy GCP SSD node.

### Proposed role split

#### Role 1: Public slim mainnet node

Purpose:

- public RPC / read service,
- current user accounts / contract state / asset state,
- optional header-only Bitcoin awareness.

Implementation direction:

- use the active-state rebase approach already discussed in:
  - `mainnet_active_state_rebase_redesign_20260313.md`
- explicitly exclude no-longer-needed heavy state domains for this role:
  - `BitcoinUTXOStore`
  - `InscriptionStore`
  - `RoochToBitcoinAddressMapping`

Expected result:

- move from `15T` class DB to a much smaller slim DB
- allow the public node to run on `1-2 TiB` balanced storage instead of `14T+` premium SSD

#### Role 2: Archive / forensic / backup node

Purpose:

- keep full history only if truly needed

Cost rule:

- do **not** keep this role on always-on GCP `pd-ssd` at current scale

Options:

- move to cheaper dedicated storage outside GCP
- keep it offline / cold most of the time
- keep only cold copies and spin up on demand

If full archive is not operationally necessary day to day, it should not remain as an always-on premium SSD VM.

#### Role 3: Testnet + auxiliary services

Current state is fragmented:

- GKE testnet stateful workloads
- `rooch-testnet` VM
- `rooch-testnet-2` VM

This should be consolidated.

Preferred target:

- one testnet runtime footprint,
- one BTC test dependency footprint,
- no duplicate long-running environments unless there is a clear operational owner.

## Phase 2: Simplify GKE And Edge Services (1-2 weeks)

Current GKE cluster is mainly hosting:

- mainnet/testnet faucet
- oracle schedule jobs
- testnet stateful services
- `litellm`

It is not the main cost driver, but after archive slimming it becomes meaningful.

### Options

#### Option A: Keep GKE, but trim aggressively

- remove any unused node pools
- keep only required namespaces/workloads
- review external LBs and ingress IPs
- move stateful test services out if possible

#### Option B: Collapse GKE into 1-2 small VMs or Cloud Run jobs

This is likely the cleaner budget path if traffic is moderate.

Good candidates to move off GKE:

- faucets
- oracle schedule jobs
- dashboards / portal frontends
- `litellm` if still required

If GKE remains only for a small number of low-throughput services, it is likely overkill.

## What Gets Us To ~$1000/month

## Minimum realistic target architecture

To get close to the budget, the following must be true:

1. the current `rooch-mainnet-sha` archive layout is retired,
2. the `14 TiB pd-ssd` and `5 TiB pd-ssd` are no longer online permanent costs,
3. the large `disk-7` snapshot is deleted or moved out of expensive snapshot storage,
4. public mainnet runs from a slim DB,
5. testnet / auxiliary services are consolidated.

### Example target budget envelope

| Component | Target monthly |
| --- | ---: |
| public slim mainnet node | `$300-$550` |
| testnet consolidated | `$150-$250` |
| faucets/oracles/dashboard/portal/misc | `$100-$200` |
| LBs, IPs, buckets, residual charges | `$50-$100` |
| optional cheap archive or cold-storage budget | `$150-$300` |
| **Total** | **`~$750-$1400/month`** |

Interpretation:

- `~$1000/month` is realistic only if the archive role is removed from always-on GCP premium SSD.
- If a full archive must remain online in GCP, the target is not realistic.

## Recommended Execution Order

### Week 1

1. stop the current mainnet snapshot job
2. decide whether the `2026-03-15` auto snapshot is actually needed
3. delete that snapshot if not required
4. copy out `snapshot-work` artifacts and delete or replace the `5 TiB pd-ssd`
5. delete terminated `babylon-integration` resources if obsolete
6. decide whether `btc-fullnode-for-rooch-mainnet` is still needed

### Week 2-3

1. implement the active-state rebase MVP
2. generate slim mainnet artifact
3. boot a public slim mainnet node on much smaller balanced storage
4. validate core RPC / account / asset behavior

### Week 3-4

1. migrate public traffic to the slim node
2. shut down or move the archive role off GCP premium SSD
3. consolidate testnet
4. collapse or simplify GKE if still oversized for the remaining workloads

## What The System Still Lacks

The missing capability is not another snapshot attempt. It is a production-ready slim-state rebuild path.

Needed implementation work:

- recursive active-state export artifact, not just flat node snapshot
- importer / rebase builder for a fresh slim DB
- explicit filter set for dropped state domains
- boot mode for a history-light or historyless public node
- operational runbook for archive/offline recovery

This aligns with the earlier active-state rebase design work.

## Recommendation

Use this as the operating decision:

- **Immediate:** stop paying for failed snapshot + oversized temporary SSD + expensive disk snapshot.
- **Near-term:** ship a slim public mainnet node using active-state rebase.
- **Structural:** move full archive out of always-on GCP premium SSD, or make it cold/offline.

Without these changes, a `$1000/month` target is not credible.

## References

- GCP Compute pricing: https://cloud.google.com/compute/all-pricing
- GCP storage pricing: https://cloud.google.com/compute/disks-image-pricing
- Active-state rebase redesign doc: `docs/dev-guide/mainnet_active_state_rebase_redesign_20260313.md`
