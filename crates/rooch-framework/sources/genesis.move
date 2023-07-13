module rooch_framework::genesis {

    use moveos_std::storage_context::StorageContext;
    use rooch_framework::auth_validator_registry;
    use rooch_framework::builtin_validators;

    const E_GENESIS_INIT: u64 = 1;

    fun init(ctx: &mut StorageContext, genesis_account: &signer){
        auth_validator_registry::genesis_init(ctx, genesis_account);
        builtin_validators::genesis_init(ctx, genesis_account);
    }


    #[test_only]
    public fun init_for_test(){
        let ctx = moveos_std::storage_context::new_test_context(@rooch_framework);
        let sender = rooch_framework::account::create_signer_for_test(@rooch_framework);
        init(&mut ctx, &sender);
        moveos_std::storage_context::drop_test_context(ctx);
    }
}