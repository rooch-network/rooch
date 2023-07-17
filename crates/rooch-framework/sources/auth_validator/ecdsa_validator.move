/// This module implements the ecdsa validator scheme.
module rooch_framework::ecdsa_validator {

   use moveos_std::storage_context::{Self, StorageContext};
   use rooch_framework::ecdsa_k1;
   use rooch_framework::auth_validator;

   const SCHEME_ECDSA: u64 = 2;

   struct EcdsaValidator has store{
   }

   public fun scheme(): u64 {
      SCHEME_ECDSA
   }

   public fun validate(ctx: &StorageContext, payload: vector<u8>){
      //FIXME check the address and public key relationship
      assert!(
      ecdsa_k1::verify(
            &payload,
            &storage_context::tx_hash(ctx),
            0 // KECCAK256:0, SHA256:1, TODO: The hash type may need to be passed through the authenticator
      ),
      auth_validator::error_invalid_authenticator());
   }

}