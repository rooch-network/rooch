// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::bitcoin_address {
    use std::option::{Self, Option};
    use std::vector;
    use std::string::{Self, String};

    friend rooch_framework::multichain_address;

    const PUBKEY_HASH_LEN: u64 = 20;
    const SCRIPT_HASH_LEN: u64 = 20;

    const P2PKH_ADDR_BYTE_LEN: u64 = 21;
    const P2SH_ADDR_BYTE_LEN: u64 = 21;

    // error code
    const ErrorInvalidAddress: u64 = 1;
    const ErrorArgNotVectorU8: u64 = 2;
    const ErrorInvalidPublicKey: u64 = 3;
    const ErrorInvalidThreshold: u64 = 4;
    const ErrorInvalidKeyEggContext: u64 = 5;
    const ErrorDeprecated: u64 = 6;

    // P2PKH address decimal prefix
    const P2PKH_ADDR_DECIMAL_PREFIX_MAIN: u8 = 0; // 0x00
    const P2PKH_ADDR_DECIMAL_PREFIX_TEST: u8 = 111; // 0x6f
    // P2SH address decimal prefix 
    const P2SH_ADDR_DECIMAL_PREFIX_MAIN: u8 = 5; // 0x05
    const P2SH_ADDR_DECIMAL_PREFIX_TEST: u8 = 196; // 0xc4

    const PAY_LOAD_TYPE_PUBKEY_HASH: u8 = 0;
    public fun pay_load_type_pubkey_hash(): u8 {
        PAY_LOAD_TYPE_PUBKEY_HASH
    }
    const PAY_LOAD_TYPE_SCRIPT_HASH: u8 = 1;
    public fun pay_load_type_script_hash(): u8 {
        PAY_LOAD_TYPE_SCRIPT_HASH
    }
    const PAY_LOAD_TYPE_WITNESS_PROGRAM: u8 = 2;
    public fun pay_load_type_witness_program(): u8 {
        PAY_LOAD_TYPE_WITNESS_PROGRAM
    }
   
    #[data_struct]
    /// BitcoinAddress is a struct that represents a Bitcoin address.
    /// We just keep the raw bytes of the address and do care about the network.
    struct BitcoinAddress has store, copy, drop {
        bytes: vector<u8>,
    }

    public fun p2pkh(pubkey_hash: vector<u8>): BitcoinAddress{
        assert!(vector::length(&pubkey_hash) == PUBKEY_HASH_LEN, ErrorInvalidAddress);
        //we do not distinguish between mainnet and testnet in Move
        let bytes = vector::singleton<u8>(PAY_LOAD_TYPE_PUBKEY_HASH);
        vector::append(&mut bytes, pubkey_hash);
        BitcoinAddress {
            bytes: bytes,
        }
    }

    public fun p2sh(script_hash: vector<u8>): BitcoinAddress{
        assert!(vector::length(&script_hash) == SCRIPT_HASH_LEN, ErrorInvalidAddress);
        let bytes = vector::singleton<u8>(PAY_LOAD_TYPE_SCRIPT_HASH);
        vector::append(&mut bytes, script_hash);
        BitcoinAddress {
            bytes: bytes,
        }
    }

    /// Derive a Bitcoin address from a internal public key and a merkle root.
    /// The internal public key is a secp256k1 public key or x-only public key.
    public fun p2tr(internal_pubkey: &vector<u8>, merkle_root: Option<address>): BitcoinAddress {
        derive_bitcoin_taproot_address(internal_pubkey, merkle_root)
    }

    public(friend) fun new(bytes: vector<u8>): BitcoinAddress {
        BitcoinAddress {
            bytes: bytes,
        }
    }

    public fun empty() : BitcoinAddress {
        BitcoinAddress {
            bytes: vector::empty()
        }
    }

    public fun pay_load_type(addr: &BitcoinAddress): u8 {
        *vector::borrow(&addr.bytes, 0)
    }

    public fun pay_load(addr: &BitcoinAddress): vector<u8> {
        vector::slice(&addr.bytes, 1, vector::length(&addr.bytes))
    }

    public fun is_p2pkh(addr: &BitcoinAddress): bool {
        let bytes = &addr.bytes;
        vector::length(bytes) == P2PKH_ADDR_BYTE_LEN && *vector::borrow(bytes, 0) == PAY_LOAD_TYPE_PUBKEY_HASH
    }

    public fun is_p2sh(addr: &BitcoinAddress): bool {
        let bytes = &addr.bytes;
        vector::length(bytes) == P2SH_ADDR_BYTE_LEN && *vector::borrow(bytes, 0) == PAY_LOAD_TYPE_SCRIPT_HASH
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
        verify_bitcoin_address_with_public_key(&bitcoin_addr, pk)
    }

    public fun to_rooch_address(addr: &BitcoinAddress): address{
        assert!(!is_empty(addr), ErrorInvalidAddress);
        let hash = moveos_std::hash::blake2b256(&addr.bytes);
        moveos_std::bcs::to_address(hash)
    }

    native fun derive_bitcoin_taproot_address(internal_pubkey: &vector<u8>, merkle_root: Option<address>) : BitcoinAddress;

    /// verify bitcoin address according to the pk bytes, the pk is Secp256k1 public key format.
    public native fun verify_bitcoin_address_with_public_key(bitcoin_addr: &BitcoinAddress, pk: &vector<u8>): bool;

 
    // derive bitcoin taproot address from a secp256k1 pubkey or x-only pubkey
    public fun derive_bitcoin_taproot_address_from_pubkey(pubkey: &vector<u8>): BitcoinAddress{
        derive_bitcoin_taproot_address(pubkey, option::none())
    }
    
    /// Parse the Bitcoin address string bytes to Move BitcoinAddress
    native fun parse(raw_addr: &vector<u8>): BitcoinAddress;

    #[test_only]
    public fun random_address_for_testing(): BitcoinAddress {
        let bytes = moveos_std::bcs::to_bytes(&moveos_std::tx_context::fresh_address_for_testing());
        BitcoinAddress{
            bytes
        }
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
    fun test_derive_bitcoin_taproot_address_from_pubkey_success() {
        let pubkey = x"034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14";

        let bitcoin_addr = derive_bitcoin_taproot_address_from_pubkey(&pubkey);

        let expected_bitcoin_addr = from_string(&string::utf8(b"bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"));

        assert!(expected_bitcoin_addr.bytes == bitcoin_addr.bytes, ErrorInvalidPublicKey);
    }

    #[test]
    fun test_derive_bitcoin_taproot_address_from_x_only_pubkey_success() {
        let pubkey = x"4cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14";

        let bitcoin_addr = derive_bitcoin_taproot_address_from_pubkey(&pubkey);

        let expected_bitcoin_addr = from_string(&string::utf8(b"bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"));

        assert!(expected_bitcoin_addr.bytes == bitcoin_addr.bytes, ErrorInvalidPublicKey);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidPublicKey)]
    fun test_derive_bitcoin_taproot_address_from_multisig_pubkey_fail() {
        let pubkey = x"3d29b653e40f5b620f9443ee05222d1e40be58f544b6fed3d464edd54db883";

        derive_bitcoin_taproot_address_from_pubkey(&pubkey);
    }
}