// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::signer {
  
    #[private_generics(T)]
    /// Returns the signer of the module address of the generic type `T`.
    /// This is safe because the generic type `T` is private, meaning it can only be used within the module that defines it.
    native public fun module_signer<T>(): signer;

    /// Returns the address of the signer.
    public fun address_of(signer: &signer): address {
        std::signer::address_of(signer)
    }

    #[test_only]
    struct TestStruct {}

    #[test]
    fun test_module_signer() {
        let signer = module_signer<TestStruct>();
        let signer_addr = address_of(&signer);

        assert!(signer_addr == @moveos_std, 0);
    }
}