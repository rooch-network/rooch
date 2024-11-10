// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::ton_proof {

    use std::vector;
    use std::string::{Self, String};
    
    use moveos_std::bcs;
    use moveos_std::byte_stream::{Self, ByteStream};
    use moveos_std::base64;

    use rooch_framework::ton_address::{TonAddress};

    const GENERIC_BOC_MAGIC: u32 = 0xb5ee9c72;

    const ErrorInvalidBocMagic: u64 = 1;

    #[data_struct]
    struct TonDomain has copy, drop, store{
        length_bytes: u64,
        value: String,
    }

    const PAYLOAD_MESSAGE_IDX: u64 = 0;
    const PAYLOAD_BITCOIN_ADDRESS_IDX: u64 = 1;
    const PAYLOAD_TX_HASH_IDX: u64 = 2;

    #[data_struct]
    struct TonProof has copy, drop, store{
        timestamp: u64,
        domain: TonDomain,
        signature: String,
        //We use a vector to store payload for future extension
        payload: vector<String>,
    }

    #[data_struct]
    struct TonProofData has copy, drop, store {
        name: String,
        proof: TonProof,
        state_init: String,
    }

    #[data_struct]
    struct RawCell has copy, drop, store {
        data: vector<u8>,
        bit_len: u64,
        references: vector<u64>,
        is_exotic: bool,
        level_mask: u32,
    }

    #[data_struct]
    struct RawBagOfCells has copy, drop, store {
        cells: vector<RawCell>,
        roots: vector<u64>,
    }

    // pub(crate) fn parse(serial: &[u8]) -> Result<RawBagOfCells, TonCellError> {
    //     let cursor = Cursor::new(serial);

    //     let mut reader: ByteReader<Cursor<&[u8]>, BigEndian> =
    //         ByteReader::endian(cursor, BigEndian);
    //     // serialized_boc#b5ee9c72
    //     let magic = reader.read::<u32>().map_boc_deserialization_error()?;

    //     let (has_idx, has_crc32c, _has_cache_bits, size) = match magic {
    //         GENERIC_BOC_MAGIC => {
    //             // has_idx:(## 1) has_crc32c:(## 1) has_cache_bits:(## 1) flags:(## 2) { flags = 0 }
    //             let header = reader.read::<u8>().map_boc_deserialization_error()?;
    //             let has_idx = (header >> 7) & 1 == 1;
    //             let has_crc32c = (header >> 6) & 1 == 1;
    //             let has_cache_bits = (header >> 5) & 1 == 1;
    //             // size:(## 3) { size <= 4 }
    //             let size = header & 0b0000_0111;

    //             (has_idx, has_crc32c, has_cache_bits, size)
    //         }
    //         magic => {
    //             return Err(TonCellError::boc_deserialization_error(format!(
    //                 "Unsupported cell magic number: {:#}",
    //                 magic
    //             )));
    //         }
    //     };
    //     //   off_bytes:(## 8) { off_bytes <= 8 }
    //     let off_bytes = reader.read::<u8>().map_boc_deserialization_error()?;
    //     //cells:(##(size * 8))
    //     let cells = read_var_size(&mut reader, size)?;
    //     //   roots:(##(size * 8)) { roots >= 1 }
    //     let roots = read_var_size(&mut reader, size)?;
    //     //   absent:(##(size * 8)) { roots + absent <= cells }
    //     let _absent = read_var_size(&mut reader, size)?;
    //     //   tot_cells_size:(##(off_bytes * 8))
    //     let _tot_cells_size = read_var_size(&mut reader, off_bytes)?;
    //     //   root_list:(roots * ##(size * 8))
    //     let mut root_list = vec![];
    //     for _ in 0..roots {
    //         root_list.push(read_var_size(&mut reader, size)?)
    //     }
    //     //   index:has_idx?(cells * ##(off_bytes * 8))
    //     let mut index = vec![];
    //     if has_idx {
    //         for _ in 0..cells {
    //             index.push(read_var_size(&mut reader, off_bytes)?)
    //         }
    //     }
    //     //   cell_data:(tot_cells_size * [ uint8 ])
    //     let mut cell_vec = Vec::with_capacity(cells);

    //     for _ in 0..cells {
    //         let cell = read_cell(&mut reader, size)?;
    //         cell_vec.push(cell);
    //     }
    //     //   crc32c:has_crc32c?uint32
    //     let _crc32c = if has_crc32c {
    //         reader.read::<u32>().map_boc_deserialization_error()?
    //     } else {
    //         0
    //     };
    //     // TODO: Check crc32

    //     Ok(RawBagOfCells {
    //         cells: cell_vec,
    //         roots: root_list,
    //     })
    // }

    fun parse_boc(serial: &vector<u8>): RawBagOfCells {
        let stream = byte_stream::new(*serial);
        // serialized_boc#b5ee9c72
        let magic = byte_stream::read_u32(&mut stream);
        assert!(magic == GENERIC_BOC_MAGIC, ErrorInvalidBocMagic);

        // has_idx:(## 1) has_crc32c:(## 1) has_cache_bits:(## 1) flags:(## 2) { flags = 0 }
        let header = byte_stream::read_u8(&mut stream);
        let has_idx = (header >> 7) & 1 == 1;
        let has_crc32c = (header >> 6) & 1 == 1;
        let _has_cache_bits = (header >> 5) & 1 == 1;
        // size:(## 3) { size <= 4 }
        let size = header & 0x07;

        // off_bytes:(## 8) { off_bytes <= 8 }
        let off_bytes = byte_stream::read_u8(&mut stream);
        // cells:(##(size * 8))
        let cells = byte_stream::read_var_size(&mut stream, size);
        // roots:(##(size * 8)) { roots >= 1 }
        let roots = byte_stream::read_var_size(&mut stream, size);
        // absent:(##(size * 8)) { roots + absent <= cells }
        let _absent = byte_stream::read_var_size(&mut stream, size);
        // tot_cells_size:(##(off_bytes * 8))
        let _tot_cells_size = byte_stream::read_var_size(&mut stream, off_bytes);

        // root_list:(roots * ##(size * 8))
        let root_list = vector::empty<u64>();
        let i = 0;
        while (i < roots) {
            vector::push_back(&mut root_list, byte_stream::read_var_size(&mut stream, size));
            i = i + 1;
        };

        // index:has_idx?(cells * ##(off_bytes * 8))
        if (has_idx) {
            let i = 0;
            while (i < cells) {
                let _index = byte_stream::read_var_size(&mut stream, off_bytes);
                i = i + 1;
            };
        };

        // cell_data:(tot_cells_size * [ uint8 ])
        let cell_vec = vector::empty<RawCell>();
        let i = 0;
        while (i < cells) {
            let cell = read_cell(&mut stream, size);
            vector::push_back(&mut cell_vec, cell);
            i = i + 1;
        };

        // crc32c:has_crc32c?uint32
        if (has_crc32c) {
            let _crc32c = byte_stream::read_u32(&mut stream);
        };

        RawBagOfCells {
            cells: cell_vec,
            roots: root_list,
        }
    }

