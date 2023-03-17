module mos_framework::account{

   public fun create_account(_auth_key: vector<u8>): signer{
      abort 0
   }

   native fun create_signer(addr: address): signer;
}