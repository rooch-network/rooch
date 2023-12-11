// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module contains the resources and functions that are used for account authentication.
/// Migrated from the account module for simplyfying the account module.
module rooch_framework::account_authentication {
   
   use std::option::{Self, Option};
   use std::signer;
   use std::vector;
   use moveos_std::context::{Self, Context};
   use moveos_std::type_table::{Self, TypeTable};
   use rooch_framework::auth_validator_registry;
   use rooch_framework::auth_validator;

   friend rooch_framework::account;

   /// max authentication key length
   const MAX_AUTHENTICATION_KEY_LENGTH: u64 = 256;

   /// The authentication validator is already installed
   const ErrorAuthValidatorAlreadyInstalled: u64 = 1;
   /// The provided authentication key has an invalid length
   const ErrorMalformedAuthenticationKey: u64 = 2;
   /// The authentication keys resource has not been found for the account address
   const ErrorAuthenticationKeysResourceNotFound: u64 = 3; 
   /// The authentication key has not been found for the specified validator
   const ErrorAuthenticationKeyNotFound: u64 = 4; 
   /// The authentication key already exists in the specified validator
   const ErrorAuthenticationKeyAlreadyExists: u64 = 5; 

   /// A resource that holds the authentication key for this account.
   /// ValidatorType is a phantom type parameter that is used to distinguish between different auth validator types.
   struct AuthenticationKey<phantom ValidatorType> has key, drop {
      authentication_key: vector<u8>
   }

   /// A resource that holds the authentication keys for this account.
   struct AuthenticationKeys has key{
      authentication_keys: TypeTable,
   }

   //TODO should we use the AuthenticationKeys to indecate the auth validator is installed for the account?
   /// A resource tha holds the auth validator ids for this account has installed.
   struct InstalledAuthValidator has key {
      validators: vector<u64>,
   }

   public(friend) fun init_authentication_keys(ctx: &mut Context, account: &signer) {
      let authentication_keys = AuthenticationKeys {
         authentication_keys: context::new_type_table(ctx),
      };
      context::move_resource_to<AuthenticationKeys>(ctx, account, authentication_keys);
   }

   public fun get_authentication_key<ValidatorType>(ctx: &Context, account_addr: address): Option<vector<u8>> {
      if(!context::exists_resource<AuthenticationKeys>(ctx, account_addr)){
         option::none<vector<u8>>()
      }else{
         let authentication_keys = context::borrow_resource<AuthenticationKeys>(ctx, account_addr);
         if(type_table::contains<AuthenticationKey<ValidatorType>>(&authentication_keys.authentication_keys)){
            option::some(type_table::borrow<AuthenticationKey<ValidatorType>>(&authentication_keys.authentication_keys).authentication_key)
         }else{
            option::none<vector<u8>>()
         }
      }
   }

   #[private_generics(ValidatorType)]
   /// This function is used to rotate a resource account's authentication key, only the module which define the `ValidatorType` can call this function.
   public fun rotate_authentication_key<ValidatorType>(ctx: &mut Context, account_addr: address, new_auth_key: vector<u8>) {
      
      assert!(
         vector::length(&new_auth_key) <= MAX_AUTHENTICATION_KEY_LENGTH,
         ErrorMalformedAuthenticationKey
      );
      assert!(
         context::exists_resource<AuthenticationKeys>(ctx, account_addr),
         ErrorAuthenticationKeysResourceNotFound
      );

      let authentication_keys = context::borrow_mut_resource<AuthenticationKeys>(ctx, account_addr);
      if(type_table::contains<AuthenticationKey<ValidatorType>>(&authentication_keys.authentication_keys)){
         let authentication_key = type_table::borrow_mut<AuthenticationKey<ValidatorType>>(&mut authentication_keys.authentication_keys);
         authentication_key.authentication_key = new_auth_key;
      }else{
         let authentication_key = AuthenticationKey<ValidatorType> {
            authentication_key: new_auth_key,
         };
         type_table::add(&mut authentication_keys.authentication_keys, authentication_key);
      };
   }

