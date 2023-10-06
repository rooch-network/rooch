module rooch_examples::box {
    use std::string::{String};
    use moveos_std::object::{Self, Object};
    use moveos_std::object_id::ObjectID;
    use moveos_std::context::{Self, Context};

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
        storage_ctx: &mut Context,
        name: String,
        count: u128,
    ): Object<Box> {
        let owner = context::sender(storage_ctx);
        let tx_ctx = context::tx_context_mut(storage_ctx);
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

    public(friend) fun add_box(storage_ctx: &mut Context, obj: Object<Box>) {
        context::add_object(storage_ctx, obj);
    }

    public(friend) fun remove_box(
        storage_ctx: &mut Context,
        obj_id: ObjectID
    ): Object<Box> {
        context::remove_object<Box>(storage_ctx, obj_id)
    }
}
