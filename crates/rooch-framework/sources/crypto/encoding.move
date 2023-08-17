/// Module which defines encoding functions.
module rooch_framework::encoding {
   /// @param address_bytes: address bytes on the Bitcoin network 
   /// Encode the address bytes with Base58Check algorithm and returns an encoded Bitcoin address
   native public fun base58check(address_bytes: &vector<u8>): vector<u8>;

   /// @param public_key: 32 bytes compressed public key
   /// Encode the public key with Bech32 or Bech32m encoding algorithm and returns 42 or 62 length Bitcoin Bech32 address.
   native public fun bech32(public_key: &vector<u8>, version: u8): vector<u8>;

   /// @param public_key: 32 bytes compressed public key
   /// Creates a pay to script hash P2SH address from a script converted from a compressed public key.
   native public fun p2sh(public_key: &vector<u8>): vector<u8>;

   /// @param public_key: 32 bytes compressed public key
   /// Creates a pay to (compressed) public key hash address from a public key.
   native public fun p2pkh(public_key: &vector<u8>): vector<u8>;

   /// TODO add tests here
}
