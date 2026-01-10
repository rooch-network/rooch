// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::merkle_proof {
    use std::vector;
    use bitcoin_move::types::{Self, MerkleProof};
    use bitcoin_move::bitcoin_hash;

    const ErrorInvalidProof: u64 = 1;

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

    #[test]
    fun test_verify_merkle_proof_empty() {
        // Test with empty proof - should only work if tx_hash == merkle_root
        let tx_hash = @0x1111111111111111111111111111111111111111111111111111111111111111;
        let proof_nodes = vector::empty();
        let proof = types::new_merkle_proof(proof_nodes);
        
        // Empty proof should only pass if tx_hash equals merkle_root
        assert!(verify_merkle_proof(tx_hash, tx_hash, &proof), 1);
    }

    #[test]
    #[expected_failure]
    fun test_verify_merkle_proof_incorrect() {
        // Test with incorrect proof - should fail
        let tx1_hash = @0x1111111111111111111111111111111111111111111111111111111111111111;
        let tx2_hash = @0x2222222222222222222222222222222222222222222222222222222222222222;
        let wrong_hash = @0x3333333333333333333333333333333333333333333333333333333333333333;
        
        let expected_root = bitcoin_hash::sha256d_concat(tx1_hash, tx2_hash);
        
        // Proof with wrong sibling hash
        let proof_nodes = vector::empty();
        vector::push_back(&mut proof_nodes, types::new_proof_node(wrong_hash, false));
        let proof = types::new_merkle_proof(proof_nodes);
        
        // This should fail
        assert!(verify_merkle_proof(tx1_hash, expected_root, &proof), 1);
    }

    #[test]
    fun test_verify_merkle_proof_wrong_position() {
        // Test with wrong sibling position
        let tx1_hash = @0x1111111111111111111111111111111111111111111111111111111111111111;
        let tx2_hash = @0x2222222222222222222222222222222222222222222222222222222222222222;
        
        let expected_root = bitcoin_hash::sha256d_concat(tx1_hash, tx2_hash);
        
        // Proof with correct sibling but wrong position (is_left = true instead of false)
        let proof_nodes = vector::empty();
        vector::push_back(&mut proof_nodes, types::new_proof_node(tx2_hash, true));
        let proof = types::new_merkle_proof(proof_nodes);
        
        // This should fail because the position is wrong
        let result = verify_merkle_proof(tx1_hash, expected_root, &proof);
        assert!(!result, 1);
    }

    #[test]
    fun test_verify_merkle_proof_multilevel() {
        // Test with a 4-transaction Merkle tree (2 levels)
        //        root
        //       /    \
        //     h12    h34
        //    /  \   /  \
        //   tx1 tx2 tx3 tx4
        
        let tx1_hash = @0x1111111111111111111111111111111111111111111111111111111111111111;
        let tx2_hash = @0x2222222222222222222222222222222222222222222222222222222222222222;
        let tx3_hash = @0x3333333333333333333333333333333333333333333333333333333333333333;
        let tx4_hash = @0x4444444444444444444444444444444444444444444444444444444444444444;
        
        // Build tree bottom-up
        let h12 = bitcoin_hash::sha256d_concat(tx1_hash, tx2_hash);
        let h34 = bitcoin_hash::sha256d_concat(tx3_hash, tx4_hash);
        let root = bitcoin_hash::sha256d_concat(h12, h34);
        
        // Proof for tx1: sibling is tx2 (right), then h34 (right)
        let proof_nodes = vector::empty();
        vector::push_back(&mut proof_nodes, types::new_proof_node(tx2_hash, false)); // Level 0: tx2 is on the right
        vector::push_back(&mut proof_nodes, types::new_proof_node(h34, false));      // Level 1: h34 is on the right
        let proof = types::new_merkle_proof(proof_nodes);
        
        assert!(verify_merkle_proof(tx1_hash, root, &proof), 1);
    }
}
