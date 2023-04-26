/// origin source from https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/object.move#L75

/// Move object identifiers
module moveos_std::object {
    use moveos_std::tx_context::{Self, TxContext};

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
        owner: address,
        //TODO define shared and immutable
        //shared: bool,
        //immutable: bool,
        // The value must be the last field
        value: T,
    }

    #[private_generic(T)]
    /// Create a new object, the object is owned by `owner`
    /// The private generic is indicate the T should be defined in the same module as the caller. This is ensured by the verifier.
    public fun new<T: key>(ctx: &mut TxContext, owner: address, value: T): Object<T> {
        let id = tx_context::fresh_address(ctx);
        Object<T>{id: ObjectID{id}, value, owner}
    }

    public(friend) fun new_with_id<T: key>(id: ObjectID, owner: address, value: T): Object<T> {
        Object<T>{id, owner, value}
    }

    //TODO should this require private generic?
    public fun borrow<T>(this: &Object<T>): &T {
        &this.value
    }

    /// Borrow the object mutable value
    public fun borrow_mut<T>(this: &mut Object<T>): &mut T {
        &mut this.value
    }

    public fun id<T>(this: &Object<T>): ObjectID {
        this.id
    }

    public fun owner<T>(this: &Object<T>): address {
        this.owner
    }

    #[private_generic(T)]
    public fun unpack<T>(obj: Object<T>): (ObjectID, address, T) {
        let Object{id, owner, value} = obj;
        (id, owner, value)
    }
 
}
