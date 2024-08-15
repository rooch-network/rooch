module rooch_nursery::bitcoin_multisign_validator{

    use std::vector;
    use moveos_std::bcs;
    use moveos_std::tx_context;
    use moveos_std::hash;
    use moveos_std::hex;
    use rooch_framework::ecdsa_k1;
    use rooch_framework::auth_validator_registry;
    use rooch_framework::auth_validator;
    use rooch_nursery::multisign_account;

    friend rooch_nursery::genesis;

    const ErrorGenesisInitError: u64 = 1;

    /// there defines auth validator id for each auth validator
    const BITCOIN_MULTISIGN_VALIDATOR_ID: u64 = 2;

    struct BitcoinMultisignValidator has store, drop {}

    #[data_struct]
    struct AuthPayload has copy, store, drop {
        // Message signature
        signatures: vector<vector<u8>>,
        // Some wallets add magic prefixes, such as unisat adding 'Bitcoin Signed Message:\n'
        message_prefix: vector<u8>,
        // Description of a user-defined signature, without the tx_hash hex
        message_info: vector<u8>,
        // Public key of address
        public_keys: vector<vector<u8>>,
    }

    struct SignData has copy, drop {
        message_prefix: vector<u8>,
        // Description of a user-defined signature, include the tx_hash hex
        message_info: vector<u8>,
    }

    public fun auth_validator_id(): u64 {
        BITCOIN_MULTISIGN_VALIDATOR_ID
    }

    public(friend) fun genesis_init(){
        let id = auth_validator_registry::register<BitcoinMultisignValidator>();
        assert!(id == BITCOIN_MULTISIGN_VALIDATOR_ID, ErrorGenesisInitError);
    }

    fun encode_full_message(self: &AuthPayload, tx_hash: vector<u8>): vector<u8> {
        let tx_hex = hex::encode(tx_hash);
        let message_info = self.message_info;
        vector::append(&mut message_info, tx_hex);
        let sign_data = SignData {
            message_prefix: self.message_prefix,
            message_info: message_info,
        };
        bcs::to_bytes(&sign_data)
    }

    /// Only validate the authenticator's signature.
    fun validate_signatures(payload: &AuthPayload, tx_hash: vector<u8>) {
        assert!(
            vector::length(&payload.signatures) == vector::length(&payload.public_keys),
            auth_validator::error_validate_invalid_authenticator()
        );

        let message = encode_full_message(payload, tx_hash);

        // The Bitcoin wallet uses sha2_256 twice, the `ecdsa_k1::verify` function also does sha2_256 once
        let message_hash = hash::sha2_256(message);
        let i = 0;
        while (i < vector::length(&payload.signatures)) {
            assert!(
                ecdsa_k1::verify(
                    vector::borrow(&payload.signatures,i),
                    vector::borrow(&payload.public_keys,i),
                    &message_hash,
                    ecdsa_k1::sha256()
                ),
                auth_validator::error_validate_invalid_authenticator()
            );
            i = i + 1;
        }
    }

    fun validate_multisign_account(multisign_address: address, public_keys: &vector<vector<u8>>) {
        assert!(
            multisign_account::is_multisign_account(multisign_address),
            auth_validator::error_validate_invalid_authenticator()
        );
        let pubkey_len = vector::length(public_keys);
        assert!(pubkey_len >= multisign_account::threshold(multisign_address), auth_validator::error_validate_invalid_authenticator());
        let i = 0;
        while (i < pubkey_len) {
            assert!(
                multisign_account::is_participant_via_public_key(multisign_address, vector::borrow(public_keys,i)),
                auth_validator::error_validate_invalid_authenticator()
            );
            i = i + 1;
        }    
    }

    public fun validate(authenticator_payload: vector<u8>) {
        let sender = tx_context::sender();
        let tx_hash = tx_context::tx_hash();
        let payload = bcs::from_bytes<AuthPayload>(authenticator_payload);
        validate_multisign_account(sender, &payload.public_keys);
        validate_signatures(&payload, tx_hash);
    }

}