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
    const EInvalidScriptVersion: u64 = 1;
    const EInvalidPublicKeyLength: u64 = 2;
    const EInvalidVersionZeroPublicKeyLength: u64 = 3;
    const EInvalidVersionOnePublicKeyLength: u64 = 4;

    struct BTCAddress has store, drop {
        bytes: vector<u8>,
    }

    public fun new_legacy(pub_key: vector<u8>, decimal_prefix: u8): BTCAddress {
        // Check the decimal_prefix, i.e. address type
        assert!(
            decimal_prefix == 0 || decimal_prefix == 5,
            error::invalid_argument(EInvalidDecimalPrefix)
        );
        // Check the public key length
        assert!(
            vector::length(&pub_key) == 32,
            error::invalid_argument(EInvalidPublicKeyLength)
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
        // Check the script version
        assert!(
            version <= 16,
            error::invalid_argument(EInvalidScriptVersion)
        );
        // Check the script version and the public key relationship
        assert!(
            version == 0 && (vector::length(&pub_key) == 20 || vector::length(&pub_key) == 32),
            error::invalid_argument(EInvalidVersionZeroPublicKeyLength)
        );
        assert!(
            version == 1 && vector::length(&pub_key) == 32,
            error::invalid_argument(EInvalidVersionOnePublicKeyLength)
        );
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

    // Function to create a Bech32 address based on the given steps: https://en.bitcoin.it/wiki/Bech32.
    // Address type depends on the pub_key and version variables. Different input pub_key lengths and versions result in different address types.
    // i.e. P2wpkh uses 20 bytes public key and P2wsh uses 32 bytes public key for witness version v0. P2tr uses 32 bytes public key for witness version v1. 
    public fun create_bech32_address(pub_key: vector<u8>, version: u8): BTCAddress {
        let address_bytes = encoding::bech32(&pub_key, version);

        BTCAddress {
            bytes: address_bytes
        }
    }
}