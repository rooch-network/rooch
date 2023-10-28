//# init --addresses test=0x42

//# publish
module test::m {
    use std::string::String;
    use moveos_std::table::{Self, Table};
    use moveos_std::context::{Self, Context};
    use moveos_std::object::ObjectID;
    use moveos_std::object_ref;

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

    public fun save_to_object_storage(ctx: &mut Context, kv: KVStore) : ObjectID {
        let object_ref = context::new_object(ctx, kv);
        let object_id = object_ref::id(&object_ref);
        object_ref::to_permanent(object_ref);
        object_id
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
        let object_id = m::save_to_object_storage(ctx, kv);
        std::debug::print(&object_id);
    }
}

//# run --signers test --args @0x1a2c876ea44c751aedab69ef139181114c79abf4fb8bca363b66969218e7d815
script {
    use std::string;
    use moveos_std::object_ref::{Self, ObjectRef};
    use test::m::{Self, KVStore};

    fun main(kv_object: &ObjectRef<KVStore>) {
        let kv = object_ref::borrow(kv_object);
        assert!(m::contains(kv, string::utf8(b"test")), 1000);
        let v = m::borrow(kv, string::utf8(b"test"));
        assert!(v == &b"value", 1001);
    }
}