// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::ord {
    use std::vector;
    use std::option::{Self, Option};
    use std::string;
    use std::string::String;
    use rooch_framework::address_mapping;
    use rooch_framework::multichain_address;
    use rooch_framework::bitcoin_address::BitcoinAddress;
    use moveos_std::bcs;
    use moveos_std::event;
    use moveos_std::context::{Self, Context};
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::json;
    use moveos_std::table_vec::{Self, TableVec};
    use bitcoin_move::types::{Self, Witness, Transaction};
    use bitcoin_move::utxo::{Self, UTXO, SealOut};
    use bitcoin_move::brc20;

    friend bitcoin_move::genesis;

    friend bitcoin_move::light_client;

    struct InscriptionID has store, copy, drop {
        txid: address,
        index: u32,
    }

    struct Inscription has key{
        txid: address,
        index: u32,
        /// Transaction input index
        input: u32,
        body: vector<u8>,
        content_encoding: Option<String>,
        content_type: Option<String>,
        metadata: vector<u8>,
        metaprotocol: Option<String>,
        parent: Option<ObjectID>,
        pointer: Option<u64>,
        /// If the Inscription body is a JSON object, this field contains the parsed JSON object as a map.
        /// Otherwise, this field is empty map
        json_body: SimpleMap<String,String>,
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

    struct InscriptionStore has key{
        inscriptions: TableVec<InscriptionID>,
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer){
        let inscriptions = context::new_table_vec<InscriptionID>(ctx);
        let store = InscriptionStore{
            inscriptions: inscriptions,
        };
        let store_obj = context::new_named_object(ctx, store);
        object::to_shared(store_obj);
    }

    // ==== Inscription ==== //

    fun record_to_inscription(txid: address, index: u32, input: u32, record: InscriptionRecord): Inscription{
        let parent = option::map(record.parent, |e| object::custom_object_id<InscriptionID,Inscription>(e));
        let json_body = parse_json_body(&record);
        Inscription{
            txid: txid,
            index: index,
            input: input,
            body: record.body,
            content_encoding: record.content_encoding,
            content_type: record.content_type,
            metadata: record.metadata,
            metaprotocol: record.metaprotocol,
            parent: parent,
            pointer: record.pointer,
            json_body: json_body,
        }
    }

    fun create_obj(ctx: &mut Context, inscription: Inscription): Object<Inscription> {
        let id = InscriptionID{
            txid: inscription.txid,
            index: inscription.index,
        };
        let store_obj_id = object::named_object_id<InscriptionStore>();
        let store_obj = context::borrow_mut_object_shared<InscriptionStore>(ctx, store_obj_id);
        let store = object::borrow_mut(store_obj);
        table_vec::push_back(&mut store.inscriptions, id);
        context::new_custom_object(ctx, id, inscription)
    }
    
    fun parse_json_body(record: &InscriptionRecord) : SimpleMap<String,String> {
        if (vector::is_empty(&record.body) || option::is_none(&record.content_type)) {
            return simple_map::new()
        };
        let content_type = option::destroy_some(record.content_type);
        if(content_type != string::utf8(b"text/plain;charset=utf-8") || content_type != string::utf8(b"text/plain") || content_type != string::utf8(b"application/json")){
            return simple_map::new()
        };
        json::to_map(record.body)
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

    public fun spend_utxo(ctx: &mut Context, utxo_obj: &mut Object<UTXO>, tx: &Transaction): vector<SealOut>{
        let utxo = object::borrow_mut(utxo_obj);
        let seal_object_ids = utxo::remove_seals<Inscription>(utxo);
        let seal_outs = vector::empty();
        if(vector::is_empty(&seal_object_ids)){
            return seal_outs
        };
        let outputs = types::tx_output(tx);
        //TODO we should track the Inscription via SatPoint, but now we just use the first output for simplicity.
        let output_index = 0;
        let first_output = vector::borrow(outputs, output_index);
        let address = types::txout_object_address(first_output);
        let bitcoin_address_opt = types::txout_address(first_output);
        let to_address = types::txout_object_address(first_output);
        let j = 0;
        let objects_len = vector::length(&seal_object_ids);
        while(j < objects_len){
            let seal_object_id = *vector::borrow(&mut seal_object_ids, j);
            let (origin_owner, inscription_obj) = context::take_object_extend<Inscription>(ctx, seal_object_id);
            let inscription = object::borrow(&inscription_obj); 
            if(brc20::is_brc20(&inscription.json_body)){
                let op = brc20::new_op(origin_owner, to_address, simple_map::clone(&inscription.json_body));
                brc20::process_utxo_op(ctx, op);
                //TODO record the execution result
            };
            object::transfer_extend(inscription_obj, to_address);
            vector::push_back(&mut seal_outs, utxo::new_seal_out(output_index, seal_object_id));
            j = j + 1;
        };
        //Auto create address mapping if not exist
        bind_multichain_address(ctx, address, bitcoin_address_opt);
        seal_outs
    }

    public fun process_transaction(ctx: &mut Context, tx: &Transaction): vector<SealOut>{
        let output_seals = vector::empty();

        let inscriptions = from_transaction(tx);
        let inscriptions_len = vector::length(&inscriptions);
        if(inscriptions_len == 0){
            vector::destroy_empty(inscriptions);
            return output_seals
        };

        let tx_outputs = types::tx_output(tx);
        let output_len = vector::length(tx_outputs);

        // ord has three mode for Inscribe:   SameSat,SeparateOutputs,SharedOutput,
        //https://github.com/ordinals/ord/blob/master/src/subcommand/wallet/inscribe/batch.rs#L533
        //TODO handle SameSat
        let is_separate_outputs = output_len > inscriptions_len;
        let idx = 0;
        //reverse inscriptions and pop from the end
        vector::reverse(&mut inscriptions);
        while(idx < inscriptions_len){
            let output_index = if(is_separate_outputs){
                idx
            }else{
                0  
            };

            let output = vector::borrow(tx_outputs, output_index);
            let to_address = types::txout_object_address(output);
            let bitcoin_address_opt = types::txout_address(output);

            let inscription = vector::pop_back(&mut inscriptions);
            //Because the previous output of inscription input is a witness program address, so we simply use the output address as the from address.
            let from = to_address;
            process_inscribe_protocol(ctx, from, to_address, &inscription);
            let inscription_obj = create_obj(ctx, inscription);
            let object_id = object::id(&inscription_obj);

            object::transfer_extend(inscription_obj, to_address);
            vector::push_back(&mut output_seals, utxo::new_seal_out(output_index, object_id));
            //Auto create address mapping if not exist
            bind_multichain_address(ctx, to_address, bitcoin_address_opt);
            idx = idx + 1;
        };
        vector::destroy_empty(inscriptions);
        output_seals
    }

    fun process_inscribe_protocol(ctx: &mut Context, from: address, to: address, inscription: &Inscription){
        if (brc20::is_brc20(&inscription.json_body)){
            let op = brc20::new_op(from, to, simple_map::clone(&inscription.json_body));
            brc20::process_inscribe_op(ctx, op);
            //TODO record the execution result
        };
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
            input: _,
            body: _,
            content_encoding: _,
            content_type: _,
            metadata: _,
            metaprotocol: _,
            parent: _,
            pointer: _,
            json_body,
        } = self;
        simple_map::drop(json_body);
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

    public fun from_transaction(tx: &Transaction): vector<Inscription>{
        let tx_id = types::tx_id(tx);
        let inscriptions = vector::empty();
        let inputs = types::tx_input(tx);
        let len = vector::length(inputs);
        let input_idx = 0;
        let index_counter = 0;
        while(input_idx < len){
            let input = vector::borrow(inputs, input_idx);
            let witness = types::txin_witness(input);
            let inscription_records_from_witness = from_witness(witness);
            if(!vector::is_empty(&inscription_records_from_witness)){
                vector::for_each(inscription_records_from_witness,|record|{
                    let inscription = record_to_inscription(tx_id, (index_counter as u32), (input_idx as u32), record);
                    vector::push_back(&mut inscriptions, inscription);
                    index_counter = index_counter + 1;
                })
            };
            input_idx = input_idx + 1;
        };
        inscriptions
    }

    public fun from_transaction_bytes(transaction_bytes: vector<u8>): vector<Inscription>{
        let transaction = bcs::from_bytes<Transaction>(transaction_bytes);
        from_transaction(&transaction)
    }

    native fun from_witness(witness: &Witness): vector<InscriptionRecord>;

    public(friend) fun bind_multichain_address(ctx: &mut Context, rooch_address: address, bitcoin_address_opt: Option<BitcoinAddress>) {
        //Auto create address mapping if not exist
        if(option::is_some(&bitcoin_address_opt)) {
            let bitcoin_address = option::extract(&mut bitcoin_address_opt);
            let maddress = multichain_address::from_bitcoin(bitcoin_address);
            if (!address_mapping::exists_mapping(ctx, maddress)) {
                let bitcoin_move_signer = moveos_std::signer::module_signer<Inscription>();
                address_mapping::bind_by_system(ctx, &bitcoin_move_signer, rooch_address, maddress);
            };
        };
    }

}