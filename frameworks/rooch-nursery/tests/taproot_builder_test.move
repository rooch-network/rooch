#[test_only]
module rooch_nursery::taproot_builder_tests {
    use rooch_nursery::taproot_builder;
    use bitcoin_move::opcode;
    use bitcoin_move::script_buf;

    #[test]
    fun test_taproot_builder() {
        let builder = taproot_builder::new();
        
        let script1 = script_buf::single(opcode::op_0());
        let script2 = script_buf::single(opcode::op_true());

        taproot_builder::add_leaf(&mut builder, 1, script1);
        taproot_builder::add_leaf(&mut builder, 1, script2);

        let root = taproot_builder::finalize(builder);

        //std::debug::print(&root);
        let expected_root = @0x15526cd6108b4765640abe555e75f4bd11d9b1453b9db4cd36cf4189577a6f63;
        assert!(root == expected_root, 1000);
    }

    #[test]
    fun test_taproot_builder_three_leaves() {
        let builder = taproot_builder::new();
        
        let script1 = script_buf::single(opcode::op_0());
        let script2 = script_buf::single(opcode::op_true());
        let script3 = script_buf::single(opcode::op_nop2());

        taproot_builder::add_leaf(&mut builder, 1, script1);
        taproot_builder::add_leaf(&mut builder, 2, script2);
        taproot_builder::add_leaf(&mut builder, 2, script3);

        let root = taproot_builder::finalize(builder);

        //std::debug::print(&root);
        let expected_root = @0xd847514fba3bdcfed383ce109a2700baafd6a629e290b22678c8c21ca93aca86;
        assert!(root == expected_root, 1000);
    }
}