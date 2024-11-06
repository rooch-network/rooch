// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::ton_proof {

    use std::vector;
    use std::string::{Self, String};
    
    use moveos_std::bcs;

    use rooch_framework::ton_address::{TonAddress};

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

    public fun decode_proof_data(proof_data_bytes: vector<u8>): TonProofData {
        bcs::from_bytes(proof_data_bytes)
    }
    
    /// verify the proof
    public fun verify_proof(_ton_addr: &TonAddress, _ton_proof_data: &TonProofData) : bool {
        //TODO
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

}