// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::auth_payload {

    use std::string::String;
    use std::vector;
    use moveos_std::bcs;
    use moveos_std::hex;

    const MessgaeInfoPrefix: vector<u8> = b"Rooch Transaction:\n";

    const ErrorInvalidSignature: u64 = 1;

    #[data_struct]
    struct AuthPayload has copy, store, drop {
        // Message signature
        signature: vector<u8>,
        // Some wallets add magic prefixes, such as unisat adding 'Bitcoin Signed Message:\n'
        message_prefix: vector<u8>,
        // Description of a user-defined signature
        message_info: vector<u8>,
        // Public key of address
        public_key: vector<u8>,
        // Wallet address
        from_address: String
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
        assert!(starts_with(&self.message_info, &MessgaeInfoPrefix), ErrorInvalidSignature);

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

    public fun signature(payload: AuthPayload): vector<u8> {
        payload.signature
    }

    public fun message_prefix(payload: AuthPayload): vector<u8> {
        payload.message_prefix
    }

    public fun message_info(payload: AuthPayload): vector<u8> {
        payload.message_info
    }

    public fun public_key(payload: AuthPayload): vector<u8> {
        payload.public_key
    }

    public fun from_address(payload: AuthPayload): String {
        payload.from_address
    }
}