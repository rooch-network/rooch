// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::kv_store {

   use moveos_std::signer;
   use moveos_std::context::{Self, Context};
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

   public fun borrow_kv_store(ctx: &Context): &KVStore {
      context::borrow_resource(ctx, @rooch_examples)
   }

   public fun borrow_kv_store_mut(ctx: &mut Context): &mut KVStore {
      context::borrow_mut_resource(ctx, @rooch_examples)
   }

   //init when module publish
   fun init(ctx: &mut Context) {
      let kv = KVStore{
         table: context::new_table(ctx),
      };
      let module_signer = signer::module_signer<KVStore>();
      context::move_resource_to(ctx, &module_signer, kv);
   }

   public entry fun add_value(ctx: &mut Context, key: String, value: String) {
      let kv = borrow_kv_store_mut(ctx);
      add(kv, key, value);
   }

   public entry fun remove_value(ctx: &mut Context, key: String) {
      let kv = borrow_kv_store_mut(ctx);
      remove(kv, key);
   }

   public fun get_value(ctx: &Context, key: String): String {
      let kv = borrow_kv_store(ctx);
      //std::debug::print(&key);
      let value = borrow(kv, key);
      *value
   }

}
