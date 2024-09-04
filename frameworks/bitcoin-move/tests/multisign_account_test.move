#[test_only]
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

    // rooch_dao: MultisignAccountConfig {
    //         multisign_bitcoin_address: BitcoinAddress::from_str(
    //             "bc1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4sugkfjt",
    //         )
    //         .unwrap(),
    //         threshold: 5,
    //         participant_public_keys: vec![
    //             hex::decode("032d4fb9f88a63f52d8bffd1a46ad40411310150a539913203265c3f46b0397f8c")
    //                 .unwrap(),
    //             hex::decode("039c9f399047d1ca911827c8c9b445ea55e84a68dcfe39641bc1f423c6a7cd99d0")
    //                 .unwrap(),
    //             hex::decode("03ad953cc82a6ed91c8eb3a6400e55965de4735bc5f8a107eabd2e4e7531f64c61")
    //                 .unwrap(),
    //             hex::decode("0346b64846c11f23ccec99811b476aaf68f421f15762287b872fcb896c92caa677")
    //                 .unwrap(),
    //             hex::decode("03730cb693e9a1bc6eaec5537c2e317a75bb6c8107a59fda018810c46c270670be")
    //                 .unwrap(),
    //             hex::decode("0259a40918150bc16ca1852fb55be383ec0fcf2b6058a73a25f0dfd87394dd92db")
    //                 .unwrap(),
    //             hex::decode("028fd25b727bf77e42d7a99cad4b1fa564d41cdb3bbddaf15219a4529f486a775a")
    //                 .unwrap(),
    //         ],
    //     },

    #[test]
    fun test_genesis_multisign_account(){
        genesis::init_for_test();
        let pk1 = x"032d4fb9f88a63f52d8bffd1a46ad40411310150a539913203265c3f46b0397f8c";
        let pk2 = x"039c9f399047d1ca911827c8c9b445ea55e84a68dcfe39641bc1f423c6a7cd99d0";
        let pk3 = x"03ad953cc82a6ed91c8eb3a6400e55965de4735bc5f8a107eabd2e4e7531f64c61";
        let pk4 = x"0346b64846c11f23ccec99811b476aaf68f421f15762287b872fcb896c92caa677";
        let pk5 = x"03730cb693e9a1bc6eaec5537c2e317a75bb6c8107a59fda018810c46c270670be";
        let pk6 = x"0259a40918150bc16ca1852fb55be383ec0fcf2b6058a73a25f0dfd87394dd92db";
        let pk7 = x"028fd25b727bf77e42d7a99cad4b1fa564d41cdb3bbddaf15219a4529f486a775a";
        
        let multisign_bitcoin_addr = bitcoin_address::from_string(&utf8(b"bc1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4sugkfjt"));
        let multisign_addr = bitcoin_address::to_rooch_address(&multisign_bitcoin_addr);
        let public_keys = vector::empty();
        let threshold = 5;
        vector::push_back(&mut public_keys, pk1);
        vector::push_back(&mut public_keys, pk2);
        vector::push_back(&mut public_keys, pk3);
        vector::push_back(&mut public_keys, pk4);
        vector::push_back(&mut public_keys, pk5);
        vector::push_back(&mut public_keys, pk6);
        vector::push_back(&mut public_keys, pk7);
        let multisign_addr_result = multisign_account::initialize_multisig_account(threshold, public_keys);
        std::debug::print(&multisign_addr_result);
        assert!(multisign_addr == multisign_addr_result, 1001);
    }
}