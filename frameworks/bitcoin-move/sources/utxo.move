// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::utxo{
    use std::vector;
    use std::string::String;
    use bitcoin_move::types::OutPoint;
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::simple_multimap::{Self, SimpleMultiMap};
    use moveos_std::type_info;
    use moveos_std::bag;

    friend bitcoin_move::light_client;

    const TEMPORARY_AREA: vector<u8> = b"temporary_area";

    /// The transaction output ID
    struct OutputID has store, copy, drop {
        /// The txid of the UTXO
        txid: address,
        /// The vout of the UTXO
        vout: u32,
    }

    /// The UTXO Object
    struct UTXO has key {
        /// The txid of the UTXO
        txid: address,
        /// The vout of the UTXO
        vout: u32,
        /// The value of the UTXO
        value: u64,
        /// Protocol seals
        seals: SimpleMultiMap<String, ObjectID>
    }

    struct UTXOSeal has store, copy, drop {
        protocol: String,
        object_id: ObjectID,
    }

    struct SealOut has store, copy, drop {
        output_index: u64,
        object_id: ObjectID,
    }

    //
    struct UTXORange has store, copy, drop {
        txid: address,
        vout: u32,
        /// [start_offset, end_offset)
        start_offset: u64,
        end_offset: u64,
    }

    public(friend) fun new(txid: address, vout: u32, value: u64) : Object<UTXO> {
        let id = OutputID{
            txid,
            vout,
        };
        let utxo = UTXO{
            txid,
            vout,
            value,
            seals: simple_multimap::new(),
        };
        object::new_custom_object(id, utxo)
    }

    public fun new_id(txid: address, vout: u32) : OutputID {
        OutputID{
            txid,
            vout,
        }
    }

    public fun new_utxo_range(txid: address, vout: u32, start_offset: u64, end_offset: u64) : UTXORange {
        UTXORange{
            txid,
            vout,
            start_offset,
            end_offset,
        }
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

    public fun exists_utxo(txid: address, vout: u32): bool{
        let id = OutputID{
            txid,
            vout,
        };
        let object_id = object::custom_object_id<OutputID,UTXO>(id);
        object::exists_object_with_type<UTXO>(object_id)
    }

    public fun borrow_utxo(txid: address, vout: u32): &Object<UTXO>{
        let id = OutputID{
            txid,
            vout,
        };
        let object_id = object::custom_object_id<OutputID,UTXO>(id);
        object::borrow_object(object_id)
    }

    /// Get the UTXORange's txid
    public fun utxo_range_txid(utxo_range: &UTXORange): address {
        utxo_range.txid
    }

    /// Get the UTXORange's vout
    public fun utxo_range_vout(utxo_range: &UTXORange): u32 {
        utxo_range.vout
    }

    /// Get the UTXORange's value range
    public fun utxo_range_range(utxo_range: &UTXORange): (u64, u64) {
        (utxo_range.start_offset, utxo_range.end_offset)
    }


    #[private_generics(T)]
    /// Seal the UTXO with a protocol, the T is the protocol object
    public fun seal<T>(utxo: &mut UTXO, seal_obj: &Object<T>){
        let protocol = type_info::type_name<T>();
        let object_id = object::id(seal_obj);
        let utxo_seal = UTXOSeal{
            protocol: protocol,
            object_id: object_id,
        };
        add_seal(utxo, utxo_seal);
    }

    public fun has_seal<T>(utxo: &UTXO) : bool {
        let protocol = type_info::type_name<T>();
        simple_multimap::contains_key(&utxo.seals, &protocol)
    }

    public fun get_seals<T>(utxo: &UTXO) : vector<ObjectID> {
        let protocol = type_info::type_name<T>();
        if(simple_multimap::contains_key(&utxo.seals, &protocol)){
            *simple_multimap::borrow(&utxo.seals, &protocol)
        }else{
            vector::empty()
        }
    }

    public fun remove_seals<T>(utxo: &mut UTXO): vector<ObjectID> {
        let protocol = type_info::type_name<T>();
        if(simple_multimap::contains_key(&utxo.seals, &protocol)){
            let(_k, value) = simple_multimap::remove(&mut utxo.seals, &protocol);
            value
        }else{
            vector::empty()
        }
    }

    public(friend) fun add_seal(utxo: &mut UTXO, utxo_seal: UTXOSeal){
        let UTXOSeal{protocol, object_id} = utxo_seal;
        simple_multimap::add(&mut utxo.seals, protocol, object_id);
    }

    // === Object<UTXO> ===    

    public(friend) fun transfer(utxo_obj: Object<UTXO>, to: address){
        object::transfer_extend(utxo_obj, to);
    }

    public(friend) fun take(object_id: ObjectID): (address, Object<UTXO>){
        object::take_object_extend<UTXO>(object_id)
    }

    public(friend) fun remove(utxo_obj: Object<UTXO>): SimpleMultiMap<String, ObjectID>{
        if(object::contains_field(&utxo_obj, TEMPORARY_AREA)){
            let bag = object::remove_field(&mut utxo_obj, TEMPORARY_AREA);
            bag::drop(bag);
        };
        let utxo = object::remove(utxo_obj);
        let UTXO{txid:_, vout:_, value:_, seals} = utxo;
        seals
    }

    // === UTXOSeal ===
    public fun new_utxo_seal(protocol: String, object_id: ObjectID) : UTXOSeal {
        UTXOSeal{
            protocol: protocol,
            object_id: object_id
        }
    }

    public fun unpack_utxo_seal(utxo_seal: UTXOSeal) : (String, ObjectID) {
        let UTXOSeal{protocol, object_id} = utxo_seal;
        (protocol, object_id)
    }

    // === SealOut ===
    public fun new_seal_out(output_index: u64, object_id: ObjectID) : SealOut {
        SealOut{
            output_index: output_index,
            object_id: object_id
        }
    }

    public fun unpack_seal_out(seal_out: SealOut) : (u64, ObjectID) {
        let SealOut{output_index, object_id} = seal_out;
        (output_index, object_id)
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
        let id = new_id(txid, vout);
        let object_id = object::custom_object_id<OutputID,UTXO>(id);
        //std::debug::print(&object_id);
        assert!(std::bcs::to_bytes(&object_id) == x"b8fc937bf3c15abe49c95fa6906aff29087149f542b48db0cf25dce671a68a63", 1);
    }

    #[test]
    fun test_remove(){
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