module rooch_framework::bitcoin_address{
    use std::error;
    use std::vector;
    use rooch_framework::encoding;

    // P2PKH addresses are 34 characters
    const P2PKH_ADDR_LENGTH: u64 = 34;
    // P2SH addresses are 34 characters
    const P2SH_ADDR_LENGTH: u64 = 34;
    // Bech32 addresses including P2WPKH and P2WSH are 42 characters
    const BECH32_ADDR_LENGTH: u64 = 42;
    // P2TR addresses with Bech32m encoding are 62 characters
    const P2TR_ADDR_LENGTH: u64 = 62;

    /// error code
    const EInvalidDecimalPrefix: u64 = 0;

    struct BTCAddress has store, drop {
        bytes: vector<u8>,
    }

    public fun new_legacy(pub_key: vector<u8>, decimal_prefix: u8): BTCAddress {
        // Check the decimal_prefix, i.e. address type
        assert!(
            decimal_prefix == 0 || decimal_prefix == 5,
            error::invalid_argument(EInvalidDecimalPrefix)
        );
        // Perform address creation
        let bitcoin_address = if (decimal_prefix == 0) { // P2PKH address
            create_p2pkh_address(pub_key)
        } else if (decimal_prefix == 5) { // P2SH address
            create_p2sh_address(pub_key)
        } else {
            BTCAddress {
                bytes: vector::empty<u8>()
            }
        };

        bitcoin_address
    }

    public fun new_bech32(pub_key: vector<u8>, version: u8): BTCAddress {
        // This will create Segwit Bech32 or Taproot Bech32m addresses depending on the public key length and the version digit
        let bitcoin_address = create_bech32_address(pub_key, version);

        bitcoin_address
    }

    public fun as_bytes(addr: &BTCAddress): &vector<u8> {
        &addr.bytes
    }

    public fun into_bytes(addr: BTCAddress): vector<u8> {
        let BTCAddress { bytes } = addr;
        bytes
    }

    public fun create_p2pkh_address(pub_key: vector<u8>): BTCAddress {
        let address_bytes = encoding::p2pkh(&pub_key);

        BTCAddress {
            bytes: address_bytes
        }
    }

    public fun create_p2sh_address(pub_key: vector<u8>): BTCAddress {
        let address_bytes = encoding::p2sh(&pub_key);

        BTCAddress {
            bytes: address_bytes
        }
    }

    // Function to create a Bech32 address based on the given steps: https://en.bitcoin.it/wiki/Bech32
    // Step 1: Having a compressed public key (0x02 or 0x03 followed by 32 byte X coordinate)
    // Step 2: Perform SHA-256 hashing on the public key
    // Step 3: Perform RIPEMD-160 hashing on the result of SHA-256
    // Step 4: The result of step 3 is an array of 8-bit unsigned integers (base 2^8=256) and Bech32 encoding converts this to an array of 5-bit unsigned integers (base 2^5=32) so we 'squash' the bytes to get
    // Step 5: Add the witness version byte in front of the step 4 result (current version is 0): 
    // Step 6: Compute the checksum by using the data from step 5 and the H.R.P (bc for MainNet and tb for TestNet)
    // Step 7: Append the checksum to result of step 5 (we now have an array of 5-bit integers):
    // Step 8: Map each value to its corresponding character in Bech32Chars (qpzry9x8gf2tvdw0s3jn54khce6mua7l) 00 -> q, 0e -> w, etc.
    // Step 9: A Bech32_encoded address consists of 3 parts: HRP + Separator + Data.
    // Address type depends on the pub_key and version variables. Different input pub_key lengths and versions result in different address types.
    // i.e. P2wpkh uses 20 bytes public key and P2wsh uses 32 bytes public key for witness version v0. P2tr uses 32 bytes public key for witness version v1. 
    public fun create_bech32_address(pub_key: vector<u8>, version: u8): BTCAddress {
        let address_bytes = encoding::bech32(&pub_key, version);

        BTCAddress {
            bytes: address_bytes
        }
    }
}