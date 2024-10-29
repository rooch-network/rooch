// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//TODO merge to rooch_framework::address_mapping.move
module rooch_nursery::ton_address_mapping {

    use std::option::{Self, Option};

    use moveos_std::object::{Self, Object};
    use moveos_std::tx_context;

    use rooch_framework::bitcoin_address;
    use rooch_nursery::ton_address::{TonAddress};
    use rooch_nursery::ton_proof::{Self, TonProof};

    const ErrorInvalidBindingProof: u64 = 1;
    const ErrorInvalidBindingAddress: u64 = 2;

    /// Mapping from rooch address to ton address
    /// The mapping record is the object field, key is the rooch address, value is the ton address
    struct RoochToTonAddressMapping has key{
        _placeholder: bool,
    }

    fun init(){
        let rooch_to_ton_mapping_id = object::named_object_id<RoochToTonAddressMapping>();
        if(!object::exists_object(rooch_to_ton_mapping_id)){
            let rooch_to_ton_mapping = object::new_named_object(RoochToTonAddressMapping{
                _placeholder: false
            });
            object::transfer_extend(rooch_to_ton_mapping, @rooch_nursery);
        };   
    }

    fun borrow_rooch_to_ton() : &Object<RoochToTonAddressMapping> {
        let object_id = object::named_object_id<RoochToTonAddressMapping>();
        object::borrow_object<RoochToTonAddressMapping>(object_id)
    }

    fun borrow_rooch_to_ton_mut() : &mut Object<RoochToTonAddressMapping> {
        let object_id = object::named_object_id<RoochToTonAddressMapping>();
        object::borrow_mut_object_extend<RoochToTonAddressMapping>(object_id)
    }

    public fun resolve_to_ton_address(sender: address): Option<TonAddress>{
        let rooch_to_ton_mapping = borrow_rooch_to_ton();
        if (object::contains_field(rooch_to_ton_mapping, sender)){
            option::some(*object::borrow_field(rooch_to_ton_mapping, sender))
        }else{
            option::none()
        }
    }

    public fun binding_ton_address(proof: TonProof, ton_address: TonAddress){
        let rooch_to_ton_mapping = borrow_rooch_to_ton_mut();
        assert!(ton_proof::verify_proof(&ton_address, &proof), ErrorInvalidBindingProof);
        let payload = ton_proof::payload(&proof);
        //The ton proof payload should be a Bitcoin address, the user wants to bing.
        let btc_addr = bitcoin_address::from_string(payload);
        let rooch_addr = bitcoin_address::to_rooch_address(&btc_addr);
        let sender = tx_context::sender();
        //The sender must be the owner of the Bitcoin address
        assert!(rooch_addr == sender, ErrorInvalidBindingAddress);
        object::add_field(rooch_to_ton_mapping, sender, ton_address);
    }

}