# Bitcoin Header-Only Import Design

## Overview

This document describes a refactoring plan to change Rooch's Bitcoin block synchronization from importing full blocks to importing only block headers. This significantly reduces state bloat while maintaining the ability to verify Bitcoin transactions through Merkle proofs.

## Problem Statement

### Current Issues

1. **Severe State Bloat**: Bitcoin has hundreds of millions of UTXOs, each stored as an independent Move object
2. **High Execution Cost**: Every transaction must be executed, creating/destroying UTXO objects
3. **Slow Synchronization**: Processing complete transaction data is time-consuming
4. **Storage Overhead**: Storing all transactions and UTXOs consumes significant storage

### Current Data Flow

```
Bitcoin Node → BitcoinRelayer → L1BlockWithBody (full block) → Executor
    → execute_l1_block → pending_block → execute_l1_tx (for each tx)
    → process_utxo → UTXO Objects + Inscriptions
```

## Solution: Header-Only Import with Merkle Verification

### Core Concepts

1. **Import Only Headers**: Sync only block headers to drive time updates on Rooch network
2. **Merkle Verification**: Verify specific transactions exist in blocks via Merkle proofs
3. **On-Demand State Creation**: Create UTXO/Inscription objects only when needed

### New Data Flow

```
Bitcoin Node → BitcoinRelayer → L1BlockHeader (header only) → Executor
    → execute_l1_block_header → Store header + Update timestamp

User submits tx + Merkle proof → verify_tx_in_block → On-demand processing
```

## Architecture

### Affected Components

| Component | File | Changes |
|-----------|------|---------|
| Bitcoin Relayer | `crates/rooch-relayer/src/actor/bitcoin_relayer.rs` | Fetch headers only |
| L1Block Types | `crates/rooch-types/src/transaction/ledger_transaction.rs` | Make block_body optional |
| Executor | `crates/rooch-executor/src/actor/executor.rs` | Handle header-only blocks |
| BitcoinBlockStore | `frameworks/bitcoin-move/sources/bitcoin.move` | New header-only entry point |
| MerkleProof | `frameworks/bitcoin-move/sources/merkle_proof.move` | New module for verification |

### Current State Storage

- `BitcoinBlockStore.blocks`: Block headers (Header)
- `BitcoinBlockStore.height_to_hash`: Height → Hash mapping
- `BitcoinBlockStore.hash_to_height`: Hash → Height mapping
- `BitcoinBlockStore.txs`: Transaction storage (to be deprecated)
- `BitcoinBlockStore.tx_ids`: Transaction ID list (to be deprecated)
- `UTXOStore`: All UTXO objects (to be deprecated)

## Implementation Plan

### Phase 1: Header-Only Import

#### 1.1 Modify Relayer Layer

**File**: `crates/rooch-relayer/src/actor/bitcoin_relayer.rs`

Change `sync_block` to fetch only block headers:

```rust
// Current
let block = self.rpc_client.get_block(next_hash).await?;
self.buffer.push(BlockResult { header_info, block });

// New
let header = self.rpc_client.get_block_header(next_hash).await?;
self.buffer.push(HeaderResult { header_info, header });
```

#### 1.2 Modify L1Block Data Structure

**File**: `crates/rooch-types/src/transaction/ledger_transaction.rs`

Make `block_body` optional:

```rust
pub struct L1BlockWithBody {
    pub block: L1Block,
    pub block_body: Option<Vec<u8>>,  // Changed to optional
}

impl L1BlockWithBody {
    pub fn new_bitcoin_header(height: u64, header: bitcoin::BlockHeader) -> Self {
        let block_hash = header.block_hash();
        let l1_block = L1Block {
            chain_id: RoochMultiChainID::Bitcoin.multichain_id(),
            block_height: height,
            block_hash: block_hash.to_byte_array().to_vec(),
        };
        Self {
            block: l1_block,
            block_body: None,  // No body for header-only
        }
    }
}
```

