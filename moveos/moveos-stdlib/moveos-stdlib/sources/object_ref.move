// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::object_ref {

    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::raw_table;

    friend moveos_std::context;

    ///TODO rename to Object
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

    public(friend) fun as_ref<T: key>(object: &Object<T>) : &ObjectRef<T>{
        as_ref_inner<ObjectRef<T>>(object::id(object))
    }
    public(friend) fun as_mut_ref<T: key>(object: &mut Object<T>) : &mut ObjectRef<T>{
        as_mut_ref_inner<ObjectRef<T>>(object::id(object))
    }

    /// Convert the ObjectID to &T or &mut T
    /// The caller must ensure the T only has one `ObjectID` field, such as `ObjectRef<T>` or `Table<K,V>`, or `TypeTable`.
    native fun as_ref_inner<T>(object_id: ObjectID): &T;
    native fun as_mut_ref_inner<T>(object_id: ObjectID): &mut T;

    #[private_generics(T)]
    /// Drop the ObjectRef<T>, and make the object to be a external object.
    /// The external object can be used as transaction argument
    public fun to_external<T: key>(self: ObjectRef<T>){
        let ObjectRef{id} = self;
        let obj = raw_table::borrow_mut_from_global<T>(&id);
        object::set_external(obj, true);
    }

    /// Borrow the object value
    public fun borrow<T: key + store>(self: &ObjectRef<T>): &T {
        let obj = raw_table::borrow_from_global<T>(&self.id);
        object::borrow(obj)
    }

    #[private_generics(T)]
    public fun borrow_extend<T: key>(self: &ObjectRef<T>): &T {
        let obj = raw_table::borrow_from_global<T>(&self.id);
        object::borrow(obj)
    }

    /// Borrow the object mutable value
    public fun borrow_mut<T: key + store>(self: &mut ObjectRef<T>): &mut T {
        let obj = raw_table::borrow_mut_from_global<T>(&self.id);
        object::borrow_mut(obj)
    }

    #[private_generics(T)]
    public fun borrow_mut_extend<T: key>(self: &mut ObjectRef<T>): &mut T {
        let obj = raw_table::borrow_mut_from_global<T>(&self.id);
        object::borrow_mut(obj)
    } 

    /// Remove the object from the global storage, and return the object value
    public fun remove<T: key + store>(self: ObjectRef<T>) : T {
        let ObjectRef{id} = self;
        let object = raw_table::remove_from_global(&id);
        let (_id, _owner, value) = object::unpack(object);
        value
    }

    #[private_generics(T)]
    public fun remove_extend<T: key>(self: ObjectRef<T>) : T {
        let ObjectRef{id} = self;
        let object = raw_table::remove_from_global(&id);
        let (_id, _owner, value) = object::unpack(object);
        value
    }

    public fun transfer<T: key + store>(self: &mut ObjectRef<T>, new_owner: address) {
        let obj = raw_table::borrow_mut_from_global<T>(&self.id);
        object::set_owner(obj, new_owner);
    }

    #[private_generics(T)]
    public fun transfer_extend<T: key>(self: &mut ObjectRef<T>, new_owner: address) {
        let obj = raw_table::borrow_mut_from_global<T>(&self.id);
        object::set_owner(obj, new_owner);
    }

    public fun id<T>(self: &ObjectRef<T>): ObjectID {
        self.id
    }

    public fun owner<T: key>(self: &ObjectRef<T>): address {
        let obj = raw_table::borrow_from_global<T>(&self.id);
        object::owner(obj)
    }

}
