// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::ton_address {

    use std::vector;
    use std::option;
    use std::string::{Self, String};

    use moveos_std::string_utils;
    use moveos_std::hex;
    use moveos_std::bcs;

    const ErrorInvalidAddress: u64 = 1;
    const ErrorInvalidWorkchain: u64 = 2;

    struct TonAddress has store, copy, drop{
        is_nagative: bool,
        //The workchain in TonAddress is i32, but No i32 in Move
        //So we use u32 instead, and use `is_nagative` to represent the sign
        workchain: u32,
        hash_part: address,
    }

    /// The split char in hex address string: `:`
    const SPLIT_CHAR: u8 = 58u8;
    /// The minus char in hex address string: `-`
    const MINUS_CHAR: u8 = 45u8;

    public fun from_hex_str(s: &String) : TonAddress {
        let bytes = string::bytes(s);
        let addr_len = vector::length(bytes);
        let (found,idx) = vector::index_of(bytes, &58u8);
        assert!(found, ErrorInvalidAddress);
        assert!(idx > 0, ErrorInvalidWorkchain);
        let wc_part = vector::slice(bytes, 0, idx);
        let hash_part = vector::slice(bytes, idx + 1, addr_len);
        let is_nagative = *vector::borrow(&wc_part,0) == 45u8;

        let wc_num_part = if(is_nagative){
            vector::slice(&wc_part, 1, idx)
        }else{
            wc_part
        };
        let wc_num_opt = string_utils::parse_u32_from_bytes(&wc_num_part);
        assert!(option::is_some(&wc_num_opt), ErrorInvalidWorkchain);
        let wc_num = option::destroy_some(wc_num_opt);
        let hash_part_bytes = hex::decode(&hash_part);
        let hash_part = bcs::from_bytes<address>(hash_part_bytes);
        TonAddress{
            is_nagative,
            workchain: wc_num,
            hash_part,
        }
    }

    public fun from_string(addr_str: &String): TonAddress{
        //TODO support base64 address string
        from_hex_str(addr_str)
    }

    #[test]
    fun test_from_hex(){
        let addr_str = string::utf8(b"0:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76");
        let addr = from_hex_str(&addr_str);
        assert!(addr.is_nagative == false, 1);
        assert!(addr.workchain == 0u32, 2);
        assert!(addr.hash_part == @0xe4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76, 3);
    }

    #[test]
    fun test_from_hex_nagitave(){
        let addr_str = string::utf8(b"-1:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76");
        let addr = from_hex_str(&addr_str);
        assert!(addr.is_nagative == true, 1);
        assert!(addr.workchain == 1u32, 2);
        assert!(addr.hash_part == @0xe4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76, 3);
    }

}