/// This module implements the schnorr validator scheme.
module rooch_framework::schnorr_validator {

   use moveos_std::storage_context::{Self, StorageContext};
   use rooch_framework::schnorr;
   use rooch_framework::auth_validator;

   const SCHEME_SCHNORR: u64 = 3;

   struct SchnorrValidator has store{
   }

   public fun scheme(): u64 {
      SCHEME_SCHNORR
   }

   public fun validate(ctx: &StorageContext, payload: vector<u8>){
      //FIXME check the address and public key relationship
      assert!(
      schnorr::verify(
            &payload,
            &storage_context::tx_hash(ctx),
            1 // KECCAK256:0, SHA256:1, TODO: The hash type may need to be passed through the authenticator
      ),
      auth_validator::error_invalid_authenticator());
   }

   fun pre_execute(
      _ctx: &mut StorageContext,
   ) { 
   }
   
   fun post_execute(
      _ctx: &mut StorageContext,
   ) {
   }
}