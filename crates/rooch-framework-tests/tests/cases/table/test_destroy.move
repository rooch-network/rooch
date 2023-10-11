//# init --addresses test=0x42

//# publish
module test::m {
    use std::string::String;
    use moveos_std::table::{Self, Table};
    use moveos_std::context::{Self, Context};
    use moveos_std::object;
    use moveos_std::object::ObjectID;
    use moveos_std::account_storage;

    struct KVStore has store, key {
        table: Table<String,vector<u8>>,
    }

    public fun make_kv_store(ctx: &mut Context): KVStore{
        KVStore{
            table: table::new(ctx),
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

    public fun save_to_object_storage(ctx: &mut Context, kv: KVStore) : ObjectID {        
        let object = context::new_object(ctx, kv);
        let object_id = object::id(&object);
        context::add_object(ctx, object);
        object_id
    }

    public fun borrow_from_object_storage(ctx: &mut Context, object_id: ObjectID): &KVStore {
        let object = context::borrow_object(ctx, object_id);
        object::borrow<KVStore>(object)
    }

    public fun save_to_account_storage(ctx: &mut Context, account: &signer, store: KVStore){
        account_storage::global_move_to(ctx, account, store);
    }

    public fun borrow_from_account_storage(ctx: &Context, account: address) : &KVStore{
        account_storage::global_borrow(ctx, account)
    }

    public fun borrow_mut_from_account_storage(ctx: &mut Context, account: address) : &mut KVStore{
        account_storage::global_borrow_mut(ctx, account)
    }

    public fun move_from_account_storage(ctx: &mut Context, account: address) : KVStore{
        account_storage::global_move_from(ctx, account)
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
    use moveos_std::context::{Context};
    use test::m;

    fun main(ctx: &mut Context, sender: signer) {
        let kv = m::make_kv_store(ctx);
        m::add(&mut kv, string::utf8(b"test"), b"value");
        assert!(m::length(&kv) == 1, 1000); // check length is correct when data in table cache
        m::save_to_account_storage(ctx, &sender, kv);
    }
}

// check contains
//# run --signers test
script {
    use std::string;
    use moveos_std::context::{Self, Context};
    use test::m;

    fun main(ctx: &mut Context) {
        let sender = context::sender(ctx);
        let kv = m::borrow_from_account_storage(ctx, sender);
        assert!(m::contains(kv, string::utf8(b"test")), 1001);
        let v = m::borrow(kv, string::utf8(b"test"));
        assert!(v == &b"value", 1002);
    }
}

// check length when data is in both remote storage and cache
//# run --signers test
script {
    use std::string;
    use moveos_std::context::{Self, Context};
    use test::m;

    fun main(ctx: &mut Context) {
        let sender = context::sender(ctx);
        let kv = m::borrow_mut_from_account_storage(ctx, sender);
        m::add(kv, string::utf8(b"test1"), b"value1");
        assert!(m::length(kv) == 2, 1003); 
        let _value = m::remove(kv, string::utf8(b"test1"));
    }
}

// destroy none empty table, should failed.
//# run --signers test
script {
    use moveos_std::context::{Self, Context};
    use test::m;

    fun main(ctx: &mut Context) {
        let sender = context::sender(ctx);
        let kv = m::move_from_account_storage(ctx, sender);
        m::destroy(kv);
    }
}

// destroy empty table, should success.
//# run --signers test
script {
    use std::string;
    use moveos_std::context::{Self, Context};
    use test::m;    

    fun main(ctx: &mut Context) {
        let sender = context::sender(ctx);
        let kv = m::move_from_account_storage(ctx, sender);
        let v = m::remove(&mut kv, string::utf8(b"test"));
        assert!(v == b"value", 1004);
        m::destroy(kv);
    }
}