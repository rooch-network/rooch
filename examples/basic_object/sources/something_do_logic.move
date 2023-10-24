// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::something_do_logic {
    use std::string::String;
    use moveos_std::object_ref::ObjectRef;
    use moveos_std::context::Context;
    use rooch_examples::something::{Self, SomethingProperties};

    friend rooch_examples::something_aggregate;

    public(friend) fun do_something(obj: ObjectRef<SomethingProperties>): ObjectRef<SomethingProperties> {
        let i = something::i(&obj) + 1;
        something::set_i(&mut obj, i);
        let j = something::j(&obj) + 1;
        something::set_j(&mut obj, j);
        obj
    }

    public(friend) fun add_foo_table_item(
        ctx: &mut Context,
        obj: ObjectRef<SomethingProperties>,
        key: String,
        val: String
    ): ObjectRef<SomethingProperties> {
        something::add_foo_table_item(ctx, &mut obj, key, val);
        obj
    }
}
