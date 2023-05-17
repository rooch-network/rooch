module rooch_framework::authenticator{

    use rooch_framework::address_mapping::MultiChainAddress;

    const SCHEME_ED25519:u64 = 0;
    const SCHEME_MULTIED25519:u64 = 1;
    const SCHEME_SECP256K1:u64 = 2;

    const EUnsupportedScheme:u64 = 1000;

    struct AuthenticatorInfo has copy, store, drop {
        sender: MultiChainAddress,
        authenticator: Authenticator,
    }

    struct Authenticator has copy, store, drop {
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
    public fun check_authenticator(authenticator: &Authenticator) {
        assert!(is_builtin_scheme(authenticator.scheme), EUnsupportedScheme);
    }

    public fun decode_authenticator_info(data: vector<u8>) : (MultiChainAddress, Authenticator) {
        let info = moveos_std::bcd::from_bytes<AuthenticatorInfo>(data);
        let AuthenticatorInfo{sender, authenticator} = info;
        (sender, authenticator)
    }

    public fun decode_ed25519_authenticator(authenticator: Authenticator) : Ed25519Authenticator {
        assert!(authenticator.scheme == SCHEME_ED25519, EUnsupportedScheme);
        moveos_std::bcd::from_bytes<Ed25519Authenticator>(authenticator.payload)
    }

    public fun decode_multied25519_authenticator(authenticator: Authenticator) : MultiEd25519Authenticator {
        assert!(authenticator.scheme == SCHEME_MULTIED25519, EUnsupportedScheme);
        moveos_std::bcd::from_bytes<MultiEd25519Authenticator>(authenticator.payload)
    }

    public fun decode_secp256k1_authenticator(authenticator: Authenticator) : Secp256k1Authenticator {
        assert!(authenticator.scheme == SCHEME_SECP256K1, EUnsupportedScheme);
        moveos_std::bcd::from_bytes<Secp256k1Authenticator>(authenticator.payload)
    }
}