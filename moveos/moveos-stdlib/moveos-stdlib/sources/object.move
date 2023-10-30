// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Move Object
/// For more details, please refer to https://rooch.network/docs/developer-guides/object
/// 
/// 
/// For more details please refer https://rooch.network/docs/developer-guides/object
/// The Object is a box style Object
/// The differents with the Object in [Sui](https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/object.move#L75):
/// 1. The Object is a struct in Move
/// 2. The Object is a use case of the Hot Potato pattern in Move. Objects do not have any ability, so they cannot be drop, copy, or store, and can only be handled by StorageContext API after creation.
module moveos_std::object {

    use std::error;
    use std::hash;
    use moveos_std::type_info;
    use moveos_std::bcs;
    use moveos_std::address;
    use moveos_std::raw_table::{Self, TableHandle};

    friend moveos_std::context;
    friend moveos_std::account_storage;
    friend moveos_std::storage_context;
    friend moveos_std::event;
    friend moveos_std::table;
    friend moveos_std::type_table;

    const ErrorObjectFrozen: u64 = 1;
    const ErrorInvalidOwnerAddress:u64 = 2;

    const SYSTEM_OWNER_ADDRESS: address = @0x0;
    
    /// ObjectEntity<T> is a box of the value of T
    /// It does not have any ability, so it can not be `drop`, `copy`, or `store`, and can only be handled by storage API after creation.
    struct ObjectEntity<T> {
        // The object id
        id: ObjectID,
        // The owner of the object
        owner: address,
        /// A flag to indicate whether the object is shared or frozen
        //flag: u8,
        // The value of the object
        // The value must be the last field
        value: T,
    }

    /// Object<T> is a pointer to the ObjectEntity<T>, It has `key` and `store` ability. 
    /// It has the same lifetime as the ObjectEntity<T>
    /// Developers only need to use Object<T> related APIs and do not need to know the ObjectEntity<T>.
    struct Object<phantom T> has key, store {
        id: ObjectID,
    }
  
    /// ObjectID is a unique identifier for the Object
    struct ObjectID has store, copy, drop {
        id: address,
    }

    /// Generate a new ObjectID from an address
    public(friend) fun address_to_object_id(address: address): ObjectID {
        ObjectID { id: address }
    }

    public(friend) fun object_id_to_table_handle(object_id: ObjectID): TableHandle {
        raw_table::new_table_handle(object_id.id)
    }

    public(friend) fun singleton_object_id<T>(): ObjectID {
        address_to_object_id(
            address::from_bytes(
                hash::sha3_256(
                    bcs::to_bytes(
                        &type_info::type_of<T>()
                    )
                )
            )
        )
    }

    /// Create a new object, the object is owned by `System` by default.
    public(friend) fun new<T: key>(id: ObjectID, value: T): Object<T> {
        let owner = SYSTEM_OWNER_ADDRESS;
        let entity = ObjectEntity<T>{id, value, owner};
        add_to_global(entity);
        Object{id}
    }

    /// Create a new singleton object, singleton object is always owned by `System` and is p
    /// Singleton object means the object of `T` is only one instance in the Object Storage.
    public(friend) fun new_singleton<T: key>(value: T): Object<T> {
        let id = singleton_object_id<T>();
        new(id, value)
    }

    /// Borrow the object value
    public fun borrow<T: key>(self: &Object<T>): &T {
        let obj_enitty = borrow_from_global<T>(self.id);
        &obj_enitty.value
    }

    /// Borrow the object mutable value
    public fun borrow_mut<T: key>(self: &mut Object<T>): &mut T {
        let obj_entity = borrow_mut_from_global<T>(self.id);
        &mut obj_entity.value
    }

    #[private_generics(T)]
    /// Remove the object from the global storage, and return the object value
    /// This function is only can be called by the module of `T`.
    public fun remove<T: key>(self: Object<T>) : T {
        let Object{id} = self;
        let object_entity = remove_from_global<T>(id);
        let ObjectEntity{id:_, owner:_, value} = object_entity;
        value
    }

