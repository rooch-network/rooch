module rooch_examples::kv_store {

   use moveos_std::storage_context::{Self, StorageContext};
   use moveos_std::account_storage;
   use moveos_std::table::{Self, Table};
   use std::string::{String};

   struct KVStore has store, key {
      table: Table<String, String>,
   }

   public fun add(store: &mut KVStore, key: String, value: String) {
      table::add(&mut store.table, key, value);
   }

   public fun remove(store: &mut KVStore, key: String) {
      table::remove(&mut store.table, key);
   }

   public fun contains(store: &KVStore, key: String): bool {
      table::contains(&store.table, key)
   }

   public fun borrow(store: &KVStore, key: String): &String {
      table::borrow(&store.table, key)
   }

   public fun borrow_kv_store(ctx: &mut StorageContext): &KVStore {
      account_storage::global_borrow(ctx, @rooch_examples)
   }

   public fun borrow_kv_store_mut(ctx: &mut StorageContext): &mut KVStore {
      account_storage::global_borrow_mut(ctx, @rooch_examples)
   }

   //init when module publish
   fun init(ctx: &mut StorageContext, sender: signer) {
      let tx_ctx = storage_context::tx_context_mut(ctx);
      let kv = KVStore{
         table: table::new(tx_ctx),
      };
      account_storage::global_move_to(ctx, &sender, kv);
   }

   public entry fun add_value(ctx: &mut StorageContext, key: String, value: String) {
      let kv = borrow_kv_store_mut(ctx);
      add(kv, key, value);
   }

   public entry fun remove_value(ctx: &mut StorageContext, key: String) {
      let kv = borrow_kv_store_mut(ctx);
      remove(kv, key);
   }

   #[view]
   public fun get_value(ctx: &mut StorageContext, key: String): String {
      let kv = borrow_kv_store(ctx);
      let value = borrow(kv, key);
      *value
   }

}