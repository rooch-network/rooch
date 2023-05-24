module rooch_examples::something {
    use moveos_std::object::{Self, Object};
    use moveos_std::object_id::{ObjectID};
    use moveos_std::object_storage;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::tx_context;

    friend rooch_examples::something_aggregate;
    friend rooch_examples::something_do_logic;

    struct SomethingProperties has key {
        i: u32,
        j: u128,
    }

    /// get object id
    public fun id(obj: &Object<SomethingProperties>): ObjectID {
        object::id(obj)
    }

    /// get property 'i' from object
    public fun i(obj: &Object<SomethingProperties>): u32 {
        object::borrow(obj).i
    }

    /// set property 'i' of object
    public(friend) fun set_i(obj: &mut Object<SomethingProperties>, i: u32) {
        object::borrow_mut(obj).i = i;
    }

    public fun j(obj: &Object<SomethingProperties>): u128 {
        object::borrow(obj).j
    }

    public(friend) fun set_j(obj: &mut Object<SomethingProperties>, j: u128) {
        object::borrow_mut(obj).j = j;
    }

    public(friend) fun create_something(
        storage_ctx: &mut StorageContext,
        i: u32,
        j: u128,
    ): Object<SomethingProperties> {
        let tx_ctx = storage_context::tx_context_mut(storage_ctx);
        let owner = tx_context::sender(tx_ctx);
        let obj = object::new(
            tx_ctx,
            owner,
            new_something_properties(i, j),
        );
        obj
    }

    fun new_something_properties(
        i: u32,
        j: u128,
    ): SomethingProperties {
        SomethingProperties {
            i,
            j,
        }
    }

    public(friend) fun add_something(storage_ctx: &mut StorageContext, obj: Object<SomethingProperties>) {
        let obj_store = storage_context::object_storage_mut(storage_ctx);
        object_storage::add(obj_store, obj);
    }

    public(friend) fun remove_something(
        storage_ctx: &mut StorageContext,
        obj_id: ObjectID
    ): Object<SomethingProperties> {
        let obj_store = storage_context::object_storage_mut(storage_ctx);
        object_storage::remove<SomethingProperties>(obj_store, obj_id)
    }
}
