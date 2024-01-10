// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::auth_payload {

    use moveos_std::bcs;
    use rooch_framework::multichain_address;

    #[data_struct]
    struct AuthPayload has copy, store, drop {
        sign: vector<u8>,
        // Some wallets add magic prefixes, such as unisat adding 'Bitcoin Signed Message:\n'
        sign_info_prefix: vector<u8>,
        // Description of a user-defined signature
        sign_info: vector<u8>,
        public_key: vector<u8>,
        multi_address: vector<u8>,
        // TODO: use multi_address, currently, sdk does not support eth, so test use from_address
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

    public fun multi_address(payload: AuthPayload): multichain_address::MultiChainAddress {
        multichain_address::from_bytes(payload.multi_address)
    }

    public fun from_address(payload: AuthPayload): vector<u8> {
        payload.from_address
    }

}