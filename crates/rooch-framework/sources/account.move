// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::account {
   use moveos_std::account::SignerCapability;
   use rooch_framework::account_coin_store;
   use rooch_framework::account_authentication;
   use moveos_std::signer::module_signer;
   use moveos_std::account;
   use moveos_std::core_addresses;

   friend rooch_framework::genesis;

   /// Just using to get Account module signer
   struct AccountPlaceholder {}

   /// Account already exists
   const ErrorAccountAlreadyExists: u64 = 1;
   /// Cannot create account because address is reserved
   const ErrorAddressReserved: u64 = 2;

   // TODO should we provide create account from arbitrary address?
   // TODO Can create accounts arbitrary. Is this a security risk?
   /// A entry function to create an account under `new_address`
   public entry fun create_account_entry(new_address: address){
      // Make sure the address is not reserved
      // Although we will check it in `create_account`, we need check it here
      // to prevent 0x0 address pass the check, as `account::exists_at(0x0)` will return true
      assert!(!core_addresses::is_reserved_address(new_address), ErrorAddressReserved);

      // If account already exists, do nothing
      // Because if the new address is the same as the sender, the account must already created in the `transaction_validator::pre_execute` function
      if(!account::exists_at(new_address)){
         create_account(new_address);
      };
   }

   /// Publishes a new `Account` resource under `new_address`. A signer representing `new_address`
   /// is returned. This way, the caller of this function can publish additional resources under
   /// `new_address`.
   public fun create_account(new_address: address): signer {
      assert!(!core_addresses::is_reserved_address(new_address), ErrorAddressReserved);
      create_account_internal(new_address)
   }

   public(friend) fun create_account_internal(new_address: address): signer {
      // Make sure the Account is not already created.
      assert!(!account::exists_at(new_address), ErrorAccountAlreadyExists);
      create_account_unchecked(new_address)      
   }

   fun create_account_unchecked(new_address: address): signer {
      let system = module_signer<AccountPlaceholder>();
      let new_account = account::create_account_by_system(&system, new_address);

      account_authentication::init_authentication_keys(&new_account);
      account_coin_store::init_account_coin_stores(&new_account);
      new_account
   }


   /// A resource account is used to manage resources independent of an account managed by a user.
   /// In Rooch a resource account is created based upon the sha3 256 of the source's address and additional seed data.
   /// A resource account can only be created once
   public fun create_resource_account(source: &signer): (signer, SignerCapability) {
      let (resource_signer, signer_cap) = account::create_resource_account(source);

      account_authentication::init_authentication_keys(&resource_signer);
      account_coin_store::init_account_coin_stores(&resource_signer);
      (resource_signer, signer_cap)
   }

   #[test_only]
   public fun create_account_for_testing(new_address: address): signer {
      create_account(new_address)
   }

   #[test_only]
   struct Test has key{
      addr: address,
      version: u64
   }

   #[test(sender=@0x42)]
   fun test_create_account_entry(sender: address){
      create_account_entry(sender);
   }

   #[test(sender=@0x0)]
   #[expected_failure(abort_code = ErrorAddressReserved, location = Self)]
   fun test_failure_entry_account_creation_reserved(sender: address){
      create_account_entry(sender);
   }
}
