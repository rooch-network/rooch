// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

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
        ctx: &mut Context,
        name: String,
        count: u128,
    ): Object<Box> {
        let obj = context::new_object(
            ctx,
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

    public(friend) fun add_box(ctx: &mut Context, obj: Object<Box>) {
        context::add_object(ctx, obj);
    }

    public(friend) fun remove_box(
        ctx: &mut Context,
        obj_id: ObjectID
    ): Object<Box> {
        context::remove_object<Box>(ctx, obj_id)
    }
}
