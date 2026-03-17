# Mainnet Header-Only State Slimming Plan (2026-03-12)

## 1. Purpose

This note proposes a practical way to reduce the active-state size for a mainnet node that is intentionally being downgraded to a slimmer role:

- keep Bitcoin header sync
- stop Bitcoin transaction processing
- drop inscription / ord / bitseed functionality
- accept a framework upgrade for one-time maintenance

The immediate motivation is operational:

- the current active-state snapshot job has already written more than `1.323B` nodes and is still not finished
- the current database footprint is around `15T`
- we need a credible path to produce a much smaller active-state snapshot

This document does **not** claim that contract-side cleanup will immediately reclaim the current `15T` on the existing database. It is primarily a plan for producing a slimmer future state and a slimmer snapshot.

## 2. Assumptions

This plan assumes all of the following are true:

1. The node will no longer execute Bitcoin transaction ingestion (`execute_l1_tx`).
2. The node will continue to ingest Bitcoin headers only.
3. It is acceptable to remove inscription / ord / bitseed user-facing functionality.
4. A framework upgrade can add privileged maintenance entry functions.
5. The resulting node is a different role from a full-featured historical or Bitcoin-enabled node.

If any of these assumptions is false, this plan should not be applied as written.

## 3. Current Evidence

### 3.1 Snapshot Baseline

From `docs/dev-guide/mainnet_snapshot_refactor_handoff_20260312.md`:

- current snapshot progress: `1,323,750,000` nodes written
- current long-run throughput: roughly `1.5M ~ 1.9M nodes/hour`
- wider observed range: roughly `1.1M ~ 2.4M nodes/hour`

This is the baseline we compare against.

### 3.2 Current Large Objects

RPC inspection on mainnet shows that the dominant system objects are:

| Object | Size |
|--------|------:|
| `0x3::address_mapping::RoochToBitcoinAddressMapping` | `213,674,521` |
| `0x4::utxo::BitcoinUTXOStore` | `164,135,687` |
| `0x4::ord::InscriptionStore` | `150,953,628` |

Other common system objects such as `ModuleStore`, `Timestamp`, `ChainID`, and `GasSchedule` are not remotely comparable in subtree size.

### 3.3 Shape of the Three Large Trees

RPC sampling (`rooch_listFieldStates`) shows:

- `RoochToBitcoinAddressMapping` is effectively a flat dynamic-field table
- `BitcoinUTXOStore` is effectively a flat object set of `UTXO`
- `InscriptionStore` is a mix of:
  - `DynamicField<u32, InscriptionID>`
  - `Inscription` objects

No meaningful nested child trees were observed in the samples. For active-state sizing, these objects behave like very large flat SMTs.

### 3.4 Rough Node Contribution

For flat large tables, a practical rough estimate is:

- `total SMT nodes ~= 1.5x ~ 2.0x field_count`

Using that approximation:

| Object | Rough Node Count |
|--------|------------------:|
| `AddressMapping` | `320M ~ 430M` |
| `BitcoinUTXOStore` | `246M ~ 328M` |
| `InscriptionStore` | `226M ~ 302M` |
| Combined | `792M ~ 1.06B` |

That combined estimate is consistent with the current month-scale snapshot runtime.

## 4. Decision Matrix

| Object | Can Clear Under This Plan? | Why | Main Loss |
|--------|-----------------------------|-----|----------|
| `BitcoinUTXOStore` | Yes, conditionally | Safe only if this node never executes `execute_l1_tx` again | Lose UTXO-based Bitcoin functionality |
| `InscriptionStore` | Yes | Accepted product decision to drop inscription / bitseed | Lose ord / bitseed / metaprotocol state |
| `RoochToBitcoinAddressMapping` | Yes | Reverse mapping is not consensus-critical for header-only role | Lose reverse address lookup and some wallet UX |

## 5. Impact Analysis

### 5.1 `BitcoinUTXOStore`

`BitcoinUTXOStore` is protocol state for Bitcoin transaction processing.

