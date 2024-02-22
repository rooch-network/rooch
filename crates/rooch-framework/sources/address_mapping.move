// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::address_mapping{
    
    use std::option::{Self, Option};
    use std::signer;
    use std::vector;
    use moveos_std::core_addresses;
    use moveos_std::object_id::ObjectID;
    use moveos_std::object_id;
    use moveos_std::bcs;
    use moveos_std::context::{Self, Context};
    use moveos_std::table::{Self, Table};
    use moveos_std::object::{Self, Object};
    use rooch_framework::multichain_address::{Self, MultiChainAddress};

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;
    friend rooch_framework::transfer;
    
    const ErrorMultiChainAddressInvalid: u64 = 1;

    struct AddressMapping has key{
        mapping: Table<MultiChainAddress, address>,
        reverse_mapping: Table<address, vector<MultiChainAddress>>,
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer) {
        let mapping = context::new_table<MultiChainAddress, address>(ctx);
        let reverse_mapping = context::new_table<address, vector<MultiChainAddress>>(ctx);
        let obj = context::new_named_object(ctx, AddressMapping{
            mapping,
            reverse_mapping
        });
        object::transfer_extend(obj, @rooch_framework);
    }

    /// Return AddressMapping table handle, including mapping and reverse_mapping table handle
    public fun address_mapping_handle(_ctx: &Context): (ObjectID, ObjectID, ObjectID) {
        let object_id = object_id::named_object_id<AddressMapping>();
        let address_mapping_obj = object::borrow_object<AddressMapping>(object_id);
        let address_mapping = object::borrow<AddressMapping>(address_mapping_obj);
        (object_id, *table::handle(&address_mapping.mapping), *table::handle(&address_mapping.reverse_mapping))
    }

    /// Borrow the address mapping object
    public fun borrow(_ctx: &Context) : &Object<AddressMapping> {
        let object_id = object_id::named_object_id<AddressMapping>();
        object::borrow_object<AddressMapping>(object_id)
    }

    fun borrow_mut(_ctx: &mut Context) : &mut Object<AddressMapping> {
        let object_id = object_id::named_object_id<AddressMapping>();
        object::borrow_mut_object_extend<AddressMapping>(object_id)
    }

    public fun resolve_address(obj: &Object<AddressMapping>, maddress: MultiChainAddress): Option<address> {
        let am = object::borrow(obj);
        if (multichain_address::is_rooch_address(&maddress)) {
            return option::some(multichain_address::into_rooch_address(maddress))
        };
        if(table::contains(&am.mapping, maddress)){
            let addr = table::borrow(&am.mapping, maddress);
            option::some(*addr)
        }else{
            option::none()
        }
    }

    public fun resolve_or_generate_address(obj: &Object<AddressMapping>, maddress: MultiChainAddress): address {
        let addr = resolve_address(obj, maddress);
        if(option::is_none(&addr)){
            multichain_address::mapping_to_rooch_address(maddress)
        }else{
            option::extract(&mut addr)
        }
    }

    /// Return the first multi chain address for the rooch address
    public fun reverse_resolve_address(obj: &Object<AddressMapping>, rooch_address: address): Option<MultiChainAddress> {
        let am = object::borrow(obj);
        if(table::contains(&am.reverse_mapping, rooch_address)){
            let maddresses = table::borrow(&am.reverse_mapping, rooch_address);
            if (!vector::is_empty(maddresses)) {
                option::some(*vector::borrow(maddresses, 0))
            } else {
                option::none()
            }
        }else{
            option::none()
        }
    }

    /// Return the first multi chain address for the rooch address with the same multichain id
    public fun reverse_resolve_address_with_multichain_id(obj: &Object<AddressMapping>, rooch_address: address, multichain_id: u64): Option<MultiChainAddress> {
        let am = object::borrow(obj);
        if (multichain_id == multichain_address::multichain_id_rooch()) {
            let raw_address = bcs::to_bytes(&rooch_address);
            return option::some(multichain_address::new(multichain_id, raw_address))
        };
        if(table::contains(&am.reverse_mapping, rooch_address)){
            let maddresses = table::borrow(&am.reverse_mapping, rooch_address);
            let (exist, first_index) = vector::find(maddresses, |v| multichain_address::multichain_id(v) == multichain_id);
            if (exist) {
                option::some(*vector::borrow(maddresses, first_index))
            } else {
                option::none()
            }
        }else{
            option::none()
        }
    }

    public fun exists_mapping_address(obj: &Object<AddressMapping>, maddress: MultiChainAddress): bool {
        if (multichain_address::is_rooch_address(&maddress)) {
            return true
        };
        let am = object::borrow(obj);
        table::contains(&am.mapping, maddress)
    }

    /// Resolve a multi-chain address to a rooch address
    public fun resolve(ctx: &Context, maddress: MultiChainAddress): Option<address> {
        let am = Self::borrow(ctx);
        Self::resolve_address(am, maddress)
    }

    /// Resolve a multi-chain address to a rooch address, if not exists, generate a new rooch address
    public fun resolve_or_generate(ctx: &Context, maddress: MultiChainAddress): address {
        let am = Self::borrow(ctx);
        Self::resolve_or_generate_address(am, maddress)
    }

    /// Check if a multi-chain address is bound to a rooch address
    public fun exists_mapping(ctx: &Context, maddress: MultiChainAddress): bool {
        let obj = Self::borrow(ctx);
        Self::exists_mapping_address(obj, maddress)
    }

    /// Bind a multi-chain address to the sender's rooch address
    /// The caller need to ensure the relationship between the multi-chain address and the rooch address
    public fun bind(ctx: &mut Context, sender: &signer, maddress: MultiChainAddress) {
        bind_no_check(ctx, signer::address_of(sender), maddress);
    }

    /// Bind a multi-chain address to the rooch address
    /// Called by system
    public fun bind_by_system(ctx: &mut Context, system: &signer, rooch_address: address, maddress: MultiChainAddress) {
        core_addresses::assert_system_reserved(system);
        bind_no_check(ctx, rooch_address, maddress);
    }

    /// Bind a rooch address to a multi-chain address
    public(friend) fun bind_no_check(ctx: &mut Context, rooch_address: address, maddress: MultiChainAddress) {
        if(multichain_address::is_rooch_address(&maddress)){
            assert!(
                multichain_address::into_rooch_address(maddress) == rooch_address, 
                ErrorMultiChainAddressInvalid
            );
        };
        let obj = Self::borrow_mut(ctx);
        let am = object::borrow_mut(obj);
        table::add(&mut am.mapping, maddress, rooch_address);
        // maintenance the reverse mapping rooch_address -> vector<MultiChainAddress>
        let maddresses = table::borrow_mut_with_default(&mut am.reverse_mapping, rooch_address, vector[]);
        vector::push_back(maddresses, maddress);
    }
   
}
