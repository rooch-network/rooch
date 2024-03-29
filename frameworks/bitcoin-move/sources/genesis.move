// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::genesis{
    use std::option;
    use std::option::Option;
    use moveos_std::tx_context;
    use moveos_std::signer;
    use bitcoin_move::light_client;
    use bitcoin_move::ord;
    use bitcoin_move::brc20;
    use bitcoin_move::utxo;

    friend bitcoin_move::network;

     /// BitcoinGenesisContext is a genesis init config in the TxContext.
    struct BitcoinGenesisContext has copy,store,drop{
        network: u8,
    }

    fun init(){
        let genesis_account = signer::module_signer<BitcoinGenesisContext>();
        utxo::genesis_init();
        brc20::genesis_init(&genesis_account);
        ord::genesis_init(&genesis_account);
        light_client::genesis_init(&genesis_account);
    }

    public(friend) fun network() : Option<u8> {
        let genesis_context_option = tx_context::get_attribute<BitcoinGenesisContext>();
        if(option::is_some(&genesis_context_option)){
            let network = option::borrow(&genesis_context_option).network;
            option::some(network)
        } else {
            option::none<u8>()
        }
    }
}