#### 1.3 Modify Executor Layer

**File**: `crates/rooch-executor/src/actor/executor.rs`

Handle header-only blocks in `validate_l1_block`:

```rust
RoochMultiChainID::Bitcoin => {
    let action = if block_body.is_some() {
        // Legacy full block processing
        VerifiedMoveAction::Function {
            call: BitcoinModule::create_execute_l1_block_call_bytes(
                block_height, block_hash, block_body.unwrap(),
            )?,
            bypass_visibility: true,
        }
    } else {
        // New header-only processing
        VerifiedMoveAction::Function {
            call: BitcoinModule::create_execute_l1_block_header_call(
                block_height, block_hash,
            )?,
            bypass_visibility: true,
        }
    };
    // ...
}
```

#### 1.4 Add Header-Only Entry Point in Move

**File**: `frameworks/bitcoin-move/sources/bitcoin.move`

```move
/// Import block header only, without processing transactions
fun execute_l1_block_header(block_height: u64, block_hash: address, header_bytes: vector<u8>) {
    let btc_block_store_obj = borrow_block_store_mut();
    let btc_block_store = object::borrow_mut(btc_block_store_obj);
    
    // Check for reorg
    assert!(!table::contains(&btc_block_store.height_to_hash, block_height), ErrorReorgTooDeep);
    
    let header = bcs::from_bytes<Header>(header_bytes);
    process_block_header(btc_block_store, block_height, block_hash, header);
    
    // Update global time
    if (!chain_id::is_test()) {
        let timestamp_seconds = (types::time(&header) as u64);
        let module_signer = signer::module_signer<BitcoinBlockStore>();
        timestamp::try_update_global_time(
            &module_signer, 
            timestamp::seconds_to_milliseconds(timestamp_seconds)
        );
    }
}
```

### Phase 2: Merkle Proof Verification

#### 2.1 Merkle Proof Data Structures

**File**: `frameworks/bitcoin-move/sources/types.move`

```move
#[data_struct]
struct MerkleProof has store, copy, drop {
    /// Path from transaction hash to merkle_root
    proof: vector<ProofNode>,
}

#[data_struct]
struct ProofNode has store, copy, drop {
    /// Sibling hash in the Merkle tree
    hash: address,
    /// True if this node is on the left side
    is_left: bool,
}
```

#### 2.2 Merkle Verification Module

**New File**: `frameworks/bitcoin-move/sources/merkle_proof.move`

```move
module bitcoin_move::merkle_proof {
    use std::vector;
    use std::option;
    use bitcoin_move::types::{Self, Header, Transaction, MerkleProof, ProofNode};
    use bitcoin_move::bitcoin_hash;
    use bitcoin_move::bitcoin;

    const ErrorInvalidProof: u64 = 1;
    const ErrorBlockNotFound: u64 = 2;

    /// Verify that a transaction is included in the specified block
    public fun verify_tx_in_block(
        block_hash: address,
        tx: &Transaction,
        proof: &MerkleProof,
    ): bool {
        let header_opt = bitcoin::get_block(block_hash);
        if (option::is_none(&header_opt)) {
            return false
        };
        let header = option::destroy_some(header_opt);
        let merkle_root = types::merkle_root(&header);
        
        verify_merkle_proof(types::tx_id(tx), merkle_root, proof)
    }

    /// Verify a Merkle proof against a known root
    public fun verify_merkle_proof(
        tx_hash: address,
        merkle_root: address,
        proof: &MerkleProof
    ): bool {
        let current_hash = tx_hash;
        let proof_nodes = types::proof_nodes(proof);
        let i = 0;
        let len = vector::length(proof_nodes);
        
        while (i < len) {
            let node = vector::borrow(proof_nodes, i);
            let sibling_hash = types::proof_node_hash(node);
            let is_left = types::proof_node_is_left(node);
            
            current_hash = if (is_left) {
                bitcoin_hash::sha256d_concat(sibling_hash, current_hash)
            } else {
                bitcoin_hash::sha256d_concat(current_hash, sibling_hash)
            };
            i = i + 1;
        };
        
        current_hash == merkle_root
    }

    /// Submit a Bitcoin transaction with Merkle proof for verification
    public entry fun submit_tx_with_proof(
        block_hash: address,
        tx_bytes: vector<u8>,
        proof_bytes: vector<u8>,
    ) {
        let tx = bcs::from_bytes<Transaction>(tx_bytes);
        let proof = bcs::from_bytes<MerkleProof>(proof_bytes);
        
        assert!(verify_tx_in_block(block_hash, &tx, &proof), ErrorInvalidProof);
        
        // Transaction verified - can now process as needed
        // e.g., create UTXO objects, process inscriptions, etc.
    }
}
```

