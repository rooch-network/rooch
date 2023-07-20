/// This module implements the schnorr validator scheme.
module rooch_framework::schnorr_validator {

   use std::vector;
   use std::option;
   use moveos_std::storage_context::{Self, StorageContext};
   use rooch_framework::account_authentication;
   use rooch_framework::hash;
   use rooch_framework::schnorr;
   use rooch_framework::auth_validator;

   const SCHEME_SCHNORR: u64 = 3;
   const SCHNORR_SCHEME_LENGTH: u64 = 1;
   const SCHNORR_PUBKEY_LENGTH: u64 = 32;
   const SCHNORR_SIG_LENGTH: u64 = 64;
   const SCHNORR_HASH_LENGTH: u64 = 1;

   struct SchnorrValidator has store{
   }

   public fun scheme(): u64 {
      SCHEME_SCHNORR
   }

   public fun schnorr_hash(payload: &vector<u8>): u8 {
      let hash = vector::empty<u8>();
      let i = SCHNORR_SCHEME_LENGTH + SCHNORR_SIG_LENGTH + SCHNORR_PUBKEY_LENGTH;
      while (i < SCHNORR_SCHEME_LENGTH + SCHNORR_SIG_LENGTH + SCHNORR_PUBKEY_LENGTH + SCHNORR_HASH_LENGTH) {
         let value = vector::borrow(payload, i);
         vector::push_back(&mut hash, *value);
         i = i + 1;
      };
      let vector_size: u64 = vector::length(&hash);
      let hash_value: u8 = *vector::borrow(&hash, vector_size - 1);

      hash_value
   }

   public fun schnorr_public_key(payload: &vector<u8>): vector<u8> {
      let public_key = vector::empty<u8>();
      let i = SCHNORR_SCHEME_LENGTH + SCHNORR_SIG_LENGTH;
      while (i < SCHNORR_SCHEME_LENGTH + SCHNORR_SIG_LENGTH + SCHNORR_PUBKEY_LENGTH) {
         let value = vector::borrow(payload, i);
         vector::push_back(&mut public_key, *value);
         i = i + 1;
      };

      public_key
   }

   public fun schnorr_signature(payload: &vector<u8>): vector<u8> {
      let sign = vector::empty<u8>();
      let i = SCHNORR_SCHEME_LENGTH;
      while (i < SCHNORR_SIG_LENGTH + 1) {
         let value = vector::borrow(payload, i);
         vector::push_back(&mut sign, *value);
         i = i + 1;
      };

      sign
   }

   /// Get the authentication key of the given authenticator.
   public fun schnorr_authentication_key(payload: &vector<u8>): vector<u8> {
      let public_key = schnorr_public_key(payload);
      let addr = schnorr_public_key_to_address(public_key);
      moveos_std::bcs::to_bytes(&addr)
   }

   public fun schnorr_public_key_to_address(public_key: vector<u8>): address {
      let bytes = vector::singleton((SCHEME_SCHNORR as u8));
      vector::append(&mut bytes, public_key);
      moveos_std::bcs::to_address(hash::blake2b256(&bytes))
   }

   public fun get_authentication_key(ctx: &StorageContext, addr: address): vector<u8> {
      let auth_key_option = account_authentication::get_authentication_key<SchnorrValidator>(ctx, addr);
      if(option::is_some(&auth_key_option)){
         option::extract(&mut auth_key_option)
      }else{
        //if AuthenticationKey does not exist, return addr as authentication key
        moveos_std::bcs::to_bytes(&addr)
      }
   }

   public fun validate(ctx: &StorageContext, payload: vector<u8>){
      let auth_key = schnorr_authentication_key(&payload);
      let auth_key_in_account = get_authentication_key(ctx, storage_context::sender(ctx));
      assert!(
         auth_key_in_account == auth_key,
         auth_validator::error_invalid_account_auth_key()
      );
      assert!(
         schnorr::verify(
         &schnorr_signature(&payload),
         &schnorr_public_key(&payload),
         &storage_context::tx_hash(ctx),
         schnorr_hash(&payload),
         ),
         auth_validator::error_invalid_account_auth_key()
      );
   }

   fun pre_execute(
      _ctx: &mut StorageContext,
   ) { 
   }
   
   fun post_execute(
      _ctx: &mut StorageContext,
   ) {
   }

   // this test ensures that the schnorr_public_key_to_address function is compatible with the one in the rust code
   #[test]
   fun test_schnorr_public_key_to_address(){
      let public_key = x"1b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f";
      let addr = schnorr_public_key_to_address(public_key);
      assert!(addr == @0x7ef99ee767314ccb4726be579ab3eabd212741b3796db40405ff421c47b0ae85, 1000);
   }
}