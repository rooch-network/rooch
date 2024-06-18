// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::network{
    use std::string::{Self, String};
    use moveos_std::object;

    const ErrorUnknownNetwork: u64 = 1;

    friend bitcoin_move::genesis;

    /// Bitcoin network onchain configuration.
    struct BitcoinNetwork has key{
        network: u8
    }

    public(friend) fun genesis_init(network: u8){
        let obj = object::new_named_object(BitcoinNetwork{network: network});
        object::to_shared(obj);
    }

    /// Get the current network from the onchain configuration.
    public fun network() : u8 {
        let id = object::named_object_id<BitcoinNetwork>();
        object::borrow(object::borrow_object<BitcoinNetwork>(id)).network 
    }

    ///Currently, Move does not support enum types, so we use constants to represent the network type.    
    /// Mainnet Bitcoin.
    const NETWORK_BITCOIN: u8 = 1;
    public fun network_bitcoin(): u8 {
        NETWORK_BITCOIN
    }

    /// Bitcoin's testnet network.
    const NETWORK_TESTNET: u8 = 2;
    public fun network_testnet(): u8 {
        NETWORK_TESTNET
    }

    /// Bitcoin's signet network.
    const NETWORK_SIGNET: u8 = 3;
    public fun network_signet(): u8 {
        NETWORK_SIGNET
    }

    /// Bitcoin's regtest network.
    const NETWORK_REGTEST: u8 = 4;
    public fun network_regtest(): u8 {
        NETWORK_REGTEST
    }

    public fun is_mainnet(): bool {
        network() == NETWORK_BITCOIN
    }

    public fun is_testnet(): bool {
        network() == NETWORK_TESTNET
    }

    public fun is_signet(): bool {
        network() == NETWORK_SIGNET
    }

    public fun from_str(network: &String): u8 {
        if (string::bytes(network) == &b"bitcoin") {
            NETWORK_BITCOIN
        } else if (string::bytes(network) == &b"testnet") {
            NETWORK_TESTNET
        } else if (string::bytes(network) == &b"signet") {
            NETWORK_SIGNET
        } else if (string::bytes(network) == &b"regtest") {
            NETWORK_REGTEST
        } else {
            abort ErrorUnknownNetwork
        }
    }

    public fun network_name(network: u8): String {
        if (network == NETWORK_BITCOIN) {
            string::utf8(b"bitcoin")
        } else if (network == NETWORK_TESTNET) {
            string::utf8(b"testnet")
        } else if (network == NETWORK_SIGNET) {
            string::utf8(b"signet")
        } else if (network == NETWORK_REGTEST) {
            string::utf8(b"regtest")
        } else {
            abort ErrorUnknownNetwork
        }
    }

    public fun bech32_hrp(network: u8): String {
        if (network == NETWORK_BITCOIN) {
            string::utf8(b"bc")
        } else if (network == NETWORK_TESTNET) {
            string::utf8(b"tb")
        } else if (network == NETWORK_SIGNET) {
            string::utf8(b"tb")
        } else if (network == NETWORK_REGTEST) {
            string::utf8(b"bcrt")
        } else {
            abort ErrorUnknownNetwork
        }
    }

}