### Phase 3: On-Demand UTXO/Inscription Processing

#### 3.1 Lazy UTXO Creation

Instead of creating all UTXOs during block sync:
- Users submit transaction proofs when they need to use a UTXO
- UTXO objects are created on-demand after verification
- An off-chain indexer can maintain UTXO state for queries

#### 3.2 Inscription Processing Strategy

For Ordinals inscriptions:
- Only create inscription objects when users submit inscription transaction proofs
- Retain core logic in `inscription_updater.move` but invoke on-demand

## Migration Considerations

### Option A: Soft Fork Upgrade

- Start header-only sync from a specific block height
- Existing UTXO state remains unchanged
- New UTXOs are no longer auto-created

### Option B: Hard Fork Reset

- Re-sync from genesis with header-only mode
- Discard existing UTXO and inscription state

### Option C: Gradual Cleanup

- Continue running but gradually prune inactive UTXO objects
- New syncs import headers only

## Impact Analysis

### Affected Features

| Feature | Impact | Solution |
|---------|--------|----------|
| UTXO Queries | ❌ No longer auto-maintained | Use Bitcoin node API or indexer |
| Inscription Queries | ❌ No longer auto-indexed | On-demand verify and create |
| Time Updates | ✅ Unchanged | Headers contain timestamp |
| Address Mapping | ❌ No longer auto-created | Create on first user interaction |
| BBN Staking | ❌ bbn.move requires full tx | On-demand verification |

### Dependent Modules

The following modules need evaluation:
- `bbn.move` - Babylon staking
- `ord.move` - Ordinals related
- `gas_faucet` / `gas_market` applications
- `grow_bitcoin` application

## Verification Plan

### Unit Tests

1. Merkle proof verification function tests
   - Correct proofs should pass
   - Invalid proofs should fail

2. Header import tests
   - Verify timestamp updates
   - Verify header storage

### Integration Tests

1. Test complete header sync flow on devnet
2. Verify compatibility with existing applications

### Manual Verification

1. Verify header sync correctly drives time
2. Test Merkle proof submission and verification flow

## Implementation Order

1. **Phase 1.4** - Modify Move layer to support header-only import
2. **Phase 1.1 - 1.3** - Modify Rust layer data flow
3. **Phase 2** - Implement Merkle proof verification
4. **Phase 3** - Implement on-demand UTXO/Inscription processing

## Performance Benefits

| Metric | Current | After Refactor |
|--------|---------|----------------|
| State per block | ~1-10 MB (tx + UTXOs) | ~80 bytes (header only) |
| Execution time per block | Seconds to minutes | Milliseconds |
| Total state size | Hundreds of GB | ~10 GB (headers only) |
| Sync speed | Days to weeks | Hours |

## Conclusion

This refactoring significantly reduces Rooch's state bloat by importing only Bitcoin block headers. The Merkle proof mechanism ensures that transactions can still be verified on-demand, maintaining security while dramatically improving performance and reducing storage requirements.

This is a breaking architectural change that requires careful evaluation of ecosystem impact and coordination with stakeholders on migration strategy.
