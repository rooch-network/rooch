// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::webauthn_validator_test {
    use std::string;
    use moveos_std::bcs;
    use moveos_std::tx_context;
    use moveos_std::json;
    use moveos_std::base64;
    use rooch_framework::webauthn_validator::{Self, WebauthnAuthPayload, ClientData};


    // Helper function to create test payload
    fun create_test_payload(
        signature: vector<u8>,
        public_key: vector<u8>,
        authenticator_data: vector<u8>,
        client_data_json: vector<u8>
    ): WebauthnAuthPayload {
        let payload = webauthn_validator::create_auth_payload_for_test(signature, public_key, authenticator_data, client_data_json);
        payload
    }

    fun get_real_webauthn_test_payload(): WebauthnAuthPayload {
        let payload_bytes = x"0240d198a3c70a3cf6eb4741136b101cc68b451e21d07958c3582a481c78eb085928189ee206c6eb67de2e3b01870208a2d38794bfd6bb1ddd18f0599fabaf3a98b82103faabaa02f39bae7cf872cbaf009ca1676ed0ced644d991a91f293bc4ccf7c1f52549960de5880e8c687434170f6476605b8fe4aeb9a28632c7995cf3ba831d97631d0000000086017b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a2238724377666f4b4375616c6e53387638317641466f4c566937495934674b68525456635f562d6b50575345222c226f726967696e223a22687474703a2f2f6c6f63616c686f73743a33303030222c2263726f73734f726967696e223a66616c73657d";
        let payload = bcs::from_bytes(payload_bytes);
        payload
    }

    #[test]
    fun test_webauthn_real_data_validation(){
        let payload = get_real_webauthn_test_payload();
        let (scheme, signature, public_key, authenticator_data, client_data_json) = webauthn_validator::unwrap_webauthn_auth_payload(payload);
        let client_data = json::from_json<ClientData>(client_data_json);
        let (challenge, origin, type) = webauthn_validator::unwrap_client_data(client_data);
        std::debug::print(&scheme);
        std::debug::print(&challenge);
        std::debug::print(&origin);
        std::debug::print(&type);
        std::debug::print(&signature);
        std::debug::print(&public_key);
        std::debug::print(&authenticator_data);
        tx_context::set_ctx_tx_hash_for_testing(base64::decode(string::bytes(&challenge)));
        let _auth_key = webauthn_validator::validate_auth_payload_for_test(&bcs::to_bytes(&payload));
    }

}