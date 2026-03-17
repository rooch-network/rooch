# Mainnet Active-State Rebase Redesign

> Consolidated execution context: see `docs/dev-guide/mainnet_state_slimming_master_plan_20260316.md`.

## 1. Goal

Design a practical `active-state rebase` path for Rooch that:

- preserves current accounts, contracts, assets, and required system state
- drops old state-node history and most transaction/event history
- supports a slim node role where Bitcoin only keeps block-header state
- does not depend on the current node-level snapshot path

This document focuses on what the current system already has, what is still missing, and how to implement the missing pieces in a safe order.

## 2. What Changed In The Design

The previous rebase direction was too optimistic in two places:

1. It treated `rooch statedb export --mode snapshot` as a full active-state export.
2. It assumed a rebuilt DB only needed `startup_info`.

After re-checking the code, both assumptions are incomplete.

### 2.1 Current `snapshot` export is not recursive

`crates/rooch/src/commands/statedb/commands/export.rs` currently implements:

- `export_snapshot()` by calling `moveos_store.get_state_store().iter(state_root, None)`
- `export_top_level_fields()` with an explicit comment: `no recursive export child field`

That means the current `snapshot` export only walks one SMT at one `state_root`.
For the global root, this exports only root-level fields, not the full nested object tree.

So the current `snapshot` export cannot be used as the input artifact for a true full active-state rebase.

### 2.2 Flat `FieldKey,ObjectState` CSV is not enough for global recursive rebuild

The current export/import helpers assume one object tree at a time:

- `parse_states_csv_fields()`
- `apply_fields(moveos_store, pre_state_root, update_set)`

This shape works only when the importer already knows which parent `state_root` the fields belong to.

For a full recursive export, `FieldKey,ObjectState` alone is ambiguous because:

- the same `FieldKey` value can appear under many different parent objects
- child objects live under their own `state_root`
- importer must know which parent object each row belongs to

So a true rebase artifact must carry parent scope, not just `FieldKey,ObjectState`.

### 2.3 Current server boot path still assumes sequencer metadata exists

The runtime does not currently support a historyless DB as a first-class boot mode:

- `RoochDB::latest_root()` reads `startup_info`
- `Service::run_start_server()` always initializes:
  - `ExecutorActor`
  - `SequencerActor`
  - `DAServerActor`
  - `ProposerActor`
- `SequencerActor::new()` fails if `sequencer_info` is missing
- RPC `status()` also reports `sequencer_info`

So a rebuilt DB cannot just write `startup_info` and call it done.

## 3. What The Current System Already Has

Even though there is no end-to-end rebase flow yet, several important building blocks already exist.

### 3.1 We already have low-level logical-state write primitives

Existing helpers:

- `parse_states_csv_fields()` in `crates/rooch/src/commands/statedb/commands/mod.rs`
- `apply_fields()` in `crates/rooch/src/commands/statedb/commands/mod.rs`
- `MoveOSStore::state_store.update_fields()` in `moveos/moveos-store/src/state_store/statedb.rs`

These are enough to rebuild one object subtree once we know:

- which parent object we are updating
- the correct rebuild order

### 3.2 We already have recursive traversal examples

`crates/rooch/src/commands/db/commands/dump_state.rs` already has a correct recursive traversal pattern:

- recursively dump one root
- detect child roots through `ObjectState.metadata.state_root`
- deduplicate visited roots

That code is node-oriented, not logical-state oriented, but the recursion pattern is the right one for a future active-state exporter.

### 3.3 We already have metadata export/restore helpers

`crates/rooch/src/commands/statedb/commands/re_genesis.rs` can already export and restore:

- `genesis_info`
- `startup_info`
- `sequencer_info`

This is useful for rebase because the rebuilt DB should explicitly decide which metadata to carry over and which to synthesize.

### 3.4 Indexer rebuild is already decoupled

`crates/rooch/src/commands/indexer/commands/rebuild.rs` already rebuilds indexer state from exported logical object rows.

