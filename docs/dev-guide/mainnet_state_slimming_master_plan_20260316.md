# Mainnet State Slimming Master Plan (2026-03-16)

## 1. Purpose

This document consolidates the current mainnet snapshot, state slimming, active-state rebase, and GCP cost reduction discussions into one execution-oriented plan.

It is the recommended entry point for anyone picking up this work.

The immediate business constraint is no longer only "finish a snapshot". It is:

- reduce infrastructure cost to around `$1000/month`,
- preserve public mainnet usability,
- preserve user account / contract / asset state,
- keep the current long-running snapshot only as a fallback until a replacement path is proven.

## 2. Current Situation

### 2.1 Existing snapshot path is still running, but is not the long-term answer

The current `db state-prune snapshot` job has already been running for over a month and remains too slow and too operationally unpredictable for this role.

Relevant doc:

- `docs/dev-guide/mainnet_snapshot_refactor_handoff_20260312.md`

What we already know:

- resume semantics work,
- throughput does not fail outright,
- the problem is the algorithm and the storage access pattern,
- this path should not be the long-term public-mainnet data reduction strategy.

### 2.2 The main cost problem is the archive/snapshot storage shape

The mainnet archive node currently pays for:

- `14 TiB pd-ssd`
- `5 TiB pd-ssd`
- a large retained disk snapshot
- a relatively expensive compute shape

Relevant doc:

- `docs/dev-guide/gcp_cost_reduction_plan_20260316.md`

Business implication:

- we cannot reach the target budget while public mainnet depends on the current archive node layout.

### 2.3 Product scope has already narrowed enough to justify a slim node role

Confirmed operating assumptions:

- Bitcoin transaction ingestion is no longer required on this node role,
- only Bitcoin header sync needs to remain,
- inscription / ord / bitseed functionality can be dropped,
- framework upgrade is acceptable if needed for one-time state cleanup.

Relevant doc:

- `docs/dev-guide/mainnet_header_only_state_slimming_plan_20260312.md`

This means a new slim public-mainnet role is now a valid product decision, not just a technical optimization.

## 3. Consolidated Decision

The recommended direction is:

1. keep the current long-running snapshot job alive only as a fallback,
2. stop investing in `db state-prune snapshot` as the primary future path,
3. build an `active-state rebase` pipeline for a slim public-mainnet database,
4. move the full archive role out of always-on premium GCP storage,
5. cut public traffic to the slim node after validation,
6. only then retire the old snapshot/archive path.

In short:

- **fallback path:** current snapshot
- **replacement path:** active-state rebase
- **cost target path:** slim public node + cold/offline archive

## 4. Existing Documents And How To Read Them

### 4.1 Snapshot history and why it is not enough

- `docs/dev-guide/mainnet_snapshot_resume_plan.md`
  - operational runbook for the current resumed snapshot
- `docs/dev-guide/mainnet_snapshot_ops_log_20260227.md`
  - actual migration and prune-disk decommission log
- `docs/dev-guide/mainnet_snapshot_refactor_handoff_20260312.md`
  - the current best summary of the snapshot bottleneck

Use these documents for:

- current running job context,
- rollback confidence,
- understanding why not to double down on the current algorithm.

### 4.2 State slimming scope and product tradeoffs

- `docs/dev-guide/mainnet_header_only_state_slimming_plan_20260312.md`

Use this document for:

- which large system objects dominate active-state size,
- which ones can be removed under the header-only product role,
- what functionality is lost when we do so.

### 4.3 Active-state rebase design

- `docs/dev-guide/mainnet_active_state_rebase_plan_20260313.md`
- `docs/dev-guide/mainnet_active_state_rebase_redesign_20260313.md`

Use these documents for:

- why logical active-state rebuild is a better fit than transaction replay,
- which assumptions from the first draft were too optimistic,
- what implementation gaps still exist in the current system.

### 4.4 Cost pressure and infrastructure decisions

- `docs/dev-guide/gcp_cost_reduction_plan_20260316.md`

Use this document for:

- the current GCP cost structure,
- immediate stop-the-bleed actions,
- why archive/public role split is mandatory for the target budget.

## 5. What Active-State Rebase Must Actually Deliver

The replacement pipeline should produce a new database with these properties:

- preserves current account state,
- preserves current contract/module state,
- preserves current asset / object ownership state,
- preserves required small system state,
- optionally preserves Bitcoin header-related state,
- does not preserve full transaction continuity,
- does not preserve full historical proofs,
- does not require the current node-level snapshot path.

This is **not**:

- a full archive database,
- a historical proof-serving node,
- a drop-in replacement for every current RPC behavior.

It is a slim bootstrap/runtime database for a different node role.

## 6. What The Current System Still Lacks

This section is the key implementation checklist.

### 6.1 Recursive logical-state exporter

Current issue:

- `statedb export --mode snapshot` is not a full recursive export of the entire object tree.

