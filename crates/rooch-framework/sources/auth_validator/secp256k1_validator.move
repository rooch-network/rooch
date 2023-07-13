/// This module implements the secp256k1 validator scheme.
module rooch_framework::secp256k1_validator {

   use moveos_std::storage_context::{Self, StorageContext};
   use rooch_framework::ecdsa_k1;
   use rooch_framework::transaction_validator;

   const SCHEME_SECP256K1: u64 = 2;

   struct Secp256k1Validator has store{
   }

   public fun validate(ctx: &StorageContext, payload: vector<u8>){
      //FIXME check the address and public key relationship
      assert!(
      ecdsa_k1::verify(
            &payload,
            &storage_context::tx_hash(ctx),
            0 // KECCAK256:0, SHA256:1, TODO: The hash type may need to be passed through the authenticator
      ),
      transaction_validator::error_invalid_authenticator());
   }

}