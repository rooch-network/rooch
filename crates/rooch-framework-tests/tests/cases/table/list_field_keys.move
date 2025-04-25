//# init --addresses test=0x42

//# publish
module test::m {
    use std::string::String;
    use std::option;

    use moveos_std::table::{Self, Table, Iterator};

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

    public fun list_field_keys(store: &KVStore): Iterator<String, vector<u8>> {
        table::list_field_keys(&store.table, option::none(), 10000)
    }

    public fun list_field_keys_with_name(store: &KVStore, key: String, limit: u64): Iterator<String, vector<u8>> {
        table::list_field_keys(&store.table, option::some(key), limit)
    }

    public fun field_keys_len(iterator: &Iterator<String, vector<u8>>): u64 {
        table::field_keys_len(iterator)
    }

    public fun next(iterator: &mut Iterator<String, vector<u8>>): (String, vector<u8>) {
        let (k, v) = table::next(iterator);
        (*k, *v)
    }

    public fun next_mut(iterator: &mut Iterator<String, vector<u8>>): (String, vector<u8>) {
        let (k, v) = table::next_mut(iterator);
        (*k, *v)
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

        let iter = m::list_field_keys(&kv);
        std::debug::print(&iter);
        assert!(m::field_keys_len(&iter) == 3, 1001);

        m::add(&mut kv, string::utf8(b"test4"), b"value4");
        let iter = m::list_field_keys(&kv);
        std::debug::print(&iter);
        assert!(m::field_keys_len(&iter) == 4, 1002);

        let (k, v) = m::next_mut(&mut iter);
        std::debug::print(&k);
        std::debug::print(&string::utf8(v));
        v = b"new_value";
        std::debug::print(&string::utf8(v));

        let (k, v) = m::next(&mut iter);
        std::debug::print(&k);
        std::debug::print(&string::utf8(v));

        let (k, v) = m::next(&mut iter);
        std::debug::print(&k);
        std::debug::print(&string::utf8(v));

        let (k, v) = m::next(&mut iter);
        std::debug::print(&k);
        std::debug::print(&string::utf8(v));

        let iter = m::list_field_keys_with_name(&kv, string::utf8(b"test"), 2);
        std::debug::print(&iter);
        assert!(m::field_keys_len(&iter) == 2, 1003);

        let (k, v) = m::next(&mut iter);
        std::debug::print(&k);
        std::debug::print(&string::utf8(v));

        let (k, v) = m::next(&mut iter);
        std::debug::print(&k);
        std::debug::print(&string::utf8(v));

        let object_id = m::save_to_object_storage(kv);
        std::debug::print(&110120);
        std::debug::print(&object_id);
    }
}

//# run --signers test --args object:0x9a14fa31b2a9bdd355f0b059f32c010c471a9f52f13af5b7d11640fd02ff2e2c

script {
    use std::string;
    use moveos_std::object::{Self, Object};
    use test::m::{Self, KVStore};

    fun main(kv_object: &Object<KVStore>) {
        let kv = object::borrow(kv_object);
        assert!(m::contains(kv, string::utf8(b"test")), 1000);

        let iter = m::list_field_keys(kv);
        assert!(m::field_keys_len(&iter) == 4, 1001);

        let size = m::length(kv);
        assert!(size == 4, 1002);

        let v = m::borrow(kv, string::utf8(b"test"));
        assert!(v == &b"value", 1003);
    }
}
