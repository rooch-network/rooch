// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Context is part of the StorageAbstraction
/// It is used to provide a context for the storage operations, make the storage abstraction, 
/// and let developers customize the storage
module moveos_std::context {

    use std::option::Option;
    use std::string::String;
    use std::vector;
    use moveos_std::storage_context::{StorageContext};
    use moveos_std::tx_context::{Self, TxContext};
    use moveos_std::object::{Self, ObjectID, UID, Object, TypedUID};
    use moveos_std::tx_meta::{TxMeta};
    use moveos_std::tx_result::{TxResult};
    use moveos_std::signer;
    use moveos_std::account_storage::{Self, AccountStorage};
    use moveos_std::move_module::{Self, MoveModule};
    use moveos_std::table::{Self, Table};
    use moveos_std::type_table::{Self, TypeTable};
    use moveos_std::table_vec::{Self, TableVec};

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

    /// Generate a new unique ObjectID
    public fun fresh_object_id(self: &mut Context): ObjectID {
        object::address_to_object_id(tx_context::fresh_address(&mut self.tx_context))
    }

    /// Generate a new unique ID
    public fun fresh_uid(self: &mut Context): UID {
        object::new_uid(fresh_object_id(self))
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

    // === Account Storage functions ===

    #[private_generics(T)]
    /// Borrow a resource from the account's storage
    /// This function equates to `borrow_global<T>(address)` instruction in Move
    public fun borrow_resource<T: key>(self: &Context, account: address): &T {
        let account_storage = borrow_account_storage(self, account);
        account_storage::borrow_resource<T>(account_storage)
    }

    #[private_generics(T)]
    /// Borrow a mut resource from the account's storage
    /// This function equates to `borrow_global_mut<T>(address)` instruction in Move
    public fun borrow_mut_resource<T: key>(self: &mut Context, account: address): &mut T {
        let account_storage = borrow_account_storage_mut(self, account);
        account_storage::borrow_mut_resource<T>(account_storage)
    }

    #[private_generics(T)]
    /// Move a resource to the account's storage
    /// This function equates to `move_to<T>(&signer, resource)` instruction in Move
    public fun move_resource_to<T: key>(self: &mut Context, account: &signer, resource: T){
        let account_address = signer::address_of(account);
        //Auto create the account storage when move resource to the account
        ensure_account_storage(self, account_address);
        let account_storage = borrow_account_storage_mut(self, account_address);
        account_storage::move_resource_to(account_storage, resource);
    }

    #[private_generics(T)]
    /// Move a resource from the account's storage
    /// This function equates to `move_from<T>(address)` instruction in Move
    public fun move_resource_from<T: key>(self: &mut Context, account: address): T {
        let account_storage = borrow_account_storage_mut(self, account);
        account_storage::move_resource_from<T>(account_storage)
    }

    #[private_generics(T)]
    /// Check if the account has a resource of the given type
    /// This function equates to `exists<T>(address)` instruction in Move
    public fun exists_resource<T: key>(self: &Context, account: address) : bool {
        if (exist_account_storage(self, account)) {
            let account_storage = borrow_account_storage(self, account);
            account_storage::exists_resource<T>(account_storage)
        }else{
            false
        }
    }

    /// Publish modules to the account's storage
    public fun publish_modules(self: &mut Context, account: &signer, modules: vector<MoveModule>) {
        let account_address = signer::address_of(account);
        ensure_account_storage(self, account_address);
        let account_storage = borrow_account_storage_mut(self, account_address);
        let upgrade_flag = account_storage::publish_modules(account_storage, account_address, modules);
        // Store ModuleUpgradeFlag in tx_context which will be fetched in VM in Rust, 
        // and then announce to the VM that the code loading cache should be considered outdated. 
        tx_context::set_module_upgrade_flag(&mut self.tx_context, upgrade_flag);
    }

    /// Check if the account has a module with the given name
    public fun exists_module(self: &Context, account: address, name: String): bool {
        if (exist_account_storage(self, account)) {
            let account_storage = borrow_account_storage(self, account);
            account_storage::exists_module(account_storage, name)
        }else{
            false
        }
    }

    /// Entry function to publish modules
    /// The order of modules must be sorted by dependency order.
    public entry fun publish_modules_entry(ctx: &mut Context, account: &signer, modules: vector<vector<u8>>) {
        let n_modules = vector::length(&modules);
        let i = 0;
        let module_vec = vector::empty<MoveModule>();
        while (i < n_modules) {
            let code_bytes = vector::pop_back(&mut modules);
            let m = move_module::new(code_bytes);
            vector::push_back(&mut module_vec, m);
            i = i + 1;
        };
        
        Self::publish_modules(ctx, account, module_vec);
    }

    // === Object functions ==

    /// Create a new Object UID, then call `object::new` to create a new Object
    public fun new_object_uid<T: key>(self: &mut Context): TypedUID<T> {
        let id = fresh_object_id(self);
        object::new_typed_uid(id)
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
        let id = object::named_object_id<T>();
        object::new_typed_uid(id)
    }

    #[private_generics(T)]
    /// Create a new named Object, the ObjectID is generated by the type_name of `T`
    public fun new_named_object<T: key>(_self: &mut Context, value: T): Object<T> {
        let id = object::named_object_id<T>();
        object::new_with_id(id, value)
    }

    /// Create a new account named Object UID, then call `object::new` to create a new Object
    public fun new_account_named_object_uid<T: key>(_self: &mut Context, account: address): TypedUID<T> {
        let id = object::account_named_object_id<T>(account);
        object::new_typed_uid(id)
    }

    #[private_generics(T)]
    /// Create a new account named object, the ObjectID is generated by the account address and type_name of `T`
    public fun new_account_named_object<T: key>(_self: &mut Context, account: address, value: T): Object<T> {
        let id = object::account_named_object_id<T>(account);
        object::new_with_id(id, value)
    }

    /// Borrow Object from object store by object_id
    /// Any one can borrow an `&Object<T>` from the global object storage
    public fun borrow_object<T: key>(_self: &Context, object_id: ObjectID): &Object<T> {
        let object_entity = object::borrow_from_global<T>(object_id);
        object::as_ref(object_entity)
    }

    /// Borrow mut Object by `owner` and `object_id`
    public fun borrow_mut_object<T: key>(self: &mut Context, owner: &signer, object_id: ObjectID): &mut Object<T> {
        let owner_address = signer::address_of(owner);
        let obj = borrow_mut_object_internal<T>(self, object_id);
        assert!(object::owner(obj) == owner_address, ErrorObjectOwnerNotMatch);
        obj
    }

    /// Take out the UserOwnedObject by `owner` and `object_id`
    /// The `T` must have `key + store` ability.
    /// Note: When the Object is taken out, the Object will auto become `SystemOwned` Object.
    public fun take_object<T: key + store>(_self: &mut Context, owner: &signer, object_id: ObjectID): Object<T> {
        let owner_address = signer::address_of(owner);
        let object_entity = object::borrow_mut_from_global<T>(object_id);
        assert!(object::owner_internal(object_entity) == owner_address, ErrorObjectOwnerNotMatch);
        assert!(!object::is_bound_internal(object_entity), ErrorObjectIsBound);
        object::to_system_owned_internal(object_entity);
        object::mut_entity_as_object(object_entity)
    }

    #[private_generics(T)]
    /// Take out the UserOwnedObject by `object_id`
    /// This function is for developer to extend, Only the module of `T` can take out the `UserOwnedObject` with object_id.
    public fun take_object_extend<T: key>(_self: &mut Context, object_id: ObjectID): Object<T> {
        let object_entity = object::borrow_mut_from_global<T>(object_id);
        assert!(object::is_user_owned_internal(object_entity), ErrorObjectOwnerNotMatch);
        assert!(!object::is_bound_internal(object_entity), ErrorObjectIsBound);
        object::to_system_owned_internal(object_entity);
        object::mut_entity_as_object(object_entity)
    }

    /// Borrow mut Shared Object by object_id
    public fun borrow_mut_object_shared<T: key>(self: &mut Context, object_id: ObjectID): &mut Object<T> {
        let obj = borrow_mut_object_internal<T>(self, object_id);
        assert!(object::is_shared(obj), ErrorObjectNotShared);
        obj
    }

    #[private_generics(T)]
    /// The module of T can borrow mut Object from object store by any object_id
    public fun borrow_mut_object_extend<T: key>(_self: &mut Context, object_id: ObjectID) : &mut Object<T> {
        let object_entity = object::borrow_mut_from_global<T>(object_id);
        object::as_mut_ref(object_entity)
    }

    /// Check if the object exists in the global object storage
    public fun exists_object<T: key>(_self: &Context, object_id: ObjectID): bool {
        object::contains_global(object_id)
        //TODO check the object type
    }

    // == Internal functions ==

    fun ensure_account_storage(self: &mut Context, account: address) {
        if (!exist_account_storage(self, account)) {
            let account_storage = account_storage::create_account_storage(account);
            let object_id = object::address_to_object_id(account); 
            let obj = object::new_with_id(object_id, account_storage);
            account_storage::transfer(obj, account);
        }
    }

    fun borrow_account_storage(self: &Context, account: address): &AccountStorage {
        let obj = borrow_object<AccountStorage>(self, object::address_to_object_id(account));
        object::borrow(obj)
    }

    fun borrow_account_storage_mut(self: &mut Context, account: address): &mut AccountStorage {
        let obj = borrow_mut_object_internal<AccountStorage>(self, object::address_to_object_id(account));
        object::borrow_mut(obj)
    }

    fun exist_account_storage(self: &Context, account: address): bool {
        exists_object<AccountStorage>(self, object::address_to_object_id(account))
    }

    fun borrow_mut_object_internal<T: key>(_self: &mut Context, object_id: ObjectID): &mut Object<T> {
        let object_entity = object::borrow_mut_from_global<T>(object_id);
        let obj = object::as_mut_ref(object_entity);
        obj
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
        let TestStruct{value:_} = object::remove(obj);
        drop_test_context(ctx);
    }

    #[test(alice = @0x42)]
    fun test_borrow_object(alice: &signer){
        let alice_addr = signer::address_of(alice);
        let ctx = new_test_context(alice_addr);
        
        let obj = new_object(&mut ctx, TestStruct{value: 1});
        let object_id = object::id(&obj);
        object::transfer_extend(obj, alice_addr);

        //test borrow_object by id
        {
            let _obj = borrow_object<TestStruct>(&mut ctx, object_id);
        };
       
        drop_test_context(ctx);
    }

    #[test(alice = @0x42, bob = @0x43)]
    #[expected_failure(abort_code = ErrorObjectOwnerNotMatch, location = Self)]
    fun test_borrow_mut_object(alice: &signer, bob: &signer){
        let alice_addr = signer::address_of(alice);
        let ctx = new_test_context(alice_addr);
        
        let obj = new_object(&mut ctx, TestStruct{value: 1});
        let object_id = object::id(&obj);
        object::transfer_extend(obj, alice_addr);

        //test borrow_mut_object by owner
        {
            let _obj = borrow_mut_object<TestStruct>(&mut ctx, alice, object_id);
        };

        // borrow_mut_object by non-owner failed 
        {
            let _obj = borrow_mut_object<TestStruct>(&mut ctx, bob, object_id);
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
            let obj = borrow_mut_object_shared<TestStruct>(&mut ctx, object_id);
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
            let _obj = borrow_object<TestStruct>(&mut ctx, object_id);
        };

        // none one can borrow_mut from the frozen object
        {
            let _obj = borrow_mut_object_extend<TestStruct>(&mut ctx, object_id);
        };
        drop_test_context(ctx);
    }


    #[test(sender=@0x42)]
    fun test_ensure_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = Self::new_test_context(sender_addr);
        ensure_account_storage(&mut ctx , sender_addr);
        assert!(exist_account_storage(&ctx , sender_addr), 1);
        Self::drop_test_context(ctx);
    }

}
