// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::ord {
    use std::vector;
    use std::option::{Self, Option};
    use std::string::{Self, String};
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

    // ==== Inscription ==== //

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

    public fun body(self: &Inscription): Option<vector<u8>>{
        self.body
    }

    public fun content_encoding(self: &Inscription): Option<vector<u8>>{
        self.content_encoding
    }

    public fun content_type(self: &Inscription): Option<String>{
        if(option::is_none(&self.content_type)){
            option::none()
        }else{
            let content_type = option::destroy_some(*&self.content_type);
            string::try_utf8(content_type)
        }
    }

    public fun duplicate_field(self: &Inscription): bool{
        self.duplicate_field
    }

    public fun incomplete_field(self: &Inscription): bool{
        self.incomplete_field
    }

    public fun metadata(self: &Inscription): Option<vector<u8>>{
        self.metadata
    }

    public fun metaprotocol(self: &Inscription): Option<vector<u8>>{
        self.metaprotocol
    }

    public fun parent(self: &Inscription): Option<vector<u8>>{
        self.parent
    }

    public fun pointer(self: &Inscription): Option<vector<u8>>{
        self.pointer
    }

    public fun unrecognized_even_field(self: &Inscription): bool{
        self.unrecognized_even_field
    }

    // === InscriptionStore === //

    public fun total_inscriptions(inscription_store_obj: &Object<InscriptionStore>): u64{
        let inscription_store = object::borrow(inscription_store_obj);
        table_vec::length(&inscription_store.inscription_ids)
    }

    public fun inscription_ids(inscription_store_obj: &Object<InscriptionStore>): &TableVec<InscriptionId>{
        let inscription_store = object::borrow(inscription_store_obj);
        &inscription_store.inscription_ids
    }

    public fun inscriptions(inscription_store_obj: &Object<InscriptionStore>): &Table<InscriptionId, Inscription>{
        let inscription_store = object::borrow(inscription_store_obj);
        &inscription_store.inscriptions
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

    #[test]
    fun test_inscription(){
        let tx_bytes = x"3d33603763560b82824746834918f6e309b051416a288e06a671185f00443ca0020000000000000001361cc743a923abc1db73f4fed4d0778cc8ccc092cb20f1c66cada177818e55b20000000000fdffffff03401500c4f407f66ec47c92e1daf34c46f2b52837819119b696e343385b6dba27682dd89f9e4d18354ce0f4a4200ddab8420457392702e1e0b6d51803d25d2bf2647f2016c3a3f18eb4efd24274941ba02c899d151b0473a1bad3512423cbe1b0648ea9ac0063036f7264010118746578742f706c61696e3b636861727365743d7574662d3800397b2270223a226272632d3230222c226f70223a227472616e73666572222c227469636b223a226f726469222c22616d74223a2231303030227d6821c102a58d972468a33a79350cf24cb991f28adbbe3e64e88ded5f58f558fff2b673022202000000000000225120e5053d2151d14399a3a4825740e14deae6f984e990e0a6872df065a6dad7009c6e04000000000000160014ad45c620bd9b6688c5a7a23e515402d39d02b552";
        let tx = bcs::from_bytes<Transaction>(tx_bytes);
        let inscriptions = from_transaction(&tx);
        let inscriptions_len = vector::length(&inscriptions);
        std::debug::print(&inscriptions);
        assert!(inscriptions_len == 1, 1);
        let inscription = vector::borrow(&inscriptions, 0);
        let body = string::utf8(option::destroy_some(Self::body(inscription)));
        std::debug::print(&body);
        assert!(body == string::utf8(b"{\"p\":\"brc-20\",\"op\":\"transfer\",\"tick\":\"ordi\",\"amt\":\"1000\"}"), 1);
        let content_type = std::option::destroy_some(Self::content_type(inscription));
        std::debug::print(&content_type);
        assert!(content_type == string::utf8(b"text/plain;charset=utf-8"), 2);
    }
}