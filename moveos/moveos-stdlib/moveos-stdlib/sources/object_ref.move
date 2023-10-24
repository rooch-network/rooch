// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::object_ref {

    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::raw_table;

    friend moveos_std::context;

    /// ObjectRef<T> is a reference of the Object<T>
    /// It likes ObjectID, but it contains the type information of the object.
    /// TODO should we support drop?
    struct ObjectRef<phantom T> has key, store, drop {
        id: ObjectID,
    }

    #[private_generics(T)]
    /// Get the object reference
    /// This function is protected by private_generics, so it can only be called by the module which defined the T
    /// Note: new ObjectRef need the &mut Object<T>, because the ObjectRef can borrow mutable value from the object
    public fun new<T: key>(object: &mut Object<T>) : ObjectRef<T> {
        //TODO should we track the reference count?
        new_internal(object)
    }

    public(friend) fun new_internal<T: key>(object: &mut Object<T>) : ObjectRef<T> {
        ObjectRef {
            id: object::id(object),
        }
    }

    /// Borrow the object value
    public fun borrow<T: key>(self: &ObjectRef<T>): &T {
        let obj = raw_table::borrow_from_global<T>(&self.id);
        object::internal_borrow(obj)
    }

    /// Borrow the object mutable value
    public fun borrow_mut<T: key>(self: &mut ObjectRef<T>): &mut T {
        let obj = raw_table::borrow_mut_from_global<T>(&self.id);
        object::internal_borrow_mut(obj)
    }

    /// Remove the object from the global storage, and return the object value
    public fun remove<T: key>(self: ObjectRef<T>) : T {
        let ObjectRef{id} = self;
        let object = raw_table::remove_from_global(&id);
        let (_id, _owner, value) = object::unpack_internal(object);
        value
    }

    public fun id<T>(self: &ObjectRef<T>): ObjectID {
        self.id
    }

    public fun owner<T: key>(self: &ObjectRef<T>): address {
        let obj = raw_table::borrow_from_global<T>(&self.id);
        object::owner(obj)
    }

    /// Check if the object is still exist in the global storage
    public fun exist_object<T: key>(self: &ObjectRef<T>): bool {
        raw_table::contains_global(&self.id)
    }

    /// Convert the ObjectRef to ObjectID
    public fun into_id<T: key>(self: ObjectRef<T>): ObjectID {
        let ObjectRef {id} = self;
        id
    }
}
