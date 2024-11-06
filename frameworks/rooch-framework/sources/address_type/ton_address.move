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

    // /// Parses url-safe base64 representation of an address
    // ///
    // /// # Returns
    // /// the address, non-bounceable flag, non-production flag.
    // pub fn from_base64_url_flags(
    //     s: &str,
    // ) -> Result<(TonAddress, bool, bool), TonAddressParseError> {
    //     if s.len() != 48 {
    //         return Err(TonAddressParseError::new(
    //             s,
    //             "Invalid base64url address: Wrong length",
    //         ));
    //     }
    //     let maybe_bytes = URL_SAFE_NO_PAD.decode(s);
    //     let bytes = match maybe_bytes {
    //         Ok(bytes) => bytes,
    //         Err(_) => {
    //             return Err(TonAddressParseError::new(
    //                 s,
    //                 "Invalid base64url address: Base64 decode error",
    //             ))
    //         }
    //     };
    //     let maybe_slice = bytes.as_slice().try_into();
    //     let slice = match maybe_slice {
    //         Ok(slice) => slice,
    //         Err(_) => {
    //             return Err(TonAddressParseError::new(
    //                 s,
    //                 "Invalid base64url address: Unexpected error",
    //             ))
    //         }
    //     };

    //     Self::from_base64_src(slice, s)
    // }

    // pub fn from_base64_std(s: &str) -> Result<TonAddress, TonAddressParseError> {
    //     Ok(Self::from_base64_std_flags(s)?.0)
    // }

    // /// Parses standard base64 representation of an address
    // ///
    // /// # Returns
    // /// the address, non-bounceable flag, non-production flag.
    // pub fn from_base64_std_flags(
    //     s: &str,
    // ) -> Result<(TonAddress, bool, bool), TonAddressParseError> {
    //     if s.len() != 48 {
    //         return Err(TonAddressParseError::new(
    //             s,
    //             "Invalid base64std address: Invalid length",
    //         ));
    //     }

    //     let maybe_vec = STANDARD_NO_PAD.decode(s);
    //     let vec = match maybe_vec {
    //         Ok(bytes) => bytes,
    //         Err(_) => {
    //             return Err(TonAddressParseError::new(
    //                 s,
    //                 "Invalid base64std address: Base64 decode error",
    //             ))
    //         }
    //     };
    //     let maybe_bytes = vec.as_slice().try_into();
    //     let bytes = match maybe_bytes {
    //         Ok(b) => b,
    //         Err(_) => {
    //             return Err(TonAddressParseError::new(
    //                 s,
    //                 "Invalid base64std: Unexpected error",
    //             ))
    //         }
    //     };

    //     Self::from_base64_src(bytes, s)
    // }

    // /// Parses decoded base64 representation of an address
    // ///
    // /// # Returns
    // /// the address, non-bounceable flag, non-production flag.
    // fn from_base64_src(
    //     bytes: &[u8; 36],
    //     src: &str,
    // ) -> Result<(TonAddress, bool, bool), TonAddressParseError> {
    //     let (non_production, non_bounceable) = match bytes[0] {
    //         0x11 => (false, false),
    //         0x51 => (false, true),
    //         0x91 => (true, false),
    //         0xD1 => (true, true),
    //         _ => {
    //             return Err(TonAddressParseError::new(
    //                 src,
    //                 "Invalid base64src address: Wrong tag byte",
    //             ))
    //         }
    //     };
    //     let workchain = bytes[1] as i8 as i32;
    //     let calc_crc = CRC_16_XMODEM.checksum(&bytes[0..34]);
    //     let addr_crc = ((bytes[34] as u16) << 8) | bytes[35] as u16;
    //     if calc_crc != addr_crc {
    //         return Err(TonAddressParseError::new(
    //             src,
    //             "Invalid base64src address: CRC mismatch",
    //         ));
    //     }
    //     let mut hash_part = [0_u8; 32];
    //     hash_part.clone_from_slice(&bytes[2..34]);
    //     let addr = TonAddress {
    //         workchain,
    //         hash_part,
    //     };
    //     Ok((addr, non_bounceable, non_production))
    // }

    fun from_base64_src(bytes: &vector<u8>) -> (TonAddress, bool, bool) {
        let addr_len = vector::length(bytes);
        assert!(addr_len == 36, ErrorInvalidAddress);
        let (non_production, non_bounceable) = match bytes[0] {
            0x11 => (false, false),
            0x51 => (false, true),
            0x91 => (true, false),
            0xD1 => (true, true),
            _ => {
                assert!(false, ErrorInvalidAddress);
            }
        };
        let workchain = bytes[1] as i8 as i32;
    }

    public fun from_string(addr_str: &String): TonAddress{
        //TODO support base64 address string
        from_hex_str(addr_str)
    }

    public fun from_bytes(bytes: vector<u8>): TonAddress {
        bcs::from_bytes<TonAddress>(bytes)
    }

    public fun into_bytes(addr: TonAddress): vector<u8> {
        bcs::to_bytes(&addr)
    }

    #[test]
    fun test_from_hex(){
        let addr_str = string::utf8(b"0:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76");
        let addr = from_hex_str(&addr_str);
        assert!(addr.workchain == i32::from(0), 2);
        assert!(addr.hash_part == @0xe4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76, 3);
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
