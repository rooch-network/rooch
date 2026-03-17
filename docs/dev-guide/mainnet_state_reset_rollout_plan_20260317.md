# Mainnet State Reset Rollout Plan (2026-03-17)

## 1. Purpose

This document records the agreed merge order and mainnet execution order for the Phase 1 slim public-mainnet rollout.

The goal is to avoid losing the operational sequence across multiple branches and documents.

## 2. Branch Strategy

Three branches now have distinct responsibilities:

- `feat/system-state-reset`
  - Move/native/runtime support for state reset
  - `object::clear_fields_by_system`
  - admin-gated reset entry functions
- `feat/statedb-rebase-phase1`
  - `statedb rebase-export`
  - `statedb rebase-build`
  - canonical-state rebase behavior
- `feat/slim-mainnet-integration`
  - end-to-end validation only
  - not intended for direct merge to `main`

Working rule:

- merge capability branches,
- keep the integration branch for rehearsal.

## 3. Merge Order

The recommended merge order is:

1. merge `feat/system-state-reset`
2. merge `feat/statedb-rebase-phase1`
3. keep `feat/slim-mainnet-integration` unmerged unless a later targeted fix must be backported

Reasoning:

- `feat/system-state-reset` is the chain-level capability change
- `feat/statedb-rebase-phase1` is the DB/tooling path that consumes the reset result
- separating the PRs keeps review, rollback, and bisect simple

## 4. Scope Of The First PR

The first PR should be `feat/system-state-reset`.

It should contain:

- `moveos_std::object::clear_fields_by_system`
- native/runtime support for field-tree clearing
- `reset_rooch_to_bitcoin_mapping`
- `reset_utxo_store`
- `reset_inscription_store`
- Move tests
- regenerated Move docs

It should not contain:

- `statedb rebase` Rust changes
- rollout-only integration fixes unrelated to reset capability

## 5. Scope Of The Second PR

The second PR should be `feat/statedb-rebase-phase1`.

It should contain:

- `rebase-export`
- `rebase-build`
- canonical-state rebuild logic
- the fix that preserves reset empty shells instead of dropping the three target objects during export

Important Phase 1 rule:

- the rebase pipeline must rebuild from canonical chain state after reset
- it must not reapply a second object-level drop policy for
  - `BitcoinUTXOStore`
  - `InscriptionStore`
  - `RoochToBitcoinAddressMapping`

## 6. Mainnet Rollout Preconditions

Before executing any mainnet reset transaction:

1. `feat/system-state-reset` must be merged
2. the framework upgrade that includes reset hooks must be deployed
3. `feat/statedb-rebase-phase1` should also be merged, or at minimum the tested rebase binary must be ready
4. the current long-running snapshot remains alive as fallback
5. at least one disk-level recovery point remains available
6. the node role remains explicitly:
   - Bitcoin header-only
   - no ord / bitseed
   - no Bitcoin transaction ingestion

## 7. Mainnet Execution Order

The recommended execution order is:

1. enter a controlled maintenance window
2. reduce or block new public writes as much as practical
3. execute reset transactions on mainnet in this exact order:
   - `0x4::utxo::reset_utxo_store`
   - `0x4::ord::reset_inscription_store`
   - `0x3::address_mapping::reset_rooch_to_bitcoin_mapping`
4. immediately verify post-reset object state
5. immediately take a checkpoint from the reset chain state
6. run `rebase-export`
7. run `rebase-build`
8. boot the slim node on a separate data dir / port
9. run cutover validation
10. switch public traffic only after validation passes

## 8. Why AddressMapping Must Be Last

`RoochToBitcoinAddressMapping` is special.

Normal Rooch transaction execution may write it again during `transaction_validator::pre_execute`.

That means:

- if the mapping is cleared too early,
- and other transactions continue to flow,
- the mapping can regrow before the checkpoint/export step

Therefore:

- `reset_rooch_to_bitcoin_mapping` should be executed last
- the checkpoint/export step should follow as soon as possible

## 9. Post-Reset Validation Checklist

Immediately after the three reset transactions:

- `BitcoinUTXOStore` exists and `size == 0`
- `InscriptionStore` exists and `size == 0`
- `InscriptionStore` counters are zero
- `RoochToBitcoinAddressMapping` is empty or near-empty
- record:
  - post-reset `state_root`
  - `last_order`
  - reset transaction hashes

## 10. Slim Node Validation Checklist

After `rebase-build` and slim node startup:

- startup succeeds without genesis re-init
- latest root equals the rebuilt root
- sequencer metadata loads correctly
- account reads work
- balance reads work
- module/object reads work
- the three reset domains have the expected canonical shape
- a normal Rooch write transaction succeeds

If any of the above fail:

- do not cut traffic
- keep the existing public/archive path serving

## 11. Rollback Posture

Rollback remains operationally simple only if the old path is preserved during the cutover window.

Therefore:

- do not stop the long-running fallback snapshot before slim validation passes
- do not immediately destroy old archive storage at first successful slim boot
- keep the old serving path alive through a stability window

## 12. Current Phase 1 Decision

The current agreed implementation model is:

- clear the heavy state domains on-chain first
- preserve the resulting empty-shell objects in canonical state
- rebuild a slim DB from that canonical state

This replaces the earlier Phase 1 idea of dropping those objects inside the exporter itself.