That means indexer should remain a post-process, not a blocker in the core rebase builder.

## 4. Hard Gaps In The Current System

The missing work falls into six concrete gaps.

### Gap A: No recursive active-state exporter

Current `statedb export --mode snapshot` is single-layer only.

We need a new exporter that:

- starts from the global root object
- recursively walks every reachable child object subtree
- can skip whole subtrees intentionally
- records enough parent context for later import

### Gap B: No rebase artifact format

We do not have a format that can represent the full nested logical state tree.

The current flat CSV is insufficient.

We need an artifact format that records, at minimum:

- parent object id
- child field key
- child object state
- traversal depth or object section boundaries

### Gap C: No importer for recursive logical state

We do not yet have a command that:

- reads the recursive active-state artifact
- rebuilds child object subtrees bottom-up
- updates each parent object with rebuilt child `state_root`
- finally writes root `startup_info`

### Gap D: No defined metadata policy for a historyless DB

Today there is no agreed answer for:

- keep original `sequencer_info`?
- synthesize new `sequencer_info`?
- keep DA metadata?
- keep proposer metadata?
- reset tx accumulator state?

This has to be formalized, otherwise boot behavior will be accidental.

### Gap E: No runtime mode for historyless slim DB

Current server startup is designed for a normal full DB.

For a historyless rebase DB, we need a mode that does not assume:

- historical transaction bodies exist
- accumulator history exists
- DA submit metadata is meaningful
- proposer can continue from old state

### Gap F: Header-only continuation path is still incomplete

If the slim DB is expected to continue syncing Bitcoin headers, the runtime still lacks a true header-only execution path.

The existing executor still validates Bitcoin L1 blocks using full block body:

- `ExecutorActor::validate_l1_block()` calls `BitcoinModule::create_execute_l1_block_call_bytes(...)`

So the header-only runtime design is still a dependency for a "live slim node", though not for the rebase artifact builder itself.

## 5. Recommended Target Product Split

Do not design one rebase flow that tries to satisfy every node role.
Split it into two products.

### Product 1: Rebased Slim DB For Read-Only / Query / Bootstrap

This is the MVP and should be the first implementation target.

Properties:

- preserves current active state only
- drops most or all old tx/event history
- can drop `BitcoinUTXOStore`, `InscriptionStore`, `RoochToBitcoinAddressMapping`
- intended for:
  - bootstrap
  - state queries
  - indexer rebuild
  - verification
  - header-only data serving after the header-only runtime lands

This product should not try to behave like a normal full-history active node.

### Product 2: Rebased Slim DB For Live Continuation

This is phase 2 and is meaningfully harder.

Additional requirements:

- tolerate missing old tx bodies and old accumulator nodes
- still sequence new txs correctly
- keep DA/proposer behavior coherent
- support Bitcoin header-only continuation

This product should not be bundled into the MVP.

## 6. New Artifact Design

The current flat CSV should not be stretched to fit recursive global state.
Use an object-scoped artifact.

### 6.1 Proposed bundle layout

```text
rebase_bundle/
  manifest.json
  objects.ndjson.zst
  metadata/
    genesis_info.json
    startup_info.json
    source_sequencer_info.json
```

### 6.2 `manifest.json`

Suggested contents:

- source chain id / network
- source root state root
- source startup size
- source tx order if known
- export timestamp
- selected role:
  - `full_active_state`
  - `header_only_slim`
- applied filters
- object count
- field count
- section count
- export version

### 6.3 `objects.ndjson.zst`

Use a sectioned stream, not a flat CSV.

Suggested record types:

- `begin_object`
- `field`
- `end_object`

Example:

```json
{"type":"begin_object","object_id":"0x0","depth":0}
{"type":"field","parent_object_id":"0x0","field_key":"...","object_state":"..."}
{"type":"field","parent_object_id":"0x0","field_key":"...","object_state":"..."}
{"type":"end_object","object_id":"0x0"}
```

The exporter should emit objects in post-order:

