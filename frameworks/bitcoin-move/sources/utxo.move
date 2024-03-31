// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::utxo{
    use std::vector;
    use std::string::String;
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::simple_multimap::{Self, SimpleMultiMap};
    use moveos_std::type_info;
    use moveos_std::bag;
    use bitcoin_move::types::{Self, OutPoint};

    friend bitcoin_move::genesis;
    friend bitcoin_move::ord;
    friend bitcoin_move::light_client;

    const TEMPORARY_AREA: vector<u8> = b"temporary_area";

    /// The UTXO Object
    struct UTXO has key {
        /// The txid of the UTXO
        txid: address,
        /// The vout of the UTXO
        vout: u32,
        /// The value of the UTXO
        value: u64,
        /// Protocol seals
        seals: SimpleMultiMap<String, SealPoint>
    }

    struct UTXOSeal has store, copy, drop {
        protocol: String,
        seal_point: SealPoint,
    }

    struct SealPoint has store, copy, drop {
        output_index: u32,
        offset: u64,
        object_id: ObjectID,
    }

    struct BitcoinUTXOStore has key{
        /// The next tx index to be processed
        next_tx_index: u64,
    }

    public(friend) fun genesis_init(){
        let btc_utxo_store = BitcoinUTXOStore{
            next_tx_index: 0,
        };
        let obj = object::new_named_object(btc_utxo_store);
        object::to_shared(obj);
    }

    // ======= UTOXStore =========

    public fun borrow_utxo_store(): &Object<BitcoinUTXOStore>{
        let id = object::named_object_id<BitcoinUTXOStore>();
        object::borrow_object(id)
    }

    public(friend) fun borrow_mut_utxo_store() : &mut Object<BitcoinUTXOStore> {
        let id = object::named_object_id<BitcoinUTXOStore>();
        let obj = object::borrow_mut_object_shared(id);
        obj
    }

    public(friend) fun next_tx_index(): u64 {
        let utxo_store = borrow_utxo_store();
        object::borrow(utxo_store).next_tx_index
    }

    public(friend) fun update_next_tx_index(next_tx_index: u64){
        let utxo_store = borrow_mut_utxo_store();
        object::borrow_mut(utxo_store).next_tx_index = next_tx_index;
    }

    // ======= UTXO =========
    public(friend) fun new(txid: address, vout: u32, value: u64) : Object<UTXO> {
        let id = types::new_outpoint(txid, vout);
        let utxo = UTXO{
            txid,
            vout,
            value,
            seals: simple_multimap::new(),
        };
        let uxto_store = borrow_mut_utxo_store();
        object::add_object_field_with_id(uxto_store, id, utxo)
    }

    public fun derive_utxo_id(outpoint: OutPoint) : ObjectID {
        let parent_id = object::named_object_id<BitcoinUTXOStore>();
        object::custom_child_object_id<OutPoint, UTXO>(parent_id, outpoint)
    }

    /// Get the UTXO's value
    public fun value(utxo: &UTXO): u64 {
        utxo.value
    }

    /// Get the UTXO's txid
    public fun txid(utxo: &UTXO): address {
        utxo.txid
    }

    /// Get the UTXO's vout
    public fun vout(utxo: &UTXO): u32 {
        utxo.vout
    }

    public fun exists_utxo(outpoint: OutPoint): bool{
        let object_id = derive_utxo_id(outpoint);
        object::exists_object_with_type<UTXO>(object_id)
    }

    public fun borrow_utxo(outpoint: OutPoint): &Object<UTXO>{
        let object_id = derive_utxo_id(outpoint);
        object::borrow_object(object_id)
    }

    /// Get the SealPoint's object_id
    public fun seal_point_object_id(seal_point: &SealPoint): ObjectID {
        seal_point.object_id
    }

    /// Get the SealPoint's offset
    public fun seal_point_offset(seal_point: &SealPoint): u64 {
        seal_point.offset
    }

    /// Get the SealPoint's output_index
    public fun seal_point_output_index(seal_point: &SealPoint): u32 {
        seal_point.output_index
    }

    #[private_generics(T)]
    /// Seal the UTXO with a protocol, the T is the protocol object
    public fun seal<T>(utxo: &mut UTXO, seal_obj: &Object<T>, offset: u64){
        let protocol = type_info::type_name<T>();
        let object_id = object::id(seal_obj);
        let output_index = vout(utxo);
        let seal_point = SealPoint {
            output_index,
            offset,
            object_id,
        };
        let utxo_seal = UTXOSeal{
            protocol,
            seal_point,
        };
        add_seal(utxo, utxo_seal);
    }

    public fun has_seal<T>(utxo: &UTXO) : bool {
        let protocol = type_info::type_name<T>();
        simple_multimap::contains_key(&utxo.seals, &protocol)
    }

    public fun get_seals<T>(utxo: &UTXO) : vector<SealPoint> {
        let protocol = type_info::type_name<T>();
        if(simple_multimap::contains_key(&utxo.seals, &protocol)){
            *simple_multimap::borrow(&utxo.seals, &protocol)
        }else{
            vector::empty()
        }
    }

    public fun remove_seals<T>(utxo: &mut UTXO): vector<SealPoint> {
        let protocol = type_info::type_name<T>();
        if(simple_multimap::contains_key(&utxo.seals, &protocol)){
            let(_k, value) = simple_multimap::remove(&mut utxo.seals, &protocol);
            value
        }else{
            vector::empty()
        }
    }

    public(friend) fun add_seal(utxo: &mut UTXO, utxo_seal: UTXOSeal){
        let UTXOSeal{protocol, seal_point} = utxo_seal;
        simple_multimap::add(&mut utxo.seals, protocol, seal_point);
    }

    // === Object<UTXO> ===    

    public(friend) fun transfer(utxo_obj: Object<UTXO>, to: address){
        object::transfer_extend(utxo_obj, to);
    }

    public(friend) fun take(object_id: ObjectID): (address, Object<UTXO>){
        object::take_object_extend<UTXO>(object_id)
    }

    public(friend) fun remove(utxo_obj: Object<UTXO>): SimpleMultiMap<String, SealPoint>{
        if(object::contains_field(&utxo_obj, TEMPORARY_AREA)){
            let bag = object::remove_field(&mut utxo_obj, TEMPORARY_AREA);
            bag::drop(bag);
        };
        let uxto_store = borrow_mut_utxo_store();
        let utxo = object::remove_object_field(uxto_store, utxo_obj);
        let UTXO{txid:_, vout:_, value:_, seals} = utxo;
        seals
    }

    // === UTXOSeal ===
    public fun new_utxo_seal(protocol: String, seal_point: SealPoint) : UTXOSeal {
        UTXOSeal{
            protocol,
            seal_point
        }
    }

    public fun unpack_utxo_seal(utxo_seal: UTXOSeal) : (String, SealPoint) {
        let UTXOSeal{protocol, seal_point} = utxo_seal;
        (protocol, seal_point)
    }

    // === SealPoint ===
    public fun new_seal_point(output_index: u32, offset: u64, object_id: ObjectID) : SealPoint {
        SealPoint{
            output_index,
            offset,
            object_id
        }
    }

    public fun unpack_seal_point(seal_point: SealPoint) : (u32, u64, ObjectID) {
        let SealPoint{output_index, offset, object_id} = seal_point;
        (output_index, offset, object_id)
    }

    // ==== Temporary Area ===

    #[private_generics(S)]
    public fun add_temp_state<S: store + drop>(utxo: &mut Object<UTXO>, state: S){
        if(object::contains_field(utxo, TEMPORARY_AREA)){
            let bag = object::borrow_mut_field(utxo, TEMPORARY_AREA);
            let name = type_info::type_name<S>();
            bag::add_dropable(bag, name, state);
        }else{
            let bag = bag::new_dropable();
            let name = type_info::type_name<S>();
            bag::add_dropable(&mut bag, name, state);
            object::add_field(utxo, TEMPORARY_AREA, bag);
        }
    }

    public fun contains_temp_state<S: store + drop>(utxo: &Object<UTXO>) : bool {
        if(object::contains_field(utxo, TEMPORARY_AREA)){
            let bag = object::borrow_field(utxo, TEMPORARY_AREA);
            let name = type_info::type_name<S>();
            bag::contains(bag, name)
        }else{
            false
        }
    }

    public fun borrow_temp_state<S: store + drop>(utxo: &Object<UTXO>) : &S {
        let bag = object::borrow_field(utxo, TEMPORARY_AREA);
        let name = type_info::type_name<S>();
        bag::borrow(bag, name)
    }

    #[private_generics(S)]
    public fun borrow_mut_temp_state<S: store + drop>(utxo: &mut Object<UTXO>) : &mut S {
        let bag = object::borrow_mut_field(utxo, TEMPORARY_AREA);
        let name = type_info::type_name<S>();
        bag::borrow_mut(bag, name)
    }

    #[private_generics(S)]
    public fun remove_temp_state<S: store + drop>(utxo: &mut Object<UTXO>) : S {
        let bag = object::borrow_mut_field(utxo, TEMPORARY_AREA);
        let name = type_info::type_name<S>();
        bag::remove(bag, name)
    }

    #[test_only]
    public fun new_for_testing(txid: address, vout: u32, value: u64) : Object<UTXO> {
        new(txid, vout, value)
    }

    #[test_only]
    public fun drop_for_testing(utxo: Object<UTXO>){
        let seals = remove(utxo);
        simple_multimap::drop(seals);
    }

    #[test]
    fun test_id(){
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let vout = 0;
        let object_id = derive_utxo_id(types::new_outpoint(txid, vout));
        std::debug::print(&std::bcs::to_bytes(&object_id));
        assert!(std::bcs::to_bytes(&object_id) == x"02826a5e56581ba5ab84c39976f27cf3578cf524308b4ffc123922dfff507e514db8fc937bf3c15abe49c95fa6906aff29087149f542b48db0cf25dce671a68a63", 1);
    }

    #[test]
    fun test_remove(){
        genesis_init();
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let vout = 0;
        let utxo = new(txid, vout, 100);
        let seals = remove(utxo);
        simple_multimap::drop(seals);
    }

    struct TempState has store, copy, drop {
        value: u64,
    }

    #[test]
    fun test_temporary_area(){
        genesis_init();
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let vout = 0;
        let utxo = new(txid, vout, 100);
        add_temp_state(&mut utxo, TempState{value: 10});
        assert!(contains_temp_state<TempState>(&utxo), 1000);
        assert!(borrow_temp_state<TempState>(&utxo).value == 10, 1001);
        {
            let state = borrow_mut_temp_state<TempState>(&mut utxo);
            state.value = 20;
        };
        let state = remove_temp_state<TempState>(&mut utxo);
        assert!(state.value == 20, 1);
        assert!(!contains_temp_state<TempState>(&utxo), 1002);
        let seals = remove(utxo);
        simple_multimap::drop(seals);
    }
}