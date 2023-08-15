module rooch_framework::builtin_validators{

    use moveos_std::storage_context::StorageContext;
    use rooch_framework::auth_validator_registry;
    use rooch_framework::ed25519;
    use rooch_framework::ecdsa_k1;
    use rooch_framework::ecdsa_k1_recoverable;
    use rooch_framework::schnorr;
    use rooch_framework::native_validator;
    use rooch_framework::bitcoin_validator;
    use rooch_framework::ethereum_validator;
    use rooch_framework::nostr_validator;
    use rooch_framework::multi_ed25519_validator;

    friend rooch_framework::genesis;

    const E_GENESIS_INIT: u64 = 1;

    public(friend) fun genesis_init(ctx: &mut StorageContext, _genesis_account: &signer){
        //SCHEME_ED25519: u64 = 0;
        let id = auth_validator_registry::register_internal<native_validator::NativeValidator>(ctx);
        assert!(id == ed25519::scheme(), std::error::internal(E_GENESIS_INIT));
        //SCHEME_MULTIED25519: u64 = 1;
        let id = auth_validator_registry::register_internal<multi_ed25519_validator::MultiEd25519Validator>(ctx);
        assert!(id == multi_ed25519_validator::scheme(), std::error::internal(E_GENESIS_INIT));
        //SCHEME_ECDSA_K1: u64 = 2;
        let id = auth_validator_registry::register_internal<bitcoin_validator::BitcoinValidator>(ctx);
        assert!(id == ecdsa_k1::scheme(), std::error::internal(E_GENESIS_INIT));
        //SCHEME_ECDSA_K1_RECOVERABLE: u64 = 3;
        let id = auth_validator_registry::register_internal<ethereum_validator::EthereumValidator>(ctx);
        assert!(id == ecdsa_k1_recoverable::scheme(), std::error::internal(E_GENESIS_INIT));
        //SCHEME_SCHNORR: u64 = 4;
        let id = auth_validator_registry::register_internal<nostr_validator::NostrValidator>(ctx);
        assert!(id == schnorr::scheme(), std::error::internal(E_GENESIS_INIT));
    }

    public fun is_builtin_scheme(scheme: u64): bool {
        scheme == ed25519::scheme() || scheme == multi_ed25519_validator::scheme() || scheme == ecdsa_k1::scheme() || scheme == ecdsa_k1_recoverable::scheme() || scheme == schnorr::scheme()
    }
}