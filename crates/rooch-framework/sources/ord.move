// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::ord {
    use std::vector;
    use std::option::{Self, Option};
    use std::string::{Self, String};
    use moveos_std::bcs;
    use moveos_std::event;
    use moveos_std::context::{Self, Context};
    use moveos_std::object::{Self, ObjectID, Object};
    use rooch_framework::bitcoin_types::{Self, Witness, Transaction};
    use rooch_framework::utxo::{Self, UTXO, SealOut};

    friend rooch_framework::genesis;

    struct InscriptionId has store, copy, drop {
        txid: address,
        index: u32,
    }

    struct Inscription has key{
        txid: address,
        index: u32,
        body: Option<vector<u8>>,
        content_encoding: Option<vector<u8>>,
        content_type: Option<vector<u8>>,
        metadata: Option<vector<u8>>,
        metaprotocol: Option<vector<u8>>,
        parent: Option<ObjectID>,
        pointer: Option<vector<u8>>,
    }

    struct InscriptionRecord has store, copy, drop {
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

    struct InvalidInscriptionEvent has store, copy, drop {
        txid: address,
        input_index: u64,
        record: InscriptionRecord,
    }

    struct MultiInscriptionEvent has store, copy, drop {
        txid: address,
        inscription_records: vector<InscriptionRecord>,
    }

    // ==== Inscription ==== //

    fun new_inscription(txid: address, index:u32, record: InscriptionRecord): Inscription {
        Inscription{
            txid: txid,
            index: index,
            body: record.body,
            content_encoding: record.content_encoding,
            content_type: record.content_type,
            metadata: record.metadata,
            metaprotocol: record.metaprotocol,
             //TODO handle parent
            parent: option::none(),
            pointer: record.pointer,
        }
    }

    public fun spend_utxo(ctx: &mut Context, utxo_obj: &Object<UTXO>, tx: &Transaction): Option<SealOut>{
        let utxo = object::borrow(utxo_obj);
        let seal_opt = utxo::get_seal<Inscription>(utxo);
        if(option::is_none(&seal_opt)){
            return option::none()
        };
        let seal_object_id = option::destroy_some(seal_opt);
        let inscription_obj = context::take_object_extend<Inscription>(ctx, seal_object_id); 
        let outputs = bitcoin_types::tx_output(tx);
        //TODO we should track the Inscription via SatPoint, but now we just use the first output for simplicity.
        let output_index = 0;
        let first_output = vector::borrow(outputs, output_index);
        let address = bitcoin_types::txout_object_address(first_output);
        object::transfer_extend(inscription_obj, address);
        option::some(utxo::new_seal_out(output_index, seal_object_id))
    }

    public fun progress_transaction(ctx: &mut Context, tx: &Transaction): vector<SealOut>{
        let tx_id = bitcoin_types::tx_id(tx);
        let output_seals = vector::empty();

        let inscription_records = from_transaction(tx);
        let inscription_records_len = vector::length(&inscription_records);
        let tx_outputs = bitcoin_types::tx_output(tx);
        let output_len = vector::length(tx_outputs);

        if(inscription_records_len != output_len && output_len != 1){
            event::emit(MultiInscriptionEvent{
                    txid: tx_id,
                    inscription_records: inscription_records,
            });
        };

        // ord has three mode for Inscribe:   SameSat,SeparateOutputs,SharedOutput,
        //https://github.com/ordinals/ord/blob/master/src/subcommand/wallet/inscribe/batch.rs#L533
        //TODO handle SameSat
        let is_separate_outputs = inscription_records_len == output_len;
        if(inscription_records_len > 0){
            let idx = 0;
            while(idx < inscription_records_len){
                let inscription_record = *vector::borrow(&mut inscription_records, idx);
                
                let inscription = new_inscription(tx_id, (idx as u32), inscription_record);
                //TODO custom Inscription ID?
                let inscription_obj = context::new_object(ctx, inscription);
                let object_id = object::id(&inscription_obj);
                let output_index = if(is_separate_outputs){
                    idx
                }else{
                    0  
                };
                let output = vector::borrow(tx_outputs, output_index);
                let address = bitcoin_types::txout_object_address(output);
                object::transfer_extend(inscription_obj, address);
                vector::push_back(&mut output_seals, utxo::new_seal_out(output_index, object_id));
                idx = idx + 1;
            };
        };
        output_seals
    }

    fun validate_inscription_records(tx_id: address, input_index: u64, record: vector<InscriptionRecord>): vector<InscriptionRecord>{
        let len = vector::length(&record);
        let idx = 0;
        let valid_records = vector::empty();
        while(idx < len){
            let record = *vector::borrow(&mut record, idx);
            if(!record.duplicate_field && !record.incomplete_field && !record.unrecognized_even_field){
                vector::push_back(&mut valid_records, record);
            }else{
               event::emit(InvalidInscriptionEvent{
                   txid: tx_id,
                   input_index: input_index,
                   record: record,
               }); 
            };
            idx = idx + 1;
        };
        valid_records
    } 

    public fun txid(self: &Inscription): address{
        self.txid
    }

    public fun index(self: &Inscription): u32{
        self.index
    }

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

    public fun metadata(self: &Inscription): Option<vector<u8>>{
        self.metadata
    }

    public fun metaprotocol(self: &Inscription): Option<vector<u8>>{
        self.metaprotocol
    }

    public fun parent(self: &Inscription): Option<ObjectID>{
        self.parent
    }

    public fun pointer(self: &Inscription): Option<vector<u8>>{
        self.pointer
    }

    fun drop(self: Inscription){
        let Inscription{
            txid: _,
            index: _,
            body: _,
            content_encoding: _,
            content_type: _,
            metadata: _,
            metaprotocol: _,
            parent: _,
            pointer: _,
        } = self;
    }

    // ==== InscriptionRecord ==== //

    public fun unpack_record(record: InscriptionRecord): 
    (Option<vector<u8>>, Option<vector<u8>>, Option<vector<u8>>, Option<vector<u8>>, Option<vector<u8>>, Option<vector<u8>>, Option<vector<u8>>){
        let InscriptionRecord{
            body: body,
            content_encoding: content_encoding,
            content_type: content_type,
            duplicate_field: _,
            incomplete_field: _,
            metadata: metadata,
            metaprotocol: metaprotocol,
            parent: parent,
            pointer: pointer,
            unrecognized_even_field: _,
        } = record;
        (body, content_encoding, content_type, metadata, metaprotocol, parent, pointer)
    }

    public fun from_transaction(tx: &Transaction): vector<InscriptionRecord>{
        let tx_id = bitcoin_types::tx_id(tx);
        let inscription_records = vector::empty();
        let inputs = bitcoin_types::tx_input(tx);
        let len = vector::length(inputs);
        let idx = 0;
        while(idx < len){
            let input = vector::borrow(inputs, idx);
            let witness = bitcoin_types::txin_witness(input);
            let inscription_records_from_witness = validate_inscription_records(tx_id, idx, from_witness(witness));
            if(vector::length(&inscription_records_from_witness) > 0){
                vector::append(&mut inscription_records, inscription_records_from_witness);
            };
            idx = idx + 1;
        };
        inscription_records
    }

    public fun from_transaction_bytes(transaction_bytes: vector<u8>): vector<InscriptionRecord>{
        let transaction = bcs::from_bytes<Transaction>(transaction_bytes);
        from_transaction(&transaction)
    }

    native fun from_witness(witness: &Witness): vector<InscriptionRecord>;



    #[test]
    fun test_inscription(){
        let tx_bytes = x"3d33603763560b82824746834918f6e309b051416a288e06a671185f00443ca0020000000000000001361cc743a923abc1db73f4fed4d0778cc8ccc092cb20f1c66cada177818e55b20000000000fdffffff03401500c4f407f66ec47c92e1daf34c46f2b52837819119b696e343385b6dba27682dd89f9e4d18354ce0f4a4200ddab8420457392702e1e0b6d51803d25d2bf2647f2016c3a3f18eb4efd24274941ba02c899d151b0473a1bad3512423cbe1b0648ea9ac0063036f7264010118746578742f706c61696e3b636861727365743d7574662d3800397b2270223a226272632d3230222c226f70223a227472616e73666572222c227469636b223a226f726469222c22616d74223a2231303030227d6821c102a58d972468a33a79350cf24cb991f28adbbe3e64e88ded5f58f558fff2b673022202000000000000225120e5053d2151d14399a3a4825740e14deae6f984e990e0a6872df065a6dad7009c6e04000000000000160014ad45c620bd9b6688c5a7a23e515402d39d02b552";
        let tx = bcs::from_bytes<Transaction>(tx_bytes);
        let tx_id = bitcoin_types::tx_id(&tx);
        let inscription_records = from_transaction(&tx);
        let inscription_records_len = vector::length(&inscription_records);
        std::debug::print(&inscription_records);
        assert!(inscription_records_len == 1, 1);
        let inscription_record = vector::remove(&mut inscription_records, 0);
        let inscription = new_inscription(tx_id, 0, inscription_record);
        let body = string::utf8(option::destroy_some(Self::body(&inscription)));
        std::debug::print(&body);
        assert!(body == string::utf8(b"{\"p\":\"brc-20\",\"op\":\"transfer\",\"tick\":\"ordi\",\"amt\":\"1000\"}"), 1);
        let content_type = std::option::destroy_some(Self::content_type(&inscription));
        std::debug::print(&content_type);
        assert!(content_type == string::utf8(b"text/plain;charset=utf-8"), 2);
        drop(inscription);
    }
}