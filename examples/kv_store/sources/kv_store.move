// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::kv_store {

   use moveos_std::signer;
   
   use moveos_std::table::{Self, Table};
   use std::string::{String};
   use moveos_std::account;

   struct KVStore has key {
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

   public fun borrow_kv_store(): &KVStore {
      account::borrow_resource(@rooch_examples)
   }

   public fun borrow_kv_store_mut(): &mut KVStore {
      account::borrow_mut_resource(@rooch_examples)
   }

   //init when module publish
   fun init() {
      let kv = KVStore{
         table: table::new(),
      };
      let module_signer = signer::module_signer<KVStore>();
      account::move_resource_to(&module_signer, kv);
   }

   public entry fun add_value(key: String, value: String) {
      let kv = borrow_kv_store_mut();
      add(kv, key, value);
   }

   public entry fun remove_value(key: String) {
      let kv = borrow_kv_store_mut();
      remove(kv, key);
   }

   public fun get_value(key: String): String {
      let kv = borrow_kv_store();
      //std::debug::print(&key);
      let value = borrow(kv, key);
      *value
   }

}
