module rooch_framework::authenticator{

    const SCHEME_ED25519:u64 = 0;
    const SCHEME_MULTIED25519:u64 = 1;
    const SCHEME_SECP256K1:u64 = 2;

    const EUnsupportedScheme:u64 = 1000;

    struct AuthenticatorInfo has copy, store, drop{
        scheme: u64,
        payload: vector<u8>,
    }

    struct Ed25519Authenticator has copy, store, drop {
        public_key: vector<u8>,
        signature: vector<u8>,
    }

    struct MultiEd25519Authenticator has copy, store, drop {
        public_key: vector<u8>,
        signature: vector<u8>,
    }

    struct Secp256k1Authenticator has copy, store, drop {
        signature: vector<u8>,
    }

    fun is_builtin_scheme(scheme: u64) : bool {
        scheme == SCHEME_ED25519 || scheme == SCHEME_MULTIED25519 || scheme == SCHEME_SECP256K1
    }

    /// Check if we can handle the given authenticator info.
    /// If not, just abort
    public fun check_authenticator(authenticator: &AuthenticatorInfo) {
        assert!(is_builtin_scheme(authenticator.scheme), EUnsupportedScheme);
    }

    public fun decode_authenticator_info(data: vector<u8>) : AuthenticatorInfo {
        moveos_std::bcd::from_bytes<AuthenticatorInfo>(data)
    }

    public fun decode_ed25519_authenticator(info: AuthenticatorInfo) : Ed25519Authenticator {
        assert!(info.scheme == SCHEME_ED25519, EUnsupportedScheme);
        moveos_std::bcd::from_bytes<Ed25519Authenticator>(info.payload)
    }

    public fun decode_multied25519_authenticator(info: AuthenticatorInfo) : MultiEd25519Authenticator {
        assert!(info.scheme == SCHEME_MULTIED25519, EUnsupportedScheme);
        moveos_std::bcd::from_bytes<MultiEd25519Authenticator>(info.payload)
    }

    public fun decode_secp256k1_authenticator(info: AuthenticatorInfo) : Secp256k1Authenticator {
        assert!(info.scheme == SCHEME_SECP256K1, EUnsupportedScheme);
        moveos_std::bcd::from_bytes<Secp256k1Authenticator>(info.payload)
    }
}