Needed:

- a recursive exporter that starts from the global root,
- walks nested object trees,
- deduplicates visited child roots,
- emits a full logical-state artifact.

### 6.2 A rebase artifact format with parent scope

Current issue:

- flat `FieldKey,ObjectState` rows are ambiguous for full recursive rebuild.

Needed:

- artifact rows that include parent object scope,
- or a grouped object-subtree format,
- enough information for deterministic rebuild ordering.

### 6.3 Importer / builder for a fresh slim DB

Current issue:

- the repo has low-level apply primitives, but no full rebase importer.

Needed:

- create a fresh DB,
- apply object groups in correct order,
- compute resulting roots,
- persist the final root and required metadata.

### 6.4 Explicit metadata strategy

Current issue:

- a rebuilt DB cannot rely on `startup_info` alone.

Needed:

- clear rules for `startup_info`,
- clear rules for `genesis_info`,
- clear rules for `sequencer_info`,
- explicit decision on whether these are copied, synthesized, or bypassed for the slim role.

### 6.5 Runtime boot mode for a history-light node

Current issue:

- current runtime still assumes sequencer- and actor-related metadata that fit a normal history-carrying node.

Needed:

- a supported slim/read-only/history-light boot mode,
- or a bounded compatibility layer that lets the slim DB serve the required public APIs safely.

### 6.6 Validation and cutover tooling

Needed:

- diff/validation between source chain state and rebuilt slim DB,
- smoke tests for account / balance / object / module reads,
- clear cutover criteria.

## 7. Recommended MVP Scope

To keep this project shippable, the MVP should target a **public slim read node**, not a full continuation node.

### MVP should include

- recursive active-state export,
- slim-state filtering,
- fresh DB rebuild,
- enough metadata to boot read services,
- indexer rebuild as a post-process if needed,
- RPC validation for the preserved state surface.

### MVP should exclude

- full transaction replay,
- full event/history continuity,
- proof completeness,
- Bitcoin transaction processing,
- inscription / ord / bitseed restoration,
- archive-node equivalence.

This keeps the first deliverable aligned with the actual budget problem.

## 8. State Domains To Keep vs Drop

### Keep

- global root and required object graph
- account objects
- modules / packages
- assets / coin stores / ownership state
- required framework/system configs
- Bitcoin header-related state only if needed for the role

### Drop for slim public-mainnet role

- `BitcoinUTXOStore`
- `InscriptionStore`
- `RoochToBitcoinAddressMapping`
- old transaction history
- old event history
- old changesets
- old state-node history
- archive/proof-oriented data not required by the new role

This list should be implemented as an explicit filter set, not as an operator convention.

## 9. Recommended Execution Order

### Phase A: Keep fallback alive, build replacement

1. keep the existing snapshot job running as-is for now
2. do not expand more storage or engineering investment around that path
3. implement the active-state rebase MVP in parallel

### Phase B: Produce and validate slim artifact

1. export recursive active state
2. apply slim filters
3. build fresh slim DB
4. boot slim read node
5. validate account / asset / module / object reads

### Phase C: Move public service

1. route public traffic to the slim node
2. observe for a stability window
3. keep archive/snapshot path only as fallback during the window

### Phase D: Retire expensive archive footprint

1. stop the legacy snapshot
2. delete or migrate expensive temporary snapshot storage
3. move archive role to cold/offline/cheap storage if still needed
4. keep public mainnet on the slim footprint

This sequence is important because it matches the user's current requirement:

- do not stop the old path until the new one exists.

## 10. Suggested Implementation Work Breakdown

### Workstream 1: Export artifact

- add recursive exporter
- define artifact schema
- add CLI entrypoint
- add resume/checkpointing if needed

### Workstream 2: Import / rebase builder

- add importer
- add ordering / scope handling
- add metadata emission
- add final root verification

### Workstream 3: Slim runtime boot

- add or adapt boot mode
- define supported RPC surface
- ensure status reporting is coherent on slim DB

### Workstream 4: Validation

- account/object/module diff checks
- deterministic fixture tests
- mainnet sample verification workflow

### Workstream 5: Cutover and cost retirement

- traffic migration plan
- archive fallback window
- old storage retirement steps

## 11. Immediate Next Engineering Step

The next implementation step should be:

1. define the recursive export artifact format,
2. implement the exporter,
3. make it produce a reusable mainnet sample artifact for validation.

Reason:

- this is the first hard blocker,
- it reduces ambiguity in the importer design,
- it lets us validate filtered state shape before changing boot/runtime code.

## 12. Final Recommendation

Use this working rule:

- treat the current snapshot as insurance, not as strategy,
- treat active-state rebase as the primary replacement project,
- treat archive/public role split as mandatory for the cost target.

If these three principles are not kept together, the project will drift back into trying to optimize the current archive snapshot path, which is exactly what the previous documents have already shown to be the wrong long-term direction.