//     fn read_cell(
//     reader: &mut ByteReader<Cursor<&[u8]>, BigEndian>,
//     size: u8,
// ) -> Result<RawCell, TonCellError> {
//     let d1 = reader.read::<u8>().map_boc_deserialization_error()?;
//     let d2 = reader.read::<u8>().map_boc_deserialization_error()?;

//     let ref_num = d1 & 0b111;
//     let is_exotic = (d1 & 0b1000) != 0;
//     let has_hashes = (d1 & 0b10000) != 0;
//     let level_mask = (d1 >> 5) as u32;
//     let data_size = ((d2 >> 1) + (d2 & 1)).into();
//     let full_bytes = (d2 & 0x01) == 0;

//     if has_hashes {
//         let hash_count = LevelMask::new(level_mask).hash_count();
//         let skip_size = hash_count * (32 + 2);

//         // TODO: check depth and hashes
//         reader
//             .skip(skip_size as u32)
//             .map_boc_deserialization_error()?;
//     }

//     let mut data = reader
//         .read_to_vec(data_size)
//         .map_boc_deserialization_error()?;

//     let data_len = data.len();
//     let padding_len = if data_len > 0 && !full_bytes {
//         // Fix last byte,
//         // see https://github.com/toncenter/tonweb/blob/c2d5d0fc23d2aec55a0412940ce6e580344a288c/src/boc/BitString.js#L302
//         let num_zeros = data[data_len - 1].trailing_zeros();
//         if num_zeros >= 8 {
//             return Err(TonCellError::boc_deserialization_error(
//                 "Last byte of binary must not be zero if full_byte flag is not set",
//             ));
//         }
//         data[data_len - 1] &= !(1 << num_zeros);
//         num_zeros + 1
//     } else {
//         0
//     };
//     let bit_len = data.len() * 8 - padding_len as usize;
//     let mut references: Vec<usize> = Vec::new();
//     for _ in 0..ref_num {
//         references.push(read_var_size(reader, size)?);
//     }
//     let cell = RawCell::new(data, bit_len, references, level_mask, is_exotic);
//     Ok(cell)
// }

    fun read_cell(stream: &mut ByteStream, size: u8): RawCell {
        let d1 = byte_stream::read_u8(stream);
        let d2 = byte_stream::read_u8(stream);

        let ref_num = d1 & 0x07;
        let is_exotic = (d1 & 0x08) != 0;
        let has_hashes = (d1 & 0x10) != 0;
        let level_mask = ((d1 >> 5) as u32);
        let data_size = (((d2 >> 1) + (d2 & 1)) as u64);
        let full_bytes = (d2 & 0x01) == 0;

        if (has_hashes) {
            let hash_index = count_ones(level_mask);
            let hash_count = hash_index + 1;
            let skip_size = hash_count * (32 + 2);
            byte_stream::skip(stream, (skip_size as u64));
        };

        let data = byte_stream::read_to_vec(stream, data_size);
        let data_len = vector::length(&data);
        let padding_len = if (data_len > 0 && !full_bytes) {
            // Fix last byte
            let last_byte = *vector::borrow(&data, data_len - 1);
            let num_zeros = trailing_zeros(last_byte);
            assert!(num_zeros < 8, 1); // Last byte must not be zero if full_byte flag is not set
            let mask = 1 << num_zeros;
            let last_byte = last_byte & (0xff ^ mask);
            vector::push_back(&mut data, last_byte);
            num_zeros + 1
        } else {
            0
        };

        let bit_len = data_len * 8 - (padding_len as u64);
        let references = vector::empty<u64>();
        let i = 0;
        while (i < ref_num) {
            vector::push_back(&mut references, byte_stream::read_var_size(stream, size));
            i = i + 1;
        };

        RawCell {
            data: data,
            bit_len: bit_len,
            references: references,
            is_exotic: is_exotic,
            level_mask: level_mask
        }
    }

    //TODO migrate this function to u32.move
    // return the number of 1s in the binary representation of the value
    fun count_ones(value: u32): u32 {
        let count = 0;
        let mut_value = value;
        while (mut_value != 0) {
            if ((mut_value & 1) == 1) {
                count = count + 1;
            };
            mut_value = mut_value >> 1;
        };
        count
    }

    fun trailing_zeros(value: u8): u8 {
        let count = 0;
        let mut_value = value;
        while ((mut_value & 1) == 0) {
            count = count + 1;
            mut_value = mut_value >> 1;
        };
        count
    }

    public fun decode_proof_data(proof_data_bytes: vector<u8>): TonProofData {
        bcs::from_bytes(proof_data_bytes)
    }
    
    /// verify the proof
    public fun verify_proof(_ton_addr: &TonAddress, ton_proof_data: &TonProofData) : bool {
        let state_init_bytes = base64::decode(string::bytes(&ton_proof_data.state_init));
        let _boc = parse_boc(&state_init_bytes);
        true
    }

    // ======================== TonProofData functions ========================

    public fun name(ton_proof_data: &TonProofData): &String {
        &ton_proof_data.name
    }

    public fun proof(ton_proof_data: &TonProofData): &TonProof {
        &ton_proof_data.proof
    }

    public fun state_init(ton_proof_data: &TonProofData): &String {
        &ton_proof_data.state_init
    }

    // ======================== TonProof functions ========================

    public fun domain(ton_proof: &TonProof): &TonDomain {
        &ton_proof.domain
    }

    public fun payload(ton_proof: &TonProof): &vector<String> {
        &ton_proof.payload
    }

    /// Get the message from the payload, if the payload is not long enough, return an empty string
    public fun payload_message(ton_proof: &TonProof): String {
        if (vector::length(&ton_proof.payload) > PAYLOAD_MESSAGE_IDX) {
            *vector::borrow(&ton_proof.payload, PAYLOAD_MESSAGE_IDX)
        } else {
            string::utf8(b"")
        }
    }

    /// Get the bitcoin address from the payload, if the payload is not long enough, return an empty string
    public fun payload_bitcoin_address(ton_proof: &TonProof): String {
        if (vector::length(&ton_proof.payload) > PAYLOAD_BITCOIN_ADDRESS_IDX) {
            *vector::borrow(&ton_proof.payload, PAYLOAD_BITCOIN_ADDRESS_IDX)
        } else {
            string::utf8(b"")
        }
    }

    /// Get the tx hash from the payload, if the payload is not long enough, return an empty string
    public fun payload_tx_hash(ton_proof: &TonProof): String {
        if (vector::length(&ton_proof.payload) > PAYLOAD_TX_HASH_IDX) {
            *vector::borrow(&ton_proof.payload, PAYLOAD_TX_HASH_IDX)
        } else {
            string::utf8(b"")
        }
    }

    public fun signature(ton_proof: &TonProof): &String {
        &ton_proof.signature
    }

    public fun timestamp(ton_proof: &TonProof): u64 {
        ton_proof.timestamp
    }

    // ======================== TonDomain functions ========================

    public fun domain_length_bytes(ton_domain: &TonDomain): u64 {
        ton_domain.length_bytes
    }

    public fun domain_value(ton_domain: &TonDomain): &String {
        &ton_domain.value
    }

    // let verify_proof_json = r#"{
    //         "name": "ton_proof",
    //         "proof": {
    //             "timestamp": 1730363765,
    //             "domain": {
    //                 "length_bytes": 21,
    //                 "value": "ton-connect.github.io"
    //             },
    //             "signature": "BvysFrBS8KgTa3bww9f5paEu6/jZr5jB1JmO6T8nqsLzJqB3hWHiqOG9OezPsiJX3kD9nifMbRhr1xkv37ICCw==",
    //             "payload": "bc1q04uaa0mveqtt4y0sltuxtauhlyl8ctstr5x3hu"
    //         },
    //         "state_init": "te6cckECFgEAAwQAAgE0ARUBFP8A9KQT9LzyyAsCAgEgAxACAUgEBwLm0AHQ0wMhcbCSXwTgItdJwSCSXwTgAtMfIYIQcGx1Z70ighBkc3RyvbCSXwXgA/pAMCD6RAHIygfL/8nQ7UTQgQFA1yH0BDBcgQEI9ApvoTGzkl8H4AXTP8glghBwbHVnupI4MOMNA4IQZHN0crqSXwbjDQUGAHgB+gD0BDD4J28iMFAKoSG+8uBQghBwbHVngx6xcIAYUATLBSbPFlj6Ahn0AMtpF8sfUmDLPyDJgED7AAYAilAEgQEI9Fkw7UTQgQFA1yDIAc8W9ADJ7VQBcrCOI4IQZHN0coMesXCAGFAFywVQA88WI/oCE8tqyx/LP8mAQPsAkl8D4gIBIAgPAgEgCQ4CAVgKCwA9sp37UTQgQFA1yH0BDACyMoHy//J0AGBAQj0Cm+hMYAIBIAwNABmtznaiaEAga5Drhf/AABmvHfaiaEAQa5DrhY/AABG4yX7UTQ1wsfgAWb0kK29qJoQICga5D6AhhHDUCAhHpJN9KZEM5pA+n/mDeBKAG3gQFImHFZ8xhAT48oMI1xgg0x/TH9MfAvgju/Jk7UTQ0x/TH9P/9ATRUUO68qFRUbryogX5AVQQZPkQ8qP4ACSkyMsfUkDLH1Iwy/9SEPQAye1U+A8B0wchwACfbFGTINdKltMH1AL7AOgw4CHAAeMAIcAC4wABwAORMOMNA6TIyx8Syx/L/xESExQAbtIH+gDU1CL5AAXIygcVy//J0Hd0gBjIywXLAiLPFlAF+gIUy2sSzMzJc/sAyEAUgQEI9FHypwIAcIEBCNcY+gDTP8hUIEeBAQj0UfKnghBub3RlcHSAGMjLBcsCUAbPFlAE+gIUy2oSyx/LP8lz+wACAGyBAQjXGPoA0z8wUiSBAQj0WfKnghBkc3RycHSAGMjLBcsCUAXPFlAD+gITy2rLHxLLP8lz+wAACvQAye1UAFEAAAAAKamjF1M4HQpWKrIhrdY9Ou9RtUmildvf4qB7qOpqgADYbRTiQD9nbsU="
    //     }"#;
    
    #[test_only]
    use rooch_framework::ton_address;

    #[test]
    fun test_verify_proof(){
        let proof = TonProofData{
            name: string::utf8(b"ton_proof"),
            proof: TonProof{
                timestamp: 1730363765,
                domain: TonDomain{
                    length_bytes: 21,
                    value: string::utf8(b"ton-connect.github.io")
                },
                signature: string::utf8(b"BvysFrBS8KgTa3bww9f5paEu6/jZr5jB1JmO6T8nqsLzJqB3hWHiqOG9OezPsiJX3kD9nifMbRhr1xkv37ICCw=="),
                payload: vector[string::utf8(b"bc1q04uaa0mveqtt4y0sltuxtauhlyl8ctstr5x3hu")]
            },
            state_init: string::utf8(b"te6cckECFgEAAwQAAgE0ARUBFP8A9KQT9LzyyAsCAgEgAxACAUgEBwLm0AHQ0wMhcbCSXwTgItdJwSCSXwTgAtMfIYIQcGx1Z70ighBkc3RyvbCSXwXgA/pAMCD6RAHIygfL/8nQ7UTQgQFA1yH0BDBcgQEI9ApvoTGzkl8H4AXTP8glghBwbHVnupI4MOMNA4IQZHN0crqSXwbjDQUGAHgB+gD0BDD4J28iMFAKoSG+8uBQghBwbHVngx6xcIAYUATLBSbPFlj6Ahn0AMtpF8sfUmDLPyDJgED7AAYAilAEgQEI9Fkw7UTQgQFA1yDIAc8W9ADJ7VQBcrCOI4IQZHN0coMesXCAGFAFywVQA88WI/oCE8tqyx/LP8mAQPsAkl8D4gIBIAgPAgEgCQ4CAVgKCwA9sp37UTQgQFA1yH0BDACyMoHy//J0AGBAQj0Cm+hMYAIBIAwNABmtznaiaEAga5Drhf/AABmvHfaiaEAQa5DrhY/AABG4yX7UTQ1wsfgAWb0kK29qJoQICga5D6AhhHDUCAhHpJN9KZEM5pA+n/mDeBKAG3gQFImHFZ8xhAT48oMI1xgg0x/TH9MfAvgju/Jk7UTQ0x/TH9P/9ATRUUO68qFRUbryogX5AVQQZPkQ8qP4ACSkyMsfUkDLH1Iwy/9SEPQAye1U+A8B0wchwACfbFGTINdKltMH1AL7AOgw4CHAAeMAIcAC4wABwAORMOMNA6TIyx8Syx/L/xESExQAbtIH+gDU1CL5AAXIygcVy//J0Hd0gBjIywXLAiLPFlAF+gIUy2sSzMzJc/sAyEAUgQEI9FHypwIAcIEBCNcY+gDTP8hUIEeBAQj0UfKnghBub3RlcHSAGMjLBcsCUAbPFlAE+gIUy2oSyx/LP8lz+wACAGyBAQjXGPoA0z8wUiSBAQj0WfKnghBkc3RycHSAGMjLBcsCUAXPFlAD+gITy2rLHxLLP8lz+wAACvQAye1UAFEAAAAAKamjF1M4HQpWKrIhrdY9Ou9RtUmildvf4qB7qOpqgADYbRTiQD9nbsU=")
        };
        let ton_addr = ton_address::from_hex_str(
            &string::utf8(b"0:b1481ee8620ebf33b7882fa749654176ef00c7e4cac95ed39f371d5775920814"),
        );
        verify_proof(&ton_addr, &proof);
    }
}