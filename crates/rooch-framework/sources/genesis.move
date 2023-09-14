module rooch_framework::genesis {

    use std::error;
    use std::option;
    use moveos_std::storage_context::{Self, StorageContext};
    use rooch_framework::account;
    use rooch_framework::auth_validator_registry;
    use rooch_framework::builtin_validators;
    use rooch_framework::chain_id;
    use rooch_framework::coin;
    use rooch_framework::gas_coin;
    use rooch_framework::transaction_fee;
    use rooch_framework::timestamp;
    use rooch_framework::ethereum_light_client;

    const ErrorGenesisInit: u64 = 1;

    /// GenesisContext is a genesis init parameters in the TxContext.
    struct GenesisContext has copy,store,drop{
        chain_id: u64,
        /// genesis timestamp in microseconds
        timestamp: u64,
    }

    fun init(ctx: &mut StorageContext){
        //TODO genesis account should be a resource account?
        let genesis_account = &account::create_account(ctx, @rooch_framework);//&moveos_signer::module_signer<GenesisContext>();
        let genesis_context_option = storage_context::get<GenesisContext>(ctx);
        assert!(option::is_some(&genesis_context_option), error::invalid_argument(ErrorGenesisInit));
        let genesis_context = option::extract(&mut genesis_context_option);
        std::debug::print(&genesis_context);
        chain_id::genesis_init(ctx, genesis_account, genesis_context.chain_id);
        auth_validator_registry::genesis_init(ctx, genesis_account);
        builtin_validators::genesis_init(ctx, genesis_account);
        coin::genesis_init(ctx, genesis_account);
        gas_coin::genesis_init(ctx, genesis_account);
        transaction_fee::genesis_init(ctx, genesis_account);
        timestamp::genesis_init(ctx, genesis_account, genesis_context.timestamp);
        ethereum_light_client::genesis_init(ctx, genesis_account);
    }


    #[test_only]
    /// init the genesis context for test, and return the StorageContext with @rooch_framework genesis account
    public fun init_for_test(): StorageContext{
        let ctx = moveos_std::storage_context::new_test_context(@rooch_framework);
        storage_context::add(&mut ctx, GenesisContext{chain_id: 20230103, timestamp: 0});
        init(&mut ctx);
        ctx
    }
}