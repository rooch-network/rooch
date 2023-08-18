/// Module which defines decoding functions.
module rooch_framework::decoding {
   /// @param encoded_address_bytes: encoded Bitcoin address bytes on the Bitcoin network 
   /// @param version_byte: version byte used on Bitcoin network for verification of different types of addresses
   /// Decode the Bitcoin address bytes with Base58Check algorithm and returns a raw address bytes
   native public fun base58check(encoded_address_bytes: &vector<u8>, version_byte: u8): vector<u8>;

}