// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::address_mapping{
    
    use std::option::{Self, Option};
    use std::signer;
    use moveos_std::context::{Self, Context};
    use moveos_std::table::{Self, Table};
    use moveos_std::object::{Self, Object};
    use rooch_framework::hash::{blake2b256};
    use rooch_framework::multichain_address::{Self, MultiChainAddress};

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;
    friend rooch_framework::transfer;
    
    const ErrorMultiChainAddressInvalid: u64 = 1;

    struct AddressMapping has key{
        mapping: Table<MultiChainAddress, address>,
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer) {
        let mapping = context::new_table<MultiChainAddress, address>(ctx);
        let obj = context::new_named_object(ctx, AddressMapping{
            mapping,
        });
        object::transfer_extend(obj, @rooch_framework);
    }

    /// Borrow the address mapping object
    public fun borrow(ctx: &Context) : &Object<AddressMapping> {
        let object_id = object::named_object_id<AddressMapping>();
        context::borrow_object<AddressMapping>(ctx, object_id)
    }

    fun borrow_mut(ctx: &mut Context) : &mut Object<AddressMapping> {
        let object_id = object::named_object_id<AddressMapping>();
        context::borrow_mut_object_extend<AddressMapping>(ctx, object_id)
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
            generate_rooch_address(&maddress)
        }else{
            option::extract(&mut addr)
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
    
    fun generate_rooch_address(maddress: &MultiChainAddress): address {
        let hash = blake2b256(multichain_address::raw_address(maddress));
        moveos_std::bcs::to_address(hash)
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
        //TODO matienance the reverse mapping rooch_address -> vector<MultiChainAddress>
    }
   
}
