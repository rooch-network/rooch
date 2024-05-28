// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::account {
   use rooch_framework::account_authentication;
   use moveos_std::signer::module_signer;
   use moveos_std::account;
   use moveos_std::core_addresses;

   friend rooch_framework::genesis;
   friend rooch_framework::transfer;
   friend rooch_framework::transaction_validator;

   /// Just using to get Account module signer
   struct AccountPlaceholder {}

   /// Cannot create account because address is reserved
   const ErrorAddressReserved: u64 = 1;

   /// Publishes a new `Account` resource under `new_address`. A signer representing `new_address`
   /// is returned. This way, the caller of this function can publish additional resources under
   /// `new_address`.
   public(friend) fun create_account(new_address: address): signer {
      assert!(!core_addresses::is_reserved_address(new_address), ErrorAddressReserved);
      create_account_internal(new_address)
   }

   public(friend) fun create_account_internal(new_address: address): signer {
      let system = module_signer<AccountPlaceholder>();
      let new_account = account::create_account_by_system(&system, new_address);

      account_authentication::init_authentication_keys(&new_account);
      new_account
   }

   #[test_only]
   public fun create_account_for_testing(new_address: address): signer {
      create_account_internal(new_address)
   }

   #[test(sender=@0x0)]
   #[expected_failure(abort_code = ErrorAddressReserved, location = Self)]
   fun test_failure_entry_account_creation_reserved(sender: address){
      create_account(sender);
   }
}
