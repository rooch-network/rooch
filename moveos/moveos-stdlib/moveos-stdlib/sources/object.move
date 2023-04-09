/// origin source from https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/object.move#L75

/// Move object identifiers
module moveos_std::object {
    use moveos_std::tx_context::{Self, TxContext};


    //TODO we need this?
    // marks newly created UIDs from hash
    //native fun record_new_uid(id: address);

    // === transfer functions ===
    //https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/transfer.move#L92

   
    /// Transfer ownership of `obj` to `recipient`.
    public fun transfer<T: store>(obj: Object<T>, recipient: address) {
        // TODO: emit event
        transfer_internal(obj, recipient)
    }

    /// Box style object
    struct Object<T: store> {
        id: address,
        value: T,
    }

    public fun new<T: store>(ctx: &mut TxContext, value: T): Object<T>{
        let id = tx_context::new_object(ctx);
        Object<T>{id, value}
    }

    public fun borrow<T: store>(this: &Object<T>): &T{
        &this.value
    }

    public fun borrow_mut<T: store>(this: &mut Object<T>): &mut T{
        &mut this.value
    }

    public fun remove_object<T: store>(obj: Object<T>): T{
        let Object{id, value} = obj;
        delete_impl(id);
        value
    }

    /// ==== Native functions ===

    /// Freeze `obj`. After freezing `obj` becomes immutable and can no
    /// longer be transferred or mutated.
    //public fun freeze_object<T: store>(obj: Object<T>);

    /// Turn the given object into a mutable shared object that everyone
    /// can access and mutate. This is irreversible, i.e. once an object
    /// is shared, it will stay shared forever.
    //public native fun share_object<T: store>(obj: Object<T>);

    native fun transfer_internal<T: store>(obj: Object<T>, recipient: address);

    // helper for delete
    native fun delete_impl(id: address);
}
