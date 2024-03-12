// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// This test module is used to test the address_mapping
module rooch_framework::address_mapping_test{

    use std::option;
    use moveos_std::signer;
    use rooch_framework::multichain_address;
    use rooch_framework::address_mapping;

    #[test(sender=@0x42)]
    fun test_address_mapping(sender: signer){
        rooch_framework::genesis::init_for_test();
        let sender_addr = signer::address_of(&sender);

        let multi_chain_address = multichain_address::new(
            multichain_address::multichain_id_bitcoin(),
            x"1234567890abcdef",
        );
        address_mapping::bind(&sender, multi_chain_address);
        let addr = option::extract(&mut address_mapping::resolve(multi_chain_address));
        assert!(addr == sender_addr, 1000);
        
    }
   
}
