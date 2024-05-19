// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::auth_payload {

    use moveos_std::bcs;

    #[data_struct]
    struct AuthPayload has copy, store, drop {
        // Message sin
        sign: vector<u8>,
        // Some wallets add magic prefixes, such as unisat adding 'Bitcoin Signed Message:\n'
        sign_info_prefix: vector<u8>,
        // Description of a user-defined signature
        sign_info: vector<u8>,
        // Public key of address
        public_key: vector<u8>,
        // Wallet address
        from_address: vector<u8>
    }

    public fun from_bytes(bytes: vector<u8>): AuthPayload {
        bcs::from_bytes<AuthPayload>(bytes)
    }

    public fun sign(payload: AuthPayload): vector<u8> {
        payload.sign
    }

    public fun sign_info_prefix(payload: AuthPayload): vector<u8> {
        payload.sign_info_prefix
    }

    public fun sign_info(payload: AuthPayload): vector<u8> {
        payload.sign_info
    }

    public fun public_key(payload: AuthPayload): vector<u8> {
        payload.public_key
    }

    public fun from_address(payload: AuthPayload): vector<u8> {
        payload.from_address
    }

}