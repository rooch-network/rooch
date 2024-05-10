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

    native public fun verify_with_pk (addr: &vector<u8>, pk: &vector<u8>): bool;

    #[test]
    fun test_verify_with_pk_success() {
        // p2tr
        let addr = x"626331703878706a706b6339757a6a3264657863786a67397377386c786a6538357861343037307a7063797335383965337266366b3230716d36676a7274";
        let pk = x"038e3d29b653e40f5b620f9443ee05222d1e40be58f544b6fed3d464edd54db883";

        verify_with_pk(&addr, &pk);

        // p2wpkh
        let addr = x"6263317139796d6c6e6132656671783561727663737a75363333727a667871373763653963337a33346c";
        let pk = x"02481521eb57656db4bc9ec81857e105cc7853fe8cad61be23667bb401840fc7f8";
        verify_with_pk(&addr, &pk);

        // p2pkh
        let addr = x"313531364d67424b5a317438784641726d475a7a796e6354714142526f7665794c47";
        let pk = x"02c3bc6ff4dec7f43dd4f587d4dc227fb171755779425ca032e0fcb2f0bb639cc2";
        verify_with_pk(&addr, &pk);

        // p2sh-p2wpkh
        let addr = x"33385972544d547051345a55736a766373776557676d48696b556d67466356374435";
        let pk = x"02ebdc1107552f81d188a2c63806cb6fa5d734eaa7316a85dc1f608fcaee412b72";
        verify_with_pk(&addr, &pk);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = 3)]
    fun test_validate_signature_fail() {
        let addr = x"616331703878706a706b6339757a6a3264657863786a67397377386c786a6538357861343037307a7063797335383965337266366b3230716d36676a7274";
        let pk = x"038e3d29b653e40f5b620f9443ee05222d1e40be58f544b6fed3d464edd54db883";

        verify_with_pk(&addr, &pk);
    }
}