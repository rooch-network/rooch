// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::address_mapping{
    
    use std::option::{Self, Option};
    use moveos_std::core_addresses;
    use moveos_std::object::{Self, Object};
    use rooch_framework::multichain_address::{Self, MultiChainAddress};
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};

    friend rooch_framework::genesis;
    friend rooch_framework::bitcoin_validator;
    friend rooch_framework::transaction_validator;
    
    const ErrorMultiChainAddressInvalid: u64 = 1;
    const ErrorUnsupportedAddress: u64 = 2;

    const NAMED_MAPPING_INDEX: u64 = 0;
    const NAMED_REVERSE_MAPPING_INDEX: u64 = 1;

    /// Mapping from multi-chain address to rooch address
    /// Not including Bitcoin address, because Bitcoin address can directly hash to rooch address
    /// The mapping record is the object field, key is the multi-chain address, value is the rooch address
    struct MultiChainAddressMapping has key{
        _placeholder: bool,
    }
    
    /// Mapping from rooch address to bitcoin address, other chain can use new table
    /// The mapping record is the object field, key is the rooch address, value is the Bitcoin address
    struct RoochToBitcoinAddressMapping has key{
        _placeholder: bool,
    }

    public(friend) fun genesis_init(_genesis_account: &signer) {
        let multichain_mapping = object::new_named_object(MultiChainAddressMapping{
            _placeholder: false
        });
        let rooch_to_bitcoin_mapping = object::new_named_object(RoochToBitcoinAddressMapping{
            _placeholder: false
        });
        object::transfer_extend(multichain_mapping, @rooch_framework);
        object::transfer_extend(rooch_to_bitcoin_mapping, @rooch_framework);
    }

    fun borrow_multichain() : &Object<MultiChainAddressMapping> {
        let object_id = object::named_object_id<MultiChainAddressMapping>();
        object::borrow_object<MultiChainAddressMapping>(object_id)
    }

    fun borrow_multichain_mut() : &mut Object<MultiChainAddressMapping> {
        let object_id = object::named_object_id<MultiChainAddressMapping>();
        object::borrow_mut_object_extend<MultiChainAddressMapping>(object_id)
    }

    fun borrow_rooch_to_bitcoin() : &Object<RoochToBitcoinAddressMapping> {
        let object_id = object::named_object_id<RoochToBitcoinAddressMapping>();
        object::borrow_object<RoochToBitcoinAddressMapping>(object_id)
    }

    fun borrow_rooch_to_bitcoin_mut() : &mut Object<RoochToBitcoinAddressMapping> {
        let object_id = object::named_object_id<RoochToBitcoinAddressMapping>();
        object::borrow_mut_object_extend<RoochToBitcoinAddressMapping>(object_id)
    }

    fun resolve_address(obj: &Object<MultiChainAddressMapping>, maddress: MultiChainAddress): Option<address> {
        if (multichain_address::is_rooch_address(&maddress)) {
            return option::some(multichain_address::into_rooch_address(maddress))
        };
        if (multichain_address::is_bitcoin_address(&maddress)) {
            return option::some(bitcoin_address::to_rooch_address(&multichain_address::into_bitcoin_address(maddress)))
        };

        if(object::contains_field(obj, maddress)){
            let addr = object::borrow_field(obj, maddress);
            option::some(*addr)
        }else{
            option::none()
        }
    }

    fun resolve_bitcoin_address(obj: &Object<RoochToBitcoinAddressMapping>, rooch_address: address): Option<BitcoinAddress> {
        if(object::contains_field(obj, rooch_address)){
            let addr = object::borrow_field(obj, rooch_address);
            option::some(*addr)
        }else{
            option::none()
        }
    }

    fun exists_mapping_address(obj: &Object<MultiChainAddressMapping>, maddress: MultiChainAddress): bool {
        if (multichain_address::is_rooch_address(&maddress) || multichain_address::is_bitcoin_address(&maddress)) {
            return true
        };
        object::contains_field(obj, maddress)
    }

    /// Resolve a multi-chain address to a rooch address
    public fun resolve(maddress: MultiChainAddress): Option<address> {
        let am = Self::borrow_multichain();
        Self::resolve_address(am, maddress)
    }

    /// Resolve a rooch address to a bitcoin address
    public fun resolve_bitcoin(rooch_address: address): Option<BitcoinAddress> {
        let am = Self::borrow_rooch_to_bitcoin();
        Self::resolve_bitcoin_address(am, rooch_address)
    }

    /// Generate a rooch address via bitcoin multi-chain address
    /// This function will deprecated in the future, client should directly generate rooch address via bitcoin address.
    public fun resolve_or_generate(maddress: MultiChainAddress): address {
        if (multichain_address::is_rooch_address(&maddress)) {
            return multichain_address::into_rooch_address(maddress)
        };
        if (multichain_address::is_bitcoin_address(&maddress)) {
            return bitcoin_address::to_rooch_address(&multichain_address::into_bitcoin_address(maddress))
        };
        abort ErrorUnsupportedAddress
    }

    /// Check if a multi-chain address is bound to a rooch address
    public fun exists_mapping(maddress: MultiChainAddress): bool {
        let obj = Self::borrow_multichain();
        Self::exists_mapping_address(obj, maddress)
    }

    public(friend) fun bind_bitcoin_address(rooch_address: address, baddress: BitcoinAddress) {
        // bitcoin address to rooch address do not need to record, we just record rooch address to bitcoin address
        let obj = Self::borrow_rooch_to_bitcoin_mut();
        if(!object::contains_field(obj, rooch_address)){
            object::add_field(obj, rooch_address, baddress);
        }
    }

    public fun bind_bitcoin_address_by_system(system: &signer, rooch_address: address, baddress: BitcoinAddress) {
        core_addresses::assert_system_reserved(system);
        Self::bind_bitcoin_address(rooch_address, baddress);
    }

}
