//# init --addresses test=0x42

//# publish
module test::m {
    use std::string::String;
    use moveos_std::table::{Self, Table};
    use moveos_std::context::{Self, Context};
    use moveos_std::object_id::ObjectID;
    use moveos_std::object;

    struct KVStore has store, key {
        table: Table<String,vector<u8>>,
    }

    public fun make_kv_store(ctx: &mut Context): KVStore {
        KVStore{
            table: context::new_table(ctx),
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
        let object = context::new_object(ctx, kv);
        let object_id = object::id(&object);
        object::to_shared(object);
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
        std::debug::print(&110120);
        std::debug::print(&object_id);
    }
}

//# run --signers test --args object:0x20190e28d7f8ca90e1b0d8cbf8d8a07f1de0f985a778138cf879aa9b10955aa4

script {
    use std::string;
    use moveos_std::object::{Self, Object};
    use test::m::{Self, KVStore};

    fun main(kv_object: &Object<KVStore>) {
        let kv = object::borrow(kv_object);
        assert!(m::contains(kv, string::utf8(b"test")), 1000);
        let v = m::borrow(kv, string::utf8(b"test"));
        assert!(v == &b"value", 1001);
    }
}