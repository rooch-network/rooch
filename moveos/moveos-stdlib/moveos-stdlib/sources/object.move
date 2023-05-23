/// Move Object
/// The Object is a box style Object
/// The differents with the Object in [Sui](https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/object.move#L75):
/// 1. The Object is a struct in Move
/// 2. The Object is a use case for the Hot Potato pattern in Move. Objects do not have any ability, so they cannot be drop, copy, or store, and can only be handled by ObjectStorage API after creation.
/// More details about the Object can be found in [Storage Abstraction](https://github.com/rooch-network/rooch/blob/main/docs/design/storage_abstraction.md)
module moveos_std::object {
    use moveos_std::tx_context::{Self, TxContext};
    use std::debug;
    use moveos_std::object_id::ObjectID;

    friend moveos_std::account_storage;
    
    /// Invalid access of object, the object is not owned by the signer or the object is not shared or immutable
    const EInvalidAccess: u64 = 0;
   
    /// Box style object
    /// The object can not be copied, droped, only can be consumed by ObjectStorage API.
    struct Object<T> {
        id: ObjectID,
        owner: address,
        //TODO define shared and immutable
        //shared: bool,
        //immutable: bool,
        // The value must be the last field
        value: T,
    }

    #[private_generics(T)]
    /// Create a new object, the object is owned by `owner`
    /// The private generic is indicate the T should be defined in the same module as the caller. This is ensured by the verifier.
    public fun new<T: key>(ctx: &mut TxContext, owner: address, value: T): Object<T> {
        let id = tx_context::fresh_object_id(ctx);
        let obj = Object<T>{id, value, owner};
        //TODO after add event, then remove the debug info
        debug::print(&obj);
        obj
    }

    public(friend) fun new_with_id<T: key>(id: ObjectID, owner: address, value: T): Object<T> {
        Object<T>{id, owner, value}
    }

    #[private_generics(T)]
    //TODO should this require private generic?
    public fun borrow<T>(this: &Object<T>): &T {
        &this.value
    }

    #[private_generics(T)]
    /// Borrow the object mutable value
    public fun borrow_mut<T>(this: &mut Object<T>): &mut T {
        &mut this.value
    }

    #[private_generics(T)]
    /// Transfer object to recipient
    public fun transfer<T: key>(this: &mut Object<T>, recipient: address) {
        this.owner = recipient;
    }

    public fun id<T>(this: &Object<T>): ObjectID {
        this.id
    }

    public fun owner<T>(this: &Object<T>): address {
        this.owner
    }

    #[private_generics(T)]
    public fun unpack<T>(obj: Object<T>): (ObjectID, address, T) {
        let Object{id, owner, value} = obj;
        (id, owner, value)
    }
 
}
