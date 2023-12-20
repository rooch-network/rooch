// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::bitcoin_address {
    use std::vector;

    friend rooch_framework::multichain_address;

    const PUBKEY_HASH_LEN: u64 = 20;
    const SCRIPT_HASH_LEN: u64 = 20;

    const P2PKH_ADDR_BYTE_LEN: u64 = 21;
    const P2SH_ADDR_BYTE_LEN: u64 = 21;

    // error code
    const ErrorAddressBytesLen: u64 = 1;

    // P2PKH address decimal prefix
    const P2PKH_ADDR_DECIMAL_PREFIX_MAIN: u8 = 0; // 0x00
    const P2PKH_ADDR_DECIMAL_PREFIX_TEST: u8 = 111; // 0x6f
    // P2SH address decimal prefix 
    const P2SH_ADDR_DECIMAL_PREFIX_MAIN: u8 = 5; // 0x05
    const P2SH_ADDR_DECIMAL_PREFIX_TEST: u8 = 196; // 0xc4

   
    #[data_struct]
    /// BitcoinAddress is a struct that represents a Bitcoin address.
    /// We just keep the raw bytes of the address and do care about the network.
    struct BitcoinAddress has store, copy, drop {
        bytes: vector<u8>,
    }

    public fun new_p2pkh(pubkey_hash: vector<u8>): BitcoinAddress{
        assert!(vector::length(&pubkey_hash) == PUBKEY_HASH_LEN, ErrorAddressBytesLen);
        //we do not distinguish between mainnet and testnet in Move
        let bytes = vector::singleton<u8>(P2PKH_ADDR_DECIMAL_PREFIX_MAIN);
        vector::append(&mut bytes, pubkey_hash);
        BitcoinAddress {
            bytes: bytes,
        }
    }

    public fun new_p2sh(script_hash: vector<u8>): BitcoinAddress{
        assert!(vector::length(&script_hash) == SCRIPT_HASH_LEN, ErrorAddressBytesLen);
        let bytes = vector::singleton<u8>(P2SH_ADDR_DECIMAL_PREFIX_MAIN);
        vector::append(&mut bytes, script_hash);
        BitcoinAddress {
            bytes: bytes,
        }
    }

    public fun new_witness_program(program: vector<u8>): BitcoinAddress{
        BitcoinAddress {
            bytes: program,
        }
    }

    public fun from_bytes(bytes: vector<u8>): BitcoinAddress {
        BitcoinAddress {
            bytes: bytes,
        }
    }

    public fun is_p2pkh(addr: &BitcoinAddress): bool {
        let bytes = &addr.bytes;
        vector::length(bytes) == P2PKH_ADDR_BYTE_LEN && *vector::borrow(bytes, 0) == P2PKH_ADDR_DECIMAL_PREFIX_MAIN
    }

    public fun is_p2sh(addr: &BitcoinAddress): bool {
        let bytes = &addr.bytes;
        vector::length(bytes) == P2SH_ADDR_BYTE_LEN && *vector::borrow(bytes, 0) == P2SH_ADDR_DECIMAL_PREFIX_MAIN
    }

    public fun is_witness_program(addr: &BitcoinAddress): bool {
        !is_p2sh(addr) && !is_p2pkh(addr)
    }

    /// Empty address is a special address that is used to if we parse address failed from script.
    public fun is_empty(addr: &BitcoinAddress): bool {
        vector::length(&addr.bytes) == 0
    }

    public fun as_bytes(addr: &BitcoinAddress): &vector<u8> {
        &addr.bytes
    }

    public fun into_bytes(addr: BitcoinAddress): vector<u8> {
        let BitcoinAddress { bytes } = addr;
        bytes
    }

    public fun to_bech32(_addr: &BitcoinAddress): std::string::String{
        //TODO we need the bech32 string address?
        //We need to add the network and address type
        abort 0
    }
 
}