//# init --addresses test=0x42

//# publish
module test::m {
    use std::string::String;
    use moveos_std::table::{Self, Table};
    use moveos_std::context::{Self, Context};
    use moveos_std::object;
    use moveos_std::object::ObjectID;
    use moveos_std::object_ref::{ObjectRef};

    struct KVStore has store, key {
        table: Table<String,vector<u8>>,
    }

    public fun make_kv_store(ctx: &mut Context): KVStore {
        KVStore{
            table: table::new(ctx),
        }
    }

    public fun add(store: &mut KVStore, key: String, value: vector<u8>) {
        table::add(&mut store.table, key, value);
    }

    public fun contains(store: &KVStore, key: String): bool {
        table::contains(&store.table, key)
    }

    public fun borrow(store: &KVStore, key: String): &vector<u8> {
        table::borrow(&store.table, key)
    }

    public fun save_to_object_storage(ctx: &mut Context, kv: KVStore) : ObjectRef<KVStore> {
        context::new_object(ctx, kv)
    }

    public fun borrow_from_object_storage(ctx: &mut Context, object_id: ObjectID): &KVStore {
        let object = context::borrow_object(ctx, object_id);
        object::borrow<KVStore>(object)
    }
}

//# run --signers test
script {
    use std::string;
    use moveos_std::context::{Context};
    use test::m;

    fun main(ctx: &mut Context) {
        let kv = m::make_kv_store(ctx);
        m::add(&mut kv, string::utf8(b"test"), b"value");
        let object_ref = m::save_to_object_storage(ctx, kv);
        std::debug::print(&object_ref);
    }
}

//# run --signers test --args @0xcc48c91b1a0f15813bed988390a2794660ae5dadcd86fdb1b55d4a28d0f74c4d
script {
    use std::string;
    use moveos_std::context::{Context};
    use moveos_std::object::ObjectID;
    use test::m;

    fun main(ctx: &mut Context, object_id: ObjectID) {
        let kv = m::borrow_from_object_storage(ctx, object_id);
        assert!(m::contains(kv, string::utf8(b"test")), 1000);
        let v = m::borrow(kv, string::utf8(b"test"));
        assert!(v == &b"value", 1001);
    }
}