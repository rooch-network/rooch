// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::counter {
   use moveos_std::context::{Self, Context};

   struct Counter has key, store {
      value:u64,
   }

   #[test_only]
   public fun init_for_test(ctx: &mut Context, account: &signer) {
      context::move_resource_to(ctx, account, Counter { value: 0 });
   }

   fun init(ctx: &mut Context, account: &signer) {
      context::move_resource_to(ctx, account, Counter { value: 0 });
   }

   public fun increase_(ctx: &mut Context) {
      let counter = context::borrow_mut_resource<Counter>(ctx, @rooch_examples);
      counter.value = counter.value + 1;
   }

   public entry fun increase(ctx: &mut Context) {
      Self::increase_(ctx)
   }

   public fun value(ctx: &Context): u64 {
      let counter = context::borrow_resource<Counter>(ctx, @rooch_examples);
      counter.value
   }
}
