// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::something {
    use std::string::String;

    use moveos_std::event;
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::context::{Self, Context};
    use moveos_std::table::{Self, Table};

    friend rooch_examples::something_aggregate;
    friend rooch_examples::something_do_logic;

    struct SomethingProperties has key {
        i: u32,
        j: u128,
        fooTable: Table<String, String>,
        barTable: Table<u8, u128>,
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

    struct SomethingCreated {
        obj_id: ObjectID,
        i: u32,
        j: u128,
    }

    struct KeyValuePair<K, V> has store {
        key: K,
        value: V,
    }

    struct BarTableItemAdded {
        item: KeyValuePair<u8, u128>
    }

    struct FooTableItemAdded {
        item: KeyValuePair<String, String>
    }

    public(friend) fun create_something(
        ctx: &mut Context,
        i: u32,
        j: u128,
    ): Object<SomethingProperties> {
        let value = new_something_properties(ctx, i, j);
        let obj = context::new_object(
            ctx,
            value,
        );
        event::emit(ctx, SomethingCreated {
            obj_id: object::id(&obj),
            i,
            j,
        });
        obj
    }

    fun new_something_properties(
        ctx: &mut Context,
        i: u32,
        j: u128,
    ): SomethingProperties {
        let ps = SomethingProperties {
            i,
            j,
            fooTable: table::new(ctx),
            barTable: table::new(ctx),
        };
        add_bar_table_item(ctx, &mut ps.barTable, 0, 0);
        add_bar_table_item(ctx, &mut ps.barTable, 1, 1);
        add_bar_table_item(ctx, &mut ps.barTable, 2, 2);
        ps
    }

    fun add_bar_table_item(ctx: &mut Context,
                           table: &mut Table<u8, u128>,
                           key: u8,
                           val: u128
    ) {
        table::add(table, key, val);
        event::emit(ctx, BarTableItemAdded {
            item: KeyValuePair {
                key,
                value: val,
            }
        });
    }

    public(friend) fun add_foo_table_item(
        ctx: &mut Context,
        obj: &mut Object<SomethingProperties>,
        key: String,
        val: String
    ) {
        table::add(&mut object::borrow_mut(obj).fooTable, key, val);
        event::emit(ctx, FooTableItemAdded {
            item: KeyValuePair {
                key,
                value: val,
            }
        });
    }

    public(friend) fun add_something(ctx: &mut Context, obj: Object<SomethingProperties>) {
        context::add_object(ctx, obj);
    }

    public(friend) fun remove_something(
        ctx: &mut Context,
        obj_id: ObjectID
    ): Object<SomethingProperties> {
        context::remove_object<SomethingProperties>(ctx, obj_id)
    }
}
