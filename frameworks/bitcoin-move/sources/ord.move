// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::ord {
    use std::vector;
    use std::option::{Self, Option};
    use std::string;
    use std::string::String;

    use moveos_std::bcs;
    use moveos_std::event;
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::json;
    use moveos_std::table_vec::{Self, TableVec};
    use moveos_std::type_info;
    use moveos_std::bag;
    use moveos_std::string_utils;
    use moveos_std::address;

    use rooch_framework::address_mapping;
    use rooch_framework::multichain_address;
    use rooch_framework::bitcoin_address::BitcoinAddress;
    use bitcoin_move::types::{Self, Witness, Transaction};
    use bitcoin_move::utxo::{Self, UTXO};
    
    friend bitcoin_move::genesis;
    friend bitcoin_move::bitcoin;

    /// How may blocks between halvings.
    const SUBSIDY_HALVING_INTERVAL: u32 = 210_000;

    const FIRST_POST_SUBSIDY_EPOCH: u32 = 33;

    const PERMANENT_AREA: vector<u8> = b"permanent_area";
    const TEMPORARY_AREA: vector<u8> = b"temporary_area";
    
    const METAPROTOCOL_VALIDITY: vector<u8> = b"metaprotocol_validity";

    /// How many satoshis are in "one bitcoin".
    const COIN_VALUE: u64 = 100_000_000;

    struct InscriptionID has store, copy, drop {
        txid: address,
        index: u32,
    }

    struct Flotsam has store, copy, drop {
        // inscription_id: InscriptionID,
        // offset: u64,
        output_index: u32,
        // start: u64,
        offset: u64,
        object_id: ObjectID,
    }

    struct SatPoint has store, copy, drop {
        output_index: u32,
        offset: u64,
        object_id: ObjectID,
    }

    struct Inscription has key {
        txid: address,
        index: u32,
        /// Transaction input index
        input: u32,
        /// inscription offset
        offset: u64,
        /// monotonically increasing
        sequence_number: u32,
        /// The curse inscription is a negative number, combined with the curse inscription flag to express the negative number
        inscription_number: u32,
        /// curse flag
        is_curse: bool,

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

    struct MetaprotocolValidity has store, copy, drop {
        protocol_type: String,
        is_valid: bool,
        invalid_reason: Option<String>,
    }

    struct InscriptionStore has key{
        inscriptions: TableVec<InscriptionID>,
    }

    public(friend) fun genesis_init(_genesis_account: &signer){
        let inscriptions = table_vec::new<InscriptionID>();
        let store = InscriptionStore{
            inscriptions,
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

    public fun derive_inscription_id(inscription_id: InscriptionID) : ObjectID {
        let parent_id = object::named_object_id<InscriptionStore>();
        object::custom_child_object_id<InscriptionID, Inscription>(parent_id, inscription_id)
    }

    // ==== Inscription ==== //
    public fun get_inscription_id_by_index(index: u64) : &InscriptionID {
        let store_obj_id = object::named_object_id<InscriptionStore>();
        let store_obj = object::borrow_mut_object_shared<InscriptionStore>(store_obj_id);
        let store = object::borrow_mut(store_obj);
        table_vec::borrow(& store.inscriptions, index)
    }

    fun record_to_inscription(txid: address, index: u32, input: u32, offset: u64, record: InscriptionRecord): Inscription{
        let parent = option::map(record.parent, |e| derive_inscription_id(e));
        Inscription{
            txid,
            index,
            input,
            offset,
            sequence_number: 0,
            inscription_number: 0,
            is_curse: false,
            body: record.body,
            content_encoding: record.content_encoding,
            content_type: record.content_type,
            metadata: record.metadata,
            metaprotocol: record.metaprotocol,
            parent,
            pointer: record.pointer,
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
        let object = object::add_object_field_with_id(store_obj, id, inscription);
        object
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

    public fun exists_inscription(id: InscriptionID): bool{
        let object_id = derive_inscription_id(id);
        object::exists_object_with_type<Inscription>(object_id)
    }

    public fun borrow_inscription(txid: address, index: u32): &Object<Inscription>{
        let id = InscriptionID{
            txid,
            index,
        };
        let object_id = derive_inscription_id(id);
        object::borrow_object(object_id)
    }

    public fun spend_utxo(utxo_obj: &mut Object<UTXO>, tx: &Transaction, input_utxo_values: vector<u64>, input_index: u64): (vector<SatPoint>, vector<Flotsam>){
        let utxo = object::borrow_mut(utxo_obj);

        let seals = utxo::remove_seals<Inscription>(utxo);
        let new_sat_points = vector::empty();
        let flotsams = vector::empty();
        if(vector::is_empty(&seals)){
            return (new_sat_points, flotsams)
        };
        let outputs = types::tx_output(tx);

        // Track the Inscription via SatPoint
        let j = 0;
        let seals_len = vector::length(&seals);
        while(j < seals_len){
            let seal_object_id = *vector::borrow(&mut seals, j);
            let (origin_owner, inscription_obj) = object::take_object_extend<Inscription>(seal_object_id);
            let inscription = object::borrow_mut(&mut inscription_obj);
            

            let (is_match, new_sat_point) = match_utxo_and_generate_sat_point(inscription.offset, seal_object_id, tx, input_utxo_values, input_index);
            if(is_match){
                let match_output_index = new_sat_point.output_index;

                let match_output = vector::borrow(outputs, (match_output_index as u64));
                let bitcoin_address_opt = types::txout_address(match_output);
                let to_address = types::txout_object_address(match_output);
                inscription.offset = new_sat_point.offset;

                // TODO handle curse inscription
                // drop the temporary area if inscription is transferred.
                drop_temp_area(&mut inscription_obj);
                object::transfer_extend(inscription_obj, to_address);
                vector::push_back(&mut new_sat_points, new_sat_point);
                // Auto create address mapping if not exist
                bind_multichain_address(to_address, bitcoin_address_opt);
            } else {
                let flotsam = new_flotsam(new_sat_point.output_index, new_sat_point.offset, new_sat_point.object_id);
                vector::push_back(&mut flotsams, flotsam);

                drop_temp_area(&mut inscription_obj);
                object::transfer_extend(inscription_obj, origin_owner);
            };
            j = j + 1;
        };

        (new_sat_points, flotsams)
    }

    public fun handle_coinbase_tx(tx: &Transaction, flotsams: vector<Flotsam>, block_height: u64): vector<SatPoint>{
        let new_sat_points = vector::empty();
        if(vector::is_empty(&flotsams)){
            return new_sat_points
        };
        let outputs = types::tx_output(tx);

        // Track the Inscription via SatPoint
        let j = 0;
        let flotsams_len = vector::length(&flotsams);
        while(j < flotsams_len){
            let flotsam = *vector::borrow(&mut flotsams, j);
            let (_origin_owner, inscription_obj) = object::take_object_extend<Inscription>(flotsam.object_id);
            let inscription = object::borrow_mut(&mut inscription_obj);

            let new_sat_point = match_coinbase_and_generate_sat_point(j, tx, flotsams, block_height);
            let match_output_index = new_sat_point.output_index;

            let match_output = vector::borrow(outputs, (match_output_index as u64));
            let bitcoin_address_opt = types::txout_address(match_output);
            let to_address = types::txout_object_address(match_output);

            inscription.offset = new_sat_point.offset;

            // TODO handle curse inscription
            object::transfer_extend(inscription_obj, to_address);
            vector::push_back(&mut new_sat_points, new_sat_point);
            // Auto create address mapping if not exist
            bind_multichain_address(to_address, bitcoin_address_opt);
            j = j + 1;
        };

        new_sat_points
    }

    /// Match UTXO, via SatPoint offset, generate new SatPoint, UTXO spent follows "First in First out"
    fun match_utxo_and_generate_sat_point(offset: u64, seal_object_id: ObjectID, tx: &Transaction, input_utxo_values: vector<u64>, input_index: u64): (bool, SatPoint){
        let txoutput = types::tx_output(tx);
        
        let idx = 0;
        let input_utxo_value_accumulator = 0;
        while(idx < input_index){
            let utxo_value = *vector::borrow(&input_utxo_values, idx);
            input_utxo_value_accumulator = input_utxo_value_accumulator + utxo_value;

            idx = idx + 1;
        };
        input_utxo_value_accumulator = input_utxo_value_accumulator + offset;

        let idx = 0;
        let output_len = vector::length(txoutput);
        let output_utxo_value_accumulator = 0;
        let new_output_index = 0;
        let new_offset = 0;
        while(idx < output_len){
            let txout = vector::borrow(txoutput, idx);
            let output_value = types::txout_value(txout);
            output_utxo_value_accumulator = output_utxo_value_accumulator + output_value;

            if(output_utxo_value_accumulator > input_utxo_value_accumulator) {
                new_output_index = idx;
                new_offset = output_value - (output_utxo_value_accumulator - input_utxo_value_accumulator);

                break
            };

            idx = idx + 1;
        };

        // match utxo output
        if(idx < output_len){
            let new_sat_point = new_sat_point((new_output_index as u32), new_offset, seal_object_id);
            (true, new_sat_point)
        }else {
            // Paid to miners as transaction fees
            let new_offset = input_utxo_value_accumulator - output_utxo_value_accumulator;
            let new_sat_point = new_sat_point((input_index as u32), new_offset, seal_object_id);
            (false, new_sat_point)
        }
    }

    /// Match Coinbase, via SatPoint offset, generate new SatPoint
    fun match_coinbase_and_generate_sat_point(flotsam_index: u64, tx: &Transaction, flotsams: vector<Flotsam>, block_height: u64): SatPoint{
        let txoutput = types::tx_output(tx);

        let idx = 0;
        let reward_value_accumulator = 0;
        let subsidy = subsidy_by_height(block_height);
        reward_value_accumulator = reward_value_accumulator + subsidy;
        while(idx <= flotsam_index){
            let flotsam = *vector::borrow(&flotsams, idx);
            reward_value_accumulator = reward_value_accumulator + flotsam.offset;

            idx = idx + 1;
        };
        // input_utxo_value_accumulator = input_utxo_value_accumulator + offset;

        let idx = 0;
        let output_len = vector::length(txoutput);
        let output_utxo_value_accumulator = 0;
        let new_output_index = 0;
        let new_offset = 0;
        while(idx < output_len){
            let txout = vector::borrow(txoutput, idx);
            let output_value = types::txout_value(txout);
            output_utxo_value_accumulator = output_utxo_value_accumulator + output_value;

            if(output_utxo_value_accumulator > reward_value_accumulator) {
                new_output_index = idx;
                new_offset = output_value - (output_utxo_value_accumulator - reward_value_accumulator);

                break
            };

            idx = idx + 1;
        };

        let flatsam = vector::borrow(&flotsams, flotsam_index);
        let new_sat_point = new_sat_point((new_output_index as u32), new_offset, flatsam.object_id);
        new_sat_point
    }

    public fun process_transaction(tx: &Transaction, input_utxo_values: vector<u64>): vector<SatPoint>{
        let sat_points = vector::empty();

        let inscriptions = from_transaction(tx, option::some(input_utxo_values));
        let inscriptions_len = vector::length(&inscriptions);
        if(inscriptions_len == 0){
            vector::destroy_empty(inscriptions);
            return sat_points
        };

        let tx_outputs = types::tx_output(tx);
        let output_len = vector::length(tx_outputs);

        // Ord has three mode for inscribe: SameSat,SeparateOutputs,SharedOutput:
        // SameSat and SharedOutput have only one output
        // When SeparateOutputs is used, the number of output and inscription is consistent.
        // https://github.com/ordinals/ord/blob/26fcf05a738e68ef8c9c18fcc0997ccf931d6f41/src/wallet/batch/plan.rs#L270-L307
        let is_separate_outputs = output_len == inscriptions_len;
        let idx = 0;
        // reverse inscriptions and pop from the end
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
            let offset = if(is_separate_outputs){
                0
            }else{
                inscription.offset
            };

            let inscription_obj = create_obj(inscription);
            let object_id = object::id(&inscription_obj);
            object::transfer_extend(inscription_obj, to_address);

            let new_sat_point = new_sat_point((output_index as u32), offset, object_id);
            vector::push_back(&mut sat_points, new_sat_point);

            //Auto create address mapping if not exist
            bind_multichain_address(to_address, bitcoin_address_opt);

            idx = idx + 1;
        };
        vector::destroy_empty(inscriptions);
        sat_points
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

    public fun input(self: &Inscription): u32{
        self.input
    }

    public fun offset(self: &Inscription): u64{
        self.offset
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
            sequence_number: _,
            inscription_number: _,
            is_curse: _,
            body: _,
            content_encoding: _,
            content_type: _,
            metadata: _,
            metaprotocol: _,
            parent: _,
            pointer: _,
        } = self;
    }

    public fun inscription_id_txid(self: &InscriptionID): address {
        self.txid
    }

    public fun inscription_id_index(self: &InscriptionID): u32 {
        self.index
    }

    // ===== SatPoint ========== //

    // === SatPoint ===
    public fun new_sat_point(output_index: u32, offset: u64, object_id: ObjectID) : SatPoint {
        SatPoint{
            output_index,
            offset,
            object_id
        }
    }

    public fun unpack_sat_point(sat_point: SatPoint) : (u32, u64, ObjectID) {
        let SatPoint{output_index, offset, object_id} = sat_point;
        (output_index, offset, object_id)
    }

    /// Get the SatPoint's object_id
    public fun sat_point_object_id(sat_point: &SatPoint): ObjectID {
        sat_point.object_id
    }

    /// Get the SatPoint's offset
    public fun sat_point_offset(sat_point: &SatPoint): u64 {
        sat_point.offset
    }

    /// Get the SatPoint's output_index
    public fun sat_point_output_index(sat_point: &SatPoint): u32 {
        sat_point.output_index
    }


    // === Flotsam ===
    public fun new_flotsam(output_index: u32, offset: u64, object_id: ObjectID) : Flotsam {
        Flotsam{
            output_index,
            offset,
            object_id
        }
    }

    public fun unpack_flotsam(flotsam: Flotsam) : (u32, u64, ObjectID) {
        let Flotsam{output_index, offset, object_id} = flotsam;
        (output_index, offset, object_id)
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

    public fun from_transaction(tx: &Transaction, input_utxo_values: Option<vector<u64>>): vector<Inscription>{
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
            let input_value = if(option::is_some(&input_utxo_values)){
                *vector::borrow(option::borrow(&input_utxo_values), input_idx)
            } else {
                0
            };

            let inscription_records_from_witness = from_witness(witness);
            let inscription_records_len = vector::length(&inscription_records_from_witness);
            let j = 0;

            while(j < inscription_records_len){
                let record = vector::borrow(&inscription_records_from_witness, j);
                let pointer = *option::borrow_with_default(&record.pointer, &0u64);
                if(pointer >= input_value) {
                    pointer = 0;
                };

                let offset = next_offset + pointer;
                let inscription = record_to_inscription(tx_id, (index_counter as u32), (input_idx as u32), offset, *record);
                vector::push_back(&mut inscriptions, inscription);
                index_counter = index_counter + 1;
                j = j + 1;
            };
            next_offset = next_offset + input_value;
            input_idx = input_idx + 1;
        };
        inscriptions
    }

    public fun from_transaction_bytes(transaction_bytes: vector<u8>): vector<Inscription>{
        let transaction = bcs::from_bytes<Transaction>(transaction_bytes);
        from_transaction(&transaction, option::none())
    }

    native fun from_witness(witness: &Witness): vector<InscriptionRecord>;

    /// Block Rewards
    public fun subsidy_by_height(height: u64): u64 {
        let epoch = (height as u32)/ SUBSIDY_HALVING_INTERVAL;
        if(epoch < FIRST_POST_SUBSIDY_EPOCH) {
            (50 * COIN_VALUE) >> (epoch as u8)
        } else {
            0
        }
    }


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

    // ===== permenent area ========== //
    #[private_generics(S)]
    public fun add_permanent_state<S: store>(inscription: &mut Object<Inscription>, state: S){
        if(object::contains_field(inscription, PERMANENT_AREA)){
            let bag = object::borrow_mut_field(inscription, PERMANENT_AREA);
            let name = type_info::type_name<S>();
            bag::add(bag, name, state);
        }else{
            let bag = bag::new();
            let name = type_info::type_name<S>();
            bag::add(&mut bag, name, state);
            object::add_field(inscription, PERMANENT_AREA, bag);
        }
    }

    public fun contains_permanent_state<S: store>(inscription: &Object<Inscription>) : bool {
        if(object::contains_field(inscription, PERMANENT_AREA)){
            let bag = object::borrow_field(inscription, PERMANENT_AREA);
            let name = type_info::type_name<S>();
            bag::contains(bag, name)
        }else{
            false
        }
    }

    public fun borrow_permanent_state<S: store>(inscription: &Object<Inscription>) : &S {
        let bag = object::borrow_field(inscription, PERMANENT_AREA);
        let name = type_info::type_name<S>();
        bag::borrow(bag, name)
    }

    #[private_generics(S)]
    public fun borrow_mut_permanent_state<S: store>(inscription: &mut Object<Inscription>) : &mut S {
        let bag = object::borrow_mut_field(inscription, PERMANENT_AREA);
        let name = type_info::type_name<S>();
        bag::borrow_mut(bag, name)
    }

    #[private_generics(S)]
    public fun remove_permanent_state<S: store>(inscription: &mut Object<Inscription>) : S {
        let bag = object::borrow_mut_field(inscription, PERMANENT_AREA);
        let name = type_info::type_name<S>();
        bag::remove(bag, name)
    }

    // TODO: remove #[test_only]?
    #[test_only]
    /// Destroy permanent area if it's empty. Aborts if it's not empty.
    public fun destroy_permanent_area(inscription: &mut Object<Inscription>){
        if (object::contains_field(inscription, PERMANENT_AREA)) {
            let bag = object::remove_field(inscription, PERMANENT_AREA);
            bag::destroy_empty(bag);
        }
    }


    // ==== Temporary Area ===

    #[private_generics(S)]
    public fun add_temp_state<S: store + drop>(inscription: &mut Object<Inscription>, state: S){
        if(object::contains_field(inscription, TEMPORARY_AREA)){
            let bag = object::borrow_mut_field(inscription, TEMPORARY_AREA);
            let name = type_info::type_name<S>();
            bag::add_dropable(bag, name, state);
        }else{
            let bag = bag::new_dropable();
            let name = type_info::type_name<S>();
            bag::add_dropable(&mut bag, name, state);
            object::add_field(inscription, TEMPORARY_AREA, bag);
        }
    }

    public fun contains_temp_state<S: store + drop>(inscription: &Object<Inscription>) : bool {
        if(object::contains_field(inscription, TEMPORARY_AREA)){
            let bag = object::borrow_field(inscription, TEMPORARY_AREA);
            let name = type_info::type_name<S>();
            bag::contains(bag, name)
        }else{
            false
        }
    }

    public fun borrow_temp_state<S: store + drop>(inscription: &Object<Inscription>) : &S {
        let bag = object::borrow_field(inscription, TEMPORARY_AREA);
        let name = type_info::type_name<S>();
        bag::borrow(bag, name)
    }

    #[private_generics(S)]
    public fun borrow_mut_temp_state<S: store + drop>(inscription: &mut Object<Inscription>) : &mut S {
        let bag = object::borrow_mut_field(inscription, TEMPORARY_AREA);
        let name = type_info::type_name<S>();
        bag::borrow_mut(bag, name)
    }

    #[private_generics(S)]
    public fun remove_temp_state<S: store + drop>(inscription: &mut Object<Inscription>) : S {
        let bag = object::borrow_mut_field(inscription, TEMPORARY_AREA);
        let name = type_info::type_name<S>();
        bag::remove(bag, name)
    }

    /// Drop the bag, whether it's empty or not
    public(friend) fun drop_temp_area(inscription: &mut Object<Inscription>){
        if (object::contains_field(inscription, TEMPORARY_AREA)) {
            let bag = object::remove_field(inscription, TEMPORARY_AREA);
            bag::drop(bag);
        }
    }

    // ==== Inscription Metaprotocol Validity ==== //

    #[private_generics(T)]
    /// Seal the metaprotocol validity for the given inscription_id.
    public fun seal_metaprotocol_validity<T>(inscription_id: InscriptionID, is_valid: bool, invalid_reason: Option<String>) {
        let store_obj_id = object::named_object_id<InscriptionStore>();
        let store_obj = object::borrow_mut_object_shared<InscriptionStore>(store_obj_id);

        let inscription_object_id = derive_inscription_id(inscription_id);
        let inscription_obj = object::borrow_mut_object_field<InscriptionStore, Inscription>(store_obj, inscription_object_id);

        let protocol_type = type_info::type_name<T>();
        let validity = MetaprotocolValidity {
            protocol_type,
            is_valid,
            invalid_reason,
        };

        object::upsert_field(inscription_obj, METAPROTOCOL_VALIDITY, validity);
    }

    /// Returns true if Inscription `object` contains metaprotocol validity
    public fun exists_metaprotocol_validity(inscription_id: InscriptionID): bool{
        let store_obj_id = object::named_object_id<InscriptionStore>();
        let store_obj = object::borrow_mut_object_shared<InscriptionStore>(store_obj_id);

        let inscription_object_id = derive_inscription_id(inscription_id);
        let exists = object::contains_object_field<InscriptionStore, Inscription>(store_obj, inscription_object_id);
        if (!exists) {
            return false
        };

        let inscription_obj = object::borrow_mut_object_field<InscriptionStore, Inscription>(store_obj, inscription_object_id);
        object::contains_field(inscription_obj, METAPROTOCOL_VALIDITY)
    }

    /// Borrow the metaprotocol validity for the given inscription_id.
    public fun borrow_metaprotocol_validity(inscription_id: InscriptionID): &MetaprotocolValidity {
        let store_obj_id = object::named_object_id<InscriptionStore>();
        let store_obj = object::borrow_mut_object_shared<InscriptionStore>(store_obj_id);

        let inscription_object_id = derive_inscription_id(inscription_id);
        let inscription_obj = object::borrow_mut_object_field<InscriptionStore, Inscription>(store_obj, inscription_object_id);

        object::borrow_field(inscription_obj, METAPROTOCOL_VALIDITY)
    }

    /// Check the MetaprotocolValidity's protocol_type whether match
    public fun metaprotocol_validity_protocol_match<T>(validity: &MetaprotocolValidity): bool {
        let protocol_type = type_info::type_name<T>();
        protocol_type == validity.protocol_type
    }

    /// Get the MetaprotocolValidity's protocol_type
    public fun metaprotocol_validity_protocol_type(validity: &MetaprotocolValidity): String {
        validity.protocol_type
    }

    /// Get the MetaprotocolValidity's is_valid
    public fun metaprotocol_validity_is_valid(validity: &MetaprotocolValidity): bool {
        validity.is_valid
    }

    /// Get the MetaprotocolValidity's invalid_reason
    public fun metaprotocol_validity_invalid_reason(validity: &MetaprotocolValidity): Option<String> {
        validity.invalid_reason
    }

    #[test_only]
    public fun init_for_test(_genesis_account: &signer){
        genesis_init(_genesis_account);
    }

    #[test_only]
    public fun drop_temp_area_for_test(inscription: &mut Object<Inscription>) {
        drop_temp_area(inscription);
    }

    #[test_only]
    public fun new_inscription_object_for_test(
        txid: address,
        index: u32,
        input: u32,
        offset: u64,
        body: vector<u8>,
        content_encoding: Option<String>,
        content_type: Option<String>,
        metadata: vector<u8>,
        metaprotocol: Option<String>,
        parent: Option<ObjectID>,
        pointer: Option<u64>,
    ): Object<Inscription> {
        let inscription = Inscription {
            txid,
            index,
            input,
            offset,
            sequence_number: 0,
            inscription_number: 0,
            is_curse: false,
            body,
            content_encoding,
            content_type,
            metadata,
            metaprotocol,
            parent,
            pointer,
        };

        object::new(inscription)
    }

    #[test_only]
    public fun drop_inscription_object_for_test(inscription: Object<Inscription>) {
        let inscription = object::remove(inscription);
        let Inscription { 
            txid: _, 
            index: _,
            input: _,
            offset: _,
            sequence_number: _,
            inscription_number: _,
            is_curse: _,
            body: _,
            content_encoding: _,
            content_type: _,
            metadata: _,
            metaprotocol: _,
            parent: _,
            pointer: _,
        } = inscription;
    }

    #[test_only]
    struct PermanentState has store {
        value: u64,
    }

    #[test]
    fun test_permanent_state(){
        // genesis_init();
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let inscription_obj = new_inscription_object_for_test(
            txid,
            0,
            0,
            0,
            vector[],
            option::none(),
            option::none(),
            vector[],
            option::none(),
            option::none(),
            option::none(),
        );
        add_permanent_state(&mut inscription_obj, PermanentState{value: 10});
        assert!(contains_permanent_state<PermanentState>(&inscription_obj), 1);
        assert!(borrow_permanent_state<PermanentState>(&inscription_obj).value == 10, 2);
        {
            let state = borrow_mut_permanent_state<PermanentState>(&mut inscription_obj);
            state.value = 20;
        };
        let state = remove_permanent_state<PermanentState>(&mut inscription_obj);
        assert!(state.value == 20, 1);
        assert!(!contains_permanent_state<PermanentState>(&inscription_obj), 3);

        let PermanentState { value: _ } = state;
        destroy_permanent_area(&mut inscription_obj);
        drop_inscription_object_for_test(inscription_obj);
    }

    #[test_only]
    struct TempState has store, copy, drop {
        value: u64,
    }

    #[test]
    fun test_temp_state(){
        // genesis_init();
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let inscription_obj = new_inscription_object_for_test(
            txid,
            0,
            0,
            0,
            vector[],
            option::none(),
            option::none(),
            vector[],
            option::none(),
            option::none(),
            option::none(),
        );
        add_temp_state(&mut inscription_obj, TempState{value: 10});
        assert!(contains_temp_state<TempState>(&inscription_obj), 1);
        assert!(borrow_temp_state<TempState>(&inscription_obj).value == 10, 2);
        {
            let state = borrow_mut_temp_state<TempState>(&mut inscription_obj);
            state.value = 20;
        };
        let state = remove_temp_state<TempState>(&mut inscription_obj);
        assert!(state.value == 20, 1);
        assert!(!contains_temp_state<TempState>(&inscription_obj), 3);

        drop_temp_area(&mut inscription_obj);
        drop_inscription_object_for_test(inscription_obj);
    }

    #[test_only]
    fun mock_inscription_transferring_along_utxo(inscription_obj: Object<Inscription>, to: address) {
        drop_temp_area(&mut inscription_obj);
        object::transfer_extend(inscription_obj, to);
    }

    // If the inscription is transferred, the permanent area will be kept and the temporary area will be dropped.
    #[test]
    fun test_transfer() {
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let inscription_obj = new_inscription_object_for_test(
            txid,
            0,
            0,
            0,
            vector[],
            option::none(),
            option::none(),
            vector[],
            option::none(),
            option::none(),
            option::none(),
        );

        add_temp_state(&mut inscription_obj, TempState{value: 10});
        add_permanent_state(&mut inscription_obj, PermanentState{value: 10});
        let object_id = object::id(&inscription_obj);

        let to_address = @0x42;
        {
            mock_inscription_transferring_along_utxo(inscription_obj, to_address);
        };

        let inscription_obj = object::borrow_object<Inscription>(object_id);
        assert!(!contains_temp_state<TempState>(inscription_obj), 1);
        assert!(contains_permanent_state<PermanentState>(inscription_obj), 2);
    }
 

    #[test_only]
    struct TestProtocol has key {}

    #[test_only]
    public fun new_inscription_id_for_test(        
        txid: address,
        index: u32,
    ) : InscriptionID {
        InscriptionID {
            txid,
            index,
        }
    }

    #[test_only]
    public fun new_inscription_for_test(
        txid: address,
        index: u32,
        input: u32,
        offset: u64,
        body: vector<u8>,
        content_encoding: Option<String>,
        content_type: Option<String>,
        metadata: vector<u8>,
        metaprotocol: Option<String>,
        parent: Option<ObjectID>,
        pointer: Option<u64>,
    ): Inscription {
        Inscription {
            txid,
            index,
            input,
            offset,
            sequence_number: 0,
            inscription_number: 0,
            is_curse: false,
            body,
            content_encoding,
            content_type,
            metadata,
            metaprotocol,
            parent,
            pointer,
        }
    }

    #[test_only]
    public fun setup_inscription_for_test(genesis_account: &signer) : (address, InscriptionID) {
        genesis_init(genesis_account);

        // prepare test inscription
        let test_address = @0x5416690eaaf671031dc609ff8d36766d2eb91ca44f04c85c27628db330f40fd1;
        let test_txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let test_inscription_id = new_inscription_id_for_test(test_txid, 0);

        let test_inscription = new_inscription_for_test(
            test_txid,
            0,
            0,
            0,
            vector[],
            option::none(),
            option::none(),
            vector[],
            option::none(),
            option::none(),
            option::none(),
        );

        let test_inscription_obj = create_obj(test_inscription);
        object::transfer_extend(test_inscription_obj, test_address);

        (test_address, test_inscription_id)
    }

    #[test(genesis_account=@0x4)]
    fun test_metaprotocol_validity(genesis_account: &signer){
        // prepare test inscription
        let (_test_address, test_inscription_id) = setup_inscription_for_test(genesis_account);

        // Check whether exists metaprotocol_validity
        let is_exists = exists_metaprotocol_validity(test_inscription_id);
        assert!(!is_exists, 1);

        // seal TestProtocol valid to test_inscription_id
        seal_metaprotocol_validity<TestProtocol>(test_inscription_id, true, option::none());

        // Check whether exists metaprotocol_validity
        let is_exists = exists_metaprotocol_validity(test_inscription_id);
        assert!(is_exists, 1);

        // borrow metaprotocol validity from test_inscription_id
        let metaprotocol_validity = borrow_metaprotocol_validity(test_inscription_id);
        let is_valid = metaprotocol_validity_is_valid(metaprotocol_validity);
        assert!(is_valid, 2);

        // seal TestProtocol not valid to test_inscription_id
        let test_invalid_reason = string::utf8(b"Claimed first by another");
        seal_metaprotocol_validity<TestProtocol>(test_inscription_id, false, option::some(test_invalid_reason));

        // borrow metaprotocol validity from test_inscription_id
        let metaprotocol_validity = borrow_metaprotocol_validity(test_inscription_id);

        let is_valid = metaprotocol_validity_is_valid(metaprotocol_validity);
        assert!(!is_valid, 31);

        let invalid_reason_option = metaprotocol_validity_invalid_reason(metaprotocol_validity);
        let invalid_reason = option::borrow(&invalid_reason_option);
        assert!(invalid_reason == &test_invalid_reason, 4);
    }

    // ==== Prase InscriptionID ==== //
    public fun parse_inscription_id(inscription_id: &String) : Option<InscriptionID> {
        let offset = string::index_of(inscription_id, &std::string::utf8(b"i"));
        if (offset == string::length(inscription_id)) {
            return option::none()
        };

        let txid_str = string::sub_string(inscription_id, 0, offset);
        let ascii_txid_option = std::ascii::try_string(string::into_bytes(txid_str));
        if (option::is_none(&ascii_txid_option)) {
            return option::none()
        };

        let txid_option = address::from_ascii_string(option::extract(&mut ascii_txid_option));
        if (option::is_none(&txid_option)) {
            return option::none()
        };

        let index_str = string::sub_string(inscription_id, offset+1, string::length(inscription_id));
        let index_option = string_utils::parse_u64_option(&index_str);
        if (option::is_none(&index_option)) {
            return option::none()
        };

        option::some(InscriptionID{
            txid: option::extract<address>(&mut txid_option),
            index: (option::extract<u64>(&mut index_option) as u32),
        })
    }

    #[test]
    fun test_parse_inscription_id_ok(){
        let inscription_id_str = std::string::utf8(b"6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0");
        let inscription_id_option = parse_inscription_id(&inscription_id_str);
        assert!(option::is_some(&inscription_id_option), 1);
    }

    #[test]
    fun test_parse_inscription_id_fail_with_invalid_txid_str(){
        let inscription_id_str = std::string::utf8(x"E4BDA0E5A5BD6930");
        let inscription_id_option = parse_inscription_id(&inscription_id_str);
        assert!(option::is_none(&inscription_id_option), 1);
    }

    #[test]
    fun test_parse_inscription_id_fail_with_invalid_txid_address(){
        let inscription_id_str = std::string::utf8(b"6x55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0");
        let inscription_id_option = parse_inscription_id(&inscription_id_str);
        assert!(option::is_none(&inscription_id_option), 1);
    }

    #[test]
    fun test_parse_inscription_id_fail_with_without_i(){
        let inscription_id_str = std::string::utf8(b"6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8");
        let inscription_id_option = parse_inscription_id(&inscription_id_str);
        assert!(option::is_none(&inscription_id_option), 1);
    }

    #[test]
    fun test_parse_inscription_id_fail_with_invalid_index(){
        let inscription_id_str = std::string::utf8(b"6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8ix");
        let inscription_id_option = parse_inscription_id(&inscription_id_str);
        assert!(option::is_none(&inscription_id_option), 1);
    }
}
