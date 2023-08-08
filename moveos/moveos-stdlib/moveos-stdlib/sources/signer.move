// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::signer {
   #[test_only]
   use std::signer;

   #[test_only]
   struct TestStruct {}

   #[private_generics(T)]
   /// Returns the signer of the module address of the generic type `T`. 
   /// This is safe because the generic type `T` is private, meaning it can only be used within the module that defines it.
   public native fun module_signer<T>(): signer;

   #[test]
   fun test_module_signer() {
    let signer = module_signer<TestStruct>();
    let signer_addr = signer::address_of(&signer);

    assert!(signer_addr == @0x2, 0);
   }

   #[test]
   #[expected_failure(abort_code=0, location=moveos_std::signer)]
   fun test_module_signer_invalid_address() {
    let signer = module_signer<TestStruct>();
    let signer_addr = signer::address_of(&signer);

    assert!(signer_addr == @0x6, 0);
   }
}