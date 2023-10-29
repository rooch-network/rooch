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
    friend moveos_std::object_ref;
    friend moveos_std::table;
    friend moveos_std::type_table;

    const ErrorInvalidOwnerAddress:u64 = 1;

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

    /// Create a new object, the object is owned by `owner`
    public(friend) fun new<T: key>(id: ObjectID, value: T): ObjectEntity<T> {
        let owner = SYSTEM_OWNER_ADDRESS;
        ObjectEntity<T>{id, value, owner}
    }

    public(friend) fun borrow<T>(self: &ObjectEntity<T>): &T {
        &self.value
    }

    public(friend) fun borrow_mut<T>(self: &mut ObjectEntity<T>): &mut T {
        &mut self.value
    }

    public(friend) fun transfer<T>(self: &mut ObjectEntity<T>, owner: address) {
        assert!(owner != SYSTEM_OWNER_ADDRESS, error::invalid_argument(ErrorInvalidOwnerAddress));
        self.owner = owner;
    }

    public(friend) fun transfer_to_system<T>(self: &mut ObjectEntity<T>){
        self.owner = SYSTEM_OWNER_ADDRESS;
    }

    public(friend) fun to_shared<T>(self: &mut ObjectEntity<T>) {
        // TODO set the flag
        transfer_to_system(self);
    }

    public(friend) fun is_shared<T>(_self: &ObjectEntity<T>) : bool {
        // TODO check the flag
        false
    }

    public(friend) fun to_frozen<T>(self: &mut ObjectEntity<T>) {
        // TODO set the flag
        transfer_to_system(self);
    }

    public(friend) fun is_frozen<T>(_self: &ObjectEntity<T>) : bool {
        // TODO check the flag
        false
    }
    
    public fun id<T>(self: &ObjectEntity<T>): ObjectID {
        self.id
    }

    public fun owner<T>(self: &ObjectEntity<T>): address {
        self.owner
    }

    /// Unpack the object, return the id, owner, and value
    public(friend) fun unpack<T>(self: ObjectEntity<T>): (ObjectID, address, T) {
        let ObjectEntity{id, owner, value} = self;
        (id, owner, value)
    }

    // === Object Storage ===

    const GlobalObjectStorageHandle: address = @0x0;

    /// The global object storage's table handle should be `0x0`
    public(friend) fun global_object_storage_handle(): TableHandle {
        raw_table::new_table_handle(GlobalObjectStorageHandle)
    }

    public(friend) fun add_to_global<T: key>(obj: ObjectEntity<T>) {
        raw_table::add<ObjectID, ObjectEntity<T>>(global_object_storage_handle(), id(&obj), obj);
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

        transfer(&mut obj, @0x10);
        let obj_owner = owner(&obj);
        assert!(obj_owner != sender_addr, 1002);

        let (_id, _owner, test_obj) = unpack(obj);
        let TestObject{count: _count} = test_obj;
    }
}
