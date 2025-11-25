// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// WebAuthn validator implementation (DEPRECATED)
///
/// This validator has been deprecated. Please use did_validator (ID=4) with WebAuthnV0 envelope instead.
/// Migration: Use Authenticator.did(txHash, signer, vmFragment, SigningEnvelope.WebAuthnV0)
module rooch_framework::webauthn_validator {
    use std::string::String;

    friend rooch_framework::transaction_validator;
    friend rooch_framework::builtin_validators;

    /// Identifier reserved for the WebAuthn validator. Must stay in sync with
    /// `builtin_validators.move`.
    const WEBAUTHN_AUTH_VALIDATOR_ID: u64 = 3;

    /// Error code indicating this validator has been deprecated
    const ErrorValidatorDeprecated: u64 = 2001;

    struct WebauthnValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        WEBAUTHN_AUTH_VALIDATOR_ID
    }

    /// Validate the incoming authenticator payload (DEPRECATED)
    /// This function always aborts with ErrorValidatorDeprecated.
    /// Please migrate to did_validator (ID=4) with WebAuthnV0 envelope.
    public(friend) fun validate(_authenticator_payload: vector<u8>): vector<u8> {
        abort ErrorValidatorDeprecated
    }

    ///////////////////////////////////////////////////////////////////////////
    //  Deprecated structs and functions - kept for compatibility          //
    ///////////////////////////////////////////////////////////////////////////

    #[data_struct]
    /// BCS-serialised payload sent by the browser / SDK (DEPRECATED)
    /// This struct is kept for compatibility but should not be used.
    struct WebauthnAuthPayload has copy, store, drop {
        scheme: u8,
        signature: vector<u8>,        // 64 B (r||s)
        public_key: vector<u8>,       // 33 B compressed P-256
        authenticator_data: vector<u8>,
        client_data_json: vector<u8>,
    }

    /// Unwrap WebAuthn auth payload (DEPRECATED)
    /// This function is kept for compatibility but should not be used.
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
    /// Client data struct (DEPRECATED)
    /// This struct is kept for compatibility but should not be used.
    struct ClientData has copy, store, drop {
        challenge: String,
        origin: String,
        type: String,
    }

    /// Unwrap client data (DEPRECATED)
    /// This function is kept for compatibility but should not be used.
    public fun unwrap_client_data(client_data: ClientData): (String, String, String) {
        let ClientData {
            challenge,
            origin,
            type,
        } = client_data;
        (challenge, origin, type)
    }
}
