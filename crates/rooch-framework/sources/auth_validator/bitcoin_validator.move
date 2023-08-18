/// This module implements bitcoin validator with the ECDSA signature over Secp256k1 crypto scheme.
module rooch_framework::bitcoin_validator {

    use std::error;
    use std::vector;
    use std::option::{Self, Option};
    use std::signer;
    use moveos_std::storage_context::{Self, StorageContext};
    use rooch_framework::account_authentication;
    use rooch_framework::ecdsa_k1;
    use rooch_framework::auth_validator;
    use rooch_framework::bitcoin_address::{Self, BTCAddress};

    /// there defines scheme for each blockchain
    const SCHEME_BITCOIN: u64 = 2;

    /// error code
    const EInvalidPublicKeyLength: u64 = 0;

    struct BitcoinValidator has store, drop {}

    public fun scheme(): u64 {
        SCHEME_BITCOIN
    }

    /// `rotate_authentication_key_entry` only supports rotating authentication key to a Bitcoin legacy address
    /// becuase ecdsa k1 scheme only supports key generation of 33-bytes compressed public key at this time.
    public entry fun rotate_authentication_key_entry(
        ctx: &mut StorageContext,
        account: &signer,
        public_key: vector<u8>,
        decimal_prefix_or_version: u8,
    ) {
        // compare newly passed public key with Bitcoin public key length to ensure it's compatible
        assert!(
            vector::length(&public_key) == ecdsa_k1::public_key_length()
            || vector::length(&public_key) == 20 // TODO support key generation of 20-bytes public key for Bitcoin bech32 addresses
            || vector::length(&public_key) == 32, // TODO support key generation of 32-bytes public key for Bitcoin bech32 addresses
            error::invalid_argument(EInvalidPublicKeyLength)
        );

        // User can rotate the authentication key arbitrarily, so we do not need to check the new public key with the account address.
        let authentication_key = public_key_to_authentication_key(public_key, decimal_prefix_or_version);
        let account_addr = signer::address_of(account);
        rotate_authentication_key(ctx, account_addr, authentication_key);
    }

    fun rotate_authentication_key(ctx: &mut StorageContext, account_addr: address, authentication_key: vector<u8>) {
        account_authentication::rotate_authentication_key<BitcoinValidator>(ctx, account_addr, authentication_key);
    }

    public entry fun remove_authentication_key_entry(ctx: &mut StorageContext, account: &signer) {
        account_authentication::remove_authentication_key<BitcoinValidator>(ctx, signer::address_of(account));
    }

    /// Get the authentication key of the given authenticator from authenticator_payload.
    public fun get_authentication_key_from_authenticator_payload(authenticator_payload: &vector<u8>, decimal_prefix_or_version: u8): vector<u8> {
        let public_key = ecdsa_k1::get_public_key_from_authenticator_payload(authenticator_payload);
        let addr = public_key_to_address(public_key, decimal_prefix_or_version);
        bitcoin_address::into_bytes(addr)
    }

    public fun public_key_to_address(public_key: vector<u8>, decimal_prefix_or_version: u8): BTCAddress {
        // Determine the public key length, 33-bytes for a legacy address and 32- and 20-bytes for a bech32 address.
        if (vector::length(&public_key) == ecdsa_k1::public_key_length()) {
            let decimal_prefix = decimal_prefix_or_version;
            bitcoin_address::new_legacy(&public_key, decimal_prefix)
        } else {
            let version = decimal_prefix_or_version;
            bitcoin_address::new_bech32(&public_key, version)
        }
    }

    /// Get the authentication key of the given public key.
    public fun public_key_to_authentication_key(public_key: vector<u8>, decimal_prefix_or_version: u8): vector<u8> {
        let addr = public_key_to_address(public_key, decimal_prefix_or_version);
        bitcoin_address::into_bytes(addr)
    }

    /// Get the authentication key option of the given account.
    public fun get_authentication_key_option_from_account(ctx: &StorageContext, addr: address): Option<vector<u8>> {
        account_authentication::get_authentication_key<BitcoinValidator>(ctx, addr)
    }

