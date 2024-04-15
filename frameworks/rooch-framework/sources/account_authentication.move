// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module contains the resources and functions that are used for account authentication.
/// Migrated from the account module for simplyfying the account module.
module rooch_framework::account_authentication {
   
   use std::option::{Self, Option};
   use std::signer;
   use std::vector;
   use moveos_std::account;
   use moveos_std::features;
   
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
   struct AuthenticationKey<phantom ValidatorType> has key, store, drop {
      authentication_key: vector<u8>
   }

   /// A resource that holds the authentication keys for this account.
   struct AuthenticationKeys has key, store{
      authentication_keys: TypeTable,
   }

   //TODO should we use the AuthenticationKeys to indecate the auth validator is installed for the account?
   /// A resource that holds the auth validator ids for this account has installed.
   struct InstalledAuthValidator has key,store {
      validators: vector<u64>,
   }

   public(friend) fun init_authentication_keys(account: &signer) {
      let authentication_keys = AuthenticationKeys {
         authentication_keys: type_table::new(),
      };
      account::move_resource_to<AuthenticationKeys>(account, authentication_keys);
   }

   public fun get_authentication_key<ValidatorType>(account_addr: address): Option<vector<u8>> {
      if(!account::exists_resource<AuthenticationKeys>(account_addr)){
         option::none<vector<u8>>()
      }else{
         let authentication_keys = account::borrow_resource<AuthenticationKeys>(account_addr);
         if(type_table::contains<AuthenticationKey<ValidatorType>>(&authentication_keys.authentication_keys)){
            option::some(type_table::borrow<AuthenticationKey<ValidatorType>>(&authentication_keys.authentication_keys).authentication_key)
         }else{
            option::none<vector<u8>>()
         }
      }
   }

   #[private_generics(ValidatorType)]
   /// This function is used to rotate a resource account's authentication key, only the module which define the `ValidatorType` can call this function.
   public fun rotate_authentication_key<ValidatorType>(account_addr: address, new_auth_key: vector<u8>) {
      // TODO: This may have some issues as we can only rotate the authentication key of Rooch accounts.
      // We have no way to change the BTC authentication.
      features::ensure_testnet_enabled();

      assert!(
         vector::length(&new_auth_key) <= MAX_AUTHENTICATION_KEY_LENGTH,
         ErrorMalformedAuthenticationKey
      );
      assert!(
         account::exists_resource<AuthenticationKeys>(account_addr),
         ErrorAuthenticationKeysResourceNotFound
      );

      let authentication_keys = account::borrow_mut_resource<AuthenticationKeys>(account_addr);
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
   public fun remove_authentication_key<ValidatorType>(account_addr: address): AuthenticationKey<ValidatorType> {
      assert!(
         account::exists_resource<AuthenticationKeys>(account_addr),
         ErrorAuthenticationKeysResourceNotFound
      );
      let authentication_keys = account::borrow_mut_resource<AuthenticationKeys>(account_addr);
      assert!(
         type_table::contains<AuthenticationKey<ValidatorType>>(&authentication_keys.authentication_keys),
         ErrorAuthenticationKeyNotFound
      );
   
      let removed_authentication_key = type_table::remove<AuthenticationKey<ValidatorType>>(&mut authentication_keys.authentication_keys);
      removed_authentication_key
   }

   /// Return if the authentication validator is installed for the account at `account_addr`.
   public fun is_auth_validator_installed(account_addr: address, auth_validator_id: u64): bool {
      if(account::exists_resource<InstalledAuthValidator>(account_addr)){
         let installed_auth_validator = account::borrow_resource<InstalledAuthValidator>(account_addr);
         vector::contains(&installed_auth_validator.validators, &auth_validator_id)
      }else{
         false
      }
   }

   //TODO should we init the AuthenticationKey when install auth validator?
   public fun install_auth_validator<ValidatorType: store>(account_signer: &signer) {
      features::ensure_testnet_enabled();

      let validator = auth_validator_registry::borrow_validator_by_type<ValidatorType>();
      let validator_id = auth_validator::validator_id(validator);
      let account_addr = signer::address_of(account_signer);

      assert!(
         !is_auth_validator_installed(account_addr, validator_id),
         ErrorAuthValidatorAlreadyInstalled);

      
      if(!account::exists_resource<InstalledAuthValidator>(account_addr)){
         let installed_auth_validator = InstalledAuthValidator {
            validators: vector::empty(),
         };
         account::move_resource_to<InstalledAuthValidator>(account_signer, installed_auth_validator);
      };
      let installed_auth_validator = account::borrow_mut_resource<InstalledAuthValidator>(account_addr);
      vector::push_back(&mut installed_auth_validator.validators, validator_id);
   }

   public entry fun install_auth_validator_entry<ValidatorType: store>(account_signer: &signer) {
      install_auth_validator<ValidatorType>(account_signer);
   }

   #[test_only]
   struct TestValidator has store {
   }
   
   #[test(sender=@0x42)]
   fun test_rotate_authentication_key(sender: signer){
      features::switch_on_all_features_for_test();
      
      init_authentication_keys(&sender);
      let sender_addr = signer::address_of(&sender);
      let authentication_key = x"0123";
      let authentication_key_option = get_authentication_key<TestValidator>(sender_addr);
      assert!(option::is_none(&authentication_key_option), ErrorAuthenticationKeyAlreadyExists);
      rotate_authentication_key<TestValidator>(sender_addr, authentication_key);
      authentication_key_option = get_authentication_key<TestValidator>(sender_addr);
      assert!(option::is_some(&authentication_key_option), ErrorAuthenticationKeyNotFound);
      
   }

   #[test(sender=@0x42)]
   fun test_remove_authentication_key(sender: signer){
      features::switch_on_all_features_for_test();
      
      init_authentication_keys(&sender);
      let sender_addr = signer::address_of(&sender);
      let authentication_key = x"1234";
      let authentication_key_option = get_authentication_key<TestValidator>(sender_addr);
      assert!(option::is_none(&authentication_key_option), ErrorAuthenticationKeyAlreadyExists);
      rotate_authentication_key<TestValidator>(sender_addr, authentication_key);
      authentication_key_option = get_authentication_key<TestValidator>(sender_addr);
      assert!(option::is_some(&authentication_key_option), ErrorAuthenticationKeyNotFound);
      let removed_authentication_key = remove_authentication_key<TestValidator>(sender_addr);
      authentication_key_option = get_authentication_key<TestValidator>(sender_addr);
      assert!(option::is_none(&authentication_key_option), ErrorAuthenticationKeyAlreadyExists);
      assert!(removed_authentication_key.authentication_key == authentication_key, ErrorMalformedAuthenticationKey);
      
   }
}