    /// Directly drop the Object, and make the Object permanent, the object will can not be removed from the object storage.
    /// If you want to remove the object, please use `remove` function.
    public fun to_permanent<T: key>(self: Object<T>) {
        let Object{id:_} = self;
    }

    /// Make the Object shared, Any one can get the &mut Object<T> from shared object
    /// The shared object also can be removed from the object storage.
    public fun to_shared<T: key>(self: Object<T>) {
        let obj_entity = borrow_mut_from_global<T>(self.id);
        // TODO set the flag
        transfer_to_system(obj_entity); 
        to_permanent(self);
    }

    /// Make the Object frozen, Any one can not get the &mut Object<T> from frozen object
    public fun to_frozen<T: key>(self: Object<T>) {
        let obj_entity = borrow_mut_from_global<T>(self.id);
        // TODO set the flag
        transfer_to_system(obj_entity); 
        to_permanent(self);
    }

    /// Transfer the object to the new owner
    /// Only the `T` with `store` can be directly transferred.
    public fun transfer<T: key + store>(self: &mut Object<T>, new_owner: address) {
        let obj_entity = borrow_mut_from_global<T>(self.id);
        transer_internal(obj_entity, new_owner);
    }

    #[private_generics(T)]
    /// Transfer the object to the new owner
    /// This function is for the module of `T` to extend the `transfer` function.
    public fun transfer_extend<T: key>(self: &mut Object<T>, new_owner: address) {
        let obj = borrow_mut_from_global<T>(self.id);
        transer_internal(obj, new_owner);
    }

    fun transer_internal<T: key>(self: &mut ObjectEntity<T>, new_owner: address) {
        assert!(new_owner != SYSTEM_OWNER_ADDRESS, error::invalid_argument(ErrorInvalidOwnerAddress));
        self.owner = new_owner;
    }

    public(friend) fun transfer_to_system<T>(self: &mut ObjectEntity<T>){
        self.owner = SYSTEM_OWNER_ADDRESS;
    }

    public fun id<T>(self: &Object<T>): ObjectID {
        self.id
    }

    public fun owner<T: key>(self: &Object<T>): address {
        let obj_enitty = borrow_from_global<T>(self.id);
        obj_enitty.owner
    }

    public(friend) fun owner_internal<T: key>(self: &ObjectEntity<T>): address {
        self.owner
    }

    public fun is_shared<T>(_self: &Object<T>) : bool {
        // TODO check the flag
        false
    }

    public(friend) fun is_shared_internal<T>(_self: &ObjectEntity<T>) : bool {
        // TODO check the flag
        false
    }

    public fun is_frozen<T>(_self: &Object<T>) : bool {
        // TODO check the flag
        false
    }

    public(friend) fun is_frozen_internal<T>(_self: &ObjectEntity<T>) : bool {
        // TODO check the flag
        false
    }
    

    // === Object Ref ===

    public(friend) fun as_ref<T: key>(object_entity: &ObjectEntity<T>) : &Object<T>{
        as_ref_inner<Object<T>>(object_entity.id)
    }
    public(friend) fun as_mut_ref<T: key>(object_entity: &mut ObjectEntity<T>) : &mut Object<T>{
        assert!(!is_frozen_internal(object_entity), error::permission_denied(ErrorObjectFrozen));
        as_mut_ref_inner<Object<T>>(object_entity.id)
    }

    /// Convert the ObjectID to &T or &mut T
    /// The caller must ensure the T only has one `ObjectID` field, such as `Object<T>` or `Table<K,V>`, or `TypeTable`.
    native fun as_ref_inner<T>(object_id: ObjectID): &T;
    native fun as_mut_ref_inner<T>(object_id: ObjectID): &mut T;

    // === Object Storage ===

    const GlobalObjectStorageHandle: address = @0x0;

    /// The global object storage's table handle should be `0x0`
    public(friend) fun global_object_storage_handle(): TableHandle {
        raw_table::new_table_handle(GlobalObjectStorageHandle)
    }

