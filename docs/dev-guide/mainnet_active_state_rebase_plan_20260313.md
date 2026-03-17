# Mainnet Active-State Rebase Plan (2026-03-13)

## 1. Purpose

This document proposes an alternative to the current mainnet snapshot path for a reduced-scope bootstrap database.

Target outcome:

- preserve current user accounts
- preserve current contract state and asset state
- preserve only the state that is live at a chosen root
- do not preserve full transaction continuity
- do not preserve full historical proofs

This is intentionally different from:

- a full archival database
- a full transaction replay
- the current node-level snapshot flow under `db state-prune snapshot`

The motivating constraint is that the current node-level snapshot pipeline is too slow and operationally unpredictable for this use case.

## 2. Core Idea

Instead of exporting and rebuilding the sparse Merkle node graph directly, or replaying all historical transactions, build a new database from the current active state materialization.

In short:

1. pick a target `state_root`
2. export the current live `FieldKey -> ObjectState` view
3. optionally filter out whole state domains that the target node role no longer needs
4. import the filtered active state into a fresh database
5. write minimal metadata so the new database can boot as a slim state snapshot

This is effectively an "active-state rebase".

The result is:

- semantically closer to "bootstrap from current balances and objects"
- not equivalent to "preserve original transaction history"

## 3. Why This Fits the Goal Better

If the goal is only:

- keep accounts
- keep assets
- keep contract state

then transaction replay is unnecessary work.

Replay is useful when we need:

- transaction continuity
- ordered history after a checkpoint
- a verifiable retained history window

That is not the requirement here.

If we do not need transaction continuity, then exporting current live state is the more direct representation of the desired outcome.

## 4. Existing Building Blocks in the Repository

### 4.1 Active State Export Already Exists

`statedb export --mode snapshot` already exports the current active state as `FieldKey,ObjectState` records.

Relevant implementation:

- `crates/rooch/src/commands/statedb/commands/export.rs`

The active-state path is:

- `ExportMode::Snapshot`
- `export_snapshot()`
- `moveos_store.get_state_store().iter(state_root, None)`

This means the export walks the current live state entries, not the full historical node graph.

### 4.2 State Parsing Already Exists

The repository already supports parsing:

- `FieldKey` from string
- `ObjectState` from string

Relevant implementation:

- `moveos/moveos-types/src/state.rs`

This is important because the snapshot CSV already uses stringified `FieldKey,ObjectState`.

### 4.3 State Apply Helpers Already Exist

The `statedb` command helpers already expose:

- `parse_states_csv_fields()`
- `apply_fields(moveos_store, pre_state_root, update_set)`

Relevant implementation:

- `crates/rooch/src/commands/statedb/commands/mod.rs`

So the project already has most of the low-level pieces needed to rebuild a new state tree from exported active-state records.

### 4.4 Existing Replay Path Solves a Different Problem

The `db state-prune replay` path:

- loads a snapshot
- replays changesets
- optionally prunes part of history

Relevant implementation:

- `crates/rooch/src/commands/db/commands/state_prune/replay.rs`
- `crates/rooch-pruner/src/state_prune/incremental_replayer.rs`

This path is useful for retaining part of transaction history.

It is not the best fit when we explicitly do not need transaction continuity.

## 5. Recommended New Workflow

### 5.1 Export Active State

Use the current state root and export live state records:

```bash
rooch statedb export \
  --mode snapshot \
  --state-root 0x<target_root> \
  --output /path/to/active_state.csv
```

This produces a logical current-state export:

- key: `FieldKey`
- value: `ObjectState`

### 5.2 Filter Unwanted State Domains

Apply a filtering step before import.

For the current reduced-scope target role, likely candidates for removal are:

- `BitcoinUTXOStore`
- `InscriptionStore`
- `RoochToBitcoinAddressMapping`
- other whole state domains that are intentionally unsupported on the target node role

This filtering step should operate on logical state entries, not on raw SMT nodes.

### 5.3 Import Into a Fresh Database

Build a new command, for example:

- `rooch statedb import --mode snapshot`
- or `rooch db state-rebase`

The import path should:

1. create a new empty output store
2. read `FieldKey,ObjectState` CSV
3. batch entries into `UpdateSet<FieldKey, ObjectState>`
4. apply them from an empty root using `apply_fields`
5. commit the resulting state tree
6. write `startup_info`
7. initialize only the minimum metadata required to boot the target node role

### 5.4 Treat the Result as a New Node Role

The resulting database should be explicitly treated as:

- a slim bootstrap database
- not a full historical database
- not an archive node
- not a full Bitcoin/ord/bitseed node

## 6. Why This May Avoid the Current Snapshot Bottleneck

The current slow path is `db state-prune snapshot`, which exports raw state nodes by traversing the node graph.

That path is operationally expensive because it is dominated by:

- point lookups into RocksDB
- poor locality in trie traversal
- repeated random-access reads

