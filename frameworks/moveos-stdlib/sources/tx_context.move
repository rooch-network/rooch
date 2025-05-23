// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Origin source https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/tx_context.move#L24
// And do refactoring

module moveos_std::tx_context {
    use std::vector;
    use std::hash;
    use std::string::String;
    use std::option::{Self, Option};
    use moveos_std::bcs;
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::copyable_any::{Self, Any};
    use moveos_std::type_info;
    use moveos_std::tx_meta::{TxMeta};
    use moveos_std::tx_result::{TxResult};

    friend moveos_std::object;
    friend moveos_std::account;
    friend moveos_std::event;
    friend moveos_std::module_store;

    const ErrorInvalidContext: u64 = 1;
    const ErrorRepeatedContextKey: u64 = 2;
 
    /// Information about the transaction currently being executed.
    struct TxContext {
        /// The address of the user that signed the current transaction
        sender: address,
        /// Sequence number of this transaction corresponding to sender's account.
        sequence_number: u64,
        // The max gas to be used. 
        max_gas_amount: u64,
        /// Hash of the current transaction
        tx_hash: vector<u8>,
        /// Data size of this transaction
        tx_size: u64,
        /// Counter recording the number of fresh id's created while executing
        /// this transaction. Always 0 at the start of a transaction
        ids_created: u64,
        /// A Key-Value map that can be used to store context information
        map: SimpleMap<String, Any>,
    }

    // Used to indicate module upgrading in this tx and then 
    // setting mark_loader_cache_as_invalid() in VM, which announce to 
    // the VM that the code loading cache should be considered outdated. 
    struct ModuleUpgradeFlag has copy, drop, store {
        is_upgrade: bool,
    }

    /// Return the address of the user that signed the current transaction
    public fun sender(): address {
        borrow().sender
    }

    /// Return the sequence number of the current transaction
    public fun sequence_number(): u64 {
        borrow().sequence_number
    }

    /// Return the max gas to be used
    public fun max_gas_amount(): u64 {
        borrow().max_gas_amount
    } 

    /// Generate a new unique address,
    public fun fresh_address(): address {
        let ctx = borrow_mut();
        let addr = derive_id(ctx.tx_hash, ctx.ids_created);
        ctx.ids_created = ctx.ids_created + 1;
        addr
    }

    public(friend) fun derive_id(hash: vector<u8>, index: u64): address {
        let bytes = hash;
        vector::append(&mut bytes, bcs::to_bytes(&index));
        let id = hash::sha3_256(bytes);
        bcs::to_address(id)
    }

    /// Return the hash of the current transaction
    public fun tx_hash(): vector<u8> {
        borrow().tx_hash
    }

    /// Return the number of ids created by the current transaction.
    /// Hidden for now, but may expose later
    fun ids_created(): u64 {
        borrow().ids_created
    }

    /// Add a value to the context map
    fun add<T: drop + store + copy>(self: &mut TxContext, value: T) {
        let any = copyable_any::pack(value);
        let type_name = *copyable_any::type_name(&any);
        assert!(!simple_map::contains_key(&self.map, &type_name), ErrorRepeatedContextKey);
        simple_map::add(&mut self.map, type_name, any)
    }

    /// Add a value to the context map via system reserved address
    public fun add_attribute_via_system<T: drop + store + copy>(system: &signer, value: T){
        moveos_std::core_addresses::assert_system_reserved(system);
        let ctx = borrow_mut();
        add(ctx, value);
    } 

    /// Get a value from the context map
    fun get<T: drop + store + copy>(self: &TxContext): Option<T> {
        let type_name = type_info::type_name<T>();
        if (simple_map::contains_key(&self.map, &type_name)) {
            let any = simple_map::borrow(&self.map, &type_name);
            option::some(copyable_any::unpack(*any))   
        }else{
            option::none()
        }
    }

    /// Get attribute value from the context map
    public fun get_attribute<T: drop + store + copy>(): Option<T> {
        let ctx = borrow();
        get(ctx)
    }

    /// Check if the key is in the context map
    fun contains<T: drop + store + copy>(self: &TxContext): bool {
        let type_name = type_info::type_name<T>();
        simple_map::contains_key(&self.map, &type_name)
    }

    /// Check if the key is in the context map
    public fun contains_attribute<T: drop + store + copy>(): bool {
        let ctx = borrow();
        contains<T>(ctx)
    }

