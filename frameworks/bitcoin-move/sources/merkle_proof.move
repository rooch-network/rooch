// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::merkle_proof {
    use std::vector;
    use std::option::{Self, Option};
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

    #[test]
    fun test_verify_merkle_proof() {
        // Simple test with a 2-transaction Merkle tree
        // Root = hash(hash(tx1) || hash(tx2))
        let tx1_hash = @0x1111111111111111111111111111111111111111111111111111111111111111;
        let tx2_hash = @0x2222222222222222222222222222222222222222222222222222222222222222;
        
        // Calculate expected root
        let expected_root = bitcoin_hash::sha256d_concat(tx1_hash, tx2_hash);
        
        // Proof for tx1: sibling is tx2 (on the right)
        let proof_nodes = vector::empty();
        vector::push_back(&mut proof_nodes, types::new_proof_node(tx2_hash, false));
        let proof = types::new_merkle_proof(proof_nodes);
        
        assert!(verify_merkle_proof(tx1_hash, expected_root, &proof), 1);
    }
}
