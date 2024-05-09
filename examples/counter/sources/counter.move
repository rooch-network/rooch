// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::counter {
   
   use moveos_std::account;

   struct Counter has key {
      value:u64,
   }

   fun init() {
      // The signer of the module address(rooch_examples)
      let signer = moveos_std::signer::module_signer<Counter>();
      account::move_resource_to(&signer, Counter { value: 0 });
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

   #[test_only]
   public fun init_for_test() {
      Self::init()
   }

   #[test]
   fun test_counter() {
      init_for_test();
      let value = Self::value();
      assert!(value == 0, 1000);
      
      Self::increase_();
      let value = Self::value();

      assert!(value == 1, 1000);
   }
}
