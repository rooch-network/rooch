//# init --addresses test=0x42

//# publish
module test::m {
    use std::string::String;
    use moveos_std::table::{Self, Table};
    use moveos_std::tx_context::{TxContext};

    struct KVStore has store, key {
        table: Table<String,vector<u8>>,
    }

    public fun make_kv_store(ctx: &mut TxContext): KVStore {
        KVStore{
            table: table::new(ctx),
        }
    }

    public fun add(store: &mut KVStore, key: String, value: vector<u8>) {
        table::add(&mut store.table, key, value);
    }

    public fun contains(store: &mut KVStore, key: String): bool {
        table::contains(&mut store.table, key)
    }

}

//check module exists
//# run --signers test
script {
    use std::string;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::object;
    use moveos_std::object_storage;
    use moveos_std::tx_context;
    use test::m;

    fun main(ctx: &mut StorageContext) {
        let tx_ctx = storage_context::tx_context_mut(ctx);
        let kv = m::make_kv_store(tx_ctx);
        m::add(&mut kv, string::utf8(b"test"), b"value");
        assert!(m::contains(&mut kv, string::utf8(b"test")), 1000);
        let sender = tx_context::sender(tx_ctx);
        let object = object::new(tx_ctx, sender, kv);
        let object_storage = storage_context::object_storage_mut(ctx);
        object_storage::add(object_storage, object);
    }
}

