module rooch_framework::builtin_validators{

    use moveos_std::storage_context::StorageContext;
    use rooch_framework::auth_validator_registry;
    use rooch_framework::native_validator;
    use rooch_framework::ethereum_validator;

    friend rooch_framework::genesis;

    const ErrorGenesisInit: u64 = 1;

    public(friend) fun genesis_init(ctx: &mut StorageContext, _genesis_account: &signer){
        // SCHEME_NATIVE: u64 = 0;
        let id = auth_validator_registry::register_internal<native_validator::NativeValidator>(ctx);
        assert!(id == native_validator::scheme(), std::error::internal(ErrorGenesisInit));

        // SCHEME_ETHEREUM: u64 = 1;
        let id = auth_validator_registry::register_internal<ethereum_validator::EthereumValidator>(ctx);
        assert!(id == ethereum_validator::scheme(), std::error::internal(ErrorGenesisInit));
    }

    public fun is_builtin_scheme(scheme: u64): bool {
        scheme == native_validator::scheme()
        || scheme == ethereum_validator::scheme()
    }
}