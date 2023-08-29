module rooch_framework::genesis {

    use std::error;
    use std::option;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::signer as moveos_signer;
    use rooch_framework::auth_validator_registry;
    use rooch_framework::builtin_validators;
    use rooch_framework::chain_id;

    const ErrorGenesisInit: u64 = 1;

    /// GenesisContext is a geneis init parameters in the TxContext.
    struct GenesisContext has copy,store,drop{
        chain_id: u64,
    }

    fun init(ctx: &mut StorageContext){
        let genesis_account = &moveos_signer::module_signer<GenesisContext>();
        let genesis_context_option = storage_context::get<GenesisContext>(ctx);
        assert!(option::is_some(&genesis_context_option), error::invalid_argument(ErrorGenesisInit));
        let genesis_context = option::extract(&mut genesis_context_option);
        chain_id::genesis_init(ctx, genesis_account, genesis_context.chain_id);
        auth_validator_registry::genesis_init(ctx, genesis_account);
        builtin_validators::genesis_init(ctx, genesis_account);
    }


    #[test_only]
    public fun init_for_test(){
        let ctx = moveos_std::storage_context::new_test_context(@rooch_framework);
        storage_context::add(&mut ctx, GenesisContext{chain_id: 20230103});
        init(&mut ctx);
        moveos_std::storage_context::drop_test_context(ctx);
    }
}