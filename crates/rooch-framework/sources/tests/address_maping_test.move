#[test_only]
/// This test module is used to test the address_mapping
module rooch_framework::address_mapping_test{

    use std::option;
    use moveos_std::signer;
    use rooch_framework::multichain_address;
    use rooch_framework::address_mapping;

    #[test(sender=@0x42)]
    fun test_address_mapping(sender: signer){
        let genesis_ctx = rooch_framework::genesis::init_for_test();
        let sender_addr = signer::address_of(&sender);

        let multi_chain_address = multichain_address::new(
            multichain_address::multichain_id_bitcoin(),
            x"1234567890abcdef",
        );
        address_mapping::bind(&mut genesis_ctx, &sender, multi_chain_address);
        let addr = option::extract(&mut address_mapping::resolve(&genesis_ctx, multi_chain_address));
        assert!(addr == sender_addr, 1000);
        moveos_std::storage_context::drop_test_context(genesis_ctx);
    }
   
}