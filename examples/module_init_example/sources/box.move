module rooch_examples::box {
    use std::string::{String};
    use moveos_std::object::{Self, Object};
    use moveos_std::object_id::ObjectID;
    use moveos_std::object_storage;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::tx_context;

    friend rooch_examples::box_fun;
    friend rooch_examples::box_friend;

    struct Box has key {
        name: String,
        count: u128,
    }

    /// get object id
    public fun id(obj: &Object<Box>): ObjectID {
        object::id(obj)
    }

    /// get property 'name' from object
    public fun name(obj: &Object<Box>): String {
        object::borrow(obj).name
    }

    /// set property 'name' of object
    public(friend) fun set_name(obj: &mut Object<Box>, name: String) {
        object::borrow_mut(obj).name = name;
    }

    public fun count(obj: &Object<Box>): u128 {
        object::borrow(obj).count
    }

    public(friend) fun set_count(obj: &mut Object<Box>, count: u128) {
        object::borrow_mut(obj).count = count;
    }

    public(friend) fun create_box(
        storage_ctx: &mut StorageContext,
        name: String,
        count: u128,
    ): Object<Box> {
        let tx_ctx = storage_context::tx_context_mut(storage_ctx);
        let owner = tx_context::sender(tx_ctx);
        let obj = object::new(
            tx_ctx,
            owner,
            new_box(name, count),
        );
        obj
    }

    fun new_box(
        name: String,
        count: u128,
    ): Box {
        Box {
            name,
            count,
        }
    }

    public(friend) fun add_box(storage_ctx: &mut StorageContext, obj: Object<Box>) {
        let obj_store = storage_context::object_storage_mut(storage_ctx);
        object_storage::add(obj_store, obj);
    }

    public(friend) fun remove_box(
        storage_ctx: &mut StorageContext,
        obj_id: ObjectID
    ): Object<Box> {
        let obj_store = storage_context::object_storage_mut(storage_ctx);
        object_storage::remove<Box>(obj_store, obj_id)
    }
}
