/// This module implements the ed25519 validator scheme.
module rooch_framework::ed25519_validator {

   use std::error;
   use std::vector;
   use std::option;
   use std::signer;
   use moveos_std::storage_context::{Self, StorageContext};
   use rooch_framework::hash;
   use rooch_framework::account_authentication;
   use rooch_framework::ed25519;
   use rooch_framework::auth_validator;

   const SCHEME_ED25519: u64 = 0;

   const V_ED25519_SCHEME_LENGTH: u64 = 1;
   const V_ED25519_PUBKEY_LENGTH: u64 = 32;
   const V_ED25519_SIG_LENGTH: u64 = 64;
   /// error code
   const EMalformedAccount: u64 = 1001;
   const EMalformedAuthenticationKey: u64 = 1002;

   struct Ed25519Validator has store{
   }

   public fun scheme(): u64 {
      SCHEME_ED25519
   }

   public entry fun rotate_authentication_key_entry<Ed25519Validator>(ctx: &mut StorageContext, account: &signer, public_key: vector<u8>) {
      // compare newly passed public key with ed25519 public key length to ensure it's compatible
      assert!(
         vector::length(&public_key) == V_ED25519_PUBKEY_LENGTH,
         error::invalid_argument(EMalformedAuthenticationKey)
      );

      // ensure that the ed25519 public key to address matched with the ed25519 account address
      let account_addr = signer::address_of(account);
      let ed25519_addr = ed25519_public_key_to_address(public_key);
      assert!(
         account_addr == ed25519_addr,
         error::invalid_argument(EMalformedAccount)
      );

      // serialize the address to an auth key and rotate it by calling rotate_authentication_key
      let ed25519_authentication_key = moveos_std::bcs::to_bytes(&ed25519_addr);
      account_authentication::rotate_authentication_key<Ed25519Validator>(ctx, account, ed25519_authentication_key);
   }


   public fun ed25519_public_key(payload: &vector<u8>): vector<u8> {
      let public_key = vector::empty<u8>();
      let i = V_ED25519_SCHEME_LENGTH + V_ED25519_SIG_LENGTH;
      while (i < V_ED25519_SCHEME_LENGTH + V_ED25519_SIG_LENGTH + V_ED25519_PUBKEY_LENGTH) {
         let value = vector::borrow(payload, i);
         vector::push_back(&mut public_key, *value);
         i = i + 1;
      };

      public_key
   }

   public fun ed25519_signature(payload: &vector<u8>): vector<u8> {
      let sign = vector::empty<u8>();
      let i = V_ED25519_SCHEME_LENGTH;
      while (i < V_ED25519_SIG_LENGTH + 1) {
         let value = vector::borrow(payload, i);
         vector::push_back(&mut sign, *value);
         i = i + 1;
      };

      sign
   }

   /// Get the authentication key of the given authenticator payload.
   public fun get_authentication_key_from_payload(payload: &vector<u8>): vector<u8> {
      let public_key = ed25519_public_key(payload);
      let addr = ed25519_public_key_to_address(public_key);
      moveos_std::bcs::to_bytes(&addr)
   }

   public fun ed25519_public_key_to_address(public_key: vector<u8>): address {
      let bytes = vector::singleton((SCHEME_ED25519 as u8));
      vector::append(&mut bytes, public_key);
      moveos_std::bcs::to_address(hash::blake2b256(&bytes))
   }

   public fun get_authentication_key(ctx: &StorageContext, addr: address): vector<u8> {
      let auth_key_option = account_authentication::get_authentication_key<Ed25519Validator>(ctx, addr);
      if(option::is_some(&auth_key_option)){
         option::extract(&mut auth_key_option)
      }else{
        //if AuthenticationKey does not exist, return addr as authentication key
        moveos_std::bcs::to_bytes(&addr)
      }
   }

   /// Only validate the authenticator's signature.
   public fun validate_signature(authenticator_payload: &vector<u8>, tx_hash: &vector<u8>){
      assert!(
            ed25519::verify(
               &ed25519_signature(authenticator_payload),
               &ed25519_public_key(authenticator_payload),
               tx_hash
            ),
            auth_validator::error_invalid_authenticator()
        );
   }

   public fun validate(ctx: &StorageContext, authenticator_payload: vector<u8>){
      let tx_hash = storage_context::tx_hash(ctx);
      validate_signature(&authenticator_payload, &tx_hash);
        
      let auth_key = get_authentication_key_from_payload(&authenticator_payload);
      let auth_key_in_account = get_authentication_key(ctx, storage_context::sender(ctx));
      assert!(
         auth_key_in_account == auth_key,
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

   // this test ensures that the ed25519_public_key_to_address function is compatible with the one in the rust code
   #[test]
   fun test_ed25519_public_key_to_address(){
      let public_key = x"3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29";
      let addr = ed25519_public_key_to_address(public_key);
      assert!(addr == @0x7a1378aafadef8ce743b72e8b248295c8f61c102c94040161146ea4d51a182b6, 1000)
   }
}