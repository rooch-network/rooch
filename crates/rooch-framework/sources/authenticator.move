module rooch_framework::authenticator {

   const SCHEME_ED25519: u64 = 0;
   const SCHEME_MULTIED25519: u64 = 1;
   const SCHEME_SECP256K1: u64 = 2;

   const EUnsupportedScheme: u64 = 1000;

   struct AuthenticatorInfo has copy, store, drop {
      sequence_number: u64,
      authenticator: Authenticator,
   }

   struct Authenticator has copy, store, drop {
      scheme: u64,
      payload: vector<u8>,
   }

   //TODO migrate this function to a more suitable place
   public fun is_builtin_scheme(scheme: u64): bool {
      scheme == SCHEME_ED25519 || scheme == SCHEME_MULTIED25519 || scheme == SCHEME_SECP256K1
   }
}