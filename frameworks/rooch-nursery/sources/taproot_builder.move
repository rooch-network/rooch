// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Taproot is a module that provides Bitcoin Taproot related functions.
module rooch_nursery::taproot_builder {

    use std::vector;
    use std::option::{Self, Option, is_none, is_some, none, destroy_some};
    use moveos_std::bcs;
    use moveos_std::compare;
    use bitcoin_move::script_buf::{Self,ScriptBuf};

    /// Tapscript leaf version.
    // https://github.com/bitcoin/bitcoin/blob/e826b22da252e0599c61d21c98ff89f366b3120f/src/script_buf/interpreter.h#L226
    const TAPROOT_LEAF_TAPSCRIPT: u8 = 0xc0;
    const TAPROOT_CONTROL_MAX_NODE_COUNT: u64 = 128;

    const TAG_TAP_LEAF:vector<u8> = b"TapLeaf";
    const TAG_TAP_BRANCH:vector<u8> = b"TapBranch";
   
    // Error codes
    const ErrorInvalidMerkleTreeDepth: u64 = 1;
    const ErrorNodeNotInDfsOrder: u64 = 2;
    const ErrorOverCompleteTree: u64 = 3;
    const ErrorUnreachable: u64 = 4;

    struct TaprootBuilder has store, drop {
        branch: vector<Option<NodeInfo>>,
    }

    struct NodeInfo has store, drop {
        hash: address,
        is_leaf: bool,
    }

    public fun new(): TaprootBuilder {
        TaprootBuilder {
            branch: vector::empty(),
        }
    }

    public fun add_leaf(builder: &mut TaprootBuilder, depth: u8, script_buf: ScriptBuf): &mut TaprootBuilder {
        let leaf = new_leaf(script_buf);
        insert(builder, leaf, depth);
        builder
    }

    fun new_leaf(script_buf: ScriptBuf): NodeInfo {
        let ver = TAPROOT_LEAF_TAPSCRIPT;
        let hash = calculate_leaf_hash(&script_buf, ver);
        NodeInfo { hash, is_leaf: true }
    }

    fun insert(builder: &mut TaprootBuilder, node: NodeInfo, depth: u8): &mut TaprootBuilder {
        let depth = (depth as u64);
        assert!(depth <= TAPROOT_CONTROL_MAX_NODE_COUNT, ErrorInvalidMerkleTreeDepth);

        // We cannot insert a leaf at a lower depth while a deeper branch is unfinished. Doing
        // so would mean the add_leaf/add_hidden invocations do not correspond to a DFS traversal of a
        // binary tree.
        assert!(depth + 1 >= vector::length(&builder.branch), ErrorNodeNotInDfsOrder);

        while (vector::length(&builder.branch) == depth + 1) {
            let child_opt = vector::pop_back(&mut builder.branch);
            if (is_none(&child_opt)) {
                vector::push_back(&mut builder.branch, child_opt);
                break
            };
            let child = destroy_some(child_opt);
            assert!(depth > 0, ErrorOverCompleteTree);
            node = combine(node, child);
            depth = depth - 1;
        };

        while (vector::length(&builder.branch) < depth + 1) {
            vector::push_back(&mut builder.branch, none());
        };
        let last_node_opt = vector::borrow_mut(&mut builder.branch, depth);
        let _pre = option::swap_or_fill(last_node_opt, node);
        
        builder
    }

    fun combine(left: NodeInfo, right: NodeInfo): NodeInfo {
        let hash = hash_internal_node(left.hash, right.hash);
        NodeInfo { hash, is_leaf: false }
    }

    fun calculate_leaf_hash(script_buf: &ScriptBuf, ver: u8): address {
        let bytes = vector::empty();
        vector::push_back(&mut bytes, ver);
        //use bcs::to_bytes to write length and then the bytes
        vector::append(&mut bytes, bcs::to_bytes(script_buf::bytes(script_buf)));
        tagged_hash(TAG_TAP_LEAF, bytes)
    }

