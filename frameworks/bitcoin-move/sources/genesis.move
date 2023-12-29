// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::genesis{
    use std::option;
    use std::option::Option;
    use moveos_std::context;
    use moveos_std::context::Context;
    use moveos_std::signer;
    use bitcoin_move::light_client;
    use bitcoin_move::ord;
    use bitcoin_move::brc20;

    friend bitcoin_move::network;

     /// BitcoinGenesisContext is a genesis init config in the TxContext.
    struct BitcoinGenesisContext has copy,store,drop{
        network: u8,
    }

    fun init(ctx: &mut Context){
        //let genesis_account = &account::create_account(ctx, @bitcoin_move);
        let genesis_account = signer::module_signer<BitcoinGenesisContext>();
        brc20::genesis_init(ctx, &genesis_account);
        ord::genesis_init(ctx, &genesis_account);
        light_client::genesis_init(ctx, &genesis_account);
        
    }

    public(friend) fun network(ctx: &Context) : Option<u8> {
        let genesis_context_option = context::get<BitcoinGenesisContext>(ctx);
        if(option::is_some(&genesis_context_option)){
            let network = option::borrow(&genesis_context_option).network;
            option::some(network)
        } else {
            option::none<u8>()
        }
    }
}