- child objects first
- parent object section last

This makes streaming bottom-up import much simpler.

### 6.4 Why this format

This format solves three problems the current CSV cannot solve:

1. parent scope is explicit
2. recursive subtree boundaries are explicit
3. importer can rebuild subtrees bottom-up without loading the entire world into memory

## 7. New Exporter Design

### 7.1 New command

Add a new command instead of overloading current `snapshot` export:

```text
rooch statedb export-active-tree
```

Suggested inputs:

- `--state-root`
- `--output-dir`
- `--role full-active-state|header-only-slim`
- `--exclude-object-id`
- `--exclude-object-type`
- `--json-progress`

### 7.2 Traversal algorithm

Start from root object metadata:

1. resolve root object
2. list fields under the current object's `state_root`
3. for each child object with `has_fields() == true`, recurse into child first
4. after children finish, emit the current object's section

Key points:

- deduplicate by object id and/or child `state_root`
- export in post-order
- collect counts per object and per subtree
- support skip rules before descending into a child subtree

### 7.3 Filter model

For the slim header-only role, filters should operate at subtree boundaries.

Initial built-in filter set:

- keep `BitcoinBlockStore`
- drop `BitcoinUTXOStore`
- drop `InscriptionStore`
- drop `RoochToBitcoinAddressMapping`

This is safer than fine-grained field filtering because it avoids half-deleted domain state.

### 7.4 Why not reuse current `ExportMode::Snapshot`

Because its current semantics are already wrong for this use case.
Reusing the same name would preserve confusion.

`ExportMode::Snapshot` should remain as-is or be explicitly renamed later, but the recursive rebase exporter needs a new command and a new artifact version.

## 8. New Importer Design

### 8.1 New command

Add a dedicated builder command:

```text
rooch db active-state-rebase build
```

Suggested inputs:

- `--bundle-dir`
- `--output-data-dir`
- `--metadata-mode copied|synthetic-readonly`
- `--verify`

### 8.2 Bottom-up import algorithm

Importer processes object sections in stream order.

Because export is post-order:

- when importing object `P`, all child objects of `P` have already been rebuilt
- importer can update each child `ObjectState.metadata.state_root` to the rebuilt child root
- then call `apply_fields()` for `P`

Import procedure:

1. initialize empty output DB and restore `genesis_info`
2. for each object section:
   - parse all child field rows
   - patch child object metadata with rebuilt child roots
   - call `apply_fields(moveos_store, GENESIS_STATE_ROOT, update_set)` for leaf objects or the appropriate empty pre-root
   - store rebuilt root for this object id
3. when root object section finishes, write `startup_info`
4. optionally write `sequencer_info`
5. rebuild indexer as a separate step

### 8.3 Important detail: imported rows must not trust old child roots blindly

For child objects with fields, importer must overwrite the embedded child `state_root` with the rebuilt one.

The exporter artifact is the logical state source of truth.
The new physical node hashes are expected to differ from the old DB.

## 9. Metadata Policy

This needs to be explicit.

### 9.1 Metadata always required

The rebuilt DB should always carry:

- `genesis_info`
- `startup_info`

Without these, normal boot behavior is fragile.

### 9.2 Metadata for MVP read-only slim DB

For MVP, use a dedicated `synthetic-readonly` metadata mode.

Recommended behavior:

- keep source `genesis_info`
- write new `startup_info`
- write a synthetic `sequencer_info`
  - `last_order = source snapshot tx_order` if known, otherwise `0`
  - `last_accumulator_info = default()` or a clearly marked synthetic value
- clear or ignore:
  - DA submit metadata
  - proposer last block metadata

This mode is only valid together with a runtime mode that does not try to continue normal full-service sequencing semantics.

### 9.3 Metadata for future live-continuation mode

For phase 2, support a `copied` metadata mode:

- keep source `sequencer_info`
- keep enough DA/proposer metadata to continue safely

But this should only be enabled after runtime boot is made tolerant of missing old transaction history.

