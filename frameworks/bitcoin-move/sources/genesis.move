// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::genesis{
    use moveos_std::context::Context;
    use moveos_std::signer;
    use bitcoin_move::light_client;

     /// BitcoinGenesisConfig is a genesis init config in the TxContext.
    struct BitcoinGenesisConfig has copy,store,drop{
        //TODO define bitcoin network config
    }

    fun init(ctx: &mut Context){
        //let genesis_account = &account::create_account(ctx, @bitcoin_move);
        let genesis_account = signer::module_signer<BitcoinGenesisConfig>();
        light_client::genesis_init(ctx, &genesis_account);
    }
}