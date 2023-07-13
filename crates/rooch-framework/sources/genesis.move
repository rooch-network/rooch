module rooch_framework::genesis {

    use moveos_std::storage_context::StorageContext;
    use rooch_framework::auth_validator_registry;

    const E_GENESIS_INIT: u64 = 1;

    fun init(ctx: &mut StorageContext, genesis_account: &signer){
        auth_validator_registry::genesis_init(ctx, genesis_account);
        
        //SCHEME_ED25519: u64 = 0;
        let id = auth_validator_registry::register_internal<rooch_framework::ed25519_validator::Ed25519Validator>(ctx);
        assert!(id == 0, std::error::internal(E_GENESIS_INIT));
        //SCHEME_MULTIED25519: u64 = 1;
        let id = auth_validator_registry::register_internal<rooch_framework::multi_ed25519_validator::MultiEd25519Validator>(ctx);
        assert!(id == 1, std::error::internal(E_GENESIS_INIT));
        //SCHEME_SECP256K1: u64 = 2;
        let id = auth_validator_registry::register_internal<rooch_framework::secp256k1_validator::Secp256k1Validator>(ctx);
        assert!(id == 2, std::error::internal(E_GENESIS_INIT));
    }


    #[test_only]
    public fun init_for_test(){
        let ctx = moveos_std::storage_context::new_test_context(@rooch_framework);
        let sender = rooch_framework::account::create_signer_for_test(@rooch_framework);
        init(&mut ctx, &sender);
        moveos_std::storage_context::drop_test_context(ctx);
    }
}