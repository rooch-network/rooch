// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::mint_get_factory {

    use std::option;
    use std::string::String;
    use moveos_std::object::{Self, Object};
    use moveos_std::tx_context;
    use moveos_std::bcs;
    use rooch_nursery::tick_info;
    use rooch_nursery::bitseed_on_l2::{Bitseed};


    const DEFAULT_AMOUNT_PER_MINT: u64 = 10000;
    
    const ErrorInvalidInitLockedArgs: u64 = 1;
    const ErrorInvalidMintFunction: u64 = 2;

    struct MintGetFactory has store {
        amount_per_mint: u64, 
    }

    #[data_struct]
    struct DeployArgs has store, copy, drop{
        amount_per_mint: u64, 
    }
    
    public entry fun mint(metaprotocol: String, tick: String) {
        let bitseed = do_mint(metaprotocol, tick);
        object::transfer(bitseed, tx_context::sender());
    }
    
    public fun do_mint(metaprotocol: String, tick: String): Object<Bitseed> {
        let tick_info = tick_info::borrow_tick_info(metaprotocol, tick);
        let deploy_args = tick_info::deploy_args(tick_info);
        let amount_per_mint = if(option::is_some(&deploy_args)){
            let deploy_args_bytes = option::destroy_some(deploy_args);
            let deploy_args = bcs::from_bytes_option<DeployArgs>(deploy_args_bytes);
            if(option::is_some(&deploy_args)){
                let deploy_args = option::destroy_some(deploy_args);
                deploy_args.amount_per_mint
            }else{
                //TODO return error 
                DEFAULT_AMOUNT_PER_MINT
            }
        }else{
            DEFAULT_AMOUNT_PER_MINT
        };
        tick_info::mint<MintGetFactory>(metaprotocol, tick, amount_per_mint)
    }
}