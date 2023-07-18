/// This module implements the multi-ed25519 validator scheme.
module rooch_framework::multi_ed25519_validator {

   use moveos_std::storage_context::StorageContext;
   
   const SCHEME_MULTIED25519: u64 = 1;

   struct MultiEd25519Validator has store{
   }

   public fun scheme(): u64 {
      SCHEME_MULTIED25519
   }

   public fun validate(_ctx: &StorageContext, _payload: vector<u8>){
      //TODO
      abort std::error::not_implemented(1)
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