   #[private_generics(ValidatorType)]
   /// This function is used to remove a resource account's authentication key, only the module which define the `ValidatorType` can call this function.
   public fun remove_authentication_key<ValidatorType>(ctx: &mut Context, account_addr: address): AuthenticationKey<ValidatorType> {
      assert!(
         context::exists_resource<AuthenticationKeys>(ctx, account_addr),
         ErrorAuthenticationKeysResourceNotFound
      );
      let authentication_keys = context::borrow_mut_resource<AuthenticationKeys>(ctx, account_addr);
      assert!(
         type_table::contains<AuthenticationKey<ValidatorType>>(&authentication_keys.authentication_keys),
         ErrorAuthenticationKeyNotFound
      );
   
      let removed_authentication_key = type_table::remove<AuthenticationKey<ValidatorType>>(&mut authentication_keys.authentication_keys);
      removed_authentication_key
   }

   /// Return if the authentication validator is installed for the account at `account_addr`.
   public fun is_auth_validator_installed(ctx: &Context, account_addr: address, auth_validator_id: u64): bool {
      if(context::exists_resource<InstalledAuthValidator>(ctx, account_addr)){
         let installed_auth_validator = context::borrow_resource<InstalledAuthValidator>(ctx, account_addr);
         vector::contains(&installed_auth_validator.validators, &auth_validator_id)
      }else{
         false
      }
   }

   //TODO should we init the AuthenticationKey when install auth validator?
   public fun install_auth_validator<ValidatorType: store>(ctx: &mut Context, account_signer: &signer) {
      let validator = auth_validator_registry::borrow_validator_by_type<ValidatorType>(ctx);
      let validator_id = auth_validator::validator_id(validator);
      let account_addr = signer::address_of(account_signer);

      assert!(
         !is_auth_validator_installed(ctx, account_addr, validator_id),
         ErrorAuthValidatorAlreadyInstalled);

      
      if(!context::exists_resource<InstalledAuthValidator>(ctx, account_addr)){
         let installed_auth_validator = InstalledAuthValidator {
            validators: vector::empty(),
         };
         context::move_resource_to<InstalledAuthValidator>(ctx, account_signer, installed_auth_validator);
      };
      let installed_auth_validator = context::borrow_mut_resource<InstalledAuthValidator>(ctx, account_addr);
      vector::push_back(&mut installed_auth_validator.validators, validator_id);
   }

   public entry fun install_auth_validator_entry<ValidatorType: store>(ctx: &mut Context, account_signer: &signer) {
      install_auth_validator<ValidatorType>(ctx, account_signer);
   }

   #[test_only]
   struct TestValidator has store {
   }
   
   #[test(sender=@0x42)]
   fun test_rotate_authentication_key(sender: signer){
      let ctx = moveos_std::context::new_test_context(@std);
      init_authentication_keys(&mut ctx, &sender);
      let sender_addr = signer::address_of(&sender);
      let authentication_key = x"0123";
      let authentication_key_option = get_authentication_key<TestValidator>(&ctx, sender_addr);
      assert!(option::is_none(&authentication_key_option), ErrorAuthenticationKeyAlreadyExists);
      rotate_authentication_key<TestValidator>(&mut ctx, sender_addr, authentication_key);
      authentication_key_option = get_authentication_key<TestValidator>(&ctx, sender_addr);
      assert!(option::is_some(&authentication_key_option), ErrorAuthenticationKeyNotFound);
      moveos_std::context::drop_test_context(ctx);
   }

   #[test(sender=@0x42)]
   fun test_remove_authentication_key(sender: signer){
      let ctx = moveos_std::context::new_test_context(@std);
      init_authentication_keys(&mut ctx, &sender);
      let sender_addr = signer::address_of(&sender);
      let authentication_key = x"1234";
      let authentication_key_option = get_authentication_key<TestValidator>(&ctx, sender_addr);
      assert!(option::is_none(&authentication_key_option), ErrorAuthenticationKeyAlreadyExists);
      rotate_authentication_key<TestValidator>(&mut ctx, sender_addr, authentication_key);
      authentication_key_option = get_authentication_key<TestValidator>(&ctx, sender_addr);
      assert!(option::is_some(&authentication_key_option), ErrorAuthenticationKeyNotFound);
      let removed_authentication_key = remove_authentication_key<TestValidator>(&mut ctx, sender_addr);
      authentication_key_option = get_authentication_key<TestValidator>(&ctx, sender_addr);
      assert!(option::is_none(&authentication_key_option), ErrorAuthenticationKeyAlreadyExists);
      assert!(removed_authentication_key.authentication_key == authentication_key, ErrorMalformedAuthenticationKey);
      moveos_std::context::drop_test_context(ctx);
   }
}