    /// Remove a value from the context map
    fun remove<T: drop + store + copy>(self: &mut TxContext) {
        let type_name = type_info::type_name<T>();
        simple_map::remove(&mut self.map, &type_name);
    }

    /// Get the transaction meta data
    /// The TxMeta is writed by the VM before the transaction execution.
    /// The meta data is only available when executing or validating a transaction, otherwise abort(eg. readonly function call).
    public fun tx_meta(): TxMeta {
        let ctx = borrow();
        let meta = get<TxMeta>(ctx);
        assert!(option::is_some(&meta), ErrorInvalidContext);
        option::extract(&mut meta)
    }

    /// Get the gas payment account of the transaction
    /// Currently, the gas payment account is the sender of the transaction.
    /// In the future, the gas payment account may be different from the sender.
    public fun tx_gas_payment_account(): address {
        let ctx = borrow();
        ctx.sender
    }

    /// The result is only available in the `post_execute` function.
    public fun tx_result(): TxResult {
        let ctx = borrow();
        let result = get<TxResult>(ctx);
        assert!(option::is_some(&result), ErrorInvalidContext);
        option::extract(&mut result)
    }

    /// Check if the current transaction is a system call
    /// The system call is a special transaction initiated by the system.
    public fun is_system_call(): bool {
        let sender = sender();
        moveos_std::core_addresses::is_vm_address(sender)
    }

    public(friend) fun set_module_upgrade_flag(is_upgrade: bool) {
        let ctx = borrow_mut();
        if(!contains<ModuleUpgradeFlag>(ctx)){
            add(ctx, ModuleUpgradeFlag{is_upgrade});
        }else{
            //If the flag is already set, means the module upgrade flag is set in the previous function call.
            //We only need to set the flag if is_upgrade is true.
            if(is_upgrade){
                let flag = get<ModuleUpgradeFlag>(ctx);
                assert!(option::is_some(&flag), ErrorInvalidContext);
                option::borrow_mut(&mut flag).is_upgrade = true;
            }
        }
        
    }

    public fun drop(self: TxContext){
        let TxContext {
            sender: _,
            sequence_number: _,
            max_gas_amount: _,
            tx_hash: _,
            tx_size: _,
            ids_created: _,
            map:_,
        } = self;
    }

    fun borrow(): &TxContext {
        Self::borrow_inner()
    }

    fun borrow_mut(): &mut TxContext {
        Self::borrow_mut_inner()
    }


    native fun borrow_inner(): &TxContext;
    native fun borrow_mut_inner(): &mut TxContext;

    #[test_only]
    /// set the TxContext sender for unit test
    public fun set_ctx_sender_for_testing(sender: address){
        let ctx = borrow_mut();
        ctx.sender = sender;
    }

    #[test_only]
    /// set the TxContext sequence_number for unit test
    public fun set_ctx_sequencer_number_for_testing(sequence_number: u64){
        let ctx = borrow_mut();
        ctx.sequence_number = sequence_number;
    }

    #[test_only]
    /// set the TxContext tx_hash for unit test
    public fun set_ctx_tx_hash_for_testing(tx_hash: vector<u8>){
        let ctx = borrow_mut();
        ctx.tx_hash = tx_hash;
    }

    #[test_only]
    /// Set an attribute value in the context map for testing
    public fun set_attribute_for_testing<T: drop + store + copy>(value: T) {
        let ctx = borrow_mut();
        add(ctx, value);
    }

    #[test_only]
    public fun remove_attribute_for_testing<T: drop + store + copy>() {
        let ctx = borrow_mut();
        remove<T>(ctx);
    }

    #[test_only]
    public fun fresh_address_for_testing(): address {
        fresh_address()
    }


    #[test_only]
    struct TestValue has store, drop, copy{
        value: u64,
    }

    #[test]
    fun test_attributes() {
        let ctx = borrow_mut();
        let value = TestValue{value: 42};
        add(ctx, value);
        let value2 = get<TestValue>(ctx);
        assert!(value == option::extract(&mut value2), 1000);
    }

    #[test(sender=@0x42)]
    fun test_fresh_address() {
        let addr1 = fresh_address();
        let addr2 = fresh_address();
        assert!(addr1 != addr2, 1000);
    }
}
