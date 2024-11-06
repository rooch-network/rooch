// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::ton_address {

    use std::vector;
    use std::option;
    use std::string::{Self, String};

    use moveos_std::string_utils;
    use moveos_std::hex;
    use moveos_std::bcs;
    use moveos_std::i32::{Self, I32};
    use moveos_std::i8;
    use moveos_std::base64;
    const ErrorInvalidAddress: u64 = 1;
    const ErrorInvalidWorkchain: u64 = 2;

    #[data_struct]
    struct TonAddress has store, copy, drop{
        //The workchain in TonAddress is i32
        workchain: I32,
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
        let workchain = if (is_nagative) {
            i32::neg_from(wc_num)
       } else {
            i32::from(wc_num)
        };
        let hash_part_bytes = hex::decode(&hash_part);
        let hash_part = bcs::from_bytes<address>(hash_part_bytes);
        TonAddress{
            workchain,
            hash_part,
        }
    }

    public fun from_base64_url(s: &String): TonAddress {
        let (addr, _, _) = from_base64_url_flags(s);
        addr
    }

    public fun from_base64_url_flags(s: &String): (TonAddress, bool, bool) {
        //Because the base64::decode do not support url-safe, we need to convert it to std first.
        let url_safe_str = to_base64_std(s);
        let bytes = string::bytes(&url_safe_str);
        let len = vector::length(bytes);
        assert!(len == 48, ErrorInvalidAddress);
        let decoded_bytes = base64::decode(bytes);
        let (addr, non_bounceable, non_production) = from_base64_src(&decoded_bytes);
        (addr, non_bounceable, non_production)
    }

    public fun from_base64_std(s: &String): TonAddress {
        let (addr, _, _) = from_base64_std_flags(s);
        addr
    }

    public fun from_base64_std_flags(s: &String): (TonAddress, bool, bool) {
        let bytes = string::bytes(s);
        let addr_len = vector::length(bytes);
        assert!(addr_len == 48, ErrorInvalidAddress);
        let decoded_bytes = base64::decode(bytes);
        let (addr, non_bounceable, non_production) = from_base64_src(&decoded_bytes);
        (addr, non_bounceable, non_production)
    }

    fun from_base64_src(bytes: &vector<u8>) : (TonAddress, bool, bool) {
        let addr_len = vector::length(bytes);
        assert!(addr_len == 36, ErrorInvalidAddress);
        let first_byte = *vector::borrow(bytes, 0);
        let (non_production, non_bounceable) = 
        if(first_byte == 0x11u8){
            (false, false)
        }else if(first_byte == 0x51u8){
            (false, true)
        }else if(first_byte == 0x91u8){
            (true, false)
        }else if(first_byte == 0xD1u8){
            (true, true)
        }else{
            abort ErrorInvalidAddress
        };
        let workchain = i32::from_i8(i8::from_u8(*vector::borrow(bytes, 1)));
        //TODO verify the checksum
        //let addr_crc = ((*vector::borrow(bytes, 34) as u16) << 8) | (*vector::borrow(bytes, 35) as u16);
        
        let hash_part_bytes = vector::slice(bytes, 2, 34);
        let hash_part = bcs::from_bytes<address>(hash_part_bytes);
        (TonAddress{
            workchain,
            hash_part,
        }, non_bounceable, non_production)
    }

    const BASE64_URL_CHAR_MINUS: u8 = 45u8; // '-'
    const BASE64_URL_CHAR_UNDERSCORE: u8 = 95u8; // '_'
    const BASE64_STD_CHAR_PLUS: u8 = 43u8; // '+'
    const BASE64_STD_CHAR_SLASH: u8 = 47u8; // '/'
    const BASE64_STD_CHAR_EQUAL: u8 = 61u8; // '='

    fun is_url_safe(s: &String): bool {
        let bytes = string::bytes(s);
        let len = vector::length(bytes);
        let i = 0;
        while(i < len){
            let c = *vector::borrow(bytes, i);
            if(c == BASE64_STD_CHAR_PLUS || c == BASE64_STD_CHAR_SLASH || c == BASE64_STD_CHAR_EQUAL){
                return false
            };
            i = i + 1;
        };
        true
    }

    fun to_base64_std(s: &String): String {
        let bytes = string::bytes(s);
        let len = vector::length(bytes);
        let new_bytes = vector::empty<u8>();
        let i = 0;
        while(i < len){
            let c = *vector::borrow(bytes, i);
            if(c == BASE64_URL_CHAR_MINUS){
                vector::push_back(&mut new_bytes, BASE64_STD_CHAR_PLUS);
            }else if(c == BASE64_URL_CHAR_UNDERSCORE){
                vector::push_back(&mut new_bytes, BASE64_STD_CHAR_SLASH);
            }else{
                vector::push_back(&mut new_bytes, c);
            };
            i = i + 1;
        };
        string::utf8(new_bytes)
    }

    public fun from_string(addr_str: &String): TonAddress{
        let len = string::length(addr_str);
        if(len == 48){
            if(is_url_safe(addr_str)){
                from_base64_url(addr_str)
            }else{
                from_base64_std(addr_str)
            }
        }else{
            from_hex_str(addr_str)
        }
    }

    public fun from_bytes(bytes: vector<u8>): TonAddress {
        bcs::from_bytes<TonAddress>(bytes)
    }

    public fun into_bytes(addr: TonAddress): vector<u8> {
        bcs::to_bytes(&addr)
    }

    #[test]
    fun test_from_string(){
        let addr_str = string::utf8(b"0:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76");
        let addr = from_hex_str(&addr_str);
        assert!(addr.workchain == i32::from(0), 2);
        assert!(addr.hash_part == @0xe4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76, 3);
        assert!(addr == from_string(&addr_str), 4);

        let addr_str_base64_std = string::utf8(b"EQDk2VTvn04SUKJrW7rXahzdF8/Qi6utb0wj43InCu9vdjrR");
        let addr2 = from_base64_std(&addr_str_base64_std);
        assert!(addr2 == addr, 2);
        assert!(addr2 == from_string(&addr_str_base64_std), 5);

        let addr_str_base64_url = string::utf8(b"EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR");
        let addr3 = from_base64_url(&addr_str_base64_url);
        assert!(addr3 == addr, 2);
        assert!(addr3 == from_string(&addr_str_base64_url), 5);

    }

    #[test]
    fun test_from_hex_nagitave(){
        let addr_str = string::utf8(b"-1:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76");
        let addr = from_hex_str(&addr_str);
        assert!(addr.workchain == i32::neg_from(1), 2);
        assert!(addr.hash_part == @0xe4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76, 3);
    }

    #[test]
    fun test_into_bytes(){
        let addr_str = string::utf8(b"0:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76");
        let addr = from_hex_str(&addr_str);
        let bytes = into_bytes(addr);
        let addr2 = from_bytes(bytes);
        assert!(addr2.workchain == addr.workchain, 2);
        assert!(addr2.hash_part == addr.hash_part, 3);
    }

}
