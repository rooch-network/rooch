// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::object_ref {

    use std::error;
    use moveos_std::object::{Self, ObjectEntity, ObjectID};

    friend moveos_std::context;

    const ErrorObjectFrozen: u64 = 1;

    ///TODO rename to Object
    /// ObjectRef<T> is a reference of the ObjectEntity<T>
    /// It likes ObjectID, but it contains the type information of the object.
    struct ObjectRef<phantom T> has key, store {
        id: ObjectID,
    }

    public(friend) fun new_internal<T: key>(object: &mut ObjectEntity<T>) : ObjectRef<T> {
        ObjectRef {
            id: object::id(object),
        }
    }

    public(friend) fun as_ref<T: key>(object: &ObjectEntity<T>) : &ObjectRef<T>{
        as_ref_inner<ObjectRef<T>>(object::id(object))
    }
    public(friend) fun as_mut_ref<T: key>(object: &mut ObjectEntity<T>) : &mut ObjectRef<T>{
        assert!(!object::is_frozen(object), error::permission_denied(ErrorObjectFrozen));
        as_mut_ref_inner<ObjectRef<T>>(object::id(object))
    }

    /// Convert the ObjectID to &T or &mut T
    /// The caller must ensure the T only has one `ObjectID` field, such as `ObjectRef<T>` or `Table<K,V>`, or `TypeTable`.
    native fun as_ref_inner<T>(object_id: ObjectID): &T;
    native fun as_mut_ref_inner<T>(object_id: ObjectID): &mut T;

    /// Borrow the object value
    public fun borrow<T: key>(self: &ObjectRef<T>): &T {
        let obj = object::borrow_from_global<T>(self.id);
        object::borrow(obj)
    }

    /// Borrow the object mutable value
    public fun borrow_mut<T: key>(self: &mut ObjectRef<T>): &mut T {
        let obj = object::borrow_mut_from_global<T>(self.id);
        object::borrow_mut(obj)
    }

    #[private_generics(T)]
    /// Remove the object from the global storage, and return the object value
    /// This function is only can be called by the module of `T`.
    public fun remove<T: key>(self: ObjectRef<T>) : T {
        let ObjectRef{id} = self;
        let object = object::remove_from_global(id);
        let (_id, _owner, value) = object::unpack(object);
        value
    }

    /// Directly drop the ObjectRef, and make the Object permanent, the object will can not be removed from the object storage.
    /// If you want to remove the object, please use `remove` function.
    public fun to_permanent<T: key>(self: ObjectRef<T>) {
        let ObjectRef{id:_} = self;
    }

    /// Make the Object shared, Any one can get the &mut ObjectRef<T> from shared object
    /// The shared object also can be removed from the object storage.
    public fun to_shared<T: key>(self: ObjectRef<T>) {
        let obj = object::borrow_mut_from_global<T>(self.id);
        object::to_shared(obj); 
        to_permanent(self);
    }

    /// Make the Object frozen, Any one can not get the &mut ObjectRef<T> from frozen object
    public fun to_frozen<T: key>(self: ObjectRef<T>) {
        let obj = object::borrow_mut_from_global<T>(self.id);
        object::to_frozen(obj);
        to_permanent(self);
    }

    /// Transfer the object to the new owner
    /// Only the `T` with `store` can be directly transferred.
    public fun transfer<T: key + store>(self: &mut ObjectRef<T>, new_owner: address) {
        let obj = object::borrow_mut_from_global<T>(self.id);
        object::transfer(obj, new_owner);
    }

    #[private_generics(T)]
    /// Transfer the object to the new owner
    /// This function is for the module of `T` to extend the `transfer` function.
    public fun transfer_extend<T: key>(self: &mut ObjectRef<T>, new_owner: address) {
        let obj = object::borrow_mut_from_global<T>(self.id);
        object::transfer(obj, new_owner);
    }

    public fun id<T>(self: &ObjectRef<T>): ObjectID {
        self.id
    }

    public fun owner<T: key>(self: &ObjectRef<T>): address {
        let obj = object::borrow_from_global<T>(self.id);
        object::owner(obj)
    }

}
