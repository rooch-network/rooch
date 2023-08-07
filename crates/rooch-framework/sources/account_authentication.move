/// This module contains the resources and functions that are used for account authentication.
/// Migrate their from the account module for simplyfying the account module.
module rooch_framework::account_authentication{
   
   use std::error;
   use std::option::{Self, Option};
   use std::signer;
   use std::vector;
   use moveos_std::account_storage;
   use moveos_std::storage_context::StorageContext;
   use rooch_framework::auth_validator_registry;
   use rooch_framework::auth_validator;

    /// max authentication key length
   const MAX_AUTHENTICATION_KEY_LENGTH: u64 = 256;

   
   const EAuthValidatorAlreadyInstalled: u64 = 1;
   /// The provided authentication key has an invalid length
   const EMalformedAuthenticationKey: u64 = 2; 

   /// A resource that holds the authentication key for this account.
   /// ValidatorType is a phantom type parameter that is used to distinguish between different auth validator types.
   struct AuthenticationKey<phantom ValidatorType> has key {
      authentication_key: vector<u8>
   }

   /// A resource tha holds the auth validator ids for this account has installed.
   struct InstalledAuthValidator has key {
      validators: vector<u64>,
   }

   public fun get_authentication_key<ValidatorType>(ctx: &StorageContext, account_addr: address): Option<vector<u8>> {
      if(!account_storage::global_exists<AuthenticationKey<ValidatorType>>(ctx, account_addr)){
         option::none<vector<u8>>()
      }else{
         option::some(account_storage::global_borrow<AuthenticationKey<ValidatorType>>(ctx, account_addr).authentication_key)
      }
   }

   #[private_generics(ValidatorType)]
   /// This function is used to rotate a resource account's authentication key, only the module which define the `ValidatorType` can call this function.
   public fun rotate_authentication_key<ValidatorType>(ctx: &mut StorageContext, account: &signer, new_auth_key: vector<u8>) {
      rotate_authentication_key_internal<ValidatorType>(ctx, account, new_auth_key);
   }

   public(friend) fun rotate_authentication_key_internal<ValidatorType>(ctx: &mut StorageContext, account: &signer, new_auth_key: vector<u8>) {
      let account_addr = signer::address_of(account);
      
      assert!(
         vector::length(&new_auth_key) <= MAX_AUTHENTICATION_KEY_LENGTH,
         error::invalid_argument(EMalformedAuthenticationKey)
      );
   
      if(account_storage::global_exists<AuthenticationKey<ValidatorType>>(ctx, account_addr)){
         let authentication_key = account_storage::global_borrow_mut<AuthenticationKey<ValidatorType>>(ctx, account_addr);
         authentication_key.authentication_key = new_auth_key;
      }else{
         let authentication_key = AuthenticationKey<ValidatorType> {
            authentication_key: new_auth_key,
         };
         account_storage::global_move_to(ctx, account, authentication_key);
      }
   }

   /// Return the authentication validator is installed for the account at `account_addr`.
   public fun is_auth_validator_installed(ctx: &StorageContext, account_addr: address, auth_validator_id: u64): bool {
      if(account_storage::global_exists<InstalledAuthValidator>(ctx, account_addr)){
         let installed_auth_validator = account_storage::global_borrow<InstalledAuthValidator>(ctx, account_addr);
         vector::contains(&installed_auth_validator.validators, &auth_validator_id)
      }else{
         false
      }
   }

   public fun install_auth_validator<ValidatorType: store>(ctx: &mut StorageContext, account_signer: &signer) {
      let validator = auth_validator_registry::borrow_validator_by_type<ValidatorType>(ctx);
      let validator_id = auth_validator::validator_id(validator);
      let account_addr = signer::address_of(account_signer);

      assert!(
         !is_auth_validator_installed(ctx, account_addr, validator_id),
         error::already_exists(EAuthValidatorAlreadyInstalled));

      
      if(!account_storage::global_exists<InstalledAuthValidator>(ctx, account_addr)){
         let installed_auth_validator = InstalledAuthValidator {
            validators: vector::empty(),
         };
         account_storage::global_move_to<InstalledAuthValidator>(ctx, account_signer, installed_auth_validator);
      };
      let installed_auth_validator = account_storage::global_borrow_mut<InstalledAuthValidator>(ctx, account_addr);
      vector::push_back(&mut installed_auth_validator.validators, validator_id);
   }

   public entry fun install_auth_validator_entry<ValidatorType: store>(ctx: &mut StorageContext, account_signer: &signer) {
      install_auth_validator<ValidatorType>(ctx, account_signer);
   }

   #[test_only]
   struct TestValidator has store {
   }
   
   #[test(sender=@0x42)]
   fun test_rotate_authentication_key_internal(sender: address){
      let ctx = moveos_std::storage_context::new_test_context(@std);
      let sender_signer = rooch_framework::account::create_signer_for_test(sender);
      rotate_authentication_key_internal<TestValidator>(&mut ctx, &sender_signer, x"0123");
      moveos_std::storage_context::drop_test_context(ctx);
   }
}