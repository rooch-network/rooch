// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::network{
    use std::string::{Self, String};

    const ErrorUnknownNetwork: u64 = 1;

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

    public fun is_mainnet(network: u8): bool {
        network == NETWORK_BITCOIN
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
            string::utf8(b"sb")
        } else if (network == NETWORK_REGTEST) {
            string::utf8(b"bcrt")
        } else {
            abort ErrorUnknownNetwork
        }
    }
   
    public fun network_magic(network: u8): vector<u8> {
        if (network == NETWORK_BITCOIN) {
            x"f9beb4d9"
        } else if (network == NETWORK_TESTNET) {
            x"0b110907"
        } else if (network == NETWORK_SIGNET) {
            x"0f1e2c3d"
        } else if (network == NETWORK_REGTEST) {
            x"fabfb5da"
        } else {
            abort ErrorUnknownNetwork
        }
    }
}