// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::account {
   use moveos_std::account::SignerCapability;
   use rooch_framework::account_coin_store;
   use rooch_framework::account_authentication;
   use moveos_std::signer::module_signer;
   use moveos_std::account;
   use moveos_std::context::{Context};
   #[test_only]
   use moveos_std::context;

   /// Just using to get Account module signer
   struct AccountPlaceholder {}

   // TODO should we provide create account from arbitrary address?
   // TODO Can create accounts arbitrary. Is this a security risk?
   /// A entry function to create an account under `new_address`
   public entry fun create_account_entry(ctx: &mut Context, new_address: address){
      // If account already exists, do nothing
      // Because if the new address is the same as the sender, the account must already created in the `transaction_validator::pre_execute` function
      if(!account::exists_at(ctx, new_address)){
         create_account(ctx, new_address);
      };
   }

   /// Publishes a new `Account` resource under `new_address`. A signer representing `new_address`
   /// is returned. This way, the caller of this function can publish additional resources under
   /// `new_address`.
   public fun create_account(ctx: &mut Context, new_address: address): signer {
      let system = module_signer<AccountPlaceholder>();
      let new_account = account::create_account_for_system(ctx, &system, new_address);

      account_authentication::init_authentication_keys(ctx, &new_account);
      account_coin_store::init_account_coin_stores(ctx, &new_account);
      new_account
   }

   /// A resource account is used to manage resources independent of an account managed by a user.
   /// In Rooch a resource account is created based upon the sha3 256 of the source's address and additional seed data.
   /// A resource account can only be created once
   public fun create_resource_account(ctx: &mut Context, source: &signer): (signer, SignerCapability) {
      let (resource_signer, signer_cap) = account::create_resource_account(ctx, source);

      account_authentication::init_authentication_keys(ctx, &resource_signer);
      account_coin_store::init_account_coin_stores(ctx, &resource_signer);
      (resource_signer, signer_cap)
   }

   #[test_only]
   public fun create_account_for_test(ctx: &mut Context, new_address: address): signer {
      create_account(ctx, new_address)
   }

   #[test_only]
   struct Test has key{
      addr: address,
      version: u64
   }

   #[test(sender=@0x42)]
   fun test_create_account_entry(sender: address){
      let ctx = context::new_test_context(sender);
      create_account_entry(&mut ctx, sender);
      context::drop_test_context(ctx);
   }

   //FIXME https://github.com/rooch-network/rooch/issues/1421
   //#[test(sender=@0x0)]
   //#[expected_failure(abort_code = 4, location = moveos_std::account)]
   // fun test_failure_entry_account_creation_reserved(sender: address){
   //    let ctx = context::new_test_context(sender);
   //    create_account_entry(&mut ctx, sender);
   //    context::drop_test_context(ctx);
   // }
}
