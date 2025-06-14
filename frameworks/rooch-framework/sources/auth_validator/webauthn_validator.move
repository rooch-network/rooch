// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// WebAuthn validator implementation (P-256 / secp256r1)
///
/// Payload layout (see docs/dev-guide/webauthn_validator.md Section 3.1):
/// ```
/// ???????????????????????????????????????????????????????????????????????????
/// ? 1 B    ? 64 B       ? 33 B      ? 4 B + *           ? *                 ?
/// ? scheme ? signature  ? publicKey ? authenticatorData ? clientDataJSON    ?
/// ???????????????????????????????????????????????????????????????????????????
///
/// After the fixed-length fields we encode `authenticatorData` length as a
/// 4-byte big-endian unsigned integer so that the boundary between
/// `authenticatorData` and `clientDataJSON` can be determined on-chain.
///
/// The validator reconstructs the message as
///     authenticatorData || SHA-256(clientDataJSON)
/// and verifies it with the provided signature and compressed P-256 public key.
///
/// On success the corresponding session authentication key is returned.
module rooch_framework::webauthn_validator {
    use std::vector;
    use std::string::{Self, String};
    use moveos_std::hash;
    use moveos_std::tx_context;
    use moveos_std::json;
    use rooch_framework::did;
    use rooch_framework::ecdsa_r1;
    use rooch_framework::session_key;
    use rooch_framework::auth_validator;
    use moveos_std::bcs;
    use std::option;
    use moveos_std::base64;

    friend rooch_framework::transaction_validator;
    friend rooch_framework::builtin_validators;

    /// Identifier reserved for the WebAuthn validator. Must stay in sync with
    /// `builtin_validators.move`.
    const WEBAUTHN_AUTH_VALIDATOR_ID: u64 = 3;


    struct WebauthnValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        WEBAUTHN_AUTH_VALIDATOR_ID
    }

    #[data_struct]
    /// BCS-serialised payload sent by the browser / SDK. This avoids manual
    /// offset parsing on-chain.
    struct WebauthnAuthPayload has copy, store, drop {
        scheme: u8,
        signature: vector<u8>,        // 64 B (r||s)
        public_key: vector<u8>,       // 33 B compressed P-256
        authenticator_data: vector<u8>,
        client_data_json: vector<u8>,
    }

    public fun unwrap_webauthn_auth_payload(payload: WebauthnAuthPayload): (u8, vector<u8>, vector<u8>, vector<u8>, vector<u8>) {
        let WebauthnAuthPayload {
            scheme,
            signature,
            public_key,
            authenticator_data,
            client_data_json,
        } = payload;
        (scheme, signature, public_key, authenticator_data, client_data_json)
    }

    #[data_struct]
    struct ClientData has copy, store, drop {
        challenge: String,
        origin: String,
        type: String,
    }

    public fun unwrap_client_data(client_data: ClientData): (String, String, String) {
        let ClientData {
            challenge,
            origin,
            type,
        } = client_data;
        (challenge, origin, type)
    }

    /// Validate the incoming authenticator payload and return the derived authentication key
    public(friend) fun validate(authenticator_payload: vector<u8>): vector<u8> {
        let auth_key = validate_internal(&authenticator_payload);

        let sender = tx_context::sender();

        // 1. Session key exists under account
        assert!(session_key::contains_session_key(sender, auth_key), auth_validator::error_validate_invalid_authenticator());

        // 2. Session key is also declared in the sender's DID document
        assert!(verify_did_auth_key(sender, &auth_key), auth_validator::error_validate_invalid_authenticator());

        auth_key
    }

    ///////////////////////////////////////////////////////////////////////////
    //  Internal helpers                                                    //
    ///////////////////////////////////////////////////////////////////////////

    fun validate_internal(bytes: &vector<u8>): vector<u8> {
        let payload = bcs::from_bytes<WebauthnAuthPayload>(*bytes);

        let WebauthnAuthPayload {
            scheme,
            signature,
            public_key,
            authenticator_data,
            client_data_json,
        } = payload;

        // Scheme check
        assert!(scheme == session_key::signature_scheme_ecdsar1(), auth_validator::error_validate_invalid_authenticator());

        // Length sanity checks
        assert!(vector::length(&signature) == ecdsa_r1::raw_signature_length(), auth_validator::error_validate_invalid_authenticator());
        assert!(vector::length(&public_key) == ecdsa_r1::public_key_length(), auth_validator::error_validate_invalid_authenticator());

        // ---------- 2. Reconstruct message ----------
        let cd_hash = hash::sha2_256(client_data_json);
        let msg = authenticator_data;
        vector::append(&mut msg, cd_hash);

        // ---------- 3. Verify ----------
        assert!(ecdsa_r1::verify(&signature, &public_key, &msg), auth_validator::error_validate_invalid_authenticator());

        let client_data = json::from_json<ClientData>(client_data_json);
        let challenge = client_data.challenge;
        let tx_hash_in_client_data = base64::decode(string::bytes(&challenge));
        let tx_hash = tx_context::tx_hash();
        assert!(tx_hash_in_client_data == tx_hash, auth_validator::error_validate_invalid_authenticator());

        // ---------- 4. Return auth_key ----------
        session_key::secp256r1_public_key_to_authentication_key(&public_key)
    }

    /// Verify that `auth_key` is linked to one of sender's authentication verification methods.
    fun verify_did_auth_key(sender: address, auth_key: &vector<u8>): bool {
        let did_exists = did::exists_did_for_address(sender);
        assert!(did_exists, auth_validator::error_validate_invalid_authenticator());
        let did_doc = did::get_did_document_by_address(sender);
        let vm_opt = did::find_verification_method_by_session_key(did_doc, auth_key);
        let find_vm = option::is_some(&vm_opt);
        find_vm
    }

    ///////////////////////////////////////////////////////////////////////////
    //  Tests                                                               //
    ///////////////////////////////////////////////////////////////////////////

    #[test_only]
    fun make_test_payload(): vector<u8> {
        let scheme = session_key::signature_scheme_ecdsar1();
        let signature = x"74133905657c1992d8d6bd72ffa7ccf8d2adf3e4a3ca25f8dc8eec175752cb5a40459f71b549a25cba3cddf4157e946bbff7b18fc82774e9c4c54e362b97ccb5";
        let public_key = x"0258a618066814098f8ddb3cbde73838b59028d843958031e50be0a5f4b0a9796d";
        let authenticator_data = x"000000020000000100000002";
        let client_data_json = x"00000003000000030000000400000005";
        let webauthn_auth_payload = WebauthnAuthPayload {
            scheme,
            signature,
            public_key,
            authenticator_data,
            client_data_json,
        };
        bcs::to_bytes(&webauthn_auth_payload)
    }

    #[test]
    #[expected_failure(location=Self, abort_code = 1010)]
    fun test_validate_fails_invalid_sig() {
        // The signature is zero so verification must fail with ErrorValidateInvalidAuthenticator (1010)
        let payload = make_test_payload();
        validate(payload);
    }

    #[test_only]
    public fun validate_auth_payload_for_test(auth_payload: &vector<u8>): vector<u8> {
        validate_internal(auth_payload)
    }

    #[test_only]
    public fun create_auth_payload_for_test(signature: vector<u8>, public_key: vector<u8>, authenticator_data: vector<u8>, client_data_json: vector<u8>): WebauthnAuthPayload {
        let scheme = session_key::signature_scheme_ecdsar1();
        WebauthnAuthPayload {
            scheme,
            signature,
            public_key,
            authenticator_data,
            client_data_json,
        }
    }

}
