module rooch_framework::builtin_validators{

    use moveos_std::storage_context::StorageContext;
    use rooch_framework::auth_validator_registry;
    use rooch_framework::native_validator;
    use rooch_framework::bitcoin_validator;
    use rooch_framework::ethereum_validator;
    use rooch_framework::nostr_validator;
    use rooch_framework::multi_ed25519_validator;

    friend rooch_framework::genesis;

    const ErrorGenesisInit: u64 = 1;

    public(friend) fun genesis_init(ctx: &mut StorageContext, _genesis_account: &signer){
        // SCHEME_NATIVE: u64 = 0;
        let id = auth_validator_registry::register_internal<native_validator::NativeValidator>(ctx);
        assert!(id == native_validator::scheme(), std::error::internal(ErrorGenesisInit));

        // SCHEME_MULTIED25519: u64 = 1;
        let id = auth_validator_registry::register_internal<multi_ed25519_validator::MultiEd25519Validator>(ctx);
        assert!(id == multi_ed25519_validator::scheme(), std::error::internal(ErrorGenesisInit));

        // SCHEME_BITCOIN: u64 = 2;
        let id = auth_validator_registry::register_internal<bitcoin_validator::BitcoinValidator>(ctx);
        assert!(id == bitcoin_validator::scheme(), std::error::internal(ErrorGenesisInit));

        // SCHEME_ETHEREUM: u64 = 3;
        let id = auth_validator_registry::register_internal<ethereum_validator::EthereumValidator>(ctx);
        assert!(id == ethereum_validator::scheme(), std::error::internal(ErrorGenesisInit));

        // SCHEME_NOSTR: u64 = 4;
        let id = auth_validator_registry::register_internal<nostr_validator::NostrValidator>(ctx);
        assert!(id == nostr_validator::scheme(), std::error::internal(ErrorGenesisInit));
    }

    public fun is_builtin_scheme(scheme: u64): bool {
        scheme == native_validator::scheme()
        || scheme == multi_ed25519_validator::scheme()
        || scheme == bitcoin_validator::scheme()
        || scheme == ethereum_validator::scheme()
        || scheme == nostr_validator::scheme()
    }
}