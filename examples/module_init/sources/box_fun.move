// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::box_fun {
    use std::string::{Self, String};
    use moveos_std::object_id::ObjectID;
    use moveos_std::context::{Context};
    use rooch_examples::box;
    use rooch_examples::box_friend;
    use std::debug;

    struct Container has key {
        // name: String,
        count: u128,
    }

    // for test
    fun init(_ctx: &mut Context) {
        debug::print<String>(&string::utf8(b"module box_fun init finish"));
    }

    public entry fun create_box(
        stoage_ctx: &mut Context,
        // name: String,
        count: u128,
    ) {
        let name = string::utf8(b"name");
        let obj = box::create_box(stoage_ctx, name ,count);
        box::add_box(stoage_ctx, obj);
    }

    public entry fun remove_box_and_update(
        ctx: &mut Context,
        object_id: ObjectID,
    ) {
        let obj = box::remove_box(ctx, object_id);
        let update_obj = box_friend::change_box(obj);
        box::add_box(ctx, update_obj);
    }
}
