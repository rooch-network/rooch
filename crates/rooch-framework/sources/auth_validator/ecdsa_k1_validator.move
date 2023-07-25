/// This module implements the ECDSA over Secpk256k1 validator scheme.
module rooch_framework::ecdsa_k1_validator {

   use std::vector;
   use std::option;
   use moveos_std::storage_context::{Self, StorageContext};
   use rooch_framework::account_authentication;
   use rooch_framework::hash;
   use rooch_framework::ecdsa_k1;
   use rooch_framework::auth_validator;

   const SCHEME_ECDSA: u64 = 2;
   const ECDSA_SCHEME_LENGTH: u64 = 1;
   const ECDSA_PUBKEY_LENGTH: u64 = 32;
   const ECDSA_SIG_LENGTH: u64 = 64;
   const ECDSA_HASH_LENGTH: u64 = 1;
   /// Hash function name that are valid for ecrecover and verify.
   const KECCAK256: u8 = 0;
   const SHA256: u8 = 1;

   struct EcdsaK1Validator has store{
   }

   public fun scheme(): u64 {
      SCHEME_ECDSA
   }

   public fun ecdsa_k1_public_key(payload: &vector<u8>): vector<u8> {
      let public_key = vector::empty<u8>();
      let i = ECDSA_SCHEME_LENGTH + ECDSA_SIG_LENGTH;
      while (i < ECDSA_SCHEME_LENGTH + ECDSA_SIG_LENGTH + ECDSA_PUBKEY_LENGTH) {
         let value = vector::borrow(payload, i);
         vector::push_back(&mut public_key, *value);
         i = i + 1;
      };

      public_key
   }

   public fun ecdsa_k1_signature(payload: &vector<u8>): vector<u8> {
      let sign = vector::empty<u8>();
      let i = ECDSA_SCHEME_LENGTH;
      while (i < ECDSA_SIG_LENGTH + 1) {
         let value = vector::borrow(payload, i);
         vector::push_back(&mut sign, *value);
         i = i + 1;
      };

      sign
   }

   /// Get the authentication key of the given authenticator.
   public fun ecdsa_k1_authentication_key(payload: &vector<u8>): vector<u8> {
      let public_key = ecdsa_k1_public_key(payload);
      let addr = ecdsa_k1_public_key_to_address(public_key);
      moveos_std::bcs::to_bytes(&addr)
   }

   public fun ecdsa_k1_public_key_to_address(public_key: vector<u8>): address {
      let bytes = vector::singleton((SCHEME_ECDSA as u8));
      vector::append(&mut bytes, public_key);
      moveos_std::bcs::to_address(hash::blake2b256(&bytes))
   }

   public fun get_authentication_key(ctx: &StorageContext, addr: address): vector<u8> {
      let auth_key_option = account_authentication::get_authentication_key<EcdsaK1Validator>(ctx, addr);
      if(option::is_some(&auth_key_option)){
         option::extract(&mut auth_key_option)
      }else{
        //if AuthenticationKey does not exist, return addr as authentication key
        moveos_std::bcs::to_bytes(&addr)
      }
   }

   public fun validate(ctx: &StorageContext, payload: vector<u8>){
      // TODO handle non-ed25519 auth key and address relationship
      // let auth_key = ecdsa_k1_authentication_key(&payload);
      // let auth_key_in_account = get_authentication_key(ctx, storage_context::sender(ctx));
      // assert!(
      //    auth_key_in_account == auth_key,
      //    auth_validator::error_invalid_account_auth_key()
      // );
      assert!(
      ecdsa_k1::verify_recoverable(
         &ecdsa_k1_signature(&payload),
         &storage_context::tx_hash(ctx),
         SHA256, // KECCAK256:0, SHA256:1, TODO: The hash type may need to be passed through the authenticator
      ),
      auth_validator::error_invalid_authenticator()
      );
      assert!(
      ecdsa_k1::verify_nonrecoverable(
         &ecdsa_k1_signature(&payload),
         &ecdsa_k1_public_key(&payload),
         &storage_context::tx_hash(ctx),
         KECCAK256, // KECCAK256:0, SHA256:1, TODO: The hash type may need to be passed through the authenticator
      ),
      auth_validator::error_invalid_authenticator()
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

   // this test ensures that the ecdsa_k1_public_key_to_address function is compatible with the one in the rust code
   #[test]
   fun test_ecdsa_k1_public_key_to_address(){
      let public_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f";
      let addr = ecdsa_k1_public_key_to_address(public_key);
      assert!(addr == @0x92718e81a52369b4bc3169161737318ddf022945391a69263e8d4289c79a0c67, 1000);
   }
}