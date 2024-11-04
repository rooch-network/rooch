// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::address_mapping{
    
    use std::option::{Self, Option};
    use std::string::{Self, String};
    use moveos_std::core_addresses;
    use moveos_std::object::{Self, Object};
    use moveos_std::tx_context; 
    use rooch_framework::multichain_address::{Self, MultiChainAddress};
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};
    use rooch_framework::ton_address::{Self, TonAddress};
    use rooch_framework::ton_proof::{Self, TonProofData};

    friend rooch_framework::genesis;
    friend rooch_framework::bitcoin_validator;
    friend rooch_framework::transaction_validator;
    friend rooch_framework::transfer;
    
    const ErrorMultiChainAddressInvalid: u64 = 1;
    const ErrorUnsupportedAddress: u64 = 2;
    const ErrorInvalidBindingProof: u64 = 3;
    const ErrorInvalidBindingAddress: u64 = 4;

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

    /// Mapping from rooch address to ton address
    /// The mapping record is the object field, key is the rooch address, value is the ton address
    struct RoochToTonAddressMapping has key{
        _placeholder: bool,
    }

    public(friend) fun genesis_init() {
        let multichain_mapping_id = object::named_object_id<MultiChainAddressMapping>();
        if(!object::exists_object(multichain_mapping_id)){
            let multichain_mapping = object::new_named_object(MultiChainAddressMapping{
                _placeholder: false
            });
            object::transfer_extend(multichain_mapping, @rooch_framework);
        };
        let rooch_to_bitcoin_mapping_id = object::named_object_id<RoochToBitcoinAddressMapping>();
        if(!object::exists_object(rooch_to_bitcoin_mapping_id)){
            let rooch_to_bitcoin_mapping = object::new_named_object(RoochToBitcoinAddressMapping{
                _placeholder: false
            });
            object::transfer_extend(rooch_to_bitcoin_mapping, @rooch_framework);
        };
        Self::init_ton_mapping();
    }

    public entry fun init_ton_mapping(){
        let rooch_to_ton_mapping_id = object::named_object_id<RoochToTonAddressMapping>();
        if(!object::exists_object(rooch_to_ton_mapping_id)){
            let rooch_to_ton_mapping = object::new_named_object(RoochToTonAddressMapping{
                _placeholder: false
            });
            object::transfer_extend(rooch_to_ton_mapping, @rooch_framework);
        };   
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

    fun borrow_rooch_to_ton() : &Object<RoochToTonAddressMapping> {
        let object_id = object::named_object_id<RoochToTonAddressMapping>();
        object::borrow_object<RoochToTonAddressMapping>(object_id)
    }

    fun borrow_rooch_to_ton_mut() : &mut Object<RoochToTonAddressMapping> {
        let object_id = object::named_object_id<RoochToTonAddressMapping>();
        object::borrow_mut_object_extend<RoochToTonAddressMapping>(object_id)
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

    /// Check if a multi-chain address is bound to a rooch address
    public fun exists_mapping(maddress: MultiChainAddress): bool {
        let obj = Self::borrow_multichain();
        Self::exists_mapping_address(obj, maddress)
    }

    public(friend) fun bind_bitcoin_address_internal(rooch_address: address, btc_address: BitcoinAddress) {
        // bitcoin address to rooch address do not need to record, we just record rooch address to bitcoin address
        let obj = Self::borrow_rooch_to_bitcoin_mut();
        if(!object::contains_field(obj, rooch_address)){
            object::add_field(obj, rooch_address, btc_address);
        }
    }

    public fun bind_bitcoin_address_by_system(system: &signer, rooch_address: address, btc_address: BitcoinAddress) {
        core_addresses::assert_system_reserved(system);
        Self::bind_bitcoin_address_internal(rooch_address, btc_address);
    }

    /// Bind a bitcoin address to a rooch address
    /// We can calculate the rooch address from bitcoin address
    /// So we call this function for record rooch address to bitcoin address mapping
    public fun bind_bitcoin_address(btc_address: BitcoinAddress){
        let rooch_addr = bitcoin_address::to_rooch_address(&btc_address);
        Self::bind_bitcoin_address_internal(rooch_addr, btc_address);
    }

    // ============================== Ton address mapping ==============================

    public fun resolve_to_ton_address(sender: address): Option<TonAddress>{
        let rooch_to_ton_mapping = borrow_rooch_to_ton();
        if (object::contains_field(rooch_to_ton_mapping, sender)){
            option::some(*object::borrow_field(rooch_to_ton_mapping, sender))
        }else{
            option::none()
        }
    }

    public fun resolve_via_ton_address(ton_address: TonAddress): Option<address>{
        let maddress = multichain_address::from_ton(ton_address);
        Self::resolve(maddress)
    }

    public fun resolve_via_ton_address_str(ton_address_str: String): Option<address>{
        let ton_address = ton_address::from_string(&ton_address_str);
        Self::resolve_via_ton_address(ton_address)
    }

    /// Bind a ton address to a rooch address
    /// The user needs to provide a valid ton proof and the ton address he wants to bind
    public fun bind_ton_address(proof_data: TonProofData, ton_address: TonAddress){
        assert!(ton_proof::verify_proof(&ton_address, &proof_data), ErrorInvalidBindingProof);
        let proof = ton_proof::proof(&proof_data);
        let btc_addr_str = ton_proof::payload_bitcoin_address(proof);
        assert!(string::length(&btc_addr_str) > 0, ErrorInvalidBindingProof);
        //The ton proof payload should be a Bitcoin address, the user wants to bing.
        let btc_addr = bitcoin_address::from_string(&btc_addr_str);
        let rooch_addr = bitcoin_address::to_rooch_address(&btc_addr);
        let sender = tx_context::sender();
        //The sender must be the owner of the Bitcoin address
        assert!(rooch_addr == sender, ErrorInvalidBindingAddress);
        Self::bind_ton_address_internal(sender, ton_address);
    }

    fun bind_ton_address_internal(addr: address, ton_address: TonAddress){
        let rooch_to_ton_mapping = borrow_rooch_to_ton_mut();
        object::add_field(rooch_to_ton_mapping, addr, ton_address);

        let multichain_mapping = Self::borrow_multichain_mut();
        let maddress = multichain_address::from_ton(ton_address);
        object::add_field(multichain_mapping, maddress, addr);
    }

    public fun bind_ton_address_entry(proof_data_bytes: vector<u8>, ton_address_str: String){
        let ton_address = ton_address::from_string(&ton_address_str);
        let proof_data = ton_proof::decode_proof_data(proof_data_bytes);
        Self::bind_ton_address(proof_data, ton_address);
    }

    #[test]
    fun test_address_mapping_for_bitcoin(){
        genesis_init();
        let btc_addr = bitcoin_address::from_string(&string::utf8(b"bc1p8xpjpkc9uzj2dexcxjg9sw8lxje85xa4070zpcys589e3rf6k20qm6gjrt"));
        bind_bitcoin_address(btc_addr);
        let rooch_addr = bitcoin_address::to_rooch_address(&btc_addr);
        let resolved_addr = resolve_bitcoin(rooch_addr);
        assert!(resolved_addr == option::some(btc_addr), 1);
    }

    #[test]
    fun test_address_mapping_for_ton(){
        genesis_init();
        let addr_str = string::utf8(b"0:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76");
        let ton_addr = ton_address::from_hex_str(&addr_str);
        let sender = @0x42;
        bind_ton_address_internal(sender, ton_addr);
        let resolved_addr = resolve_via_ton_address(ton_addr);
        assert!(resolved_addr == option::some(sender), 1);
    }
}
