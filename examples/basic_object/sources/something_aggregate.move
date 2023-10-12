// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::something_aggregate {
    use std::string::String;

    use moveos_std::object::ObjectID;
    use moveos_std::context::Context;
    use rooch_examples::something;
    use rooch_examples::something_do_logic;

    public entry fun create_something(
        stoage_ctx: &mut Context,
        i: u32,
        j: u128,
    ) {
        let obj = something::create_something(stoage_ctx, i, j);
        something::add_something(stoage_ctx, obj);
    }

    public entry fun add_foo_table_item(
        ctx: &mut Context,
        object_id: ObjectID,
        key: String,
        val: String,
    ) {
        let obj = something::remove_something(ctx, object_id);
        let update_obj = something_do_logic::add_foo_table_item(ctx, obj, key, val);
        something::add_something(ctx, update_obj);
    }

    public entry fun remove_do_something_add(
        ctx: &mut Context,
        object_id: ObjectID,
    ) {
        let obj = something::remove_something(ctx, object_id);
        let update_obj = something_do_logic::do_something(obj);
        something::add_something(ctx, update_obj);
    }
}
