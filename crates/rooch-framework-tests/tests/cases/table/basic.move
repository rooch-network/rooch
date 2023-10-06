//# init --addresses test=0x42

//# publish
module test::m {
    use std::string::String;
    use moveos_std::table::{Self, Table};
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::object;
    use moveos_std::object_id::{ObjectID};

    struct KVStore has store, key {
        table: Table<String,vector<u8>>,
    }

    public fun make_kv_store(ctx: &mut StorageContext): KVStore {
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

    public fun save_to_object_storage(ctx: &mut StorageContext, kv: KVStore) : ObjectID {
        let sender = storage_context::sender(ctx);        
        let tx_ctx = storage_context::tx_context_mut(ctx);
        let object = object::new(tx_ctx, sender, kv);
        let object_id = object::id(&object);
        storage_context::add_object(ctx, object);
        object_id
    }

    public fun borrow_from_object_storage(ctx: &mut StorageContext, object_id: ObjectID): &KVStore {
        let object = storage_context::borrow_object(ctx, object_id);
        object::borrow<KVStore>(object)
    }
}

//# run --signers test
script {
    use std::string;
    use moveos_std::storage_context::{StorageContext};
    use test::m;

    fun main(ctx: &mut StorageContext) {
        let kv = m::make_kv_store(ctx);
        m::add(&mut kv, string::utf8(b"test"), b"value");
        let object_id = m::save_to_object_storage(ctx, kv);
        std::debug::print(&object_id);
    }
}

//# run --signers test --args @0xc1c9be4d48a51830d3af349c14cbd3c4614fdc9e998e7ed9b14bd34ea483bdf9
script {
    use std::string;
    use moveos_std::storage_context::{StorageContext};
    use moveos_std::object_id::{ObjectID};
    use test::m;

    fun main(ctx: &mut StorageContext, object_id: ObjectID) {
        let kv = m::borrow_from_object_storage(ctx, object_id);
        assert!(m::contains(kv, string::utf8(b"test")), 1000);
        let v = m::borrow(kv, string::utf8(b"test"));
        assert!(v == &b"value", 1001);
    }
}