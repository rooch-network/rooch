/// This module implements the schnorr validator scheme.
module rooch_framework::schnorr_validator {

   use std::error;
   use std::vector;
   use std::option;
   use std::signer;
   use moveos_std::storage_context::{Self, StorageContext};
   use rooch_framework::account_authentication;
   use rooch_framework::hash;
   use rooch_framework::schnorr;
   use rooch_framework::auth_validator;

   const SCHEME_SCHNORR: u64 = 4;
   const V_SCHNORR_SCHEME_LENGTH: u64 = 1;
   const V_SCHNORR_PUBKEY_LENGTH: u64 = 32;
   const V_SCHNORR_SIG_LENGTH: u64 = 64;
   const V_SCHNORR_HASH_LENGTH: u64 = 1;
   /// Hash function name that are valid for verify.
   const KECCAK256: u8 = 0;
   const SHA256: u8 = 1;
   /// error code
   const EMalformedAccount: u64 = 1001;
   const EMalformedAuthenticationKey: u64 = 1002;

   struct SchnorrValidator has store{
   }

   public fun scheme(): u64 {
      SCHEME_SCHNORR
   }

   public entry fun rotate_authentication_key_entry<SchnorrValidator>(ctx: &mut StorageContext, account: &signer, public_key: vector<u8>) {
      // compare newly passed public key with schnorr public key length to ensure it's compatible
      assert!(
         vector::length(&public_key) == V_SCHNORR_PUBKEY_LENGTH,
         error::invalid_argument(EMalformedAuthenticationKey)
      );

      // ensure that the schnorr public key to address isn't matched with the ed25519 account address
      let account_addr = signer::address_of(account);
      let schnorr_addr = schnorr_public_key_to_address(public_key);
      assert!(
         account_addr != schnorr_addr,
         error::invalid_argument(EMalformedAccount)
      );

      // serialize the address to an auth key and rotate it by calling rotate_authentication_key
      let schnorr_authentication_key = moveos_std::bcs::to_bytes(&schnorr_addr);
      account_authentication::rotate_authentication_key<SchnorrValidator>(ctx, account, schnorr_authentication_key);
   }

   public fun schnorr_public_key(authenticator_payload: &vector<u8>): vector<u8> {
      let public_key = vector::empty<u8>();
      let i = V_SCHNORR_SCHEME_LENGTH + V_SCHNORR_SIG_LENGTH;
      while (i < V_SCHNORR_SCHEME_LENGTH + V_SCHNORR_SIG_LENGTH + V_SCHNORR_PUBKEY_LENGTH) {
         let value = vector::borrow(authenticator_payload, i);
         vector::push_back(&mut public_key, *value);
         i = i + 1;
      };

      public_key
   }

   public fun schnorr_signature(authenticator_payload: &vector<u8>): vector<u8> {
      let sign = vector::empty<u8>();
      let i = V_SCHNORR_SCHEME_LENGTH;
      while (i < V_SCHNORR_SIG_LENGTH + 1) {
         let value = vector::borrow(authenticator_payload, i);
         vector::push_back(&mut sign, *value);
         i = i + 1;
      };

      sign
   }

   /// Get the authentication key of the given authenticator.
   public fun schnorr_authentication_key(authenticator_payload: &vector<u8>): vector<u8> {
      let public_key = schnorr_public_key(authenticator_payload);
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

   public fun validate(ctx: &StorageContext, authenticator_payload: vector<u8>){
      // TODO handle non-ed25519 auth key and address relationship
      // let auth_key = schnorr_authentication_key(&authenticator_payload);
      // let auth_key_in_account = get_authentication_key(ctx, storage_context::sender(ctx));
      // assert!(
      //    auth_key_in_account == auth_key,
      //    auth_validator::error_invalid_account_auth_key()
      // );
      assert!(
         schnorr::verify(
            &schnorr_signature(&authenticator_payload),
            &schnorr_public_key(&authenticator_payload),
            &storage_context::tx_hash(ctx),
            SHA256,
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
      assert!(addr == @0xa519b36bbecc294726bbfd962ab46ca4e09baacca7cd90d5d2da2331afb363e6, 1000);
   }
}