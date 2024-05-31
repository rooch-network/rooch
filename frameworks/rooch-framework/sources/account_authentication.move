// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module contains the resources and functions that are used for account authentication.
/// Migrated from the account module for simplyfying the account module.
module rooch_framework::account_authentication {
   
   use std::signer;
   use std::vector;
   use moveos_std::account;
   use moveos_std::features;
   use rooch_framework::auth_validator_registry;
   use rooch_framework::auth_validator;

   friend rooch_framework::account;


   /// The authentication validator is already installed
   const ErrorAuthValidatorAlreadyInstalled: u64 = 1;


   /// A resource that holds the auth validator ids for this account has installed.
   struct InstalledAuthValidator has key {
      validators: vector<u64>,
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


}
