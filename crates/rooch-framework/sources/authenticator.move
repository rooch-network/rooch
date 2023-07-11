module rooch_framework::authenticator {

   use std::vector;
   use rooch_framework::hash;

   const SCHEME_ED25519: u64 = 0;
   const SCHEME_MULTIED25519: u64 = 1;
   const SCHEME_SECP256K1: u64 = 2;

   const ED25519_SCHEME_LENGTH: u64 = 1;
   const ED25519_PUBKEY_LENGTH: u64 = 32;
   const ED25519_SIG_LENGTH: u64 = 64;

   const EUnsupportedScheme: u64 = 1000;

   struct AuthenticatorInfo has copy, store, drop {
      sequence_number: u64,
      authenticator: Authenticator,
   }

   struct Authenticator has copy, store, drop {
      scheme: u64,
      payload: vector<u8>,
   }

   struct Ed25519Authenticator has copy, store, drop {
      signature: vector<u8>,
   }

   struct MultiEd25519Authenticator has copy, store, drop {
      public_key: vector<u8>,
      signature: vector<u8>,
   }

   struct Secp256k1Authenticator has copy, store, drop {
      signature: vector<u8>,
   }

   fun is_builtin_scheme(scheme: u64): bool {
      scheme == SCHEME_ED25519 || scheme == SCHEME_MULTIED25519 || scheme == SCHEME_SECP256K1
   }

   /// Check if we can handle the given authenticator info.
   /// If not, just abort
   public fun check_authenticator(authenticator: &Authenticator) {
      assert!(is_builtin_scheme(authenticator.scheme), EUnsupportedScheme);
   }

   public fun scheme(self: &Authenticator): u64 {
      self.scheme
   }

   public fun decode_authenticator_info(data: vector<u8>): (u64, Authenticator) {
      let info = moveos_std::bcs::from_bytes<AuthenticatorInfo>(data);
      let AuthenticatorInfo { sequence_number, authenticator } = info;
      (sequence_number, authenticator)
   }

   public fun decode_ed25519_authenticator(authenticator: Authenticator): Ed25519Authenticator {
      assert!(authenticator.scheme == SCHEME_ED25519, EUnsupportedScheme);
      moveos_std::bcs::from_bytes<Ed25519Authenticator>(authenticator.payload)
   }

   public fun ed25519_public(self: &Ed25519Authenticator): vector<u8> {
      let public_key = vector::empty<u8>();
      let i = ED25519_SCHEME_LENGTH + ED25519_SIG_LENGTH;
      while (i < ED25519_SCHEME_LENGTH + ED25519_SIG_LENGTH + ED25519_PUBKEY_LENGTH) {
         let value = vector::borrow(&self.signature, i);
         vector::push_back(&mut public_key, *value);
         i = i + 1;
      };

      public_key
   }

   public fun ed25519_signature(self: &Ed25519Authenticator): vector<u8> {
      let sign = vector::empty<u8>();
      let i = ED25519_SCHEME_LENGTH;
      while (i < ED25519_SIG_LENGTH + 1) {
         let value = vector::borrow(&self.signature, i);
         vector::push_back(&mut sign, *value);
         i = i + 1;
      };

      sign
   }

   /// Get the authentication key of the given authenticator.
   public fun ed25519_authentication_key(self: &Ed25519Authenticator): vector<u8> {
      let public_key = ed25519_public(self);
      let addr = ed25519_public_key_to_address(public_key);
      moveos_std::bcs::to_bytes(&addr)
   }

   public fun ed25519_public_key_to_address(public_key: vector<u8>): address {
      let bytes = vector::singleton((SCHEME_ED25519 as u8));
      vector::append(&mut bytes, public_key);
      moveos_std::bcs::to_address(hash::blake2b256(&bytes))
   }

   public fun decode_multied25519_authenticator(authenticator: Authenticator): MultiEd25519Authenticator {
      assert!(authenticator.scheme == SCHEME_MULTIED25519, EUnsupportedScheme);
      moveos_std::bcs::from_bytes<MultiEd25519Authenticator>(authenticator.payload)
   }

   public fun decode_secp256k1_authenticator(authenticator: Authenticator): Secp256k1Authenticator {
      assert!(authenticator.scheme == SCHEME_SECP256K1, EUnsupportedScheme);
      moveos_std::bcs::from_bytes<Secp256k1Authenticator>(authenticator.payload)
   }

   public fun secp256k1_signature(self: &Secp256k1Authenticator): vector<u8> {
      self.signature
   }

   // this test ensures that the ed25519_public_key_to_address function is compatible with the one in the rust code
   #[test]
   fun test_ed25519_public_key_to_address(){
      let public_key = x"3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29";
      let addr = ed25519_public_key_to_address(public_key);
      assert!(addr == @0x7a1378aafadef8ce743b72e8b248295c8f61c102c94040161146ea4d51a182b6, 1000)
   }
}