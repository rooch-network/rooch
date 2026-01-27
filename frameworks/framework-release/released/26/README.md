# Rooch Move Framework v26

This release introduces a major enhancement to the Bitcoin integration with header-only import mode and comprehensive Merkle proof verification capabilities.

## Major Features

### [bitcoin-move] Header-Only Import & Merkle Proof Verification (#3928)
- **Header-Only Block Import**: Modified `execute_l1_block` to skip transaction processing and only process block headers
  - Significantly reduces state bloat by not creating UTXO objects automatically
  - Improves sync performance and reduces storage requirements
- **Merkle Proof Module**: Created new `merkle_proof.move` module with:
  - `verify_tx_in_block`: Verify transactions using Merkle proofs against block headers
  - `verify_merkle_proof`: Core Merkle proof validation function
  - Support for empty proof validation and error handling
- **On-Demand Transaction Verification**: Implemented `submit_tx_with_proof` entry function for minimal transaction verification
- **Enhanced Bitcoin Types**: Extended `types.move` with:
  - `MerkleProof` data structure for proof verification
  - `ProofNode` for Merkle tree path representation
- **Bitcoin Hash Improvements**: Added `sha256d_concat` helper function to `bitcoin_hash.move`

## Technical Improvements

### Bitcoin Framework Enhancements
- Removed `execute_l1_block_header` function due to security concerns (no authorization check)
- Fixed circular dependency between `merkle_proof` and `bitcoin` modules
- Added `ErrorBlockNotFound` error constant for better error handling
- Updated `pending_block.move` to support header-only processing mode
- Enhanced transaction sequencer for better block processing

### Documentation Updates
- Added comprehensive documentation for new modules:
  - `merkle_proof.md`: Merkle proof verification documentation
  - Updated `bitcoin.md`: Reflect header-only import changes
  - Updated `bitcoin_hash.md`: Document new hash helper functions
  - Updated `types.md`: Document new data structures
  - Updated `pending_block.md`: Header processing documentation

## Testing

### Comprehensive Merkle Proof Tests
- Empty proof validation
- Incorrect proof verification (properly fails)
- Wrong sibling position handling
- Multi-level Merkle tree verification

### Updated Test Coverage
- Simplified `bitcoin_test.rs` to focus on header processing
- Updated `binding_test.rs` for header-only mode:
  - `execute_l1_block` now returns empty Vec
  - `execute_l1_block_and_tx` returns empty results
- Disabled integration tests dependent on UTXO creation:
  - `bitcoin.feature` → `bitcoin.feature.bak`
  - `multisign.feature` → `multisign.feature.bak`

## Breaking Changes

### Integration Test Compatibility
Integration tests that rely on automatic UTXO creation have been temporarily disabled:
- Bitcoin feature scenarios (4 test scenarios)
- Multisign account scenarios

These tests will need to be updated to work with the new header-only import mode and on-demand transaction verification using Merkle proofs.

## Migration Notes

This release changes how Bitcoin blocks are imported:
- **Before**: Full transaction processing with automatic UTXO object creation
- **After**: Header-only import with optional on-demand transaction verification

Applications that previously relied on automatic UTXO creation should use the new `submit_tx_with_proof` function to verify transactions when needed, providing Merkle proofs as part of the verification process.

This release significantly improves Rooch's Bitcoin integration efficiency and scalability while maintaining security through cryptographic proof verification.
