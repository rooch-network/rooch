module rooch_framework::builtin_validators{

    use moveos_std::storage_context::StorageContext;
    use rooch_framework::auth_validator_registry;
    use rooch_framework::ed25519_validator;
    use rooch_framework::multi_ed25519_validator;
    use rooch_framework::ecdsa_k1_validator;
    use rooch_framework::ecdsa_k1_recoverable_validator;
    use rooch_framework::schnorr_validator;

    friend rooch_framework::genesis;

    const E_GENESIS_INIT: u64 = 1;

    public(friend) fun genesis_init(ctx: &mut StorageContext, _genesis_account: &signer){
        //SCHEME_ED25519: u64 = 0;
        let id = auth_validator_registry::register_internal<ed25519_validator::Ed25519Validator>(ctx);
        assert!(id == ed25519_validator::scheme(), std::error::internal(E_GENESIS_INIT));
        //SCHEME_MULTIED25519: u64 = 1;
        let id = auth_validator_registry::register_internal<multi_ed25519_validator::MultiEd25519Validator>(ctx);
        assert!(id == multi_ed25519_validator::scheme(), std::error::internal(E_GENESIS_INIT));
        //SCHEME_ECDSA: u64 = 2;
        let id = auth_validator_registry::register_internal<ecdsa_k1_validator::EcdsaK1Validator>(ctx);
        assert!(id == ecdsa_k1_validator::scheme(), std::error::internal(E_GENESIS_INIT));
        //SCHEME_ECDSA_RECOVERABLE: u64 = 3;
        let id = auth_validator_registry::register_internal<ecdsa_k1_recoverable_validator::EcdsaK1RecoverableValidator>(ctx);
        assert!(id == ecdsa_k1_recoverable_validator::scheme(), std::error::internal(E_GENESIS_INIT));
        //SCHEME_SCHNORR: u64 = 4;
        let id = auth_validator_registry::register_internal<schnorr_validator::SchnorrValidator>(ctx);
        assert!(id == schnorr_validator::scheme(), std::error::internal(E_GENESIS_INIT));
    }

    public fun is_builtin_scheme(scheme: u64): bool {
        scheme == ed25519_validator::scheme() || scheme == multi_ed25519_validator::scheme() || scheme == ecdsa_k1_validator::scheme() || scheme == ecdsa_k1_recoverable_validator::scheme() || scheme == schnorr_validator::scheme()
    }
}