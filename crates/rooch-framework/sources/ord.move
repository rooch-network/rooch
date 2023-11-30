// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::ord {
    use std::vector;
    use std::option::{Option};
    use moveos_std::bcs;
    use moveos_std::context::{Self, Context};
    use moveos_std::object::{Self, Object};
    use moveos_std::table::{Self, Table};
    use moveos_std::table_vec::{Self, TableVec};
    use rooch_framework::bitcoin_types::{Self, Witness, Transaction};
    use rooch_framework::bitcoin_light_client::{Self, BitcoinBlockStore};

    friend rooch_framework::genesis;

    struct InscriptionId has store, copy, drop {
        txid: address,
        index: u32,
    }

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

    struct InscriptionStore has key{
        /// The next transaction index to be processed
        next_tx_index: u64,
        inscriptions: Table<InscriptionId, Inscription>,
        inscription_ids: TableVec<InscriptionId>,
    }

     public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer){
        let inscription_store = InscriptionStore{
            next_tx_index: 0,
            inscriptions: context::new_table(ctx),
            inscription_ids: context::new_table_vec(ctx),
        }; 
        let obj = context::new_named_object(ctx, inscription_store);
        object::to_shared(obj);
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

    native fun from_witness(witness: &Witness): vector<Inscription>;

    public fun total_inscriptions(inscription_store_obj: &Object<InscriptionStore>): u64{
        let inscription_store = object::borrow(inscription_store_obj);
        table_vec::length(&inscription_store.inscription_ids)
    }

    public fun remaining_tx_count(btc_block_store_obj: &Object<BitcoinBlockStore>, inscription_store_obj: &Object<InscriptionStore>): u64{
        let inscription_store = object::borrow(inscription_store_obj);
        let start_tx_index = inscription_store.next_tx_index;
        let max_tx_count = table_vec::length(bitcoin_light_client::tx_ids(btc_block_store_obj));
        if(start_tx_index < max_tx_count){
            max_tx_count - start_tx_index
        }else{
            0
        }
    }

    entry fun progress_inscriptions(btc_block_store_obj:&Object<BitcoinBlockStore>, inscription_store_obj: &mut Object<InscriptionStore>,batch_size: u64){
        let inscription_store = object::borrow_mut(inscription_store_obj);
        let txs:&Table<address, Transaction> = bitcoin_light_client::txs(btc_block_store_obj);
        let tx_ids: &TableVec<address> = bitcoin_light_client::tx_ids(btc_block_store_obj);
        let start_tx_index = inscription_store.next_tx_index;
        let max_tx_count = table_vec::length(tx_ids);
        if(start_tx_index >= max_tx_count){
            return
        };
        let progressed_tx_count = 0;
        let progress_tx_index = start_tx_index;
        while(progressed_tx_count < batch_size && progress_tx_index < max_tx_count){
            let tx_id = *table_vec::borrow(tx_ids, progress_tx_index);
            let tx = table::borrow(txs, tx_id);
            let inscriptions = from_transaction(tx);
            let inscriptions_len = vector::length(&inscriptions);
            let inscription_idx = 0;
            while(inscription_idx < inscriptions_len){
                let inscription = vector::borrow(&inscriptions, inscription_idx);
                let inscription_id = InscriptionId{
                    txid: tx_id,
                    index: (inscription_idx as u32),
                };
                table::add(&mut inscription_store.inscriptions, inscription_id, *inscription);
                table_vec::push_back(&mut inscription_store.inscription_ids, inscription_id);
                inscription_idx = inscription_idx + 1;
            };
            progressed_tx_count = progressed_tx_count + 1;
            progress_tx_index = progress_tx_index + 1;
        };
        inscription_store.next_tx_index = progress_tx_index;
    }
}