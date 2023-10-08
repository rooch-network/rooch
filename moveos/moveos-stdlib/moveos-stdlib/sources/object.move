// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Move Object
/// The Object is a box style Object
/// The differents with the Object in [Sui](https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/object.move#L75):
/// 1. The Object is a struct in Move
/// 2. The Object is a use case of the Hot Potato pattern in Move. Objects do not have any ability, so they cannot be drop, copy, or store, and can only be handled by StorageContext API after creation.
module moveos_std::object {
    
    use moveos_std::tx_context::{Self, TxContext};
    use moveos_std::object_id::ObjectID;

    friend moveos_std::context;
    friend moveos_std::account_storage;
    friend moveos_std::storage_context;
    friend moveos_std::event;
    friend moveos_std::object_ref;
   
    /// Box style object
    /// The object can not be copied, droped and stored. It only can be consumed by StorageContext API.
    struct Object<T> {
        // The object id
        id: ObjectID,
        // The owner of the object
        owner: address,
        // The value of the object
        // The value must be the last field
        value: T,
    }

    /// Create a new object, the object is owned by `owner`
    public(friend) fun new<T: key>(ctx: &mut TxContext, owner: address, value: T): Object<T> {
        let id = tx_context::fresh_object_id(ctx);
        Object<T>{id, value, owner}
    }

    public(friend) fun new_with_id<T: key>(id: ObjectID, owner: address, value: T): Object<T> {
        Object<T>{id, owner, value}
    }

    #[private_generics(T)]
    // Borrow the object value
    public fun borrow<T>(self: &Object<T>): &T {
        &self.value
    }

    public(friend) fun internal_borrow<T>(self: &Object<T>): &T {
        &self.value
    }

    #[private_generics(T)]
    /// Borrow the mutable object value
    public fun borrow_mut<T>(self: &mut Object<T>): &mut T {
        &mut self.value
    }

    public(friend) fun internal_borrow_mut<T>(self: &mut Object<T>): &mut T {
        &mut self.value
    }

    #[private_generics(T)]
    /// Transfer object to recipient
    public fun transfer<T: key>(self: &mut Object<T>, recipient: address) {
        self.owner = recipient;
    }

    public fun id<T>(self: &Object<T>): ObjectID {
        self.id
    }

    public fun owner<T>(self: &Object<T>): address {
        self.owner
    }

    #[private_generics(T)]
    /// Unpack the object, return the id, owner, and value
    public fun unpack<T>(self: Object<T>): (ObjectID, address, T) {
        let Object{id, owner, value} = self;
        (id, owner, value)
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
        let obj = new<TestObject>(&mut tx_context, sender_addr, object);

        let borrow_object = borrow_mut(&mut obj);
        assert!(borrow_object.count == object_count, 1001);

        transfer(&mut obj, @0x10);
        let obj_owner = owner(&obj);
        assert!(obj_owner != sender_addr, 1002);

        let (_id, _owner, test_obj) = unpack(obj);
        let TestObject{count: _count} = test_obj;
    }
}
