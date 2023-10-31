// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Context is part of the StorageAbstraction
/// It is used to provide a context for the storage operations, make the storage abstraction, 
/// and let developers customize the storage
module moveos_std::context {

    use std::option::Option;
    use std::error;
    use moveos_std::storage_context::{StorageContext};
    use moveos_std::tx_context::{Self, TxContext};
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::tx_meta::{TxMeta};
    use moveos_std::tx_result::{TxResult};
    use moveos_std::signer;

    friend moveos_std::table;
    friend moveos_std::type_table;
    friend moveos_std::account_storage;
    friend moveos_std::event;

    const ErrorObjectOwnerNotMatch: u64 = 1;

    /// Information about the global context include TxContext and StorageContext
    /// We can not put the StorageContext to TxContext, because object module depends on tx_context module,
    /// and storage_context module depends on object module.
    /// We put both TxContext and StorageContext to Context, for convenience of developers.
    /// The Context can not be `drop` or `store`, so developers need to pass the `&Context` or `&mut Context` to the `entry` function.
    struct Context {
        tx_context: TxContext,
        /// The Global Object Storage
        storage_context: StorageContext,
    }

    /// Get an immutable reference to the transaction context from the storage context
    public(friend) fun tx_context(self: &Context): &TxContext {
        &self.tx_context
    }

    /// Get a mutable reference to the transaction context from the storage context
    public(friend) fun tx_context_mut(self: &mut Context): &mut TxContext {
        &mut self.tx_context
    }

    // Wrap functions for TxContext

    /// Return the address of the user that signed the current transaction
    public fun sender(self: &Context): address {
        tx_context::sender(&self.tx_context)
    } 

    /// Return the sequence number of the current transaction
    public fun sequence_number(self: &Context): u64 {
        tx_context::sequence_number(&self.tx_context)
    }

    /// Return the maximum gas amount that can be used by the current transaction
    public fun max_gas_amount(self: &Context): u64 {
        tx_context::max_gas_amount(&self.tx_context)
    }

    /// Generate a new unique address
    public fun fresh_address(self: &mut Context): address {
        tx_context::fresh_address(&mut self.tx_context)
    }

    /// Generate a new unique object ID
    public fun fresh_object_id(self: &mut Context): ObjectID {
        object::address_to_object_id(tx_context::fresh_address(&mut self.tx_context))
    }

    /// Return the hash of the current transaction
    public fun tx_hash(self: &Context): vector<u8> {
        tx_context::tx_hash(&self.tx_context)
    } 

    /// Add a value to the context map
    public fun add<T: drop + store + copy>(self: &mut Context, value: T) {
        tx_context::add(&mut self.tx_context, value); 
    }

    /// Get a value from the context map
    public fun get<T: drop + store + copy>(self: &Context): Option<T> {
        tx_context::get(&self.tx_context)
    }

    public fun tx_meta(self: &Context): TxMeta {
        tx_context::tx_meta(&self.tx_context)
    }

    public fun tx_gas_payment_account(self: &Context): address {
        tx_context::tx_gas_payment_account(&self.tx_context)
    }

    public fun tx_result(self: &Context): TxResult {
        tx_context::tx_result(&self.tx_context)
    }


    // Wrap functions for Object

    #[private_generics(T)]
    /// Create a new Object, Add the Object to the global object storage and return the Object
    /// Note: the default owner is the `System`, the caller should explicitly transfer the Object to the owner.
    /// The owner can get the `&mut Object` by `borrow_mut_object`
    public fun new_object<T: key>(self: &mut Context, value: T): Object<T> {
        let id = fresh_object_id(self);
        object::new(id, value)
    }

    public(friend) fun new_object_with_id<T: key>(_self: &mut Context, id: ObjectID, value: T) : Object<T> {
        object::new(id, value)
    }