    public(friend) fun add_to_global<T: key>(obj: ObjectEntity<T>) {
        raw_table::add<ObjectID, ObjectEntity<T>>(global_object_storage_handle(), obj.id, obj);
    }

    public(friend) fun borrow_from_global<T: key>(object_id: ObjectID): &ObjectEntity<T> {
        raw_table::borrow<ObjectID, ObjectEntity<T>>(global_object_storage_handle(), object_id)
    }

    public(friend) fun borrow_mut_from_global<T: key>(object_id: ObjectID): &mut ObjectEntity<T> {
        raw_table::borrow_mut<ObjectID, ObjectEntity<T>>(global_object_storage_handle(), object_id)
    }

    public(friend) fun remove_from_global<T: key>(object_id: ObjectID): ObjectEntity<T> {
        raw_table::remove<ObjectID, ObjectEntity<T>>(global_object_storage_handle(), object_id)
    }

    public(friend) fun contains_global(object_id: ObjectID): bool {
        raw_table::contains<ObjectID>(global_object_storage_handle(), object_id)
    }

    #[test_only]
    struct TestStruct has key {
        count: u64,
    }

    #[test_only]
    struct TestStruct2 has key {
        count: u64,
    }

    #[test(sender = @0x2)]
    fun test_object(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let tx_context = moveos_std::tx_context::new_test_context(sender_addr);
        let init_count = 12;
        let test_struct = TestStruct {
            count: init_count,
        };
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(&mut tx_context));
        let obj = new<TestStruct>(object_id, test_struct);
        assert!(contains_global(object_id), 1000);
        {
            transfer_extend(&mut obj, sender_addr);
            assert!(owner(&obj) == sender_addr, 1001);
        };
        {
            let test_struct_mut = borrow_mut(&mut obj);
            test_struct_mut.count = test_struct_mut.count + 1;
        };
        {
            let test_struct_ref = borrow(&obj);
            assert!(test_struct_ref.count == init_count + 1, 1002);
        };
        { 
            transfer_extend(&mut obj, @0x10);
            assert!(owner(&obj) != sender_addr, 1003);
        };

        let test_obj = remove(obj);
        let TestStruct{count: _count} = test_obj;
    }


    #[test(sender = @0x42)]
    #[expected_failure(abort_code = 393218, location = moveos_std::raw_table)]
    fun test_borrow_not_exist_failure(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let ctx = moveos_std::tx_context::new_test_context(sender_addr);
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(&mut ctx));
        let obj = new(object_id, TestStruct { count: 1 });
        let TestStruct { count : _ } = remove(obj); 
        let _obj_ref = borrow_from_global<TestStruct>(object_id);
    }

    #[test(sender = @0x42)]
    #[expected_failure(abort_code = 393218, location = moveos_std::raw_table)]
    fun test_double_remove_failure(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let ctx = moveos_std::tx_context::new_test_context(sender_addr);
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(&mut ctx));
        let object = new(object_id, TestStruct { count: 1 });
        
        let ObjectEntity{ id:_,owner:_,value:test_struct1} = remove_from_global<TestStruct>(object_id);
        let test_struct2 = remove(object);
        let TestStruct { count : _ } = test_struct1;
        let TestStruct { count : _ } = test_struct2;
    }

    #[test(sender = @0x42)]
    #[expected_failure(abort_code = 393218, location = moveos_std::raw_table)]
    fun test_type_mismatch(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let ctx = moveos_std::tx_context::new_test_context(sender_addr);
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(&mut ctx));
        let obj = new(object_id, TestStruct { count: 1 });
        {
            let test_struct_ref = borrow(&obj);
            assert!(test_struct_ref.count == 1, 1001);
        };
        {
            let test_struct2_object_entity = borrow_from_global<TestStruct2>(object_id);
            assert!(test_struct2_object_entity.value.count == 1, 1002);
        };
        to_permanent(obj);
    }
}