## 10. Runtime Changes Needed

### 10.1 New slim-readonly boot mode

Current server startup always initializes sequencer, DA, and proposer.

Add a new runtime mode, for example:

```text
rooch server start --service-status read-only-mode --historyless
```

Behavior:

- initialize `ExecutorActor` and read RPC services
- initialize a lightweight status provider
- do not require full DA/proposer continuity
- do not call `process_sequenced_tx_on_startup()`
- do not start relayer/proposer timers
- do not assume old tx bodies exist

This is the cleanest way to make the MVP usable.

### 10.2 Optional proxy/sequencer shim

If RPC still requires `sequencer_info`, provide it from stored metadata without demanding a fully functional historical sequencer pipeline.

Do not make MVP depend on transaction accumulator proof continuity.

### 10.3 Header-only continuation remains separate work

For a live slim node that still ingests Bitcoin headers:

- executor must support a header-only block call
- Move framework must expose a header-only entry point

This work is already outlined in `docs/dev-guide/bitcoin_header_only_import_design.md`, but it is not yet the same as active-state rebase.

## 11. What Data Should Be Preserved In The Slim Role

For the `header-only-slim` role, preserve:

- root object and all reachable application/account/asset state
- package/module state
- onchain config
- gas schedule
- timestamp
- auth/session/did/account state
- `BitcoinBlockStore`

Drop:

- `BitcoinUTXOStore`
- `InscriptionStore`
- `RoochToBitcoinAddressMapping`
- old tx history CFs
- old event history CFs
- stale state nodes

This matches the business constraint already accepted for the slim role.

## 12. Recommended Implementation Order

### Phase 0: Correct the export model

Deliverables:

- new recursive exporter command
- new bundle format
- manifest generation
- subtree filters

This is the first blocker. Without it there is no correct active-state input artifact.

### Phase 1: Build MVP rebase importer

Deliverables:

- `rooch db active-state-rebase build`
- bottom-up subtree rebuild
- `genesis_info` + `startup_info` restore
- synthetic-readonly metadata mode
- verification report

### Phase 2: Add slim-readonly boot mode

Deliverables:

- runtime mode that does not assume full tx history
- no relayer/proposer dependence
- status endpoint remains usable

At this point, the rebased DB becomes operational for query/bootstrap use.

### Phase 3: Add automatic indexer rebuild

Deliverables:

- `rooch indexer rebuild` integration from the rebase bundle
- role-aware filtering for skipped domains

### Phase 4: Live continuation support

Deliverables:

- header-only block execution path
- metadata continuity policy for sequencer/DA/proposer
- startup logic tolerant of missing historical tx bodies

Do not start here.

## 13. Verification Plan

Every build should produce a machine-readable verification report.

Minimum checks:

1. rebuilt root object exists
2. rebuilt `startup_info.state_root` equals imported root object state root
3. exported object count equals imported object count after filters
4. spot-check selected critical objects:
   - root object
   - module store
   - account objects
   - `BitcoinBlockStore`
5. if slim role is used, verify dropped root fields are absent:
   - `BitcoinUTXOStore`
   - `InscriptionStore`
   - `RoochToBitcoinAddressMapping`

Optional later checks:

- hash inventory on rebuilt state nodes
- sampled RPC equivalence tests against source DB for preserved objects

## 14. Final Recommendation

The correct active-state rebase plan is:

1. stop treating current `statedb export --mode snapshot` as a full-state export
2. build a new recursive logical-state exporter with parent-scoped sections
3. build a bottom-up importer that reconstructs object subtrees into a fresh DB
4. ship MVP only for a read-only/header-only slim role
5. defer live continuation until the runtime explicitly supports historyless boot and header-only Bitcoin ingestion

In short:

- the current system already has enough low-level storage primitives
- the missing pieces are exporter recursion, artifact format, importer orchestration, and runtime boot policy
- the MVP is feasible now
- trying to make the first version behave like a normal full active node would unnecessarily expand the scope
