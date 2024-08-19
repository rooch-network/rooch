module bitcoin_move::multisign_account_test{
    
    use std::vector;
    use std::string::{utf8};
    use rooch_framework::bitcoin_address;
    use bitcoin_move::multisign_account;
    use bitcoin_move::genesis;

    #[test]
    public fun test_multisign_account(){
        genesis::init_for_test();
        let u1 = @0x09d7a0b046555338a1f4e62d4597d94dd7a70eb2084083cd82adf1669521e1f3;
        let u2 = @0x364611452c544b067d6d0541f239adf8067f51c6150335fb60de468051b47d88;
        let u3 = @0x49f56b4d1d0ad7320f69fe97b325c74fd9f572d395cd44d5187c457c12f969ef;
        let multisign_addr = @0x20642dfdb35337e145eebab5d0cad7e222c18afbc699b235a3e445befeb2ce0;
        let multisign_bitcoin_addr = bitcoin_address::from_string(&utf8(b"bc1payxx6patknsl8eqmr9gh9alnh50d63rcm0cj9kc6eynxkjrywq2qlm2v7m"));

        let pk1 = x"038aa3b276c6bc2117c635a92e4865baeeabf7af142350ed888723cac0a87d9c19";
        let pk2 = x"02558724fba42d673c4a91f4ef41d40025675cf50f6ab08a6b11b57770d26388d4";
        let pk3 = x"0277b379fcad0f087d2868856a04f2024055d595d7319496d9ca8e861639141cce";
        let public_keys = vector::empty();
        let threshold = 2;
        vector::push_back(&mut public_keys, pk1);
        vector::push_back(&mut public_keys, pk2);
        vector::push_back(&mut public_keys, pk3);
        let multisign_addr_result = multisign_account::initialize_multisig_account(threshold, public_keys);
        //std::debug::print(&multisign_addr_result);
        assert!(multisign_addr == multisign_addr_result, 1001);
        assert!(multisign_account::is_participant(multisign_addr, u1), 1002);
        assert!(multisign_account::is_participant(multisign_addr, u2), 1003);
        assert!(multisign_account::is_participant(multisign_addr, u3), 1004);

        assert!(multisign_bitcoin_addr == multisign_account::bitcoin_address(multisign_addr), 1005);
    }
}