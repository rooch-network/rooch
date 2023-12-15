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
        let expected = option::none<BitcoinAddress>();

        assert!(Self::from_script(&bad_p2wpkh) == expected, 1000);
        assert!(Self::from_script(&bad_p2wsh) == expected, 1001);
        //TODO fix this test
        //assert!(Self::from_script(&invalid_segwitv0_script) == expected, 1002);
    }
}