    fun hash_internal_node(left: address, right: address): address {
        let bytes = vector::empty();
        let left_bytes = bcs::to_bytes(&left);
        let right_bytes = bcs::to_bytes(&right);
        let cmp = compare::compare_vector_u8(&left_bytes, &right_bytes);
        if (cmp == compare::result_less_than()) {
            vector::append(&mut bytes, left_bytes);
            vector::append(&mut bytes, right_bytes);
        } else {
            vector::append(&mut bytes, right_bytes);
            vector::append(&mut bytes, left_bytes);
        };
        let hash = tagged_hash(TAG_TAP_BRANCH, bytes);
        hash
    }

    public fun finalize(builder: TaprootBuilder): address {
        let len = vector::length(&builder.branch);
        if (len == 0) {
            //TODO return Result<address, TaprootBuilder> after refactor the Result
            return @0x0
        };
        let last_node_opt = vector::pop_back(&mut builder.branch);
        //This should not happen, Builder guarantees the last element is Some
        assert!(is_some(&last_node_opt), ErrorUnreachable);
        let last_node = destroy_some(last_node_opt);
        last_node.hash
    }

    fun tagged_hash(tag: vector<u8>, msg: vector<u8>): address {
        let tag_hash = sha256(tag);
        let bytes = vector::empty();
        vector::append(&mut bytes, tag_hash);
        vector::append(&mut bytes, tag_hash);
        vector::append(&mut bytes, msg);
        bcs::to_address(sha256(bytes))
    }

    fun sha256(bytes: vector<u8>): vector<u8> {
        moveos_std::hash::sha2_256(bytes)
    }

    #[test]
    fun test_tapnode_hash(){
        let script_buf = script_buf::single(bitcoin_move::opcode::op_0());
        let leaf = new_leaf(script_buf);
        //std::debug::print(&leaf.hash);
        let expected_hash = @0xe7e4d593fcb72926eedbe0d1e311f41acd6f6ef161dcba081a75168ec4dcd379;
        assert!(leaf.hash == expected_hash, 1000);
    }

    #[test]
    fun test_internal_node_hash(){
        let script_buf1 = script_buf::single(bitcoin_move::opcode::op_0());
        let script_buf2 = script_buf::single(bitcoin_move::opcode::op_true());
        let leaf1 = new_leaf(script_buf1);
        let leaf2 = new_leaf(script_buf2);
        let internal_node = combine(leaf1, leaf2);
        //std::debug::print(&internal_node.hash);
        let expected_hash = @0x15526cd6108b4765640abe555e75f4bd11d9b1453b9db4cd36cf4189577a6f63;
        assert!(internal_node.hash == expected_hash, 1001);
    }

    #[test]
    fun test_internal_node_hash2(){
        let a = @0xe7e4d593fcb72926eedbe0d1e311f41acd6f6ef161dcba081a75168ec4dcd379;
        let b = @0xa85b2107f791b26a84e7586c28cec7cb61202ed3d01944d832500f363782d675;
        let c = @0x529d993be5090bb76ae9334283c3796b24169ec184caa6dcf04f39d7dcde9e3d;
        let hash = hash_internal_node(b, c);
        let hash2 = hash_internal_node(c, b);
        assert!(hash == hash2, 1002);
        let expected_hash = @0x9de4ce9cf96062eda41e0a9f2d977e38ca1486cc5dc3a66ff6c2fe8dc5301ccf;
        assert!(hash == expected_hash, 1003);
        let hash3 = hash_internal_node(a, hash);
        let expected_hash2 = @0xd847514fba3bdcfed383ce109a2700baafd6a629e290b22678c8c21ca93aca86;
        assert!(hash3 == expected_hash2, 1004);
    }

    //Make sure the cmp function is same as the rust version
    // frameworks/rooch-nursery/tests/taproot_builder_test.rs
    #[test]
    fun test_cmp(){
        let a = @0xe7e4d593fcb72926eedbe0d1e311f41acd6f6ef161dcba081a75168ec4dcd379;
        let b = @0x9de4ce9cf96062eda41e0a9f2d977e38ca1486cc5dc3a66ff6c2fe8dc5301ccf;
        let a_bytes = bcs::to_bytes(&a);
        let b_bytes = bcs::to_bytes(&b);
        let order = compare::compare_vector_u8(&a_bytes, &b_bytes);
        assert!(order == compare::result_greater_than(), 1005);
    }
}