    #[private_generics(T)]
    /// Create a new singleton object, the object is owned by `System` by default.
    /// Singleton object means the object of `T` is only one instance in the Object Storage.
    public fun new_singleton<T: key>(_self: &mut Context, value: T): Object<T> {
        object::new_singleton(value)
    }

    /// Borrow Object from object store with object_id
    /// Any one can borrow an `&Object<T>` from the global object storage
    public fun borrow_object<T: key>(_self: &Context, object_id: ObjectID): &Object<T> {
        let object_entity = object::borrow_from_global<T>(object_id);
        object::as_ref(object_entity)
    }

    /// Borrow singleton Object from global object storage
    public fun borrow_singleton<T: key>(self: &Context): &Object<T> {
        let object_id = object::singleton_object_id<T>();
        borrow_object(self, object_id)
    }

    /// Borrow mut Object from object store with object_id
    /// If the object is not shared, only the owner can borrow an `&mut Object<T>` from the global object storage
    public fun borrow_mut_object<T: key>(_self: &mut Context, owner: &signer, object_id: ObjectID): &mut Object<T> {
        let object_entity = object::borrow_mut_from_global<T>(object_id);
        let obj = object::as_mut_ref(object_entity);
        if(!object::is_shared(obj)) {
            let owner_address = signer::address_of(owner);
            assert!(object::owner(obj) == owner_address, error::permission_denied(ErrorObjectOwnerNotMatch));
        };
        obj
    }

    #[private_generics(T)]
    /// The module of T can borrow mut Object from object store with any object_id
    public fun borrow_mut_object_extend<T: key>(_self: &mut Context, object_id: ObjectID) : &mut Object<T> {
        let object_entity = object::borrow_mut_from_global<T>(object_id);
        object::as_mut_ref(object_entity)
    }

    #[private_generics(T)]
    /// Borrow mut singleton Object from global object storage
    /// Only the module of T can borrow mut singleton Object from object store
    public fun borrow_mut_singleton<T: key>(_self: &mut Context): &mut Object<T> {
        let object_id = object::singleton_object_id<T>();
        let object_entity = object::borrow_mut_from_global<T>(object_id);
        object::as_mut_ref(object_entity)
    }

    /// Check if the object exists in the global object storage
    public fun exist_object<T: key>(_self: &Context, object_id: ObjectID): bool {
        object::contains_global(object_id)
        //TODO check the object type
    }

    /// Check if the singleton object exists in the global object storage
    public fun exist_singleton<T: key>(_self: &Context): bool {
        let object_id = object::singleton_object_id<T>();
        exist_object<T>(_self, object_id)
    }

    #[test_only]
    /// Create a Context for unit test
    public fun new_test_context(sender: address): Context {
        // We need to ensure the tx_hash is unique, so we append the sender to the seed
        // If a sender create two Context, the tx_hash will be the same.
        // Maybe the test function need to pass a type parameter as seed.
        let seed = b"test_tx";
        std::vector::append(&mut seed, moveos_std::bcs::to_bytes(&sender));
        new_test_context_random(sender, seed)
    }

    #[test_only]
    /// Create a Context for unit test with random seed
    public fun new_test_context_random(sender: address, seed: vector<u8>): Context {
        let tx_context = tx_context::new_test_context_random(sender, seed);
        let storage_context = moveos_std::storage_context::new(&mut tx_context);
        Context {
            tx_context,
            storage_context,
        }
    }

    #[test_only]
    /// Testing only: allow to drop Context
    public fun drop_test_context(self: Context) {
        moveos_std::test_helper::destroy<Context>(self);
    }

    #[test_only]
    struct TestStruct has key {
        value: u64,
    }

    #[test(sender = @0x42)]
    fun test_object_mut(sender: address){
        let ctx = new_test_context(sender);
        
        let obj = new_object(&mut ctx, TestStruct{value: 1});
        
        {
            let obj_value = object::borrow_mut(&mut obj);
            obj_value.value = 2;
        };
        {
            let obj_value = object::borrow(&obj);
            assert!(obj_value.value == 2, 1000);
        };
        object::to_permanent(obj);
        drop_test_context(ctx);
    }

