// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Context is part of the StorageAbstraction
/// It is used to provide a context for the storage operations, make the storage abstraction, 
/// and let developers customize the storage
module moveos_std::context {

    use std::option::Option;
    use moveos_std::object_id::{TypedUID, UID, ObjectID};
    use moveos_std::object_id;
    use moveos_std::storage_context::{StorageContext};
    use moveos_std::tx_context::{Self, TxContext};
    use moveos_std::object::{Self, Object};
    use moveos_std::tx_meta::{TxMeta};
    use moveos_std::tx_result::{TxResult}; 
    use moveos_std::table::{Self, Table};
    use moveos_std::type_table::{Self, TypeTable};
    use moveos_std::table_vec::{Self, TableVec};

    friend moveos_std::move_module;

    const ErrorObjectOwnerNotMatch: u64 = 1;
    const ErrorObjectNotShared: u64 = 2;
    ///Can not take out the object which is bound to the account
    const ErrorObjectIsBound: u64 = 3;

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
    public(friend) fun tx_context(_self: &Context): &TxContext {
        tx_context::borrow()
    }

    /// Get a mutable reference to the transaction context from the storage context
    public(friend) fun tx_context_mut(_self: &mut Context): &mut TxContext {
        tx_context::borrow_mut()
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
    public fun fresh_address(_self: &mut Context): address {
        tx_context::fresh_address(tx_context::borrow_mut())
    }

    /// Generate a new unique ObjectID
    public fun fresh_object_id(self: &mut Context): ObjectID {
        object_id::address_to_object_id(fresh_address(self))
    }

    /// Generate a new unique ID
    public fun fresh_uid(self: &mut Context): UID {
        object_id::new_uid(fresh_object_id(self))
    }

    /// Return the hash of the current transaction
    public fun tx_hash(self: &Context): vector<u8> {
        tx_context::tx_hash(&self.tx_context)
    } 

    /// Add a value to the context map
    public fun add<T: drop + store + copy>(_self: &mut Context, value: T) {
        tx_context::add(tx_context::borrow_mut(), value); 
    }

    /// Get a value from the context map
    public fun get<T: drop + store + copy>(_self: &Context): Option<T> {
        tx_context::get(tx_context::borrow())
    }

    public fun tx_meta(_self: &Context): TxMeta {
        tx_context::tx_meta(tx_context::borrow())
    }

    public fun tx_gas_payment_account(_self: &Context): address {
        tx_context::tx_gas_payment_account(tx_context::borrow())
    }

    public fun tx_result(_self: &Context): TxResult {
        tx_context::tx_result(tx_context::borrow())
    }

    // === Table functions ===

    public fun new_table<K: copy + drop, V: store>(self: &mut Context): Table<K, V>{
        let uid = fresh_uid(self);
        table::new(uid)
    }

    public fun new_type_table(self: &mut Context): TypeTable {
        let uid = fresh_uid(self);
        type_table::new(uid)
    }

    public fun new_table_vec<V: store>(self: &mut Context): TableVec<V>{
        let uid = fresh_uid(self);
        table_vec::new(uid)
    }

    // === Object functions ==

    /// Create a new Object UID, then call `object::new` to create a new Object
    public fun new_object_uid<T: key>(self: &mut Context): TypedUID<T> {
        let id = fresh_object_id(self);
        object_id::new_typed_uid(id)
    }

    #[private_generics(T)]
    /// Create a new Object, Add the Object to the global object storage and return the Object
    /// Note: the default owner is the `System`, the caller should explicitly transfer the Object to the owner.
    public fun new_object<T: key>(self: &mut Context, value: T): Object<T> {
        let id = fresh_object_id(self);
        object::new_with_id(id, value)
    }

    /// Create a new named Object UID, then call `object::new` to create a new Object
    public fun new_named_object_uid<T: key>(_self: &mut Context): TypedUID<T> {
        let id = object_id::named_object_id<T>();
        object_id::new_typed_uid(id)
    }

    #[private_generics(T)]
    /// Create a new named Object, the ObjectID is generated by the type_name of `T`
    public fun new_named_object<T: key>(_self: &mut Context, value: T): Object<T> {
        let id = object_id::named_object_id<T>();
        object::new_with_id(id, value)
    }

    /// Create a new account named Object UID, then call `object::new` to create a new Object
    public fun new_account_named_object_uid<T: key>(_self: &mut Context, account: address): TypedUID<T> {
        let id = object_id::account_named_object_id<T>(account);
        object_id::new_typed_uid(id)
    }

    #[private_generics(T)]
    /// Create a new account named object, the ObjectID is generated by the account address and type_name of `T`
    public fun new_account_named_object<T: key>(_self: &mut Context, account: address, value: T): Object<T> {
        let id = object_id::account_named_object_id<T>(account);
        object::new_with_id(id, value)
    }

    /// Create a new custom Object UID, then call `object::new` to create a new Object
    public fun new_custom_object_uid<ID: drop, T: key>(_self: &mut Context, id: ID): TypedUID<T> {
        let id = object_id::custom_object_id<ID, T>(id);
        object_id::new_typed_uid(id)
    }

    #[private_generics(T)]
    /// Create a new custom object, the ObjectID is generated by the `id` and type_name of `T`
    public fun new_custom_object<ID: drop, T: key>(_self: &mut Context, id: ID, value: T): Object<T> {
        let id = object_id::custom_object_id<ID, T>(id);
        object::new_with_id(id, value)
    }

    #[private_generics(T)]
    /// Take out the UserOwnedObject by `object_id`, return the owner and Object
    /// This function is for developer to extend, Only the module of `T` can take out the `UserOwnedObject` with object_id.
    public fun take_object_extend<T: key>(_self: &mut Context, object_id: ObjectID): (address, Object<T>) {
        let object_entity = object::borrow_mut_from_global<T>(object_id);
        assert!(object::is_user_owned_internal(object_entity), ErrorObjectOwnerNotMatch);
        assert!(!object::is_bound_internal(object_entity), ErrorObjectIsBound);
        let owner = object::owner_internal(object_entity);
        object::to_system_owned_internal(object_entity);
        (owner, object::mut_entity_as_object(object_entity))
    }

    /// Check if the object exists in the global object storage
    public fun exists_object<T: key>(_self: &Context, object_id: ObjectID): bool {
        object::contains_global(object_id)
        //TODO check the object type
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
    use moveos_std::signer;

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
        let TestStruct{value:_} = object::remove(obj);
        drop_test_context(ctx);
    }

    #[test(alice = @0x42)]
    fun test_borrow_object(alice: signer){
        let alice_addr = signer::address_of(&alice);
        let ctx = new_test_context(alice_addr);
        
        let obj = new_object(&mut ctx, TestStruct{value: 1});
        let object_id = object::id(&obj);
        object::transfer_extend(obj, alice_addr);

        //test borrow_object by id
        {
            let _obj = object::borrow_object<TestStruct>(object_id);
        };
       
        drop_test_context(ctx);
    }

    #[test(alice = @0x42, bob = @0x43)]
    #[expected_failure(abort_code = 4, location = moveos_std::object)]
    fun test_borrow_mut_object(alice: &signer, bob: &signer){
        let alice_addr = signer::address_of(alice);
        let ctx = new_test_context(alice_addr);
        
        let obj = new_object(&mut ctx, TestStruct{value: 1});
        let object_id = object::id(&obj);
        object::transfer_extend(obj, alice_addr);

        //test borrow_mut_object by owner
        {
            let _obj = object::borrow_mut_object<TestStruct>(alice, object_id);
        };

        // borrow_mut_object by non-owner failed 
        {
            let _obj = object::borrow_mut_object<TestStruct>(bob, object_id);
        };
        drop_test_context(ctx);
    }

    #[test(alice = @0x42)] 
    fun test_shared_object(alice: &signer){
        let alice_addr = signer::address_of(alice);
        let ctx = new_test_context(alice_addr);
        
        let obj = new_object(&mut ctx, TestStruct{value: 1});
        let object_id = object::id(&obj);
        
        object::to_shared(obj);
        // any one can borrow_mut the shared object
        {
            let obj = object::borrow_mut_object_shared<TestStruct>(object_id);
            assert!(object::is_shared(obj), 1000);
        };
        drop_test_context(ctx);
    }


    #[test(alice = @0x42)]
    #[expected_failure(abort_code = 2, location =  moveos_std::object)]
    fun test_frozen_object_by_extend(alice: &signer){
        let alice_addr = signer::address_of(alice);
        let ctx = new_test_context(alice_addr);
        
        let obj = new_object(&mut ctx, TestStruct{value: 1});
        let object_id = object::id(&obj);
        object::to_frozen(obj);
        //test borrow_object
        {
            let _obj = object::borrow_object<TestStruct>(object_id);
        };

        // none one can borrow_mut from the frozen object
        {
            let _obj = object::borrow_mut_object_extend<TestStruct>(object_id);
        };
        drop_test_context(ctx);
    }
}
