// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Move Object
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
    
    /// Box style object
    /// The object can not be copied, droped and stored. It only can be consumed by StorageContext API.
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

    /// Object<T> is a reference of the ObjectEntity<T>
    /// It likes ObjectID, but it contains the type information of the object.
    struct Object<phantom T> has key, store {
        id: ObjectID,
    }
  
    /// An object ID
    struct ObjectID has store, copy, drop {
        // TODO should use u256 to replace address?
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

    /// Create a new object, the object is owned by `system` by default.
    public(friend) fun new<T: key>(id: ObjectID, value: T): Object<T> {
        let owner = SYSTEM_OWNER_ADDRESS;
        let entity = ObjectEntity<T>{id, value, owner};
        add_to_global(entity);
        Object{id}
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
    struct TestObject has key {
        count: u64,
    }

    #[test(sender = @0x2)]
    fun test_object(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let tx_context = moveos_std::tx_context::new_test_context(sender_addr);
        let object_count = 12;
        let object = TestObject {
            count: object_count,
        };
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(&mut tx_context));
        let obj = new<TestObject>(object_id, object);

        let borrow_object = borrow_mut(&mut obj);
        assert!(borrow_object.count == object_count, 1001);

        transfer_extend(&mut obj, @0x10);
        let obj_owner = owner(&obj);
        assert!(obj_owner != sender_addr, 1002);

        let test_obj = remove(obj);
        let TestObject{count: _count} = test_obj;
    }
}
