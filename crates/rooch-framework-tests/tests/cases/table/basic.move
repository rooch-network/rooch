//# init --addresses test=0x42

//# publish
module test::m {
    use std::string::String;
    use moveos_std::table::{Self, Table};
    
    use moveos_std::object::ObjectID;
    use moveos_std::object;

    struct KVStore has store, key {
        table: Table<String,vector<u8>>,
    }

    public fun make_kv_store(): KVStore {
        KVStore{
            table: table::new(),
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

    public fun save_to_object_storage(kv: KVStore) : ObjectID {
        let object = object::new(kv);
        let object_id = object::id(&object);
        object::to_shared(object);
        object_id
    }
}

//# run --signers test
script {
    use std::string;
    
    use test::m;

    fun main() {
        let kv = m::make_kv_store();
        m::add(&mut kv, string::utf8(b"test"), b"value");
        let object_id = m::save_to_object_storage(kv);
        std::debug::print(&110120);
        std::debug::print(&object_id);
    }
}

//# run --signers test --args @0x3a9b55b929a30c88bac41ad553a2ef54bfdbc05b795619189b3e7f435cbd0d98

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