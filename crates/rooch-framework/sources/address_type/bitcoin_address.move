module rooch_framework::bitcoin_address{
    use std::error;
    use std::vector;
    use rooch_framework::encoding;
    use rooch_framework::ecdsa_k1;

    // P2PKH addresses are 34 characters
    const P2PKH_ADDR_LENGTH: u64 = 34;
    // P2SH addresses are 34 characters
    const P2SH_ADDR_LENGTH: u64 = 34;
    // Bech32 addresses including P2WPKH and P2WSH are 42 characters
    const BECH32_ADDR_LENGTH: u64 = 42;
    // P2TR addresses with Bech32m encoding are 62 characters
    const P2TR_ADDR_LENGTH: u64 = 62;

    /// error code
    const ErrorInvalidDecimalPrefix: u64 = 0;
    const ErrorInvalidScriptVersion: u64 = 1;
    const ErrorInvalidCompressedPublicKeyLength: u64 = 2;
    const ErrorInvalidHashedPublicKeyLength: u64 = 3;
    const ErrorInvalidSchnorrPublicKeyLength: u64 = 4;

    // P2PKH address decimal prefix
    const P2PKH_ADDR_DECIMAL_PREFIX: u8 = 0;
    // P2SH address decimal prefix
    const P2SH_ADDR_DECIMAL_PREFIX: u8 = 5;

    struct BTCAddress has store, drop {
        bytes: vector<u8>,
    }

    public fun new_legacy(pub_key: &vector<u8>, decimal_prefix: u8): BTCAddress {
        // Check the decimal_prefix, i.e. address type
        assert!(
            decimal_prefix == P2PKH_ADDR_DECIMAL_PREFIX
            || decimal_prefix == P2SH_ADDR_DECIMAL_PREFIX,
            error::invalid_argument(ErrorInvalidDecimalPrefix)
        );
        // Check the public key length
        assert!(
            vector::length(pub_key) == ecdsa_k1::public_key_length(),
            error::invalid_argument(ErrorInvalidCompressedPublicKeyLength)
        );
        // Perform address creation
        let bitcoin_address = if (decimal_prefix == P2PKH_ADDR_DECIMAL_PREFIX) { // P2PKH address
            create_p2pkh_address(pub_key)
        } else if (decimal_prefix == P2SH_ADDR_DECIMAL_PREFIX) { // P2SH address
            create_p2sh_address(pub_key)
        } else {
            BTCAddress {
                bytes: vector::empty<u8>()
            }
        };

        bitcoin_address
    }

    public fun new_bech32(pub_key: &vector<u8>, version: u8): BTCAddress {
        // Check the script version
        assert!(
            version <= 16,
            error::invalid_argument(ErrorInvalidScriptVersion)
        );
        // Check the script version and the public key relationship
        if (version == 0) {
            assert!(
                vector::length(pub_key) == 20 || vector::length(pub_key) == 32,
                error::invalid_argument(ErrorInvalidHashedPublicKeyLength)
            );
        };
        if (version == 1) {
            assert!(
                vector::length(pub_key) == 32,
                error::invalid_argument(ErrorInvalidSchnorrPublicKeyLength)
            );
        };
        // This will create Segwit Bech32 or Taproot Bech32m addresses depending on the public key length and the script version
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

    public fun create_p2pkh_address(pub_key: &vector<u8>): BTCAddress {
        let address_bytes = encoding::p2pkh(pub_key);

        BTCAddress {
            bytes: address_bytes
        }
    }

    public fun create_p2sh_address(pub_key: &vector<u8>): BTCAddress {
        let address_bytes = encoding::p2sh(pub_key);

        BTCAddress {
            bytes: address_bytes
        }
    }

    // Function to create a Bech32 address based on the given steps: https://en.bitcoin.it/wiki/Bech32.
    // Address type depends on the pub_key and version variables. Different input pub_key lengths and versions result in different address types.
    // i.e. P2wpkh uses 20 bytes public key and P2wsh uses 32 bytes public key for witness version v0. P2tr uses 32 bytes public key for witness version v1. 
    public fun create_bech32_address(pub_key: &vector<u8>, version: u8): BTCAddress {
        let address_bytes = encoding::bech32(pub_key, version);

        BTCAddress {
            bytes: address_bytes
        }
    }

    #[test]
    fun test_p2pkh_legacy_address() {
        let pub_key = x"021b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f";
        let address = new_legacy(&pub_key, 0);
        let expected_address = b"1LZ6VbuNSP53FRQmtd6fTGH5m6iLdK68PP";
        assert!(address.bytes == expected_address, 1000);
    }

    #[test]
    fun test_p2sh_legacy_address() {
        let pub_key = x"03a819b6f0eb5f22167fffa53e1628cfbf645db9a4c50b3a226e5d20c9984e63a2";
        let address = new_legacy(&pub_key, 5);
        let expected_address = b"38Kf3LA93erdEXwH1x8cKhyRwDTaWbBKam";
        assert!(address.bytes == expected_address, 1001);
    }

    #[test]
    fun test_bech32_p2wpkh_address() {
        let pub_key = x"751e76e8199196d454941c45d1b3a323f1433bd6";
        let address = new_bech32(&pub_key, 0);
        let expected_address = b"bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
        assert!(address.bytes == expected_address, 1002);
    }

    #[test]
    fun test_bech32_p2wsh_address() {
        let pub_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd07";
        let address = new_bech32(&pub_key, 0);
        let expected_address = b"bc1qqvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rsgej0cd";
        assert!(address.bytes == expected_address, 1003);
    }

    #[test]
    fun test_bech32m_p2tr_address() {
        let pub_key = x"036d70f73022e2097b22b5c4263638ed88732de69a715cfe2f18b3c3dbf5a2a5";
        let address = new_bech32(&pub_key, 1);
        let expected_address = b"bc1pqdkhpaesyt3qj7ezkhzzvd3caky8xt0xnfc4el30rzeu8kl452jspu6fl4";
        assert!(address.bytes == expected_address, 1004);
    }
}