/// This module implements the native validator scheme.
module rooch_framework::native_validator {

    use std::error;
    use std::vector;
    use std::option;
    use std::signer;
    use moveos_std::storage_context::{Self, StorageContext};
    use rooch_framework::hash;
    use rooch_framework::account_authentication;
    use rooch_framework::ed25519;
    use rooch_framework::auth_validator;

    /// there defines scheme for each blockchain
    const NATIVE_SCHEME: u64 = 0;

    /// error code
    const EInvalidPublicKeyLength: u64 = 0;

    struct NativeValidator has store, drop {}

    public fun scheme(): u64 {
        NATIVE_SCHEME
    }

    public entry fun rotate_authentication_key_entry(
        ctx: &mut StorageContext,
        account: &signer,
        public_key: vector<u8>
    ) {
        // compare newly passed public key with Rooch public key length to ensure it's compatible
        assert!(
            vector::length(&public_key) == ed25519::public_key_length(),
            error::invalid_argument(EInvalidPublicKeyLength)
        );

        // User can rotate the authentication key arbitrarily, so we do not need to check the new public key with the account address.
        let authentication_key = public_key_to_authentication_key(public_key);
        let account_addr = signer::address_of(account);
        rotate_authentication_key(ctx, account_addr, authentication_key);
    }

    fun rotate_authentication_key(ctx: &mut StorageContext, account_addr: address, authentication_key: vector<u8>) {
        account_authentication::rotate_authentication_key<NativeValidator>(ctx, account_addr, authentication_key);
    }

    public entry fun remove_authentication_key_entry(ctx: &mut StorageContext, account: &signer) {
        account_authentication::remove_authentication_key<NativeValidator>(ctx, signer::address_of(account));
    }

    /// Get the authentication key of the given authenticator from authenticator_payload.
    public fun get_authentication_key_from_authenticator_payload(authenticator_payload: &vector<u8>): vector<u8> {
        let public_key = ed25519::get_public_key_from_authenticator_payload(authenticator_payload);
        let addr = public_key_to_address(public_key);
        moveos_std::bcs::to_bytes(&addr)
    }

    public fun public_key_to_address(public_key: vector<u8>): address {
        moveos_std::bcs::to_address(public_key_to_authentication_key(public_key))
    }

    /// Get the authentication key of the given public key.
    public fun public_key_to_authentication_key(public_key: vector<u8>): vector<u8> {
        let bytes = vector::singleton((scheme() as u8));
        vector::append(&mut bytes, public_key);
        hash::blake2b256(&bytes)
    }

    /// Get the authentication key of the given account, if it not exist, return the account address as authentication key.
    public fun get_authentication_key_with_default(ctx: &StorageContext, addr: address): vector<u8> {
        let auth_key_option = account_authentication::get_authentication_key<NativeValidator>(ctx, addr);
        if (option::is_some(&auth_key_option)) {
            option::extract(&mut auth_key_option)
        }else {
            default_authentication_key(addr)
        }
    }

    public fun default_authentication_key(addr: address): vector<u8> {
        moveos_std::bcs::to_bytes(&addr)
    }

    /// Only validate the authenticator's signature.
    public fun validate_signature(authenticator_payload: &vector<u8>, tx_hash: &vector<u8>) {
        assert!(
            ed25519::verify(
                &ed25519::get_signature_from_authenticator_payload(authenticator_payload),
                &ed25519::get_public_key_from_authenticator_payload(authenticator_payload),
                tx_hash
            ),
            auth_validator::error_invalid_authenticator()
        );
    }

    public fun validate(ctx: &StorageContext, authenticator_payload: vector<u8>) {
        let tx_hash = storage_context::tx_hash(ctx);
        validate_signature(&authenticator_payload, &tx_hash);

        let auth_key_from_authenticator_payload = get_authentication_key_from_authenticator_payload(&authenticator_payload);
        let auth_key_in_account = get_authentication_key_with_default(ctx, storage_context::sender(ctx));
        assert!(
            auth_key_in_account == auth_key_from_authenticator_payload,
            auth_validator::error_invalid_account_auth_key()
        );
    }

    fun pre_execute(
        _ctx: &mut StorageContext,
    ) {}

    fun post_execute(
        ctx: &mut StorageContext,
    ) {
        let account_addr = storage_context::sender(ctx);
        let auth_key_option = account_authentication::get_authentication_key<NativeValidator>(ctx, account_addr);
        // If the account does not have an authentication key, set the account address as the authentication key after the first transaction is executed.
        if (option::is_none(&auth_key_option)) {
            let authentication_key = default_authentication_key(account_addr);
            rotate_authentication_key(ctx, account_addr, authentication_key);
        }
    }

    // this test ensures that the Rooch native public_key_to_address function is compatible with the one in the rust code
    #[test]
    fun test_public_key_to_address() {
        let public_key = x"3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29";
        let addr = public_key_to_address(public_key);
        assert!(addr == @0x7a1378aafadef8ce743b72e8b248295c8f61c102c94040161146ea4d51a182b6, 1000)
    }
}
