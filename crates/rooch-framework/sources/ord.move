// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::ord {
    use std::vector;
    use std::option::{Option};
    use moveos_std::bcs;
    use rooch_framework::bitcoin_types::{Self, Witness, Transaction};

    struct Inscription has store, copy, drop {
        body: Option<vector<u8>>,
        content_encoding: Option<vector<u8>>,
        content_type: Option<vector<u8>>,
        duplicate_field: bool,
        incomplete_field: bool,
        metadata: Option<vector<u8>>,
        metaprotocol: Option<vector<u8>>,
        parent: Option<vector<u8>>,
        pointer: Option<vector<u8>>,
        unrecognized_even_field: bool,
    }

    public fun from_transaction(transaction: &Transaction): vector<Inscription>{
        let inscriptions = vector::empty();
        let inputs = bitcoin_types::tx_input(transaction);
        let len = vector::length(inputs);
        let idx = 0;
        while(idx < len){
            let input = vector::borrow(inputs, idx);
            let witness = bitcoin_types::txin_witness(input);
            let inscriptions_from_witness = from_witness(witness);
            if(vector::length(&inscriptions_from_witness) > 0){
                vector::append(&mut inscriptions, inscriptions_from_witness);
            };
            idx = idx + 1;
        };
        inscriptions
    }

    public fun from_transaction_bytes(transaction_bytes: vector<u8>): vector<Inscription>{
        let transaction = bcs::from_bytes<Transaction>(transaction_bytes);
        from_transaction(&transaction)
    }

    public native fun from_witness(witness: &Witness): vector<Inscription>;
}