// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// WebAuthn validator implementation (P-256 / secp256r1 or RS256 / RSASSA-PKCS1-V1_5)
///
/// Payload layout (see docs/dev-guide/webauthn_validator.md Section 3.1):
/// ```
/// ???????????????????????????????????????????????????????????????????????????
/// ? 1 B    ? 64/* B     ? 33/* B    ? 4 B + *           ? *                 ?
/// ? scheme ? signature  ? publicKey ? authenticatorData ? clientDataJSON    ?
/// ???????????????????????????????????????????????????????????????????????????
///
/// After the fixed-length fields we encode `authenticatorData` length as a
/// 4-byte big-endian unsigned integer so that the boundary between
/// `authenticatorData` and `clientDataJSON` can be determined on-chain.
///
/// The validator reconstructs the message as
///     authenticatorData || SHA-256(clientDataJSON)
/// and verifies it with the provided signature and compressed P-256 public key or RS256 public key modulus.
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
    use rooch_framework::rs256;
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
        public_key: vector<u8>,       // 33 B compressed P-256 or above 2048 bits RSASSA-PKCS1-v1_5 public key modulus
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

        // Scheme condition
        if (scheme == session_key::signature_scheme_ecdsar1()) { // ECDSA R1
            // Length sanity checks for ECDSA R1
            assert!(vector::length(&signature) == ecdsa_r1::raw_signature_length(), auth_validator::error_validate_invalid_authenticator());
            assert!(vector::length(&public_key) == ecdsa_r1::public_key_length(), auth_validator::error_validate_invalid_authenticator());
            // ---------- 2. Reconstruct message ----------
            let cd_hash = hash::sha2_256(client_data_json);
            let msg = authenticator_data;
            vector::append(&mut msg, cd_hash);

            // ---------- 3. Verify for ECDSA R1 ----------
            assert!(ecdsa_r1::verify(&signature, &public_key, &msg), auth_validator::error_validate_invalid_authenticator());

            let client_data = json::from_json<ClientData>(client_data_json);
            let challenge = client_data.challenge;
            let tx_hash_in_client_data = base64::decode(string::bytes(&challenge));
            let tx_hash = tx_context::tx_hash();
            assert!(tx_hash_in_client_data == tx_hash, auth_validator::error_validate_invalid_authenticator());

            // ---------- 4. Return auth_key for ECDSA R1 ----------
            session_key::secp256r1_public_key_to_authentication_key(&public_key)
        } else if (scheme == session_key::signature_scheme_rs256()) { // RS256
            // Length sanity checks for RS256 assuming public key is modulus
            assert!(vector::length(&signature) == vector::length(&public_key) * 8 / 8, auth_validator::error_validate_invalid_authenticator()); // cast to bits and to bytes
            assert!(vector::length(&public_key) >= rs256::rsassa_pkcs1_v1_5_minimum_modulus_length() / 8, auth_validator::error_validate_invalid_authenticator()); // cast to bytes first
            // ---------- 2. Reconstruct message ----------
            let cd_hash = hash::sha2_256(client_data_json);
            let msg = authenticator_data;
            vector::append(&mut msg, cd_hash);

            // ---------- 3. Verify for RS256 ----------
            let default_public_exponent = x"10001" // use a default public exponent value 65537 from rsa's author code, otherwise it won't work
            assert!(rs256::verify(&signature, &public_key, &default_public_exponent, &msg), auth_validator::error_validate_invalid_authenticator());

            let client_data = json::from_json<ClientData>(client_data_json);
            let challenge = client_data.challenge;
            let tx_hash_in_client_data = base64::decode(string::bytes(&challenge));
            let tx_hash = tx_context::tx_hash();
            assert!(tx_hash_in_client_data == tx_hash, auth_validator::error_validate_invalid_authenticator());

            // ---------- 4. Return auth_key for RS256 ----------
            session_key::rs256_public_key_to_authentication_key(public_key, default_public_exponent)
        } else { // Unsupported
            abort auth_validator::error_validate_invalid_authenticator()
        };
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
    fun make_test_payload_ecdsa_r1(): vector<u8> {
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
    fun test_validate_fails_invalid_sig_ecdsa_r1() {
        // The signature is zero so verification must fail with ErrorValidateInvalidAuthenticator (1010)
        let payload = make_test_payload_ecdsa_r1();
        validate(payload);
    }

    #[test_only]
    fun make_test_payload_rs256(): vector<u8> {
        let scheme = session_key::signature_scheme_rs256();
        let signature = x"634334686955506f6a394565746467747633684638304547726875425f5f647a45526174305846396732567451677239504a627533584f695a6a35525a6d683741417548496d3442682d3051635f6c4635594b745f4f3857324670356a756a4762647339754a6462463943554172377431646e5a634163516a624b42594e58344241796e5246646975422d2d665f6e5a4c67726e62795479577a4f373576524b356836784241724c4941524e50766b536a7451424d486c62314c30375165374b304761725a526d425f65534e393338334c634f4c6e365f644f2d2d786931326a7a44777573432d654f6b485745737174465a45536336426649376e6f4f507176684a317068436e7657683649655949327739514f5945556970555449386e70364c626747593946733938727156743541584c4968576b5779776c566d7456724270306967634e5f496f7970476c555051476537375341";
        let public_key = x"6f66675743754c6a7962526c7a6f30745a574a6a4e697553666234703466416b645f77574a6379516f54626a69396b306c385732366d50646478486d664851702d5661772d347150434a726353326d4a504d457a5031507430426d346434516c4c2d7952542d534664326c5a532d7043674e4d734431575f5970525045774f5776473662333236393072326a5a3437736f4d5a6f3977477a6a625f374f4d67304c4f4c2d62536636336b7061534853586e6453357a357265784d6462425955734c4139652d4b584264514f532d55546f37575442454d61325232436170486736363578736d7464564d544251593475445a6c7876623371436f355a774b68396b47344c54365f493549686c4a48376147687978584676554b2d44574e6d6f756446384e41636f395f68396961474e6a387132657468466b4d4c7339316b7a6b3250416344545739676235346834465257797558706f51";
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
    fun test_validate_fails_invalid_sig_rs256() {
        // The signature is zero so verification must fail with ErrorValidateInvalidAuthenticator (1010)
        let payload = make_test_payload_rs256();
        validate(payload);
    }

    #[test_only]
    public fun validate_auth_payload_for_test(auth_payload: &vector<u8>): vector<u8> {
        validate_internal(auth_payload)
    }

    #[test_only]
    public fun create_auth_payload_for_test_ecdsa_r1(signature: vector<u8>, public_key: vector<u8>, authenticator_data: vector<u8>, client_data_json: vector<u8>): WebauthnAuthPayload {
        let scheme = session_key::signature_scheme_ecdsar1();
        WebauthnAuthPayload {
            scheme,
            signature,
            public_key,
            authenticator_data,
            client_data_json,
        }
    }

    #[test_only]
    public fun create_auth_payload_for_test_rs256(signature: vector<u8>, public_key: vector<u8>, authenticator_data: vector<u8>, client_data_json: vector<u8>): WebauthnAuthPayload {
        let scheme = session_key::signature_scheme_rs256();
        WebauthnAuthPayload {
            scheme,
            signature,
            public_key,
            authenticator_data,
            client_data_json,
        }
    }
}
