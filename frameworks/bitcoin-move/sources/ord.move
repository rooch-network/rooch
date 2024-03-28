// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::ord {
    use std::debug;
    use std::vector;
    use std::option::{Self, Option};
    use std::string;
    use std::string::String;
    use moveos_std::big_vector;
    use moveos_std::big_vector::BigVector;
    use moveos_std::table;
    use moveos_std::table::Table;

    use moveos_std::bcs;
    use moveos_std::event;
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::json;
    use moveos_std::table_vec::{Self, TableVec};

    use rooch_framework::address_mapping;
    use rooch_framework::multichain_address;
    use rooch_framework::bitcoin_address::BitcoinAddress;

    use bitcoin_move::types::{Self, Witness, Transaction, OutPoint, TxOut};
    use bitcoin_move::utxo::{Self, SealOut};
    use bitcoin_move::brc20;

    friend bitcoin_move::genesis;

    friend bitcoin_move::light_client;

    const OUTPOINT_TO_SATPOINT_BUCKET_SIZE: u64 = 1000;

    struct InscriptionID has store, copy, drop {
        txid: address,
        index: u32,
    }

    struct Inscription has key{
        txid: address,
        index: u32,
        /// Transaction input index
        input: u32,
        offset: u64,
        /// utxo value
        value: u64,

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
        satpoint_to_inscription: Table<SatPoint, InscriptionID>,
        outpoint_to_satpoint: Table<OutPoint, BigVector<SatPoint>>,
        // outpoint_to_satpoint: Table<OutPoint, vector<SatPoint>>,
    }

    #[data_struct]
    struct SatPoint has store, copy, drop {
        outpoint: OutPoint,
        offset: u64,
    }

    struct SatPointRange has store, copy, drop {
        txid: address,
        /// The index of the referenced output in its transaction's vout.
        vout: u32,
        offset: u64,
    }

    struct SatPointMapping has store, copy, drop {
        old_satpoint: SatPoint,
        new_satpoint: SatPoint,
    }

    public(friend) fun genesis_init(_genesis_account: &signer){
        let inscriptions = table_vec::new<InscriptionID>();
        let satpoint_to_inscription = table::new<SatPoint, InscriptionID>();
        let outpoint_to_satpoint = table::new<OutPoint, BigVector<SatPoint>>();
        // let outpoint_to_satpoint = table::new<OutPoint, vector<SatPoint>>();
        let store = InscriptionStore{
            inscriptions,
            satpoint_to_inscription,
            outpoint_to_satpoint
        };
        let store_obj = object::new_named_object(store);
        object::to_shared(store_obj);
    }

    fun borrow_mut_inscription_store() : &mut InscriptionStore {
        let inscription_store_object_id = object::named_object_id<InscriptionStore>();
        let inscription_store_obj = object::borrow_mut_object_shared<InscriptionStore>(inscription_store_object_id);
        object::borrow_mut(inscription_store_obj)
    }


    fun borrow_inscription_store() : &InscriptionStore {
        let inscription_store_object_id = object::named_object_id<InscriptionStore>();
        let inscription_store_obj = object::borrow_object<InscriptionStore>(inscription_store_object_id);
        object::borrow(inscription_store_obj)
    }


    public fun new_inscription_id(txid: address, index: u32) : InscriptionID {
        InscriptionID{
            txid,
            index,
        }
    }

    public fun new_satpoint(txid: address, vout: u32, offset: u64) : SatPoint {
        let outpoint = types::new_outpoint(txid, vout);
        SatPoint{
            outpoint,
            offset,
        }
    }

    /// Get the SatPoint's offset
    public fun satpoint_offset(satpoint: &SatPoint): u64 {
        satpoint.offset
    }

    /// Get the SatPoint's offset
    public fun satpoint_txid(satpoint: &SatPoint): address {
        types::outpoint_txid(&satpoint.outpoint)
    }

    public fun new_satpoint_mapping(old_satpoint: SatPoint, new_satpoint: SatPoint) : SatPointMapping {
        SatPointMapping{
            old_satpoint,
            new_satpoint,
        }
    }

    /// Get the SatPoint's mapping
    public fun unpack_satpoint_mapping(satpoint_mapping: &SatPointMapping): (SatPoint, SatPoint) {
        (satpoint_mapping.old_satpoint, satpoint_mapping.new_satpoint)
    }

    // ==== Inscription ==== //

    fun record_to_inscription(txid: address, index: u32, input: u32, offset: u64, value: u64, record: InscriptionRecord): Inscription{
        let parent = option::map(record.parent, |e| object::custom_object_id<InscriptionID,Inscription>(e));
        let json_body = parse_json_body(&record);
        Inscription{
            txid,
            index,
            input,
            offset,
            value,
            body: record.body,
            content_encoding: record.content_encoding,
            content_type: record.content_type,
            metadata: record.metadata,
            metaprotocol: record.metaprotocol,
            parent,
            pointer: record.pointer,
            json_body,
        }
    }

    fun create_obj(inscription: Inscription): Object<Inscription> {
        let id = InscriptionID{
            txid: inscription.txid,
            index: inscription.index,
        };
        let store_obj_id = object::named_object_id<InscriptionStore>();
        let store_obj = object::borrow_mut_object_shared<InscriptionStore>(store_obj_id);
        let store = object::borrow_mut(store_obj);
        table_vec::push_back(&mut store.inscriptions, id);
        object::new_custom_object(id, inscription)
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

    public fun exists_inscription(txid: address, index: u32): bool{
        let id = InscriptionID{
            txid: txid,
            index: index,
        };
        let object_id = object::custom_object_id<InscriptionID,Inscription>(id);
        object::exists_object_with_type<Inscription>(object_id)
    }

    public fun borrow_inscription(txid: address, index: u32): &Object<Inscription>{
        let id = InscriptionID{
            txid: txid,
            index: index,
        };
        let object_id = object::custom_object_id<InscriptionID,Inscription>(id);
        object::borrow_object(object_id)
    }

    // public fun spend_utxo(utxo_obj: &mut Object<UTXO>, tx: &Transaction): vector<SealOut>{
    //     let utxo = object::borrow_mut(utxo_obj);
    //
    //     let seal_object_ids = utxo::remove_seals<Inscription>(utxo);
    //     let seal_outs = vector::empty();
    //     if(vector::is_empty(&seal_object_ids)){
    //         return seal_outs
    //     };
    //     let outputs = types::tx_output(tx);
    //     //TODO we should track the Inscription via SatPoint, but now we just use the first output for simplicity.
    //
    //     let output_index = 0;
    //     let first_output = vector::borrow(outputs, output_index);
    //     let address = types::txout_object_address(first_output);
    //     let bitcoin_address_opt = types::txout_address(first_output);
    //     let to_address = types::txout_object_address(first_output);
    //     let j = 0;
    //     let objects_len = vector::length(&seal_object_ids);
    //     while(j < objects_len){
    //         let seal_object_id = *vector::borrow(&mut seal_object_ids, j);
    //         let (origin_owner, inscription_obj) = object::take_object_extend<Inscription>(seal_object_id);
    //         let inscription = object::borrow(&inscription_obj);
    //         if(brc20::is_brc20(&inscription.json_body)){
    //             let op = brc20::new_op(origin_owner, to_address, simple_map::clone(&inscription.json_body));
    //             brc20::process_utxo_op(op);
    //             //TODO record the execution result
    //         };
    //         // TODO handle curse inscription
    //         object::transfer_extend(inscription_obj, to_address);
    //         vector::push_back(&mut seal_outs, utxo::new_seal_out((output_index as u32), seal_object_id));
    //         j = j + 1;
    //     };
    //     //Auto create address mapping if not exist
    //     bind_multichain_address(address, bitcoin_address_opt);
    //
    //     seal_outs
    // }

    public fun update_inscription_index(txout: &TxOut, outpoint: OutPoint, old_satpoint: SatPoint, new_satpoint: SatPoint, _tx: &Transaction): SealOut{
        // Track the Inscription via SatPoint
        let vout = types::outpoint_vout(&outpoint);
        let address = types::txout_object_address(txout);
        let bitcoin_address_opt = types::txout_address(txout);
        let to_address = types::txout_object_address(txout);

        let inscription_store = borrow_mut_inscription_store();
        let inscription_id = table::remove(&mut inscription_store.satpoint_to_inscription, old_satpoint);
        let inscription_obj_id = object::custom_object_id<InscriptionID,Inscription>(inscription_id);

        let (origin_owner, inscription_obj) = object::take_object_extend<Inscription>(inscription_obj_id);
        let inscription = object::borrow(&inscription_obj);
        if(brc20::is_brc20(&inscription.json_body)){
            let op = brc20::new_op(origin_owner, to_address, simple_map::clone(&inscription.json_body));
            brc20::process_utxo_op(op);
            //TODO record the execution result
        };
        // TODO handle curse inscription
        object::transfer_extend(inscription_obj, to_address);
        table::add(&mut inscription_store.satpoint_to_inscription, new_satpoint, inscription_id);

        if(table::contains(&inscription_store.outpoint_to_satpoint, outpoint)) {
            let satpoints = table::borrow_mut(&mut inscription_store.outpoint_to_satpoint, outpoint);
            big_vector::push_back(satpoints, new_satpoint);
        } else {
            let satpoints = big_vector::singleton(new_satpoint, OUTPOINT_TO_SATPOINT_BUCKET_SIZE);
            table::add(&mut inscription_store.outpoint_to_satpoint, outpoint, satpoints);
        };
        //Auto create address mapping if not exist
        bind_multichain_address(address, bitcoin_address_opt);

        // seal_out
        utxo::new_seal_out(vout, inscription_obj_id)
    }

    public fun remove_inscription_index(outpoint: OutPoint) {
        let inscription_store = borrow_mut_inscription_store();
        if(table::contains(&inscription_store.outpoint_to_satpoint, outpoint)) {
            let satpoints= table::remove(&mut inscription_store.outpoint_to_satpoint, outpoint);
            big_vector::destroy(satpoints)
        }
    }

        /// Find existing inscriptions on input (transfers of inscriptions)
    public fun inscriptions_on_output(outpoint: &OutPoint) : vector<SatPoint>{
        let inscription_store = borrow_inscription_store();
        if (table::contains(&inscription_store.outpoint_to_satpoint, *outpoint)){
            let outpoint_to_satpoint = table::borrow(&inscription_store.outpoint_to_satpoint, *outpoint);
            let all_satpoint = big_vector::to_vector(outpoint_to_satpoint);
            all_satpoint
        } else {
            vector[]
        }
    }

    public fun process_transaction(tx: &Transaction): vector<SealOut>{
        let output_seals = vector::empty();

        let inscriptions = from_transaction(tx);
        let inscriptions_len = vector::length(&inscriptions);
        if(inscriptions_len == 0){
            vector::destroy_empty(inscriptions);
            return output_seals
        };

        let txid = types::tx_id(tx);
        let tx_outputs = types::tx_output(tx);
        let output_len = vector::length(tx_outputs);

        // ord has three mode for Inscribe:   SameSat,SeparateOutputs,SharedOutput,
        //https://github.com/ordinals/ord/blob/master/src/subcommand/wallet/inscribe/batch.rs#L533
        //TODO handle SameSat
        let is_separate_outputs = output_len > inscriptions_len;
        let idx = 0;
        let next_offset: u64 = 0;
        // reverse inscriptions and pop from the end
        vector::reverse(&mut inscriptions);
        while(idx < inscriptions_len){
            let output_index = if(is_separate_outputs){
                // reset offset
                next_offset = 0;
                idx
            }else{
                0  
            };

            let output = vector::borrow(tx_outputs, output_index);
            // let txout_value = types::txout_value(output);
            let to_address = types::txout_object_address(output);
            let bitcoin_address_opt = types::txout_address(output);

            let inscription = vector::pop_back(&mut inscriptions);
            //Because the previous output of inscription input is a witness program address, so we simply use the output address as the from address.
            let from = to_address;
            let utxo_value = inscription.value;
            let inscription_id = new_inscription_id(inscription.txid, inscription.index);

            // TODO Since the brc20 transfer protocol corresponds to two transactions, `inscribe transfer` and `transfer`,
            // There is no definite receiver when inscribe transfer, so the transfer logic needs to be completed in spend UTXO flow.
            process_inscribe_protocol(from, to_address, &inscription);
            let inscription_obj = create_obj(inscription);
            let object_id = object::id(&inscription_obj);

            object::transfer_extend(inscription_obj, to_address);
            vector::push_back(&mut output_seals, utxo::new_seal_out((output_index as u32), object_id));

            // update inscription index
            let offset = next_offset;
            let satpoint = new_satpoint(txid, (output_index as u32), offset);
            let outpoint= types::new_outpoint(txid, (output_index as u32));
            let inscription_store = borrow_mut_inscription_store();
            // FIXME: if utxo have not yet sync, will cause to duplicate satpoint
            if(!table::contains(&inscription_store.satpoint_to_inscription, satpoint)) {
                table::add(&mut inscription_store.satpoint_to_inscription, satpoint, inscription_id);
            };
            if(table::contains(&inscription_store.outpoint_to_satpoint, outpoint)) {
                let satpoints = table::borrow_mut(&mut inscription_store.outpoint_to_satpoint, outpoint);
                big_vector::push_back(satpoints, satpoint);
            } else {
                let satpoints = big_vector::singleton(satpoint, OUTPOINT_TO_SATPOINT_BUCKET_SIZE);
                table::add(&mut inscription_store.outpoint_to_satpoint, outpoint, satpoints);
            };

            //Auto create address mapping if not exist
            bind_multichain_address(to_address, bitcoin_address_opt);
            next_offset = next_offset + utxo_value;
            idx = idx + 1;
        };
        vector::destroy_empty(inscriptions);
        output_seals
    }

    fun process_inscribe_protocol(from: address, to: address, inscription: &Inscription){
        if (brc20::is_brc20(&inscription.json_body)){
            let op = brc20::new_op(from, to, simple_map::clone(&inscription.json_body));
            brc20::process_inscribe_op(op);
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
                   input_index,
                   record,
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
            offset: _,
            value: _,
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
            body,
            content_encoding,
            content_type,
            duplicate_field: _,
            incomplete_field: _,
            metadata,
            metaprotocol,
            parent,
            pointer,
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
        let next_offset :u64 = 0;
        while(input_idx < len){
            let input = vector::borrow(inputs, input_idx);
            let witness = types::txin_witness(input);
            let previous_output = types::txin_previous_output(input);
            let input_value = if(utxo::exists_utxo(types::outpoint_txid(previous_output), types::outpoint_vout(previous_output))) {
                let previous_utxo = utxo::borrow_utxo(types::outpoint_txid(previous_output), types::outpoint_vout(previous_output));
                utxo::value(object::borrow(previous_utxo))
            } else {
                0
            };

            let inscription_records_from_witness = from_witness(witness);
            let inscription_records_len = vector::length(&inscription_records_from_witness);
            if(inscription_records_len > 1) {
                debug::print(&b"inscription_records_len greater 1");
                debug::print(&tx_id);
            };
            if(!vector::is_empty(&inscription_records_from_witness)){
                vector::for_each(inscription_records_from_witness,|record|{
                    // let _pointer:u64 = if(option::is_some(&record.pointer)) {
                    //     *option::borrow(&record.pointer)
                    // } else {
                    //     0
                    // };
                    // FIXME How to calculate how many sats in certain inscription when there are multi inscription from one input?
                    let offset = next_offset;
                    let value = input_value ;

                    let inscription = record_to_inscription(tx_id, (index_counter as u32), (input_idx as u32), offset, value, record);
                    vector::push_back(&mut inscriptions, inscription);
                    index_counter = index_counter + 1;
                })
            };
            next_offset = next_offset + input_value;
            input_idx = input_idx + 1;
        };
        inscriptions
    }

    public fun from_transaction_bytes(transaction_bytes: vector<u8>): vector<Inscription>{
        let transaction = bcs::from_bytes<Transaction>(transaction_bytes);
        from_transaction(&transaction)
    }

    native fun from_witness(witness: &Witness): vector<InscriptionRecord>;

    public fun pack_inscribe_generate_args(deploy_args: vector<u8>, seed: vector<u8>, user_input: vector<u8>): vector<u8>{
        native_pack_inscribe_generate_args(deploy_args, b"attrs", seed, b"seed",
            user_input, b"user_input")
    }

    native fun native_pack_inscribe_generate_args(
        deploy_args: vector<u8>, deploy_args_key: vector<u8>,
        seed: vector<u8>, seed_key: vector<u8>,
        user_input: vector<u8>, user_input_key: vector<u8>,
    ): vector<u8>;

    public(friend) fun bind_multichain_address(rooch_address: address, bitcoin_address_opt: Option<BitcoinAddress>) {
        //Auto create address mapping if not exist
        if(option::is_some(&bitcoin_address_opt)) {
            let bitcoin_address = option::extract(&mut bitcoin_address_opt);
            let maddress = multichain_address::from_bitcoin(bitcoin_address);
            if (!address_mapping::exists_mapping(maddress)) {
                let bitcoin_move_signer = moveos_std::signer::module_signer<Inscription>();
                address_mapping::bind_by_system(&bitcoin_move_signer, rooch_address, maddress);
            };
        };
    }


}