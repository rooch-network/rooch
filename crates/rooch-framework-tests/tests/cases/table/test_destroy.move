//# init --addresses test=0x42

//# publish
module test::m {
    use std::string::String;
    use moveos_std::table::{Self, Table};
    
    use moveos_std::account;

    struct KVStore has key {
        table: Table<String,vector<u8>>,
    }

    public fun make_kv_store(): KVStore{
        KVStore{
            table: table::new(),
        }
    }

    public fun add(store: &mut KVStore, key: String, value: vector<u8>) {
        table::add(&mut store.table, key, value);
    }

    public fun remove(store: &mut KVStore, key: String): vector<u8> {
        table::remove(&mut store.table, key)
    }

    public fun contains(store: &KVStore, key: String): bool {
        table::contains(&store.table, key)
    }

    public fun borrow(store: &KVStore, key: String): &vector<u8> {
        table::borrow(&store.table, key)
    }
    
    public fun save_to_resource_object(account: &signer, store: KVStore){
        account::move_resource_to(account, store);
    }

    public fun borrow_from_resource_object(account: address) : &KVStore{
        account::borrow_resource(account)
    }

    public fun borrow_mut_from_resource_object(account: address) : &mut KVStore{
        account::borrow_mut_resource(account)
    }

    public fun move_from_resource_object(account: address) : KVStore{
        account::move_resource_from(account)
    }

    public fun length(kv: &KVStore): u64 {
        table::length(&kv.table)
    }

    public fun is_empty(kv: &KVStore): bool {
        table::length(&kv.table) == 0
    }

    public fun destroy(kv: KVStore){
        let KVStore{table} = kv;
        table::destroy_empty(table);
    }
}

//# run --signers test
script {
    use std::string;
    
    use test::m;

    fun main(sender: signer) {
        let kv = m::make_kv_store();
        m::add(&mut kv, string::utf8(b"test"), b"value");
        assert!(m::length(&kv) == 1, 1000); // check length is correct when data in table cache
        m::save_to_resource_object(&sender, kv);
    }
}

// check contains
//# run --signers test
script {
    use std::string;
    
    use test::m;

    fun main() {
        let sender = moveos_std::tx_context::sender();
        let kv = m::borrow_from_resource_object(sender);
        assert!(m::contains(kv, string::utf8(b"test")), 1001);
        let v = m::borrow(kv, string::utf8(b"test"));
        assert!(v == &b"value", 1002);
    }
}

// check length when data is in both remote storage and cache
//# run --signers test
script {
    use std::string;
    
    use test::m;

    fun main() {
        let sender = moveos_std::tx_context::sender();
        let kv = m::borrow_mut_from_resource_object(sender);
        m::add(kv, string::utf8(b"test1"), b"value1");
        assert!(m::length(kv) == 2, 1003); 
        let _value = m::remove(kv, string::utf8(b"test1"));
    }
}

// destroy none empty table, should failed.
//# run --signers test
script {
    
    use test::m;

    fun main() {
        let sender = moveos_std::tx_context::sender();
        let kv = m::move_from_resource_object(sender);
        m::destroy(kv);
    }
}

// destroy empty table, should success.
//# run --signers test
script {
    use std::string;
    
    use test::m;    

    fun main() {
        let sender = moveos_std::tx_context::sender();
        let kv = m::move_from_resource_object(sender);
        let v = m::remove(&mut kv, string::utf8(b"test"));
        assert!(v == b"value", 1004);
        m::destroy(kv);
    }
}