By contrast, `statedb export --mode snapshot` works at the logical active-state layer.

Conceptually this means:

- export the current materialized state
- not the historical node graph needed to reconstruct every reachable internal node

This does not guarantee a trivial runtime, but it is much closer to the actual target artifact.

In other words:

- current node-level snapshot is optimized for trie reconstruction
- active-state rebase is optimized for preserving current state semantics

## 7. Important Tradeoffs

### 7.1 What We Keep

The rebase result should preserve:

- account objects
- contract/module state required by the target chain role
- user asset state
- object ownership
- current on-chain configuration needed by runtime
- any small system objects required for boot

### 7.2 What We Intentionally Lose

The rebase result may discard:

- historical transactions
- old changesets
- execution info history
- old events
- accumulator continuity
- historical proof compatibility
- state domains intentionally excluded from the slim role

### 7.3 This Is Not a Chain-Preserving Rewrite

This is the key boundary.

The output is not "the same chain with some unimportant transactions removed".

It is:

- a new database built from the current logical state

This is why it works for bootstrap and slim-node scenarios, but not for archive or proof-serving scenarios.

## 8. Comparison With Other Approaches

### 8.1 Compared With Full Transaction Replay

Full replay is better when we need:

- continuity from a known checkpoint
- recent history retention
- stronger compatibility with transaction-order semantics

Active-state rebase is better when we need:

- only the current account and asset state
- no old transaction continuity
- the smallest implementation aligned with the actual goal

### 8.2 Compared With Current `db state-prune snapshot`

Current node snapshot:

- exports raw SMT nodes
- preserves trie reconstruction semantics
- is currently too slow for the mainnet use case

Active-state rebase:

- exports logical live state
- is closer to "preserve balances and objects"
- requires a new import path, but matches the reduced requirement more directly

### 8.3 Compared With Contract-Side Mass Cleanup

Contract-side cleanup helps reduce future active-state size, but:

- it mutates the current chain state
- it creates additional history before physical reclamation
- it is not the fastest path if the only goal is to build a slim bootstrap database

Active-state rebase can often be cleaner because:

- filtering happens at export/import time
- the canonical database does not need to absorb the full mutation

## 9. Proposed Command Design

Two implementation shapes are reasonable.

### Option A: Extend `statedb`

Add:

- `rooch statedb import --mode snapshot`

Responsibilities:

- read active-state CSV
- rebuild logical state tree
- emit final `state_root`
- initialize startup metadata

### Option B: Add a Purpose-Built Rebase Command

Add:

- `rooch db state-rebase`

Responsibilities:

1. export active state at a chosen root
2. optionally filter by keep/drop rules
3. rebuild a fresh database
4. optionally write a role manifest:
   - `history_preserved = false`
   - `bitcoin_utxo_enabled = false`
   - `ord_enabled = false`

Option B is cleaner if we want this to be an operator-facing workflow rather than just a low-level import utility.

## 10. Minimal Viable Implementation

The smallest useful implementation would be:

1. export active-state CSV using existing `statedb export --mode snapshot`
2. implement CSV import into a new empty store using existing parsers and `apply_fields`
3. support a drop-list for known top-level domains:
   - `BitcoinUTXOStore`
   - `InscriptionStore`
   - `RoochToBitcoinAddressMapping`
4. write `startup_info`
5. boot the output database in a restricted role

This is enough to validate whether active-state rebase is a practical escape hatch from the current snapshot bottleneck.

## 11. Known Gaps

Before this becomes production-ready, we still need to define:

1. exactly which system objects must always be preserved for boot correctness
2. how to initialize minimal metadata beyond `startup_info`
3. how to validate the rebuilt state:
   - object counts
   - root consistency
   - boot-time smoke tests
4. how to encode the target node role so unsupported APIs fail explicitly
5. whether filtering happens:
   - during export
   - during import
   - or in a separate transform step

## 12. Recommendation

Given the current requirement set, the recommended direction is:

1. stop thinking in terms of replaying or preserving transaction continuity
2. treat the target artifact as a logical current-state bootstrap
3. use `statedb export --mode snapshot` as the export basis
4. build a new active-state import/rebase path
5. validate the output as a slim node role, not as a historical mainnet replica

This is the most direct path toward:

- keeping user accounts
- keeping asset state
- dropping historical weight
- avoiding the current node-level snapshot bottleneck

## 13. References

- `docs/dev-guide/mainnet_snapshot_refactor_handoff_20260312.md`
- `docs/dev-guide/mainnet_header_only_state_slimming_plan_20260312.md`
- `crates/rooch/src/commands/statedb/commands/export.rs`
- `crates/rooch/src/commands/statedb/commands/mod.rs`
- `moveos/moveos-types/src/state.rs`
- `crates/rooch/src/commands/db/commands/state_prune/replay.rs`
- `crates/rooch-pruner/src/state_prune/incremental_replayer.rs`
