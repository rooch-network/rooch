//# init --addresses test=0x42

//# publish
module test::m {
    use std::string::String;
    use std::option;

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

    public fun list_field_keys(store: &KVStore): vector<address> {
        table::list_field_keys(&store.table, option::none(), 10000)
    }

    public fun length(store: &KVStore): u64 {
        table::length(&store.table)
    }
}

//# run --signers test
script {
    use std::string;

    use test::m;

    fun main() {
        let kv = m::make_kv_store();
        m::add(&mut kv, string::utf8(b"test"), b"value");
        m::add(&mut kv, string::utf8(b"test2"), b"value2");
        m::add(&mut kv, string::utf8(b"test3"), b"value3");

        let keys = m::list_field_keys(&kv);
        std::debug::print(&keys);

        let object_id = m::save_to_object_storage(kv);
        std::debug::print(&110120);
        std::debug::print(&object_id);
    }
}

//# run --signers test --args object:0x79c5dcbfa65443bac7cb2bea1b85bbc0dfd82ab17969c5d620ac6261a458735f

script {
    use std::string;
    use std::vector;
    use moveos_std::object::{Self, Object};
    use test::m::{Self, KVStore};

    fun main(kv_object: &Object<KVStore>) {
        let kv = object::borrow(kv_object);
        assert!(m::contains(kv, string::utf8(b"test")), 1000);

        let keys = m::list_field_keys(kv);
        assert!(vector::length(&keys) == 3, 1001);

        let size = m::length(kv);
        assert!(size == 3, 1002);

        let v = m::borrow(kv, string::utf8(b"test"));
        assert!(v == &b"value", 1003);
    }
}

//# run --signers test --args object:0x79c5dcbfa65443bac7cb2bea1b85bbc0dfd82ab17969c5d620ac6261a458735f

script {
    use std::string;
    use std::vector;
    use moveos_std::object::{Self, Object};
    use test::m::{Self, KVStore};

    fun main(kv_object: &mut Object<KVStore>) {
        let kv = object::borrow_mut(kv_object);
        assert!(m::contains(kv, string::utf8(b"test")), 1000);

        m::add(kv, string::utf8(b"test4"), b"value4");
        m::add(kv, string::utf8(b"test5"), b"value5");

        let keys = m::list_field_keys(kv);
        assert!(vector::length(&keys) == 3, 1001);

        let size = m::length(kv);
        assert!(size == 5, 1002);

        let v = m::borrow(kv, string::utf8(b"test4"));
        assert!(v == &b"value4", 1003);

        let v = m::borrow(kv, string::utf8(b"test5"));
        assert!(v == &b"value5", 1004);
    }
}
