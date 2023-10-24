// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::object_ref {

    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::raw_table;

    friend moveos_std::context;

    /// ObjectRef<T> is a reference of the Object<T>
    /// It likes ObjectID, but it contains the type information of the object.
    struct ObjectRef<phantom T> has key, store {
        id: ObjectID,
    }

    public(friend) fun new_internal<T: key>(object: &mut Object<T>) : ObjectRef<T> {
        ObjectRef {
            id: object::id(object),
        }
    }

    #[private_generics(T)]
    public fun to_user_owner<T: key>(self: ObjectRef<T>, owner: address){
        let ObjectRef{id} = self;
        let obj = raw_table::borrow_mut_from_global<T>(&id);
        object::set_owner(obj, owner);
    }

    /// Borrow the object value
    public fun borrow<T: key + store>(self: &ObjectRef<T>): &T {
        let obj = raw_table::borrow_from_global<T>(&self.id);
        object::internal_borrow(obj)
    }

    #[private_generics(T)]
    public fun borrow_extend<T: key>(self: &ObjectRef<T>): &T {
        let obj = raw_table::borrow_from_global<T>(&self.id);
        object::internal_borrow(obj)
    }

    /// Borrow the object mutable value
    public fun borrow_mut<T: key + store>(self: &mut ObjectRef<T>): &mut T {
        let obj = raw_table::borrow_mut_from_global<T>(&self.id);
        object::internal_borrow_mut(obj)
    }

    #[private_generics(T)]
    public fun borrow_mut_extend<T: key>(self: &mut ObjectRef<T>): &mut T {
        let obj = raw_table::borrow_mut_from_global<T>(&self.id);
        object::internal_borrow_mut(obj)
    } 

    /// Remove the object from the global storage, and return the object value
    public fun remove<T: key + store>(self: ObjectRef<T>) : T {
        let ObjectRef{id} = self;
        let object = raw_table::remove_from_global(&id);
        let (_id, _owner, value) = object::unpack_internal(object);
        value
    }

    #[private_generics(T)]
    public fun remove_extend<T: key>(self: ObjectRef<T>) : T {
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

}
