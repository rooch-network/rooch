// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::utxo{
    use std::vector;
    use std::string::String;
    use std::option::{Self, Option};
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::simple_multimap::{Self, SimpleMultiMap};
    use moveos_std::type_info;
    use moveos_std::event_queue;
    use moveos_std::address::to_string;
    use bitcoin_move::types::{Self, OutPoint};
    use bitcoin_move::temp_state;

    friend bitcoin_move::genesis;
    friend bitcoin_move::ord;
    friend bitcoin_move::bitcoin;
    friend bitcoin_move::inscription_updater;

    const TEMPORARY_AREA: vector<u8> = b"temporary_area";

    const ErrorDeprecatedFunction: u64 = 1;

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
        vout: u32,
        seal: UTXOSeal,
    }

    /// Event emitted when a UTXO is spent
    /// In the Bitcoin UTXO model, there's no inherent concept of sender and receiver.
    /// However, for simplifying payment scenarios, we define sender and receiver as follows:
    /// - Sender: The address of the first input UTXO that can be identified
    /// - Receiver: The address of each output UTXO that can be identified
    struct SpendUTXOEvent has drop, store, copy {
        txid: address,
        sender: address,
        receiver: Option<address>,
        value: u64
    }

    /// Event emitted when a UTXO is received
    /// In the Bitcoin UTXO model, there's no inherent concept of sender and receiver.
    /// However, for simplifying payment scenarios, we define sender and receiver as follows:
    /// - Sender: The address of the first input UTXO that can be identified
    /// - Receiver: The address of each output UTXO that can be identified
    struct ReceiveUTXOEvent has drop, store, copy {
        txid: address,
        sender: Option<address>,
        receiver: address,
        value: u64
    }

    /// Event emitted when the temporary state of a UTXO is dropped
    /// The temporary state is dropped when the UTXO is spent
    /// The event is onchain event, and the event_queue name is type_name of the temporary state
    struct TempStateDropEvent has drop, store, copy {
        utxo_obj_id: ObjectID,
        /// The outpoint of the UTXO
        outpoint: OutPoint,
        /// The value of the UTXO
        value: u64,
    } 

    struct BitcoinUTXOStore has key{
    }

    public(friend) fun genesis_init(){
        let btc_utxo_store = BitcoinUTXOStore{
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
        let utxo_obj = object::new_with_parent_and_id(uxto_store, id, utxo);
        utxo_obj
    }

    public(friend) fun mock_utxo(outpoint: OutPoint, value: u64): UTXO {
        let (txid, vout) = types::unpack_outpoint(outpoint);
        UTXO{
            txid,
            vout,
            value,
            seals: simple_multimap::new(),
        }
    }

    public fun derive_utxo_id(outpoint: OutPoint) : ObjectID {
        let parent_id = object::named_object_id<BitcoinUTXOStore>();
        object::custom_object_id_with_parent<OutPoint, UTXO>(parent_id, outpoint)
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

    public(friend) fun remove_seals_internal<T>(utxo: &mut UTXO): vector<ObjectID>{
        let protocol = type_info::type_name<T>();
        if(simple_multimap::contains_key(&utxo.seals, &protocol)){
            let(_k, value) = simple_multimap::remove(&mut utxo.seals, &protocol);
            value
        }else{
            vector::empty()
        }
    }

    public(friend) fun add_seal_internal(utxo: &mut UTXO, utxo_seal: UTXOSeal){
        let UTXOSeal{protocol, object_id} = utxo_seal;
        simple_multimap::add(&mut utxo.seals, protocol, object_id);
    }

    // === Object<UTXO> ===    

    public(friend) fun transfer(utxo_obj: Object<UTXO>, sender: Option<address>, receiver: address){
        let utxo = object::borrow(&utxo_obj);
        let value = utxo.value;
        let txid = utxo.txid;
        object::transfer_extend(utxo_obj, receiver);

        if (option::is_some(&sender) && option::borrow(&sender) == &receiver){
            return
        };
        if (receiver != @bitcoin_move){
            event_queue::emit(to_string(&receiver), ReceiveUTXOEvent {
                txid,
                sender,
                receiver,
                value
            });
        };
        if (option::is_some(&sender)){
            let sender_address = option::destroy_some(sender);
            if (sender_address != @bitcoin_move){
                let receiver = if (receiver == @bitcoin_move) {
                    option::none()
                } else {
                    option::some(receiver)
                };
                event_queue::emit(to_string(&sender_address), SpendUTXOEvent {
                    txid,
                    sender: sender_address,
                    receiver,
                    value
                });
            }
        };
    }

    public(friend) fun take(object_id: ObjectID): Object<UTXO>{
        object::take_object_extend<UTXO>(object_id)
    }

    public(friend) fun remove(utxo_obj: Object<UTXO>): UTXO{
        drop_temp_area(&mut utxo_obj);
        object::remove(utxo_obj)
    }

    fun drop_temp_area(utxo_obj: &mut Object<UTXO>){
        if(object::contains_field(utxo_obj, TEMPORARY_AREA)){
            let utxo_obj_id = object::id(utxo_obj);
            let utxo = object::borrow(utxo_obj);
            let value = utxo.value;
            let outpoint = types::new_outpoint(utxo.txid, utxo.vout);
            let temp_state = object::remove_field(utxo_obj, TEMPORARY_AREA);
            let state_type_names = temp_state::remove(temp_state);
            let idx = 0;
            let len = vector::length(&state_type_names);
            while(idx < len){
                let state_type_name = vector::pop_back(&mut state_type_names);
                event_queue::emit(state_type_name, TempStateDropEvent{
                    utxo_obj_id,
                    outpoint,
                    value,
                });
                idx = idx + 1;
            };
        }
    }

    public(friend) fun drop(utxo: UTXO) {
        let UTXO{txid:_, vout:_, value:_, seals} = utxo;
        simple_multimap::destroy_empty(seals);
    }

    // === UTXOSeal ===
    public(friend) fun new_utxo_seal(protocol: String, seal_object_id: ObjectID) : UTXOSeal {
        UTXOSeal{
            protocol,
            object_id: seal_object_id,
        }
    }

    public(friend) fun unpack_utxo_seal(utxo_seal: UTXOSeal) : (String, ObjectID) {
        let UTXOSeal{protocol, object_id} = utxo_seal;
        (protocol, object_id)
    }

    public(friend) fun new_seal_out(vout: u32, seal: UTXOSeal) : SealOut {
        SealOut{
            vout,
            seal,
        }
    }

    public(friend) fun unpack_seal_out(seal_out: SealOut) : (u32, UTXOSeal) {
        let SealOut{vout, seal} = seal_out;
        (vout, seal)
    } 

    // ==== Temporary Area ===

    #[private_generics(S)]
    public fun add_temp_state<S: store + drop>(utxo: &mut Object<UTXO>, state: S){
        if(object::contains_field(utxo, TEMPORARY_AREA)){
            let temp_state = object::borrow_mut_field(utxo, TEMPORARY_AREA);
            temp_state::add_state(temp_state, state);
        }else{
            let temp_state = temp_state::new();
            temp_state::add_state(&mut temp_state, state);
            object::add_field(utxo, TEMPORARY_AREA, temp_state);
        }
    }

    public fun contains_temp_state<S: store + drop>(utxo: &Object<UTXO>) : bool {
        if(object::contains_field(utxo, TEMPORARY_AREA)){
            let temp_state = object::borrow_field(utxo, TEMPORARY_AREA);
            temp_state::contains_state<S>(temp_state)
        }else{
            false
        }
    }

    public fun borrow_temp_state<S: store + drop>(utxo: &Object<UTXO>) : &S {
        let temp_state = object::borrow_field(utxo, TEMPORARY_AREA);
        temp_state::borrow_state(temp_state)
    }

    #[private_generics(S)]
    public fun borrow_mut_temp_state<S: store + drop>(utxo: &mut Object<UTXO>) : &mut S {
        let temp_state = object::borrow_mut_field(utxo, TEMPORARY_AREA);
        temp_state::borrow_mut_state(temp_state)
    }

    #[private_generics(S)]
    public fun remove_temp_state<S: store + drop>(utxo: &mut Object<UTXO>) : S {
        let temp_state = object::borrow_mut_field(utxo, TEMPORARY_AREA);
        temp_state::remove_state(temp_state)
    }

    // Should we require the input utxo exists
    // Sometimes, we may not sync the Bitcoin block from genesis
    public(friend) fun check_utxo_input(): bool{
        //TODO make this to be configurable
        rooch_framework::chain_id::is_main()
    }

    public fun unpack_spend_utxo_event(event: SpendUTXOEvent): (address, address, Option<address>, u64) {
        let SpendUTXOEvent { txid, sender, receiver, value } = event;
        (txid, sender, receiver, value)
    }

    public fun unpack_receive_utxo_event(event: ReceiveUTXOEvent): (address, Option<address>, address, u64) {
        let ReceiveUTXOEvent { txid, sender, receiver, value } = event;
        (txid, sender, receiver, value)
    }

    public fun unpack_temp_state_drop_event(event: TempStateDropEvent): (ObjectID, OutPoint, u64) {
        let TempStateDropEvent { utxo_obj_id, outpoint, value } = event;
        (utxo_obj_id, outpoint, value)
    }

    #[test_only]
    public fun new_for_testing(txid: address, vout: u32, value: u64) : Object<UTXO> {
        new(txid, vout, value)
    }

    #[test_only]
    public fun drop_for_testing(utxo_obj: Object<UTXO>){
        let utxo = remove(utxo_obj);
        drop(utxo);
    }

    #[test]
    fun test_id(){
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let vout = 0;
        let object_id = derive_utxo_id(types::new_outpoint(txid, vout));
        std::debug::print(&std::bcs::to_bytes(&object_id));
        assert!(std::bcs::to_bytes(&object_id) == x"02f74d177bfec2d8de0c4893f6502d3e5b55f12f75e158d53b035dcbe33782ef166056a4a7b33326d5fb811c95b39cbca0743662e14fa3b904c41fa07d4b5c3956", 1);
    }

    #[test]
    fun test_remove(){
        genesis_init();
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let vout = 0;
        let utxo_obj = new(txid, vout, 100);
        let utxo = remove(utxo_obj);
        drop(utxo);
    }

    #[test_only]
    struct TestTempState has store, copy, drop {
        value: u64,
    }

    #[test]
    fun test_temporary_area(){
        genesis_init();
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let vout = 0;
        let value = 100;
        let utxo_obj = new(txid, vout, value);
        add_temp_state(&mut utxo_obj, TestTempState{value: 10});
        assert!(contains_temp_state<TestTempState>(&utxo_obj), 1000);
        assert!(borrow_temp_state<TestTempState>(&utxo_obj).value == 10, 1001);
        {
            let state = borrow_mut_temp_state<TestTempState>(&mut utxo_obj);
            state.value = 20;
        };
        let state = remove_temp_state<TestTempState>(&mut utxo_obj);
        assert!(state.value == 20, 1);
        assert!(!contains_temp_state<TestTempState>(&utxo_obj), 1002);
        let utxo = remove(utxo_obj);
        drop(utxo);
    }

    #[test]
    fun test_temp_state_drop_event(){
        genesis_init();
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let vout = 0;
        let value = 100;
        let utxo_obj = new(txid, vout, value);
        let utxo_obj_id = object::id(&utxo_obj);
        let outpoint = types::new_outpoint(txid, vout);
        let subscriber = event_queue::subscribe<TempStateDropEvent>(type_info::type_name<TestTempState>());
        add_temp_state(&mut utxo_obj, TestTempState{value: 10});
        let utxo = remove(utxo_obj);
        

        let event_option = event_queue::consume<TempStateDropEvent>(&mut subscriber);
        assert!(option::is_some(&event_option), 2);
        let event = option::destroy_some(event_option);
        assert!(event.utxo_obj_id == utxo_obj_id, 3);
        assert!(event.outpoint == outpoint, 4);
        assert!(event.value == value, 5);
        drop(utxo);
        event_queue::unsubscribe(subscriber);
    }
}