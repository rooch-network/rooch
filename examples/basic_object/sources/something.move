module rooch_examples::something {
    use std::string::String;

    use moveos_std::event;
    use moveos_std::object::{Self, Object};
    use moveos_std::object_id::ObjectID;
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

    struct BarTableItemAdded has key {
        item: KeyValuePair<u8, u128>
    }

    public(friend) fun create_something(
        storage_ctx: &mut Context,
        i: u32,
        j: u128,
    ): Object<SomethingProperties> {
        let value = new_something_properties(storage_ctx, i, j);
        let owner = context::sender(storage_ctx);
        let tx_ctx = context::tx_context_mut(storage_ctx);
        let obj = object::new(
            tx_ctx,
            owner,
            value,
        );
        event::emit(storage_ctx, SomethingCreated {
            obj_id: object::id(&obj),
            i,
            j,
        });
        obj
    }

    fun new_something_properties(
        storage_ctx: &mut Context,
        i: u32,
        j: u128,
    ): SomethingProperties {
        let ps = SomethingProperties {
            i,
            j,
            fooTable: table::new(storage_ctx),
            barTable: table::new(storage_ctx),
        };
        add_bar_table_item(storage_ctx, &mut ps.barTable, 0, 0);
        add_bar_table_item(storage_ctx, &mut ps.barTable, 1, 1);
        add_bar_table_item(storage_ctx, &mut ps.barTable, 2, 2);
        ps
    }

    fun add_bar_table_item(storage_ctx: &mut Context,
                           table: &mut Table<u8, u128>,
                           key: u8,
                           val: u128
    ) {
        table::add(table, key, val);
        event::emit(storage_ctx, BarTableItemAdded {
            item: KeyValuePair {
                key,
                value: val,
            }
        });
    }

    public(friend) fun add_foo_table_item(
        storage_ctx: &mut Context,
        obj: &mut Object<SomethingProperties>,
        key: String,
        val: String
    ) {
        table::add(&mut object::borrow_mut(obj).fooTable, key, val);
        // event::emit(storage_ctx, FooTableItemAdded {
        //     key
        // });
        let _ = storage_ctx;
    }

    public(friend) fun add_something(storage_ctx: &mut Context, obj: Object<SomethingProperties>) {
        context::add_object(storage_ctx, obj);
    }

    public(friend) fun remove_something(
        storage_ctx: &mut Context,
        obj_id: ObjectID
    ): Object<SomethingProperties> {
        context::remove_object<SomethingProperties>(storage_ctx, obj_id)
    }
}
