// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::genesis {

    use std::option;
    use moveos_std::context::{Self, Context};
    use rooch_framework::account;
    use rooch_framework::auth_validator_registry;
    use rooch_framework::builtin_validators;
    use rooch_framework::chain_id;
    use rooch_framework::coin;
    use rooch_framework::account_coin_store;
    use rooch_framework::gas_coin;
    use rooch_framework::transaction_fee;
    use rooch_framework::timestamp;
    use rooch_framework::address_mapping;
    use rooch_framework::ethereum_light_client;
    use rooch_framework::onchain_config;
    use rooch_framework::gas_schedule;

    const ErrorGenesisInit: u64 = 1;

    /// GenesisContext is a genesis init parameters in the TxContext.
    struct GenesisContext has copy,store,drop{
        chain_id: u64,
        /// genesis timestamp in microseconds
        timestamp: u64,
        /// Sequencer account
        sequencer: address,
        /// Gas Schedule
        gas_schedule_blob: vector<u8>
    }

    fun init(ctx: &mut Context){
        //TODO genesis account should be a resource account?
        let genesis_account = &account::create_account(ctx, @rooch_framework);
        let genesis_context_option = context::get<GenesisContext>(ctx);
        assert!(option::is_some(&genesis_context_option), ErrorGenesisInit);
        let genesis_context = option::extract(&mut genesis_context_option);
        chain_id::genesis_init(ctx, genesis_account, genesis_context.chain_id);
        auth_validator_registry::genesis_init(ctx, genesis_account);
        builtin_validators::genesis_init(ctx, genesis_account);
        coin::genesis_init(ctx, genesis_account);
        account_coin_store::genesis_init(ctx, genesis_account);
        gas_coin::genesis_init(ctx, genesis_account);
        transaction_fee::genesis_init(ctx, genesis_account);
        timestamp::genesis_init(ctx, genesis_account, genesis_context.timestamp);
        address_mapping::genesis_init(ctx, genesis_account);
        ethereum_light_client::genesis_init(ctx, genesis_account);
        onchain_config::genesis_init(ctx, genesis_account, genesis_context.sequencer);
        gas_schedule::gas_schedule_init(ctx, genesis_account, genesis_context.gas_schedule_blob);
    }


    #[test_only]
    use std::vector;

    #[test_only]
    /// init the genesis context for test, and return the Context with @rooch_framework genesis account
    public fun init_for_test(): Context{
        let ctx = moveos_std::context::new_test_context(@rooch_framework);
        context::add(&mut ctx, GenesisContext{chain_id: 20230103, timestamp: 0, sequencer: @rooch_framework,
            gas_schedule_blob: vector::empty()});
        init(&mut ctx);
        ctx
    }
}
