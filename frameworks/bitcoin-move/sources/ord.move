// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::ord {
    use std::option::{Self, Option};
    use std::string;
    use std::string::String;
    use std::vector;

    use moveos_std::bag;
    use moveos_std::json;
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::string_utils;
    use moveos_std::type_info;

    use bitcoin_move::bitcoin_hash;
    use bitcoin_move::types::{Self, Transaction, Witness, OutPoint};

    friend bitcoin_move::genesis;
    friend bitcoin_move::bitcoin;
    friend bitcoin_move::inscription_updater;


    const PERMANENT_AREA: vector<u8> = b"permanent_area";
    const TEMPORARY_AREA: vector<u8> = b"temporary_area";

    const METAPROTOCOL_VALIDITY: vector<u8> = b"metaprotocol_validity";


    const ErrorMetaprotocolAlreadyRegistered: u64 = 1;
    const ErrorMetaprotocolProtocolMismatch: u64 = 2;
    const ErrorInscriptionNotExists: u64 = 3;

    struct InscriptionID has store, copy, drop {
        txid: address,
        index: u32,
    }

    struct SatPoint has store, copy, drop {
        outpoint: OutPoint,
        offset: u64,
    }

    struct Inscription has key {
        id: InscriptionID,
        /// The location of the inscription
        location: SatPoint,
        /// monotonically increasing
        sequence_number: u32,
        /// The curse inscription is a negative number, combined with the curse inscription flag to express the negative number
        inscription_number: u32,
        /// Is the inscription cursed
        is_cursed: bool,
        /// inscription charms flag
        charms: u16,

        body: vector<u8>,
        content_encoding: Option<String>,
        content_type: Option<String>,
        metadata: vector<u8>,
        metaprotocol: Option<String>,
        parents: vector<InscriptionID>,
        pointer: Option<u64>,
        // Reserved for extending the Rune protocol
        rune: Option<u128>,
    }

    struct Envelope<T> has store, copy, drop {
        input: u32,
        offset: u32,
        pushnum: bool,
        stutter: bool,
        payload: T,
    }

    struct InscriptionRecord has store, copy, drop {
        body: vector<u8>,
        content_encoding: Option<String>,
        content_type: Option<String>,
        duplicate_field: bool,
        incomplete_field: bool,
        metadata: vector<u8>,
        metaprotocol: Option<String>,
        parents: vector<InscriptionID>,
        pointer: Option<u64>,
        unrecognized_even_field: bool,
        rune: Option<u128>,
    }

    struct InvalidInscriptionEvent has store, copy, drop {
        txid: address,
        input_index: u64,
        record: InscriptionRecord,
    }

    struct MetaprotocolRegistry has key {}

    struct MetaprotocolValidity has store, copy, drop {
        protocol_type: String,
        is_valid: bool,
        invalid_reason: Option<String>,
    }

    struct InscriptionStore has key {
        cursed_inscription_count: u32,
        blessed_inscription_count: u32,
        unbound_inscription_count: u32,
        lost_sats: u64,
        next_sequence_number: u32,
    }

    const InscriptionEventTypeNew: u8 = 0;
    const InscriptionEventTypeBurn: u8 = 1;

    struct InscriptionEvent has store, copy, drop {
        metaprotocol: String,
        sequence_number: u32,
        inscription_obj_id: ObjectID,
        event_type: u8,
    }

    public(friend) fun genesis_init() {
        let store = InscriptionStore {
            cursed_inscription_count: 0,
            blessed_inscription_count: 0,
            unbound_inscription_count: 0,
            lost_sats: 0,
            next_sequence_number: 0,
        };
        let store_obj = object::new_named_object(store);
        object::to_shared(store_obj);
    }

    public(friend) fun borrow_mut_inscription_store(): &mut InscriptionStore {
        let inscription_store_object_id = object::named_object_id<InscriptionStore>();
        let inscription_store_obj = object::borrow_mut_object_shared<InscriptionStore>(inscription_store_object_id);
        object::borrow_mut(inscription_store_obj)
    }


    public(friend) fun borrow_inscription_store(): &InscriptionStore {
        let inscription_store_object_id = object::named_object_id<InscriptionStore>();
        let inscription_store_obj = object::borrow_object<InscriptionStore>(inscription_store_object_id);
        object::borrow(inscription_store_obj)
    }

    public(friend) fun blessed_inscription_count(inscription_store: &InscriptionStore): u32 {
        inscription_store.blessed_inscription_count
    }

    public(friend) fun cursed_inscription_count(inscription_store: &InscriptionStore): u32 {
        inscription_store.cursed_inscription_count
    }

    public(friend) fun unbound_inscription_count(inscription_store: &InscriptionStore): u32 {
        inscription_store.unbound_inscription_count
    }

    public(friend) fun lost_sats(inscription_store: &InscriptionStore): u64 {
        inscription_store.lost_sats
    }

    public(friend) fun next_sequence_number(inscription_store: &InscriptionStore): u32 {
        inscription_store.next_sequence_number
    }

    public(friend) fun update_cursed_inscription_count(inscription_store: &mut InscriptionStore, count: u32) {
        inscription_store.cursed_inscription_count = count;
    }

    public(friend) fun update_blessed_inscription_count(inscription_store: &mut InscriptionStore, count: u32) {
        inscription_store.blessed_inscription_count = count;
    }

    public(friend) fun update_next_sequence_number(inscription_store: &mut InscriptionStore, count: u32) {
        inscription_store.next_sequence_number = count;
    }

    public(friend) fun update_unbound_inscription_count(inscription_store: &mut InscriptionStore, count: u32) {
        inscription_store.unbound_inscription_count = count;
    }

    public(friend) fun update_lost_sats(inscription_store: &mut InscriptionStore, count: u64) {
        inscription_store.lost_sats = count;
    }

    //===== InscriptionID =====//

    public fun new_inscription_id(txid: address, index: u32): InscriptionID {
        InscriptionID {
            txid,
            index,
        }
    }

    public fun derive_inscription_id(inscription_id: InscriptionID): ObjectID {
        let parent_id = object::named_object_id<InscriptionStore>();
        object::custom_object_id_with_parent<InscriptionID, Inscription>(parent_id, inscription_id)
    }

    /// Prase InscriptionID from String
    public fun parse_inscription_id(inscription_id: &String): Option<InscriptionID> {
        let offset = string::index_of(inscription_id, &std::string::utf8(b"i"));
        if (offset == string::length(inscription_id)) {
            return option::none()
        };

        let txid_str = string::sub_string(inscription_id, 0, offset);

        // Bitcoin tx id hex string is reversed
        let txid_option = bitcoin_hash::from_ascii_bytes_option(string::bytes(&txid_str));
        if (option::is_none(&txid_option)) {
            return option::none()
        };

        let index_str = string::sub_string(inscription_id, offset + 1, string::length(inscription_id));
        let index_option = string_utils::parse_u64_option(&index_str);
        if (option::is_none(&index_option)) {
            return option::none()
        };

        option::some(InscriptionID {
            txid: option::extract<address>(&mut txid_option),
            index: (option::extract<u64>(&mut index_option) as u32),
        })
    }

    public fun inscription_id_to_string(inscription_id: &InscriptionID): String {
        let txid_str = bitcoin_hash::to_string(inscription_id.txid);
        let index_str = string_utils::to_string_u32(inscription_id.index);
        string::append(&mut txid_str, std::string::utf8(b"i"));
        string::append(&mut txid_str, index_str);
        txid_str
    }

    // ==== Inscription ==== //
    public fun get_inscription_id_by_sequence_number(sequence_number: u32): &InscriptionID {
        let store_obj_id = object::named_object_id<InscriptionStore>();
        let store_obj = object::borrow_object<InscriptionStore>(store_obj_id);
        object::borrow_field(store_obj, sequence_number)
    }

    public fun get_inscription_next_sequence_number(): u32 {
        let store_obj_id = object::named_object_id<InscriptionStore>();
        let store_obj = object::borrow_object<InscriptionStore>(store_obj_id);
        let store = object::borrow(store_obj);
        store.next_sequence_number
    }

    public(friend) fun create_object(
        id: InscriptionID,
        location: SatPoint,
        sequence_number: u32,
        inscription_number: u32,
        is_cursed: bool,
        charms: u16,
        envelope: Envelope<InscriptionRecord>,
        owner: address
    ): ObjectID {
        
        let metaprotocol = envelope.payload.metaprotocol;
        let inscription = Inscription {
            id,
            location,
            sequence_number: sequence_number,
            inscription_number: inscription_number,
            is_cursed: is_cursed,
            charms: charms,
            body: envelope.payload.body,
            content_encoding: envelope.payload.content_encoding,
            content_type: envelope.payload.content_type,
            metadata: envelope.payload.metadata,
            metaprotocol,
            parents: envelope.payload.parents,
            pointer: envelope.payload.pointer,
            rune: envelope.payload.rune,
        };
        
        let obj = create_object_internal(inscription);
        let inscription_obj_id = object::id(&obj);
        
        if (option::is_some(&metaprotocol)) {
            let metaprotocol = option::destroy_some(metaprotocol);
            moveos_std::event_queue::emit(metaprotocol, InscriptionEvent {
                metaprotocol: metaprotocol,
                sequence_number: sequence_number,
                inscription_obj_id,
                event_type: InscriptionEventTypeNew,
            });
        };
        object::transfer_extend(obj, owner);
        inscription_obj_id
    }

    fun create_object_internal(inscription: Inscription): Object<Inscription>{
        let id = inscription.id;
        let store_obj_id = object::named_object_id<InscriptionStore>();
        let store_obj = object::borrow_mut_object_shared<InscriptionStore>(store_obj_id);
        // record a sequence_number to InscriptionID mapping
        object::add_field(store_obj, inscription.sequence_number, id);
        let obj = object::new_with_parent_and_id(store_obj, id, inscription);
        obj
    }

    public(friend) fun transfer_object(inscription_obj: Object<Inscription>, to: address, new_location: SatPoint, is_op_return: bool){
        //drop the temp area when inscription is transferred
        drop_temp_area(&mut inscription_obj);
        let inscription = object::borrow_mut(&mut inscription_obj);
        inscription.location = new_location;
        if (is_op_return){
            //if the output is OP_RETURN, set the burn flag and freeze the inscription
            inscription.charms = set_charm(inscription.charms, charm_burned_flag());
            let metaprotocol = inscription.metaprotocol;
            let sequence_number = inscription.sequence_number;
            let inscription_obj_id = object::id(&inscription_obj);
            if (option::is_some(&metaprotocol)) {
                let metaprotocol = option::destroy_some(metaprotocol);
                moveos_std::event_queue::emit(metaprotocol, InscriptionEvent {
                    metaprotocol: metaprotocol,
                    sequence_number,
                    inscription_obj_id,
                    event_type: InscriptionEventTypeBurn,
                });
            };
            object::to_frozen(inscription_obj);
        }else{
            object::transfer_extend(inscription_obj, to);
        };
    }

    public(friend) fun take_object(inscription_obj_id: ObjectID): Object<Inscription>{
        object::take_object_extend(inscription_obj_id)
    }

    public(friend) fun borrow_object(inscription_obj_id: ObjectID): &Object<Inscription>{
        object::borrow_object(inscription_obj_id)
    }

    fun parse_json_body(record: &InscriptionRecord): SimpleMap<String, String> {
        if (vector::is_empty(&record.body) || option::is_none(&record.content_type)) {
            return simple_map::new()
        };
        let content_type = option::destroy_some(record.content_type);
        if (content_type != string::utf8(b"text/plain;charset=utf-8") || content_type != string::utf8(
            b"text/plain"
        ) || content_type != string::utf8(b"application/json")) {
            return simple_map::new()
        };
        json::to_map(record.body)
    }

    public fun exists_inscription(id: InscriptionID): bool {
        let object_id = derive_inscription_id(id);
        object::exists_object_with_type<Inscription>(object_id)
    }

    public fun borrow_inscription(id: InscriptionID): &Inscription {
        let object_id = derive_inscription_id(id);
        assert!(object::exists_object_with_type<Inscription>(object_id), ErrorInscriptionNotExists);
        let inscription_obj = object::borrow_object(object_id);
        object::borrow(inscription_obj)
    }

    // =============== Inscription Getter =============== //

    public fun txid(self: &Inscription): address {
        self.id.txid
    }

    public fun index(self: &Inscription): u32 {
        self.id.index
    }

    public fun location(self: &Inscription): &SatPoint {
        &self.location
    }

    public fun sequence_number(self: &Inscription): u32 {
        self.sequence_number
    }

    public fun inscription_number(self: &Inscription): u32 {
        self.inscription_number
    }

    public fun is_cursed(self: &Inscription): bool {
        self.is_cursed
    }

    public fun charms(self: &Inscription): u16 {
        self.charms
    }

    public fun offset(self: &Inscription): u64 {
        self.location.offset
    }

    public fun body(self: &Inscription): vector<u8> {
        self.body
    }

    public fun content_encoding(self: &Inscription): Option<String> {
        self.content_encoding
    }

    public fun content_type(self: &Inscription): Option<String> {
        self.content_type
    }

    public fun metadata(self: &Inscription): vector<u8> {
        self.metadata
    }

    public fun metaprotocol(self: &Inscription): Option<String> {
        self.metaprotocol
    }

    public fun parents(self: &Inscription): vector<InscriptionID> {
        self.parents
    }

    public fun pointer(self: &Inscription): Option<u64> {
        self.pointer
    }

    public fun id(self: &Inscription): &InscriptionID {
        &self.id
    }

    fun drop(self: Inscription) {
        let Inscription {
            id: _,
            location: _,
            sequence_number: _,
            inscription_number: _,
            is_cursed: _,
            charms: _,
            body: _,
            content_encoding: _,
            content_type: _,
            metadata: _,
            metaprotocol: _,
            parents: _,
            pointer: _,
            rune: _,
        } = self;
    }

    public fun inscription_id_txid(self: &InscriptionID): address {
        self.txid
    }

    public fun inscription_id_index(self: &InscriptionID): u32 {
        self.index
    }

    // ===== SatPoint ========== //

    public fun new_satpoint(outpoint: OutPoint, offset: u64): SatPoint {
        SatPoint {
            outpoint,
            offset,
        }
    }

    public fun unpack_satpoint(satpoint: SatPoint): (OutPoint, u64) {
        let SatPoint { outpoint, offset } = satpoint;
        (outpoint, offset)
    }

    /// Get the SatPoint's offset
    public fun satpoint_offset(satpoint: &SatPoint): u64 {
        satpoint.offset
    }

    /// Get the SatPoint's outpoint
    public fun satpoint_outpoint(satpoint: &SatPoint): &OutPoint {
        &satpoint.outpoint
    }

    public fun satpoint_vout(satpoint: &SatPoint): u32 {
        types::outpoint_vout(&satpoint.outpoint)
    }

    // ======= Envelope and InscriptionRecord

    native fun parse_inscription_from_witness(witness: &Witness): vector<Envelope<InscriptionRecord>>;

    public(friend) fun parse_inscription_from_tx(tx: &Transaction): vector<Envelope<InscriptionRecord>> {
        let inputs = types::tx_input(tx);
        let len = vector::length(inputs);
        let input_idx = 0;
        let records = vector::empty();
        while (input_idx < len) {
            let input = vector::borrow(inputs, input_idx);
            let witness = types::txin_witness(input);
            let inscription_records = parse_inscription_from_witness(witness);
            let record_len = vector::length(&inscription_records);
            let record_idx = 0;
            while (record_idx < record_len) {
                let record = vector::borrow_mut(&mut inscription_records, record_idx);
                record.input = (input_idx as u32);
                record_idx = record_idx + 1;
            };
            vector::append(&mut records, inscription_records);
            input_idx = input_idx + 1;
        };
        records
    }

    public(friend) fun envelope_input<T>(envelope: &Envelope<T>): u32 {
        envelope.input
    }

    public(friend) fun envelope_offset<T>(envelope: &Envelope<T>): u32 {
        envelope.offset
    }

    public(friend) fun envelope_payload<T>(envelope: &Envelope<T>): &T {
        &envelope.payload
    }

    public(friend) fun envelope_pushnum<T>(envelope: &Envelope<T>): bool {
        envelope.pushnum
    }

    public(friend) fun envelope_stutter<T>(envelope: &Envelope<T>): bool {
        envelope.stutter
    }

    public(friend) fun inscription_record_pointer(record: &InscriptionRecord): &Option<u64> {
        &record.pointer
    }

    public(friend) fun inscription_record_parents(record: &InscriptionRecord): &vector<InscriptionID> {
        &record.parents
    }

    public(friend) fun inscription_record_unrecognized_even_field(record: &InscriptionRecord): bool {
        record.unrecognized_even_field
    }

    public(friend) fun inscription_record_duplicate_field(record: &InscriptionRecord): bool {
        record.duplicate_field
    }

    public(friend) fun inscription_record_incomplete_field(record: &InscriptionRecord): bool {
        record.incomplete_field
    }

    public(friend) fun inscription_record_metaprotocol(record: &InscriptionRecord): &Option<String> {
        &record.metaprotocol
    }

    public(friend) fun inscription_record_rune(record: &InscriptionRecord): &Option<u128> {
        &record.rune
    }

    public(friend) fun inscription_record_metadata(record: &InscriptionRecord): &vector<u8> {
        &record.metadata
    }

    public(friend) fun inscription_record_content_type(record: &InscriptionRecord): &Option<String> {
        &record.content_type
    }

    public(friend) fun inscription_record_content_encoding(record: &InscriptionRecord): &Option<String> {
        &record.content_encoding
    }

    public(friend) fun inscription_record_body(record: &InscriptionRecord): &vector<u8> {
        &record.body
    }

    // ===== permenent area ========== //
    #[private_generics(S)]
    public fun add_permanent_state<S: store>(inscription: &mut Object<Inscription>, state: S) {
        if (object::contains_field(inscription, PERMANENT_AREA)) {
            let bag = object::borrow_mut_field(inscription, PERMANENT_AREA);
            let name = type_info::type_name<S>();
            bag::add(bag, name, state);
        }else {
            let bag = bag::new();
            let name = type_info::type_name<S>();
            bag::add(&mut bag, name, state);
            object::add_field(inscription, PERMANENT_AREA, bag);
        }
    }

    public fun contains_permanent_state<S: store>(inscription: &Object<Inscription>): bool {
        if (object::contains_field(inscription, PERMANENT_AREA)) {
            let bag = object::borrow_field(inscription, PERMANENT_AREA);
            let name = type_info::type_name<S>();
            bag::contains(bag, name)
        }else {
            false
        }
    }

    public fun borrow_permanent_state<S: store>(inscription: &Object<Inscription>): &S {
        let bag = object::borrow_field(inscription, PERMANENT_AREA);
        let name = type_info::type_name<S>();
        bag::borrow(bag, name)
    }

    #[private_generics(S)]
    public fun borrow_mut_permanent_state<S: store>(inscription: &mut Object<Inscription>): &mut S {
        let bag = object::borrow_mut_field(inscription, PERMANENT_AREA);
        let name = type_info::type_name<S>();
        bag::borrow_mut(bag, name)
    }

    #[private_generics(S)]
    public fun remove_permanent_state<S: store>(inscription: &mut Object<Inscription>): S {
        let bag = object::borrow_mut_field(inscription, PERMANENT_AREA);
        let name = type_info::type_name<S>();
        bag::remove(bag, name)
    }

    /// Destroy permanent area if it's empty. Aborts if it's not empty.
    public fun destroy_permanent_area(inscription: &mut Object<Inscription>) {
        if (object::contains_field(inscription, PERMANENT_AREA)) {
            let bag = object::remove_field(inscription, PERMANENT_AREA);
            bag::destroy_empty(bag);
        }
    }


    // ==== Temporary Area ===

    #[private_generics(S)]
    public fun add_temp_state<S: store + drop>(inscription: &mut Object<Inscription>, state: S) {
        if (object::contains_field(inscription, TEMPORARY_AREA)) {
            let bag = object::borrow_mut_field(inscription, TEMPORARY_AREA);
            let name = type_info::type_name<S>();
            bag::add_dropable(bag, name, state);
        }else {
            let bag = bag::new_dropable();
            let name = type_info::type_name<S>();
            bag::add_dropable(&mut bag, name, state);
            object::add_field(inscription, TEMPORARY_AREA, bag);
        }
    }

    public fun contains_temp_state<S: store + drop>(inscription: &Object<Inscription>): bool {
        if (object::contains_field(inscription, TEMPORARY_AREA)) {
            let bag = object::borrow_field(inscription, TEMPORARY_AREA);
            let name = type_info::type_name<S>();
            bag::contains(bag, name)
        }else {
            false
        }
    }

    public fun borrow_temp_state<S: store + drop>(inscription: &Object<Inscription>): &S {
        let bag = object::borrow_field(inscription, TEMPORARY_AREA);
        let name = type_info::type_name<S>();
        bag::borrow(bag, name)
    }

    #[private_generics(S)]
    public fun borrow_mut_temp_state<S: store + drop>(inscription: &mut Object<Inscription>): &mut S {
        let bag = object::borrow_mut_field(inscription, TEMPORARY_AREA);
        let name = type_info::type_name<S>();
        bag::borrow_mut(bag, name)
    }

    #[private_generics(S)]
    public fun remove_temp_state<S: store + drop>(inscription: &mut Object<Inscription>): S {
        let bag = object::borrow_mut_field(inscription, TEMPORARY_AREA);
        let name = type_info::type_name<S>();
        bag::remove(bag, name)
    }

    /// Drop the bag, whether it's empty or not
    public(friend) fun drop_temp_area(inscription: &mut Object<Inscription>) {
        if (object::contains_field(inscription, TEMPORARY_AREA)) {
            let bag = object::remove_field(inscription, TEMPORARY_AREA);
            bag::drop(bag);
        }
    }

    // ==== Inscription Metaprotocol Validity ==== //

    /// Currently, Only the framework can register metaprotocol.
    /// We need to find a way to allow the user to register metaprotocol.
    public fun register_metaprotocol_via_system<T>(system: &signer, metaprotocol: String) {
        moveos_std::core_addresses::assert_system_reserved(system);
        let registry_object_id = object::named_object_id<MetaprotocolRegistry>();
        if (!object::exists_object(registry_object_id)) {
            let registry_obj = object::new_named_object(MetaprotocolRegistry {});
            object::transfer_extend(registry_obj, @bitcoin_move);
        };
        let metaprotocol_registry_obj = object::borrow_mut_object_extend<MetaprotocolRegistry>(registry_object_id);
        let protocol_type = type_info::type_name<T>();
        assert!(!object::contains_field(metaprotocol_registry_obj, metaprotocol), ErrorMetaprotocolAlreadyRegistered);
        object::add_field(metaprotocol_registry_obj, metaprotocol, protocol_type);
    }

    public fun is_metaprotocol_register(metaprotocol: String): bool {
        let registry_object_id = object::named_object_id<MetaprotocolRegistry>();
        if (!object::exists_object(registry_object_id)) {
            return false
        };
        let metaprotocol_registry_obj = object::borrow_object<MetaprotocolRegistry>(registry_object_id);
        object::contains_field(metaprotocol_registry_obj, metaprotocol)
    }

    /// Borrow the metaprotocol Move type for the given metaprotocol.
    fun get_metaprotocol_type(metaprotocol: String): Option<String> {
        let registry_object_id = object::named_object_id<MetaprotocolRegistry>();
        if (!object::exists_object(registry_object_id)) {
            return option::none()
        };
        let metaprotocol_registry_obj = object::borrow_object<MetaprotocolRegistry>(registry_object_id);
        if (!object::contains_field(metaprotocol_registry_obj, metaprotocol)) {
            return option::none()
        };
        option::some(*object::borrow_field(metaprotocol_registry_obj, metaprotocol))
    }

    #[private_generics(T)]
    /// Seal the metaprotocol validity for the given inscription_id.
    public fun seal_metaprotocol_validity<T>(
        inscription_id: InscriptionID,
        is_valid: bool,
        invalid_reason: Option<String>
    ) {
        let inscription_object_id = derive_inscription_id(inscription_id);
        let inscription_obj = object::borrow_mut_object_extend<Inscription>(inscription_object_id);

        let protocol_type = type_info::type_name<T>();
        assert!(metaprotocol_protocol_match(inscription_obj, protocol_type), ErrorMetaprotocolProtocolMismatch);
        let validity = MetaprotocolValidity {
            protocol_type,
            is_valid,
            invalid_reason,
        };

        object::upsert_field(inscription_obj, METAPROTOCOL_VALIDITY, validity);
    }

    #[private_generics(T)]
    public fun add_metaprotocol_attachment<T>(inscription_id: InscriptionID, attachment: Object<T>) {
        let inscription_object_id = derive_inscription_id(inscription_id);
        let inscription_obj = object::borrow_mut_object_extend<Inscription>(inscription_object_id);
        let protocol_type = type_info::type_name<T>();
        assert!(metaprotocol_protocol_match(inscription_obj, protocol_type), ErrorMetaprotocolProtocolMismatch);
        object::add_field(inscription_obj, protocol_type, attachment);
    }

    /// Returns true if Inscription `object` contains metaprotocol validity
    public fun exists_metaprotocol_validity(inscription_id: InscriptionID): bool {
        let inscription_object_id = derive_inscription_id(inscription_id);
        let exists = object::exists_object_with_type<Inscription>(inscription_object_id);
        if (!exists) {
            return false
        };

        let inscription_obj = object::borrow_object<Inscription>(inscription_object_id);
        object::contains_field(inscription_obj, METAPROTOCOL_VALIDITY)
    }

    /// Borrow the metaprotocol validity for the given inscription_id.
    public fun borrow_metaprotocol_validity(inscription_id: InscriptionID): &MetaprotocolValidity {
        let inscription_object_id = derive_inscription_id(inscription_id);
        let inscription_obj = object::borrow_object<Inscription>(inscription_object_id);

        object::borrow_field(inscription_obj, METAPROTOCOL_VALIDITY)
    }

    fun metaprotocol_protocol_match(inscription_obj: &Object<Inscription>, protocol_type: String): bool {
        let inscription = object::borrow(inscription_obj);
        if (option::is_none(&inscription.metaprotocol)) {
            return false
        };
        let metaprotocol = option::destroy_some(*&inscription.metaprotocol);
        let protocol_type_in_registry = get_metaprotocol_type(metaprotocol);
        if (option::is_none(&protocol_type_in_registry)) {
            return false
        };
        protocol_type == option::destroy_some(protocol_type_in_registry)
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

    public fun view_validity(inscription_id_str: String): Option<MetaprotocolValidity> {
        let inscription_id_option = parse_inscription_id(&inscription_id_str);
        if (option::is_none(&inscription_id_option)) {
            return option::none()
        };

        let inscription_id = option::destroy_some(inscription_id_option);
        if (!exists_metaprotocol_validity(inscription_id)) {
            return option::none()
        };

        let validity = borrow_metaprotocol_validity(inscription_id);

        option::some(*validity)
    }

    // ======================== Events =====================================

    public fun unpack_inscription_event(event: InscriptionEvent): (String, u32, ObjectID, u8) {
        let InscriptionEvent { metaprotocol, sequence_number, inscription_obj_id, event_type } = event;
        (metaprotocol, sequence_number, inscription_obj_id, event_type)
    }

    public fun inscription_event_type_new(): u8 {
        InscriptionEventTypeNew
    }

    public fun inscription_event_type_burn(): u8 {
        InscriptionEventTypeBurn
    }

    // ==== Inscription Charm ==== //

    //The charm flags are based on the ordinals library
    //https://github.com/ordinals/ord/blob/75bf04b22107155f8f8ab6c77f6eefa8117d9ace/crates/ordinals/src/charm.rs

    //   pub enum Charm {
    //   Coin = 0,
    //   Cursed = 1,
    //   Epic = 2,
    //   Legendary = 3,
    //   Lost = 4,
    //   Nineball = 5,
    //   Rare = 6,
    //   Reinscription = 7,
    //   Unbound = 8,
    //   Uncommon = 9,
    //   Vindicated = 10,
    //   Mythic = 11,
    //   Burned = 12,
    // }

    const CHARM_COIN_FLAG: u16 = 1<<0;
    public fun charm_coin_flag() : u16 {CHARM_COIN_FLAG}

    const CHARM_CURSED_FLAG: u16 = 1 << 1;
    public fun charm_cursed_flag() : u16 {CHARM_CURSED_FLAG}

    const CHARM_EPIC_FLAG: u16 = 1 << 2;
    public fun charm_epic_flag() : u16 {CHARM_EPIC_FLAG}

    const CHARM_LEGENDARY_FLAG: u16 = 1 << 3;
    public fun charm_legendary_flag() : u16 {CHARM_LEGENDARY_FLAG}

    const CHARM_LOST_FLAG: u16 = 1 << 4;
    public fun charm_lost_flag() : u16 {CHARM_LOST_FLAG}

    const CHARM_NINEBALL_FLAG: u16 = 1 << 5;
    public fun charm_nineball_flag() : u16 {CHARM_NINEBALL_FLAG}

    const CHARM_RARE_FLAG: u16 = 1 << 6;
    public fun charm_rare_flag() : u16 {CHARM_RARE_FLAG}

    const CHARM_REINSCRIPTION_FLAG: u16 = 1 << 7;
    public fun charm_reinscription_flag() : u16 {CHARM_REINSCRIPTION_FLAG}

    const CHARM_UNBOUND_FLAG: u16 = 1 << 8;
    public fun charm_unbound_flag() : u16 {CHARM_UNBOUND_FLAG}

    const CHARM_UNCOMMON_FLAG: u16 = 1 << 9;
    public fun charm_uncommon_flag() : u16 {CHARM_UNCOMMON_FLAG}

    const CHARM_VINDICATED_FLAG: u16 = 1 << 10;
    public fun charm_vindicated_flag() : u16 {CHARM_VINDICATED_FLAG}

    const CHARM_MYTHIC_FLAG: u16 = 1 << 11;
    public fun charm_mythic_flag() : u16 {CHARM_MYTHIC_FLAG}

    const CHARM_BURNED_FLAG: u16 = 1 << 12;
    public fun charm_burned_flag() : u16 {CHARM_BURNED_FLAG}

    public fun set_charm(charms: u16, flag: u16) : u16 {
        charms | flag
    }

    // public fun unset_charm(charms: u16, flag: u16) : u16 {
    //     charms & ^flag
    // }

    public fun is_set_charm(charms: u16, flag: u16) : bool {
        (charms & flag) != 0
    }

    /// A struct to represent the Inscription Charm
    struct InscriptionCharm has copy, drop, store{
        coin: bool,
        cursed: bool,
        epic: bool,
        legendary: bool,
        lost: bool,
        nineball: bool,
        rare: bool,
        reinscription: bool,
        unbound: bool,
        uncommon: bool,
        vindicated: bool,
        mythic: bool,
        burned: bool,
    }

    fun view_charms(charms: u16) : InscriptionCharm {
        InscriptionCharm {
            coin: is_set_charm(charms, CHARM_COIN_FLAG),
            cursed: is_set_charm(charms, CHARM_CURSED_FLAG),
            epic: is_set_charm(charms, CHARM_EPIC_FLAG),
            legendary: is_set_charm(charms, CHARM_LEGENDARY_FLAG),
            lost: is_set_charm(charms, CHARM_LOST_FLAG),
            nineball: is_set_charm(charms, CHARM_NINEBALL_FLAG),
            rare: is_set_charm(charms, CHARM_RARE_FLAG),
            reinscription: is_set_charm(charms, CHARM_REINSCRIPTION_FLAG),
            unbound: is_set_charm(charms, CHARM_UNBOUND_FLAG),
            uncommon: is_set_charm(charms, CHARM_UNCOMMON_FLAG),
            vindicated: is_set_charm(charms, CHARM_VINDICATED_FLAG),
            mythic: is_set_charm(charms, CHARM_MYTHIC_FLAG),
            burned: is_set_charm(charms, CHARM_BURNED_FLAG),
        }
    }
    

    /// Views the Inscription charms for a given inscription ID string.
    /// Returns None if the inscription doesn't exist
    /// 
    /// @param inscription_id_str - String representation of the inscription ID
    /// @return Option<InscriptionCharm> - Some(charm) if exists, None otherwise
    public fun view_inscription_charm(inscription_id_str: String): Option<InscriptionCharm> {
        let inscription_id_option = parse_inscription_id(&inscription_id_str);
        if (option::is_none(&inscription_id_option)) {
            return option::none()
        };

        let inscription_id = option::destroy_some(inscription_id_option);
        if (!exists_inscription(inscription_id)) {
            return option::none()
        };
        let inscription = borrow_inscription(inscription_id);
        let charms = view_charms(inscription.charms);
        option::some(charms)
    }

    #[test_only]
    public fun init_for_test(_genesis_account: &signer) {
        genesis_init();
    }

    #[test_only]
    public fun drop_temp_area_for_test(inscription: &mut Object<Inscription>) {
        drop_temp_area(inscription);
    }

    #[test_only]
    public fun new_inscription_object_for_test(
        id: InscriptionID,
        vout: u32,
        offset: u64,
        body: vector<u8>,
        content_encoding: Option<String>,
        content_type: Option<String>,
        metadata: vector<u8>,
        metaprotocol: Option<String>,
        parents: vector<InscriptionID>,
        pointer: Option<u64>,
    ): Object<Inscription> {
        let inscription = new_inscription_for_test(
            id,
            vout,
            offset,
            body,
            content_encoding,
            content_type,
            metadata,
            metaprotocol,
            parents,
            pointer,
        );
        create_object_internal(inscription)
    }

    #[test_only]
    public fun drop_inscription_object_for_test(inscription: Object<Inscription>) {
        let inscription = object::remove(inscription);
        drop(inscription);
    }

    #[test_only]
    public fun new_inscription_for_test(
        id: InscriptionID,
        vout: u32,
        offset: u64,
        body: vector<u8>,
        content_encoding: Option<String>,
        content_type: Option<String>,
        metadata: vector<u8>,
        metaprotocol: Option<String>,
        parents: vector<InscriptionID>,
        pointer: Option<u64>,
    ): Inscription {
        let location = SatPoint {
            outpoint: types::new_outpoint(id.txid, vout),
            offset,
        };
        Inscription {
            id,
            location,
            sequence_number: 0,
            inscription_number: 0,
            is_cursed: false,
            charms: 0,
            body,
            content_encoding,
            content_type,
            metadata,
            metaprotocol,
            parents,
            pointer,
            rune: option::none(),
        }
    }

    #[test_only]
    public fun register_metaprotocol_for_test<T>(metaprotocol: String) {
        let system = moveos_std::signer::module_signer<TestProtocol>();
        register_metaprotocol_via_system<T>(&system, metaprotocol);
    }

    #[test_only]
    public fun setup_inscription_for_test<T>(_genesis_account: &signer, metaprotocol: String): (address, InscriptionID) {
        genesis_init();

        // prepare test inscription
        let test_address = @0x5416690eaaf671031dc609ff8d36766d2eb91ca44f04c85c27628db330f40fd1;
        let test_txid = @0x21da2ae8cc773b020b4873f597369416cf961a1896c24106b0198459fec2df77;
        let test_inscription_id = new_inscription_id(test_txid, 0);

        let content_type = b"application/wasm";
        let body = x"0061736d0100000001080260017f00600000020f0107636f6e736f6c65036c6f670000030201010503010001071702066d656d6f727902000a68656c6c6f576f726c6400010a08010600410010000b0b14010041000b0e48656c6c6f2c20576f726c642100";
        if (!is_metaprotocol_register(metaprotocol)) {
            register_metaprotocol_for_test<T>(metaprotocol);
        };

        let ins_obj = new_inscription_object_for_test(
            test_inscription_id,
            0,
            0,
            body,
            option::none(),
            option::some(string::utf8(content_type)),
            vector[],
            option::some(metaprotocol),
            vector[],
            option::none(),
        );
        object::transfer_extend(ins_obj, test_address);
        
        (test_address, test_inscription_id)
    }

     #[test_only]
    struct PermanentState has store {
        value: u64,
    }

    #[test]
    fun test_permanent_state() {
        genesis_init();
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let id = new_inscription_id(txid, 0);
        let inscription_obj = new_inscription_object_for_test(
            id,
            0,
            0,
            vector[],
            option::none(),
            option::none(),
            vector[],
            option::none(),
            vector[],
            option::none(),
        );
        add_permanent_state(&mut inscription_obj, PermanentState { value: 10 });
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
    fun test_temp_state() {
        genesis_init();
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let id = new_inscription_id(txid, 0);
        let inscription_obj = new_inscription_object_for_test(
            id,
            0,
            0,
            vector[],
            option::none(),
            option::none(),
            vector[],
            option::none(),
            vector[],
            option::none(),
        );
        add_temp_state(&mut inscription_obj, TempState { value: 10 });
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
        genesis_init();
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let id = new_inscription_id(txid, 0);
        let inscription_obj = new_inscription_object_for_test(
            id,
            0,
            0,
            vector[],
            option::none(),
            option::none(),
            vector[],
            option::none(),
            vector[],
            option::none(),
        );

        add_temp_state(&mut inscription_obj, TempState { value: 10 });
        add_permanent_state(&mut inscription_obj, PermanentState { value: 10 });
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

    #[test(genesis_account= @0x4)]
    fun test_metaprotocol_validity(genesis_account: &signer): InscriptionID {
        // prepare test inscription
        let (_test_address, test_inscription_id) = setup_inscription_for_test<TestProtocol>(
            genesis_account,
            string::utf8(b"TestProtocol")
        );

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
        test_inscription_id
    }

    #[test]
    fun test_parse_inscription_id_ok() {
        let inscription_id_str = std::string::utf8(
            b"6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0"
        );
        let inscription_id_option = parse_inscription_id(&inscription_id_str);
        assert!(option::is_some(&inscription_id_option), 1);
        let inscription_id = option::destroy_some(inscription_id_option);
        assert!(inscription_id_str == inscription_id_to_string(&inscription_id), 2);
    }

    #[test]
    fun test_parse_inscription_id_fail_with_invalid_txid_str() {
        let inscription_id_str = std::string::utf8(x"E4BDA0E5A5BD6930");
        let inscription_id_option = parse_inscription_id(&inscription_id_str);
        assert!(option::is_none(&inscription_id_option), 1);
    }

    #[test]
    fun test_parse_inscription_id_fail_with_invalid_txid_address() {
        let inscription_id_str = std::string::utf8(
            b"6x55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0"
        );
        let inscription_id_option = parse_inscription_id(&inscription_id_str);
        assert!(option::is_none(&inscription_id_option), 1);
    }

    #[test]
    fun test_parse_inscription_id_fail_with_without_i() {
        let inscription_id_str = std::string::utf8(b"6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8");
        let inscription_id_option = parse_inscription_id(&inscription_id_str);
        assert!(option::is_none(&inscription_id_option), 1);
    }

    #[test]
    fun test_parse_inscription_id_fail_with_invalid_index() {
        let inscription_id_str = std::string::utf8(
            b"6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8ix"
        );
        let inscription_id_option = parse_inscription_id(&inscription_id_str);
        assert!(option::is_none(&inscription_id_option), 1);
    }

    #[test(genesis_account= @0x1)]
    fun test_inscription_charm(genesis_account: &signer) {
        // Setup
        let (_addr, test_inscription_id) = setup_inscription_for_test<TestProtocol>(genesis_account, string::utf8(b"TestProtocol"));

        // Test view_inscription_charm
        let inscription_id_str = inscription_id_to_string(&test_inscription_id);
        let viewed_charm_option = view_inscription_charm(inscription_id_str);
        assert!(option::is_some(&viewed_charm_option), 5);
        let viewed_charm = option::destroy_some(viewed_charm_option);
        assert!(!viewed_charm.burned, 6);

        // Test view_inscription_charm with non-existent inscription
        let non_existent_id_str = string::utf8(b"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdefi0");
        let non_existent_charm_option = view_inscription_charm(non_existent_id_str);
        assert!(option::is_none(&non_existent_charm_option), 7);
    }

    #[test(genesis_account= @0x1)]
    fun test_inscription_charm_edge_cases(genesis_account: &signer) {
        // Setup
        setup_inscription_for_test<TestProtocol>(genesis_account, string::utf8(b"TestProtocol"));

        // Test with invalid inscription ID string
        let invalid_id_str = string::utf8(b"invalid_id");
        let invalid_charm_option = view_inscription_charm(invalid_id_str);
        assert!(option::is_none(&invalid_charm_option), 1);
    }
}