Relevant code paths:

- UTXO objects are created under `BitcoinUTXOStore` in `bitcoin_move::utxo`:
  - `frameworks/bitcoin-move/sources/utxo.move`
- mainnet transaction processing checks UTXO existence and aborts if an expected UTXO is missing:
  - `frameworks/bitcoin-move/sources/bitcoin.move`

Important distinction:

- `execute_l1_block` is now header-only and does not process transactions
- `execute_l1_tx` is the path that actually depends on the active UTXO set

Therefore:

- if this node remains header-only forever, clearing `BitcoinUTXOStore` is operationally acceptable
- if this node ever resumes transaction ingestion, clearing `BitcoinUTXOStore` is not safe

Expected losses after clearing:

- no chain-side live UTXO set
- no Bitcoin transaction building / signing / selection that depends on on-chain UTXO state
- no reliable Bitcoin wallet functionality on this node role

This is a hard product boundary, not a temporary degradation.

### 5.2 `InscriptionStore`

`InscriptionStore` is not just an index. It holds:

- the inscription objects themselves
- the `sequence_number -> inscription_id` mapping
- metaprotocol attachments and validity data

It is read by ord / nursery / bitseed flows. Removing it will break:

- inscription queries
- inscription object reads
- bitseed and other metaprotocol features that attach to inscription objects

However, this plan explicitly accepts that loss.

There is also an additional favorable condition:

- the Bitcoin Move pipeline already pauses ordinals processing on mainnet after height `859001`

So for a header-only node that has already abandoned inscription-related features, `InscriptionStore` is removable.

### 5.3 `RoochToBitcoinAddressMapping`

This mapping stores:

- `rooch address -> bitcoin address`

It does **not** store the forward mapping from Bitcoin address to Rooch address because that direction is derived.

Consequences of clearing it:

- object owner display loses `owner_bitcoin_address`
- RPC / CLI reverse lookup from Rooch address to Bitcoin address fails
- Bitcoin wallet helper commands lose convenience behavior
- some validator paths will return an empty Bitcoin address instead of the historical one

Consequences that do **not** happen:

- Bitcoin address to Rooch address derivation still works
- transfer-to-Bitcoin-address flows can still derive the Rooch destination

Partial recovery is possible:

- new transfers to Bitcoin addresses can re-bind entries
- new transactions that carry Bitcoin auth context can re-bind entries

But global recovery is not automatic. Old dormant addresses will remain unmapped unless rebuilt from an external source.

## 6. Recommended Workflow

### 6.1 Preferred Workflow: Slim-State Build on an Isolated Copy

This is the recommended path.

1. Create an isolated copy of the canonical state source.
2. Upgrade the framework on that isolated environment with privileged cleanup entry functions.
3. Execute batched cleanup for:
   - `BitcoinUTXOStore`
   - `InscriptionStore`
   - `RoochToBitcoinAddressMapping`
4. Generate the active-state snapshot from the cleaned state.
5. Use the result only for:
   - slim bootstrap
   - header-only nodes
   - specialized low-footprint deployment roles
6. Keep the archival or full-featured source node separate.

Why this is preferred:

- the cleanup serves snapshot slimming directly
- the isolated environment can be discarded after export
- the canonical database avoids a giant one-time mutation

### 6.2 Not Recommended: Directly Mutate the Production 15T Database for Immediate Space Relief

This is not the primary recommendation.

Reasons:

- mass on-chain deletion creates huge state change history
- active state becomes smaller, but the current physical database does not immediately become small
- the existing `15T` footprint still requires later GC / rebuild / compaction to reclaim space
- the mutation itself may enlarge history before any physical reclamation happens

If the goal is "shrink the currently running 15T database right now", state cleanup by contract is the wrong first tool.

If the goal is "produce a much smaller future active-state snapshot", state cleanup is useful.

## 7. Framework Upgrade Design

### 7.1 New Maintenance Module

Add a small privileged maintenance module, for example:

- `rooch_framework::state_slimming`

Required properties:

