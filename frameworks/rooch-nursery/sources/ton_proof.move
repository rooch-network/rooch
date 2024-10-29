// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::ton_proof {

    use std::string::{String};
    
    use moveos_std::bcs;

    use rooch_nursery::ton_address::{TonAddress};

    #[data_struct]
    struct TonDomain has copy, drop, store{
        length_bytes: u64,
        value: String,
    }

    #[data_struct]
    struct TonProof has copy, drop, store{
        domain: TonDomain,
        payload: String,
        signature: String,
        state_init: String,
        timestamp: u64,
    }

    public fun decode_proof(ton_proof_bytes: vector<u8>): TonProof {
        bcs::from_bytes(ton_proof_bytes)
    }
    
    /// verify the proof
    public fun verify_proof(_ton_addr: &TonAddress, _ton_proof: &TonProof) : bool {
        //TODO
        true
    }

    // ======================== TonProof functions ========================

    public fun domain(ton_proof: &TonProof): &TonDomain {
        &ton_proof.domain
    }

    public fun payload(ton_proof: &TonProof): &String {
        &ton_proof.payload
    }

    public fun signature(ton_proof: &TonProof): &String {
        &ton_proof.signature
    }

    public fun state_init(ton_proof: &TonProof): &String {
        &ton_proof.state_init
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