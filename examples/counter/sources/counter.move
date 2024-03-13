// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::counter {
   
   use moveos_std::account;

   struct Counter has key, store {
      value:u64,
   }

   public fun init_for_test(account: &signer) {
      account::move_resource_to(account, Counter { value: 0 });
   }

   fun init(account: &signer) {
      account::move_resource_to(account, Counter { value: 0 });
   }

   public fun increase_() {
      let counter = account::borrow_mut_resource<Counter>(@rooch_examples);
      counter.value = counter.value + 1;
   }

   public entry fun increase() {
      Self::increase_()
   }

   public fun value(): u64 {
      let counter = account::borrow_resource<Counter>(@rooch_examples);
      counter.value
   }
}
