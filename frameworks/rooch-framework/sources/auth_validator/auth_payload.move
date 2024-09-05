// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::auth_payload {

    use std::string::String;
    use std::vector;
    use moveos_std::consensus_codec;
    use moveos_std::bcs;
    use moveos_std::hex;

    const MessagePrefix : vector<u8> = b"Bitcoin Signed Message:\n";
    const MessageInfoPrefix: vector<u8> = b"Rooch Transaction:\n";

    const ErrorInvalidSignature: u64 = 1;

    #[data_struct]
    struct AuthPayload has copy, store, drop {
        // Message signature
        signature: vector<u8>,
        // Some wallets add magic prefixes, such as unisat adding 'Bitcoin Signed Message:\n'
        message_prefix: vector<u8>,
        // Description of a user-defined signature, without the tx_hash hex
        message_info: vector<u8>,
        // Public key of address
        public_key: vector<u8>,
        // Wallet address
        from_address: String
    }

    #[data_struct]
    struct MultisignAuthPayload has copy, store, drop {
        // Message signature
        signatures: vector<vector<u8>>,
        // Some wallets add magic prefixes, such as unisat adding 'Bitcoin Signed Message:\n'
        message_prefix: vector<u8>,
        // Description of a user-defined signature, without the tx_hash hex
        message_info: vector<u8>,
        // Public key of address
        public_keys: vector<vector<u8>>,
    }

    #[data_struct]
    struct SignData has copy, drop {
        message_prefix: vector<u8>,
        // Description of a user-defined signature, include the tx_hash hex
        message_info: vector<u8>,
    }

    public fun new_sign_data(message_prefix: vector<u8>, message_info: vector<u8>): SignData {
        SignData {
            message_prefix,
            message_info
        }
    }

    public fun from_bytes(bytes: vector<u8>): AuthPayload {
        bcs::from_bytes<AuthPayload>(bytes)
    }

    fun starts_with(haystack: &vector<u8>, needle: &vector<u8>): bool {
        let haystack_len = vector::length(haystack);
        let needle_len = vector::length(needle);

        if (needle_len > haystack_len) {
            return false
        };

        let i = 0;
        while (i < needle_len) {
            if (vector::borrow(haystack, i) != vector::borrow(needle, i)) {
                return false
            };
            i = i + 1;
        };

        true
    }

    public fun encode_full_message(self: &AuthPayload, tx_hash: vector<u8>): vector<u8> {
        // The signature description must start with Rooch Transaction:\n
        assert!(starts_with(&self.message_info, &MessageInfoPrefix), ErrorInvalidSignature);
        let message_prefix = self.message_prefix;
        let full_message = if (message_prefix != MessagePrefix) {
            // For compatibility with the old version
            // The old version contains length information, so it needs to be removed in the future
            // After the js sdk is update, we can remove this branch
            encode_full_message_legacy(self, tx_hash)
        }else{
            encode_full_message_consensus(self, tx_hash)
        };
        full_message
    }
    
    // The bitcoin consensus encoding the full message, which has `1a` prefix than the legacy version
    fun encode_full_message_consensus(self: &AuthPayload, tx_hash: vector<u8>): vector<u8> {
        let tx_hex = hex::encode(tx_hash);
        let message_info = self.message_info;
        vector::append(&mut message_info, tx_hex);
        let sign_data = SignData {
            message_prefix: self.message_prefix,
            message_info: message_info,
        };

        conseneus_encode_sign_data(&sign_data)
    }

    fun encode_full_message_legacy(self: &AuthPayload, tx_hash: vector<u8>): vector<u8> {
        
        let tx_hex = hex::encode(tx_hash);
        let message_prefix_len = vector::length(&self.message_prefix);

        let full_message = vector<u8>[];
        if (message_prefix_len > 0) {
            vector::append(&mut full_message, self.message_prefix);
        };

        vector::append(&mut full_message, self.message_info);
        vector::append(&mut full_message, tx_hex);

        full_message
    }

    public fun signature(payload: &AuthPayload): vector<u8> {
        payload.signature
    }

    public fun message_prefix(payload: &AuthPayload): vector<u8> {
        payload.message_prefix
    }

    public fun message_info(payload: &AuthPayload): vector<u8> {
        payload.message_info
    }

    public fun public_key(payload: &AuthPayload): vector<u8> {
        payload.public_key
    }

    public fun from_address(payload: &AuthPayload): String {
        payload.from_address
    }

    // ======= MultisignAuthPayload =======

    public fun multisign_from_bytes(bytes: vector<u8>): MultisignAuthPayload {
        bcs::from_bytes<MultisignAuthPayload>(bytes)
    }

    public fun multisign_signatures(payload: &MultisignAuthPayload): &vector<vector<u8>> {
        &payload.signatures
    }

    public fun multisign_message_prefix(payload: &MultisignAuthPayload): &vector<u8> {
        &payload.message_prefix
    }

    public fun multisign_message_info(payload: &MultisignAuthPayload): &vector<u8> {
        &payload.message_info
    }

    public fun multisign_public_keys(payload: &MultisignAuthPayload): &vector<vector<u8>> {
        &payload.public_keys
    }

    public fun multisign_encode_full_message(self: &MultisignAuthPayload, tx_hash: vector<u8>): vector<u8> {
        let tx_hex = hex::encode(tx_hash);
        let message_info = self.message_info;
        vector::append(&mut message_info, tx_hex);
        let sign_data = SignData {
            message_prefix: self.message_prefix,
            message_info: message_info,
        };
        conseneus_encode_sign_data(&sign_data)
    }

    fun conseneus_encode_sign_data(sign_data: &SignData): vector<u8> {
        let encoder = consensus_codec::encoder();
        consensus_codec::emit_var_slice(&mut encoder, sign_data.message_prefix);
        consensus_codec::emit_var_slice(&mut encoder, sign_data.message_info);
        consensus_codec::unpack_encoder(encoder)
    }
}