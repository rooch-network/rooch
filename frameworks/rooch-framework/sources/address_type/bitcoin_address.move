// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::bitcoin_address {
    use std::vector;
    use std::string::{Self, String};

    friend rooch_framework::multichain_address;

    const PUBKEY_HASH_LEN: u64 = 20;
    const SCRIPT_HASH_LEN: u64 = 20;

    const P2PKH_ADDR_BYTE_LEN: u64 = 21;
    const P2SH_ADDR_BYTE_LEN: u64 = 21;

    // error code
    const E_INVALID_ADDRESS: u64 = 1;
    const E_ARG_NOT_VECTOR_U8: u64 = 2;
    const E_INVALID_PUBLIC_KEY: u64 = 3;
    const E_INVALID_THRESHOLD: u64 = 4;
    const E_INVALID_KEY_EGG_CONTEXT: u64 = 5;
    const E_INVALID_XONLY_PUBKEY: u64 = 6;

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
        assert!(vector::length(&pubkey_hash) == PUBKEY_HASH_LEN, E_INVALID_ADDRESS);
        //we do not distinguish between mainnet and testnet in Move
        let bytes = vector::singleton<u8>(P2PKH_ADDR_DECIMAL_PREFIX_MAIN);
        vector::append(&mut bytes, pubkey_hash);
        BitcoinAddress {
            bytes: bytes,
        }
    }

    public fun new_p2sh(script_hash: vector<u8>): BitcoinAddress{
        assert!(vector::length(&script_hash) == SCRIPT_HASH_LEN, E_INVALID_ADDRESS);
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

    public(friend) fun new(bytes: vector<u8>): BitcoinAddress {
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

    public fun from_string(addr: &String): BitcoinAddress {
        let raw_bytes = string::bytes(addr);
        parse(raw_bytes)
    }

    public fun verify_with_public_key(addr: &String, pk: &vector<u8>): bool {
        let bitcoin_addr = from_string(addr);
        verify_with_pk(&bitcoin_addr, pk)
    }

    public fun to_rooch_address(addr: &BitcoinAddress): address{
        let hash = moveos_std::hash::blake2b256(&addr.bytes);
        moveos_std::bcs::to_address(hash)
    }

    // verify bitcoin address according to the pk bytes
    public native fun verify_with_pk(bitcoin_addr: &BitcoinAddress, pk: &vector<u8>): bool;

    // derive multisig xonly public key from public keys
    public native fun derive_multisig_xonly_pubkey_from_xonly_pubkeys(public_keys: vector<vector<u8>>, threshold: u64): vector<u8>;
 
    // derive bitcoin taproot address from the multisig xonly public key
    public native fun derive_bitcoin_taproot_address_from_multisig_xonly_pubkey(xonly_pubkey: &vector<u8>): BitcoinAddress;

    /// Parse the Bitcoin address string bytes to Move BitcoinAddress
    native fun parse(raw_addr: &vector<u8>): BitcoinAddress;

    #[test_only]
    public fun random_address_for_testing(): BitcoinAddress {
        let bytes = moveos_std::bcs::to_bytes(&moveos_std::tx_context::fresh_address_for_testing());
        new_witness_program(bytes)
    }

    #[test]
    fun test_verify_with_public_key_success() {
        // p2tr
        let addr = string::utf8(b"bc1p8xpjpkc9uzj2dexcxjg9sw8lxje85xa4070zpcys589e3rf6k20qm6gjrt");
        let pk = x"038e3d29b653e40f5b620f9443ee05222d1e40be58f544b6fed3d464edd54db883";

        assert!(verify_with_public_key(&addr, &pk), 1000);

        // // p2wpkh
        let addr = string::utf8(b"bc1q9ymlna2efqx5arvcszu633rzfxq77ce9c3z34l");
        let pk = x"02481521eb57656db4bc9ec81857e105cc7853fe8cad61be23667bb401840fc7f8";
        assert!(verify_with_public_key(&addr, &pk), 1001);

        // // p2pkh
        let addr = string::utf8(b"1516MgBKZ1t8xFArmGZzyncTqABRoveyLG");
        let pk = x"02c3bc6ff4dec7f43dd4f587d4dc227fb171755779425ca032e0fcb2f0bb639cc2";
        assert!(verify_with_public_key(&addr, &pk), 1002);

        // // p2sh-p2wpkh
        let addr = string::utf8(b"38YrTMTpQ4ZUsjvcsweWgmHikUmgFcV7D5");
        let pk = x"02ebdc1107552f81d188a2c63806cb6fa5d734eaa7316a85dc1f608fcaee412b72";
        assert!(verify_with_public_key(&addr, &pk), 1003);
    }

    #[test]
    fun test_validate_signature_fail() {
        let addr = string::utf8(b"bc1p8xpjpkc9uzj2dexcxjg9sw8lxje85xa4070zpcys589e3rf6k20qm6gjrt");
        let pk = x"038e3d29b653e40f5b620f9443ee05222d1e40be58f544b6fed3d464edd54db884";
        assert!(!verify_with_public_key(&addr, &pk), 1004);
    }

    #[test]
    fun test_derive_multisig_xonly_pubkey_from_xonly_pubkeys_success() {
        let expected_xonly_pubkey = x"7b6474bd9206ad07c0bc6b0ac90d43f6f232235c9e9cbf0c47775bf47ca9c402";
        let pk_list = vector::empty<vector<u8>>();

        let pk_1 = x"f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9";
        let pk_2 = x"ffa540e2d3df158dfb202fc1a2cbb20c4920ba35e8f75bb11101bfa47d71449a";

        vector::push_back(&mut pk_list, pk_1);
        vector::push_back(&mut pk_list, pk_2);

        let xonly_pubkey = derive_multisig_xonly_pubkey_from_xonly_pubkeys(pk_list, 2);

        assert!(expected_xonly_pubkey == xonly_pubkey, E_INVALID_KEY_EGG_CONTEXT);
    }

    #[test]
    fun test_derive_bitcoin_taproot_address_from_multisig_xonly_pubkey_success() {
        let xonly_pubkey = x"7b6474bd9206ad07c0bc6b0ac90d43f6f232235c9e9cbf0c47775bf47ca9c402";

        let bitcoin_addr = derive_bitcoin_taproot_address_from_multisig_xonly_pubkey(&xonly_pubkey);

        let expected_bitcoin_addr = BitcoinAddress {
            bytes: x"020102f97a0a664c8493bfa28cfcf3450628bdc0ba7b3b0af2b57d4d057f15cb41f9",
        };

        assert!(expected_bitcoin_addr.bytes == bitcoin_addr.bytes, E_INVALID_XONLY_PUBKEY);
    }
}