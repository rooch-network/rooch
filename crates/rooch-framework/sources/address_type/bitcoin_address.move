// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::bitcoin_address {
    use std::error;
    use std::vector;
    use std::option::{Self, Option};
    use rooch_framework::bitcoin_script_buf::{Self, ScriptBuf};

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
    /// BTCAddress is a struct that represents a Bitcoin address.
    /// We just keep the raw bytes of the address and do care about the network.
    struct BTCAddress has store, copy, drop {
        bytes: vector<u8>,
    }

    fun new_p2pkh(pubkey_hash: vector<u8>): BTCAddress{
        assert!(vector::length(&pubkey_hash) == PUBKEY_HASH_LEN, error::invalid_argument(ErrorAddressBytesLen));
        //TDDO do we need to distinguish between mainnet and testnet?
        //OR find a way to define same module for different networks
        let bytes = vector::singleton<u8>(P2PKH_ADDR_DECIMAL_PREFIX_MAIN);
        vector::append(&mut bytes, pubkey_hash);
        BTCAddress {
            bytes: bytes,
        }
    }

    fun new_p2sh(script_hash: vector<u8>): BTCAddress{
        assert!(vector::length(&script_hash) == SCRIPT_HASH_LEN, error::invalid_argument(ErrorAddressBytesLen));
        let bytes = vector::singleton<u8>(P2SH_ADDR_DECIMAL_PREFIX_MAIN);
        vector::append(&mut bytes, script_hash);
        BTCAddress {
            bytes: bytes,
        }
    }

    fun new_witness_program(program: vector<u8>): BTCAddress{
        BTCAddress {
            bytes: program,
        }
    }

    /// from_script returns a BTCAddress from a ScriptBuf.
    public fun from_script(s: &ScriptBuf): Option<BTCAddress> {
        if(bitcoin_script_buf::is_p2pkh(s)){
            let pubkey_hash = bitcoin_script_buf::p2pkh_pubkey_hash(s);
            option::some(new_p2pkh(pubkey_hash))
        }else if(bitcoin_script_buf::is_p2sh(s)){
            let script_hash = bitcoin_script_buf::p2sh_script_hash(s);
            option::some(new_p2sh(script_hash))
        }else if(bitcoin_script_buf::is_witness_program(s)){
            let program = bitcoin_script_buf::witness_program(s);
            option::some(new_witness_program(program))
        }else{
            option::none()
        }
    }

    public(friend) fun from_bytes(bytes: vector<u8>): BTCAddress {
        BTCAddress {
            bytes: bytes,
        }
    }

    public fun is_p2pkh(addr: &BTCAddress): bool {
        let bytes = &addr.bytes;
        vector::length(bytes) == P2PKH_ADDR_BYTE_LEN && *vector::borrow(bytes, 0) == P2PKH_ADDR_DECIMAL_PREFIX_MAIN
    }

    public fun is_p2sh(addr: &BTCAddress): bool {
        let bytes = &addr.bytes;
        vector::length(bytes) == P2SH_ADDR_BYTE_LEN && *vector::borrow(bytes, 0) == P2SH_ADDR_DECIMAL_PREFIX_MAIN
    }

    public fun is_witness_program(addr: &BTCAddress): bool {
        !is_p2sh(addr) && !is_p2pkh(addr)
    }

    public fun as_bytes(addr: &BTCAddress): &vector<u8> {
        &addr.bytes
    }

    public fun into_bytes(addr: BTCAddress): vector<u8> {
        let BTCAddress { bytes } = addr;
        bytes
    }

    public fun to_bech32(_addr: &BTCAddress): std::string::String{
        //TODO we need the bech32 string address?
        //We need to add the network and address type
        abort 0
    }

    #[test]
    fun test_from_script_p2pkh(){
        let script_buf = bitcoin_script_buf::new(x"76a914010966776006953d5567439e5e39f86a0d273bee88ac");
        let addr_opt = from_script(&script_buf);
        assert!(option::is_some(&addr_opt), 1000);
        let addr = option::extract(&mut addr_opt);
        assert!(is_p2pkh(&addr), 1001);
        let addr_bytes = into_bytes(addr);
        std::debug::print(&addr_bytes);
        let expected_addr_bytes = x"00010966776006953d5567439e5e39f86a0d273bee";
        assert!(addr_bytes == expected_addr_bytes, 1002);
    }

    #[test]
    fun test_from_script_p2sh(){
        let script_buf = bitcoin_script_buf::new(x"a91474d691da1574e6b3c192ecfb52cc8984ee7b6c4887");
        let addr_opt = from_script(&script_buf);
        assert!(option::is_some(&addr_opt), 1000);
        let addr = option::extract(&mut addr_opt);
        assert!(is_p2sh(&addr), 1001);
        let addr_bytes = into_bytes(addr);
        std::debug::print(&addr_bytes);
        let expected_addr_bytes = x"0574d691da1574e6b3c192ecfb52cc8984ee7b6c48";
        assert!(addr_bytes == expected_addr_bytes, 1002);
    }

    #[test]
    fun test_p2wpkh_address(){
        let script_buf = bitcoin_script_buf::new(x"001497cdff4fd3ed6f885d54a52b79d7a2141072ae3f");
        let addr_opt = from_script(&script_buf);
        assert!(option::is_some(&addr_opt), 1000);
        let addr = option::extract(&mut addr_opt);
        assert!(is_witness_program(&addr), 1001);
        let addr_bytes = into_bytes(addr);
        //std::debug::print(&addr_bytes);
        let expected_addr_bytes = x"97cdff4fd3ed6f885d54a52b79d7a2141072ae3f";
        assert!(addr_bytes == expected_addr_bytes, 1002);
    }

    #[test]
    fun test_fail_address_from_script() {

        let bad_p2wpkh = bitcoin_script_buf::new(x"0014dbc5b0a8f9d4353b4b54c3db48846bb15abfec");
        let bad_p2wsh = bitcoin_script_buf::new(x"00202d4fa2eb233d008cc83206fa2f4f2e60199000f5b857a835e3172323385623");
        //let invalid_segwitv0_script = bitcoin_script_buf::new(x"001161458e330389cd0437ee9fe3641d70cc18");
        let expected = option::none<BTCAddress>();

        assert!(Self::from_script(&bad_p2wpkh) == expected, 1000);
        assert!(Self::from_script(&bad_p2wsh) == expected, 1001);
        //TODO fix this test
        //assert!(Self::from_script(&invalid_segwitv0_script) == expected, 1002);
    }
}