/// origin source from https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/object.move#L75

/// Move object identifiers
module moveos_std::object {
    use moveos_std::tx_context::{Self, TxContext};
    use moveos_std::any_table::{Self};

    friend moveos_std::account_storage;
    
    /// Invalid access of object, the object is not owned by the signer or the object is not shared or immutable
    const EInvalidAccess: u64 = 0;
   
    struct ObjectID has store, copy, drop {
        //TODO should use u256 to replace address?
        id: address,
    }

    public(friend) fun address_to_object_id(address: address): ObjectID {
        ObjectID{id: address}
    }

    /// Box style object
    /// The object can not be copied, droped, only can be consumed by `add`
    struct Object<T> {
        id: ObjectID,
        value: T,
        owner: address,
        //TODO define shared and immutable
        //shared: bool,
        //immutable: bool,
    }

    #[private_generic(T)]
    /// Create a new object, the object is owned by `owner`
    /// The private generic is indicate the T should be defined in the same module as the caller. This is ensured by the verifier.
    public fun new<T: key>(ctx: &mut TxContext, owner: address, value: T): Object<T>{
        let id = tx_context::derive_id(ctx);
        Object<T>{id: ObjectID{id}, value, owner}
    }

    public(friend) fun new_with_id<T: key>(id: ObjectID, owner: address, value: T): Object<T>{
        Object<T>{id, value, owner}
    }

    //TODO should this require private generic?
    public fun borrow_value<T>(this: &Object<T>): &T{
        &this.value
    }

    /// Borrow the object mutable value
    public fun borrow_value_mut<T>(this: &mut Object<T>): &mut T{
        &mut this.value
    }

    /// ==== Object Store ====

    struct ObjectStore has store {
        table: any_table::Table<ObjectID>,
    }

    #[private_generic(T)]
    public fun borrow<T: key>(this: &ObjectStore, object_id: ObjectID): &Object<T>{
        any_table::borrow(&this.table, object_id)
    }

    #[private_generic(T)]
    public fun borrow_mut<T: key>(this: &mut ObjectStore, object_id: ObjectID): &mut Object<T>{
        any_table::borrow_mut(&mut this.table, object_id)
    }
    
    #[private_generic(T)]
    /// Remove object from object store, only the owner can move the object
    public fun remove<T: key>(this: &mut ObjectStore, object_id: ObjectID): Object<T>{
        any_table::remove(&mut this.table, object_id)
    }

    #[private_generic(T)]
    public fun unpack<T>(obj: Object<T>): (ObjectID, T, address) {
        let Object{id, value, owner} = obj;
        (id, value, owner)
    }

    
    #[private_generic(T)]
    /// Add object to object store
    public fun add<T: key>(this: &mut ObjectStore, obj: Object<T>) {
        any_table::add(&mut this.table, obj.id, obj);
    }

    public fun contains(this: &mut ObjectStore, object_id: ObjectID): bool{
        any_table::contains(&this.table, object_id)
    }
 
}
