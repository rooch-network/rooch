// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::utxo{
    use std::vector;
    use std::string::String;
    use moveos_std::object_id;
    use moveos_std::object_id::ObjectID;
    use moveos_std::object::{Self, Object};
    use moveos_std::simple_multimap::{Self, SimpleMultiMap};
    use moveos_std::type_info;

    friend bitcoin_move::light_client;

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

    public(friend) fun new(txid: address, vout: u32, value: u64) : Object<UTXO> {
        let id = OutputID{
            txid: txid,
            vout: vout,
        };
        let utxo = UTXO{
            txid: txid,
            vout: vout,
            value: value,
            seals: simple_multimap::new(),
        };
        object::new_custom_object(id, utxo)
    }

    public fun new_id(txid: address, vout: u32) : OutputID {
        OutputID{
            txid: txid,
            vout: vout,
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
            txid: txid,
            vout: vout,
        };
        let object_id = object_id::custom_object_id<OutputID,UTXO>(id);
        object::exists_object_with_type<UTXO>(object_id)
    }

    public fun borrow_utxo(txid: address, vout: u32): &Object<UTXO>{
        let id = OutputID{
            txid: txid,
            vout: vout,
        };
        let object_id = object_id::custom_object_id<OutputID,UTXO>(id);
        object::borrow_object(object_id)
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

    #[test]
    fun test_id(){
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let vout = 0;
        let id = new_id(txid, vout);
        let object_id = object_id::custom_object_id<OutputID,UTXO>(id);
        //std::debug::print(&object_id);
        assert!(std::bcs::to_bytes(&object_id) == x"b8fc937bf3c15abe49c95fa6906aff29087149f542b48db0cf25dce671a68a63", 1);
    }
}