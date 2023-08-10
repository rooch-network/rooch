/// This module implements the ECDSA Recoverable over Secpk256k1 validator scheme.
module rooch_framework::ecdsa_k1_recoverable_validator {

    use std::error;
    use std::vector;
    use std::option;
    use std::signer;
    use moveos_std::storage_context::{Self, StorageContext};
    use rooch_framework::account_authentication;
    use rooch_framework::hash;
    use rooch_framework::ecdsa_k1_recoverable;
    use rooch_framework::auth_validator;

    const SCHEME_ECDSA_RECOVERABLE: u64 = 3;

    const V_ECDSA_RECOVERABLE_SCHEME_LENGTH: u64 = 1;
    const V_ECDSA_RECOVERABLE_PUBKEY_LENGTH: u64 = 33;
    const V_ECDSA_RECOVERABLE_SIG_LENGTH: u64 = 65;
    const V_ECDSA_RECOVERABLE_HASH_LENGTH: u64 = 1;
    /// Hash function name that are valid for ecrecover and verify.
    const KECCAK256: u8 = 0;
    const SHA256: u8 = 1;
    /// error code
    const EMalformedAccount: u64 = 1001;
    const EMalformedAuthenticationKey: u64 = 1002;

    struct EcdsaK1RecoverableValidator has store, drop {}

    public fun scheme(): u64 {
        SCHEME_ECDSA_RECOVERABLE
    }

    public entry fun rotate_authentication_key_entry<EcdsaK1RecoverableValidator>(
        ctx: &mut StorageContext,
        account: &signer,
        public_key: vector<u8>
    ) {
        // compare newly passed public key with ecdsa recoverable public key length to ensure it's compatible
        assert!(
            vector::length(&public_key) == V_ECDSA_RECOVERABLE_PUBKEY_LENGTH,
            error::invalid_argument(EMalformedAuthenticationKey)
        );

        // ensure that the ecdsa recoverable public key to address isn't matched with the ed25519 account address
        let account_addr = signer::address_of(account);
        let ecdsa_recoverable_addr = ecdsa_k1_recoverable_public_key_to_address(public_key);
        assert!(
            account_addr != ecdsa_recoverable_addr,
            error::invalid_argument(EMalformedAccount)
        );

        // serialize the address to an auth key and rotate it by calling rotate_authentication_key
        let ecdsa_k1_recoverable_authentication_key = moveos_std::bcs::to_bytes(&ecdsa_recoverable_addr);
        account_authentication::rotate_authentication_key<EcdsaK1RecoverableValidator>(
            ctx,
            account_addr,
            ecdsa_k1_recoverable_authentication_key
        );
    }

    public entry fun remove_authentication_key_entry<EcdsaK1RecoverableValidator>(ctx: &mut StorageContext, account: &signer) {
        let account_addr = signer::address_of(account);
        account_authentication::remove_authentication_key<EcdsaK1RecoverableValidator>(ctx, account_addr);
    }

    public fun ecdsa_k1_recoverable_public_key(authenticator_payload: &vector<u8>): vector<u8> {
        let public_key = vector::empty<u8>();
        let i = V_ECDSA_RECOVERABLE_SCHEME_LENGTH + V_ECDSA_RECOVERABLE_SIG_LENGTH;
        while (i < V_ECDSA_RECOVERABLE_SCHEME_LENGTH + V_ECDSA_RECOVERABLE_SIG_LENGTH + V_ECDSA_RECOVERABLE_PUBKEY_LENGTH) {
            let value = vector::borrow(authenticator_payload, i);
            vector::push_back(&mut public_key, *value);
            i = i + 1;
        };

        public_key
    }

    public fun ecdsa_k1_recoverable_signature(authenticator_payload: &vector<u8>): vector<u8> {
        let sign = vector::empty<u8>();
        let i = V_ECDSA_RECOVERABLE_SCHEME_LENGTH;
        while (i < V_ECDSA_RECOVERABLE_SIG_LENGTH + 1) {
            let value = vector::borrow(authenticator_payload, i);
            vector::push_back(&mut sign, *value);
            i = i + 1;
        };

        sign
    }

    /// Get the authentication key of the given authenticator.
    public fun ecdsa_k1_recoverable_authentication_key(authenticator_payload: &vector<u8>): vector<u8> {
        let public_key = ecdsa_k1_recoverable_public_key(authenticator_payload);
        let addr = ecdsa_k1_recoverable_public_key_to_address(public_key);
        moveos_std::bcs::to_bytes(&addr)
    }

    public fun ecdsa_k1_recoverable_public_key_to_address(public_key: vector<u8>): address {
        let bytes = vector::singleton((SCHEME_ECDSA_RECOVERABLE as u8));
        vector::append(&mut bytes, public_key);
        moveos_std::bcs::to_address(hash::blake2b256(&bytes))
    }

    public fun get_authentication_key(ctx: &StorageContext, addr: address): vector<u8> {
        let auth_key_option = account_authentication::get_authentication_key<EcdsaK1RecoverableValidator>(ctx, addr);
        if (option::is_some(&auth_key_option)) {
            option::extract(&mut auth_key_option)
        }else {
            //if AuthenticationKey does not exist, return addr as authentication key
            moveos_std::bcs::to_bytes(&addr)
        }
    }

    public fun validate(ctx: &StorageContext, authenticator_payload: vector<u8>) {
        // TODO handle non-ed25519 auth key and address relationship
        // let auth_key = ecdsa_k1_recoverable_authentication_key(&authenticator_payload);
        // let auth_key_in_account = get_authentication_key(ctx, storage_context::sender(ctx));
        // assert!(
        //    auth_key_in_account == auth_key,
        //    auth_validator::error_invalid_account_auth_key()
        // );
        assert!(
            ecdsa_k1_recoverable::verify(
                &ecdsa_k1_recoverable_signature(&authenticator_payload),
                &storage_context::tx_hash(ctx),
                SHA256, // KECCAK256:0, SHA256:1, TODO: The hash type may need to be passed through the authenticator
            ),
            auth_validator::error_invalid_authenticator()
        );
    }

    fun pre_execute(
        _ctx: &mut StorageContext,
    ) {}

    fun post_execute(
        _ctx: &mut StorageContext,
    ) {}

    // this test ensures that the ecdsa_k1_recoverable_public_key_to_address function is compatible with the one in the rust code
    #[test]
    fun test_ecdsa_k1_recoverable_public_key_to_address() {
        let public_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f";
        let addr = ecdsa_k1_recoverable_public_key_to_address(public_key);
        assert!(addr == @0x8c891976da9498ec1d3ff778a5d6c40c217d63cc8c48539c959f8b683eedf5a4, 1000);
    }
}