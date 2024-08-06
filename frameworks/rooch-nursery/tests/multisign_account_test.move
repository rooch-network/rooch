module rooch_nursery::multisign_account_test{
    
    use std::vector;
    use std::string::{utf8};
    use rooch_framework::bitcoin_address;
    use rooch_nursery::multisign_account;

    #[test]
    public fun test_multisign_account(){
        let u1 = @0x343580de1dc22f1498da3841f3ddaa28357f054674630cfbe33c05e1a79d54f5;
        let u2 = @0xabe4831690b358d020eb6000fc9a5111934a79f3cf5e4c97c9586b8257f899ff;
        let u3 = @0xd72665e223ebbfc1de8d5293dca6e827ad9688d782e510b08aa2336f1c391b78;
        let multisign_addr = @0x9f76765bc03904392279284cebe0a7958df8281513c6eee222145134f9e9ab9f;
        let multisign_bitcoin_addr = bitcoin_address::from_string(&utf8(b"bc1pjj7wsqrn9a4yf7q3k8j50vl5d9yyg4at8ncqqvr6lwlzr7uytz0q9rad79"));

        let pk1 = x"025b160ffad5633ecc0ac554a0dddfc626a346b9e5e554680d22f0c31572bc6a28";
        let pk2 = x"0345e5939127a2fa34664a1289126799f0bf949dfdf5b45eebe222bd387c906e05";
        let pk3 = x"0277152b26d9b7b98e7b9c01fbdbd60957428ecc6e477b6f7692847c53e80c9fea";
        let public_keys = vector::empty();
        let threshold = 2;
        vector::push_back(&mut public_keys, pk1);
        vector::push_back(&mut public_keys, pk2);
        vector::push_back(&mut public_keys, pk3);
        let multisign_addr_result = multisign_account::initialize_multisig_account(threshold, public_keys);
        std::debug::print(&multisign_addr_result);
        assert!(multisign_addr == multisign_addr_result, 1001);
        assert!(multisign_account::is_participant(multisign_addr, u1), 1002);
        assert!(multisign_account::is_participant(multisign_addr, u2), 1003);
        assert!(multisign_account::is_participant(multisign_addr, u3), 1004);

        assert!(multisign_bitcoin_addr == multisign_account::bitcoin_address(multisign_addr), 1005);
    }
}