    /// The authentication key exists in account or not.
    public fun is_authentication_key_in_account(ctx: &StorageContext, addr: address): bool {
        option::is_some(&get_authentication_key_option_from_account(ctx, addr))
    }

    /// Extract the authentication key of the authentication key option.
    public fun get_authentication_key_from_account(ctx: &StorageContext, addr: address): vector<u8> {
        option::extract(&mut get_authentication_key_option_from_account(ctx, addr))
    }

    /// Only validate the authenticator's signature.
    public fun validate_signature(authenticator_payload: &vector<u8>, tx_hash: &vector<u8>) {
        assert!(
            ecdsa_k1::verify(
                &ecdsa_k1::get_signature_from_authenticator_payload(authenticator_payload),
                &ecdsa_k1::get_public_key_from_authenticator_payload(authenticator_payload),
                tx_hash,
                ecdsa_k1::sha256()
            ),
            auth_validator::error_invalid_authenticator()
        );
    }

    public fun validate(ctx: &StorageContext, authenticator_payload: vector<u8>) {
        let tx_hash = storage_context::tx_hash(ctx);
        validate_signature(&authenticator_payload, &tx_hash);

        // TODO compare the auth_key from the payload with the auth_key from the account
    }

    fun pre_execute(
        _ctx: &mut StorageContext,
    ) {}

    fun post_execute(
        ctx: &mut StorageContext,
    ) {
        let account_addr = storage_context::sender(ctx);
        if (is_authentication_key_in_account(ctx, account_addr)) {
            let auth_key_in_account = get_authentication_key_from_account(ctx, account_addr);
            std::debug::print(&auth_key_in_account);
        }
    }

    // this test ensures that the Bitcoin public_key_to_address function is compatible with the one in the rust code
    #[test]
    fun test_public_key_to_address() {
        use rooch_framework::hash;
        let public_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f";

        // P2PKH address
        let addr = public_key_to_address(public_key, 0);
        let address_bytes = bitcoin_address::into_bytes(addr);
        let p2pkh_address = b"1C6Rc3w25VHud3dLDamutaqfKWqhrLRTaD";
        assert!(address_bytes == p2pkh_address, 1000);

        // P2SH address
        let addr = public_key_to_address(public_key, 5);
        let address_bytes = bitcoin_address::into_bytes(addr);
        let p2sh_address = b"3DedZ8SErqfunkjqnv8Pta1MKgEuHi22W5";
        assert!(address_bytes == p2sh_address, 1001);

        // P2WPKH address
        let hash160_public_key = hash::ripemd160(&hash::sha2_256(public_key));
        let addr = public_key_to_address(hash160_public_key, 0);
        let address_bytes = bitcoin_address::into_bytes(addr);
        let p2wpkh_address = b"bc1q0xcqpzrky6eff2g52qdye53xkk9jxkvrh6yhyw";
        assert!(address_bytes == p2wpkh_address, 1002);

        // P2WSH address
        let wscript_hash_public_key = x"6f1b349d7fed5240ad719948529e8b06abf038438f9b523820489375af513a3f";
        let addr = public_key_to_address(wscript_hash_public_key, 0);
        let address_bytes = bitcoin_address::into_bytes(addr);
        let p2wsh_address = b"bc1qdudnf8tla4fyptt3n9y9985tq64lqwzr37d4ywpqfzfhtt638glsqaednx";
        assert!(address_bytes == p2wsh_address, 1003);

        // P2TR address
        let tweaked_schnorr_serialized_x_only_public_key = x"8c5db7f797196d6edc4dd7df6048f4ea6b883a6af6af032342088f436543790f";
        let addr = public_key_to_address(tweaked_schnorr_serialized_x_only_public_key, 1);
        let address_bytes = bitcoin_address::into_bytes(addr);
        let p2tr_address = b"bc1p33wm0auhr9kkahzd6l0kqj85af4cswn276hsxg6zpz85xe2r0y8syx4e5t";
        assert!(address_bytes == p2tr_address, 1004);
    }
}