- callable only by a system signer or explicit governance capability
- resumable in batches
- emits progress events
- idempotent when retried

### 7.2 Entry Functions

The module should expose batched entry points such as:

- `clear_rooch_to_bitcoin_address_mapping(cursor, limit)`
- `clear_bitcoin_utxo_store(cursor, limit)`
- `clear_inscription_store_fields(cursor, limit)`
- `reset_inscription_store_metadata()`
- `set_header_only_mode(enabled)` or equivalent explicit state marker

Notes:

- `InscriptionStore` cleanup must also reset metadata counters such as:
  - `cursed_inscription_count`
  - `blessed_inscription_count`
  - `unbound_inscription_count`
  - `lost_sats`
  - `next_sequence_number`
- otherwise the store object would remain semantically stale even after fields are removed

### 7.3 Progress Model

Each cleanup transaction should:

- remove at most `N` fields
- emit:
  - object id
  - fields removed in this batch
  - cumulative removed count
  - next cursor
- stop well below block gas limits

This gives resumability and an audit trail.

### 7.4 Optional Safety Guard

Add an explicit one-way or sticky configuration flag that marks the node state as:

- `header_only_bitcoin = true`
- `ord_disabled = true`
- `bitseed_disabled = true`

The purpose is not only documentation. It prevents accidental future re-use of this state as a full-featured node.

## 8. Runtime and Product Guardrails

If we clear these stores, we should also gate or downgrade related features.

Recommended changes:

- disable or clearly fail Bitcoin transaction build / sign / transfer features on slim nodes
- disable inscription / ord / bitseed RPC and CLI paths
- document that reverse Rooch-to-Bitcoin address lookup is not guaranteed on slim nodes
- stop presenting the result as a full-featured mainnet state

Without these guardrails, the node may appear healthy while many features silently return empty or partial data.

## 9. Expected Benefit

If all three stores are cleared, the rough active-state reduction is:

- `792M ~ 1.06B` nodes removed

A practical planning range for the cleaned active state is:

- roughly `300M ~ 800M` remaining nodes

At the current snapshot throughput baseline (`1.5M ~ 1.9M nodes/hour`), that still implies a multi-day export:

- roughly `6.6 ~ 22` days

So the cleanup is materially useful, but it is **not** a silver bullet.

What it likely changes:

- from month-scale, poorly predictable snapshot runs
- to week-scale, still-heavy but more manageable runs

If operations need a normal short maintenance window, exporter refactoring is still needed on top of state slimming.

## 10. Concrete Recommendation

Given the stated product decision, the recommended path is:

1. Treat the target as a new node role: header-only, non-UTXO, non-ord, non-bitseed.
2. Implement a framework maintenance upgrade with batched privileged cleanup entry points.
3. Run the cleanup on an isolated copy or dedicated slim-state build environment.
4. Generate a new active-state snapshot from the cleaned state.
5. Keep archival / full-featured responsibilities on a separate node role.

## 11. Open Questions

Before implementation, we should still decide:

1. Whether to preserve a tiny allowlist of reverse mappings for special accounts such as sequencer or multisig identities.
2. Whether to make the slim-state flag permanent or governance-reversible.
3. Whether the cleanup should be done by contract execution or by an offline state rewrite tool that produces the same result without inflating history.
4. Whether snapshot export improvements should be developed in parallel, since cleanup alone still leaves a large multi-day export.

## 12. References

- `docs/dev-guide/mainnet_snapshot_refactor_handoff_20260312.md`
- `frameworks/bitcoin-move/sources/bitcoin.move`
- `frameworks/bitcoin-move/sources/utxo.move`
- `frameworks/bitcoin-move/sources/ord.move`
- `frameworks/rooch-framework/sources/address_mapping.move`
- `frameworks/rooch-framework/sources/transaction_validator.move`
- `frameworks/rooch-framework/sources/transfer.move`
- `frameworks/rooch-nursery/sources/bitseed.move`
- `frameworks/rooch-nursery/sources/inscribe_factory.move`
