// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::object_ref {

    use moveos_std::object_id::ObjectID;
    use moveos_std::object::{Self, Object};
    use moveos_std::raw_table;

    /// ObjectRef<T> is a reference of the Object<T>
    /// It likes ObjectID, but it contains the type information of the object.
    /// TODO should we support drop?
    struct ObjectRef<phantom T> has key, store, copy, drop {
        id: ObjectID,
    }

    #[private_generics(T)]
    /// Get the object reference
    public fun new<T: key>(object: &Object<T>) : ObjectRef<T> {
        //TODO should we track the reference count?
        ObjectRef {
            id: object::id(object),
        }
    }

    public(friend) fun new_with_id<T>(id: ObjectID): ObjectRef<T> {
        ObjectRef {
            id: id,
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

    public fun id<T>(self: &ObjectRef<T>): ObjectID {
        self.id
    }

    public fun owner<T: key>(self: &ObjectRef<T>): address {
        let obj = raw_table::borrow_from_global<T>(&self.id);
        object::owner(obj)
    }

    /// Check if the object is still contains in the global storage
    public fun contains<T: key>(self: &ObjectRef<T>): bool {
        raw_table::contains_global(&self.id)
    }

    /// Convert the ObjectRef to ObjectID
    public fun into_id<T: key>(self: ObjectRef<T>): ObjectID {
        let ObjectRef {id} = self;
        id
    }
}
