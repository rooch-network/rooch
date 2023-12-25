// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::ord {
    use std::vector;
    use std::option::{Self, Option};
    use std::string::String;
    use moveos_std::bcs;
    use moveos_std::event;
    use moveos_std::context::{Self, Context};
    use moveos_std::object::{Self, ObjectID, Object};
    use bitcoin_move::types::{Self, Witness, Transaction};
    use bitcoin_move::utxo::{Self, UTXO, SealOut};

    struct InscriptionID has store, copy, drop {
        txid: address,
        index: u32,
    }

    struct Inscription has key{
        txid: address,
        index: u32,
        body: vector<u8>,
        content_encoding: Option<String>,
        content_type: Option<String>,
        metadata: vector<u8>,
        metaprotocol: Option<String>,
        parent: Option<ObjectID>,
        pointer: Option<u64>,
    }

    struct InscriptionRecord has store, copy, drop {
        body: vector<u8>,
        content_encoding: Option<String>,
        content_type: Option<String>,
        duplicate_field: bool,
        incomplete_field: bool,
        metadata: vector<u8>,
        metaprotocol: Option<String>,
        parent: Option<InscriptionID>,
        pointer: Option<u64>,
        unrecognized_even_field: bool,
    }

    struct InvalidInscriptionEvent has store, copy, drop {
        txid: address,
        input_index: u64,
        record: InscriptionRecord,
    }

    // ==== Inscription ==== //

    fun new_inscription(ctx: &mut Context, txid: address, index:u32, record: InscriptionRecord): Object<Inscription> {
        let parent = option::map(record.parent, |e| object::custom_object_id<InscriptionID,Inscription>(e));
        let inscription = Inscription{
            txid: txid,
            index: index,
            body: record.body,
            content_encoding: record.content_encoding,
            content_type: record.content_type,
            metadata: record.metadata,
            metaprotocol: record.metaprotocol,
            parent: parent,
            pointer: record.pointer,
        };
        let id = InscriptionID{
            txid: txid,
            index: index,
        };
        context::new_custom_object(ctx, id, inscription)
    }

    public fun exists_inscription(ctx: &Context, txid: address, index: u32): bool{
        let id = InscriptionID{
            txid: txid,
            index: index,
        };
        let object_id = object::custom_object_id<InscriptionID,Inscription>(id);
        context::exists_object<Inscription>(ctx, object_id)
    }

    public fun borrow_inscription(ctx: &Context, txid: address, index: u32): &Object<Inscription>{
        let id = InscriptionID{
            txid: txid,
            index: index,
        };
        let object_id = object::custom_object_id<InscriptionID,Inscription>(id);
        context::borrow_object(ctx, object_id)
    }

    public fun spend_utxo(ctx: &mut Context, utxo_obj: &Object<UTXO>, tx: &Transaction): vector<SealOut>{
        let utxo = object::borrow(utxo_obj);
        let seal_object_ids = utxo::get_seals<Inscription>(utxo);
        let seal_outs = vector::empty();
        if(vector::is_empty(&seal_object_ids)){
            return seal_outs
        };
        let outputs = types::tx_output(tx);
        //TODO we should track the Inscription via SatPoint, but now we just use the first output for simplicity.
        let output_index = 0;
        let first_output = vector::borrow(outputs, output_index);
        let address = types::txout_object_address(first_output);
        let j = 0;
        let objects_len = vector::length(&seal_object_ids);
        while(j < objects_len){
            let seal_object_id = *vector::borrow(&mut seal_object_ids, j);
            let inscription_obj = context::take_object_extend<Inscription>(ctx, seal_object_id); 
            object::transfer_extend(inscription_obj, address);
            vector::push_back(&mut seal_outs, utxo::new_seal_out(output_index, seal_object_id));
            j = j + 1;
        };
        seal_outs
    }

    public fun progress_transaction(ctx: &mut Context, tx: &Transaction): vector<SealOut>{
        let tx_id = types::tx_id(tx);
        let output_seals = vector::empty();

        let inscription_records = from_transaction(tx);
        let inscription_records_len = vector::length(&inscription_records);
        if(inscription_records_len == 0){
            return output_seals
        };

        let tx_outputs = types::tx_output(tx);
        let output_len = vector::length(tx_outputs);

        // ord has three mode for Inscribe:   SameSat,SeparateOutputs,SharedOutput,
        //https://github.com/ordinals/ord/blob/master/src/subcommand/wallet/inscribe/batch.rs#L533
        //TODO handle SameSat
        let is_separate_outputs = output_len > inscription_records_len;
        let idx = 0;
        while(idx < inscription_records_len){
            let inscription_record = *vector::borrow(&mut inscription_records, idx);
            
            let inscription_obj = new_inscription(ctx, tx_id, (idx as u32), inscription_record);
            let object_id = object::id(&inscription_obj);
            let output_index = if(is_separate_outputs){
                idx
            }else{
                0  
            };
            let output = vector::borrow(tx_outputs, output_index);
            let address = types::txout_object_address(output);
            object::transfer_extend(inscription_obj, address);
            vector::push_back(&mut output_seals, utxo::new_seal_out(output_index, object_id));
            idx = idx + 1;
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

    public fun body(self: &Inscription): vector<u8>{
        self.body
    }

    public fun content_encoding(self: &Inscription): Option<String>{
        self.content_encoding
    }

    public fun content_type(self: &Inscription): Option<String>{
        self.content_type
    }

    public fun metadata(self: &Inscription): vector<u8>{
        self.metadata
    }

    public fun metaprotocol(self: &Inscription): Option<String>{
        self.metaprotocol
    }

    public fun parent(self: &Inscription): Option<ObjectID>{
        self.parent
    }

    public fun pointer(self: &Inscription): Option<u64>{
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
    (vector<u8>, Option<String>, Option<String>, vector<u8>, Option<String>, Option<InscriptionID>, Option<u64>){
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
        let tx_id = types::tx_id(tx);
        let inscription_records = vector::empty();
        let inputs = types::tx_input(tx);
        let len = vector::length(inputs);
        let idx = 0;
        while(idx < len){
            let input = vector::borrow(inputs, idx);
            let witness = types::txin_witness(input);
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


}