    #[test(alice = @0x42)]
    fun test_borrow_object(alice: &signer){
        let alice_addr = signer::address_of(alice);
        let ctx = new_test_context(alice_addr);
        
        let obj = new_object(&mut ctx, TestStruct{value: 1});
        let object_id = object::id(&obj);
        object::transfer_extend(&mut obj, alice_addr);

        //test borrow_object by id
        {
            let _obj = borrow_object<TestStruct>(&mut ctx, object_id);
        };
       
        object::to_permanent(obj);
        drop_test_context(ctx);
    }

    #[test(alice = @0x42, bob = @0x43)]
    #[expected_failure(abort_code = 327681, location = Self)]
    fun test_borrow_mut_object(alice: &signer, bob: &signer){
        let alice_addr = signer::address_of(alice);
        let ctx = new_test_context(alice_addr);
        
        let obj = new_object(&mut ctx, TestStruct{value: 1});
        let object_id = object::id(&obj);
        object::transfer_extend(&mut obj, alice_addr);

        //test borrow_mut_object by owner
        {
            let _obj = borrow_mut_object<TestStruct>(&mut ctx, alice, object_id);
        };

        // borrow_mut_object by non-owner failed 
        {
            let _obj = borrow_mut_object<TestStruct>(&mut ctx, bob, object_id);
        };
        object::to_permanent(obj);
        drop_test_context(ctx);
    }

    #[test(alice = @0x42, bob = @0x43)] 
    fun test_shared_object(alice: &signer, bob: &signer){
        let alice_addr = signer::address_of(alice);
        let ctx = new_test_context(alice_addr);
        
        let obj = new_object(&mut ctx, TestStruct{value: 1});
        let object_id = object::id(&obj);
        object::transfer_extend(&mut obj, alice_addr);

        //test borrow_mut_object by owner
        {
            let _obj = borrow_mut_object<TestStruct>(&mut ctx, alice, object_id);
        };

        object::to_shared(obj);
        // any one can borrow_mut the shared object
        {
            let obj = borrow_mut_object<TestStruct>(&mut ctx, bob, object_id);
            assert!(object::is_shared(obj), 1000);
        };
        drop_test_context(ctx);
    }

    #[test(alice = @0x42)]
    #[expected_failure(abort_code = 327681, location =  moveos_std::object)]
    fun test_frozen_object_by_owner(alice: &signer){
        let alice_addr = signer::address_of(alice);
        let ctx = new_test_context(alice_addr);
        
        let obj = new_object(&mut ctx, TestStruct{value: 1});
        let object_id = object::id(&obj);
        object::transfer_extend(&mut obj, alice_addr);
        object::to_frozen(obj);
        //test borrow_object by owner
        {
            let _obj = borrow_object<TestStruct>(&mut ctx, object_id);
        };

        // none one can borrow_mut from the frozen object
        {
            let _obj = borrow_mut_object<TestStruct>(&mut ctx, alice, object_id);
        };
        drop_test_context(ctx);
    }

    #[test(alice = @0x42)]
    #[expected_failure(abort_code = 327681, location =  moveos_std::object)]
    fun test_frozen_object_by_extend(alice: &signer){
        let alice_addr = signer::address_of(alice);
        let ctx = new_test_context(alice_addr);
        
        let obj = new_object(&mut ctx, TestStruct{value: 1});
        let object_id = object::id(&obj);
        object::transfer_extend(&mut obj, alice_addr);
        object::to_frozen(obj);
        //test borrow_object by owner
        {
            let _obj = borrow_object<TestStruct>(&mut ctx, object_id);
        };

        // none one can borrow_mut from the frozen object
        {
            let _obj = borrow_mut_object_extend<TestStruct>(&mut ctx, object_id);
        };
        drop_test_context(ctx);
    }
}
