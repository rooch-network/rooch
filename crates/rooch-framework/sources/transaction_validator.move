module rooch_framework::transaction_validator {
    use std::error;
    use std::option;
    use moveos_std::storage_context::{Self,StorageContext};
    use rooch_framework::account;
    use rooch_framework::authenticator;
    use rooch_framework::ed25519;
    use rooch_framework::ecdsa_k1;
    use rooch_framework::address_mapping::{Self,MultiChainAddress};

    const MAX_U64: u128 = 18446744073709551615;

    /// Scheme identifier for Ed25519 signatures used to derive authentication keys for Ed25519 public keys.
    const ED25519_SCHEME: u64 = 0;
    /// Scheme identifier for MultiEd25519 signatures used to derive authentication keys for MultiEd25519 public keys.
    const MULTI_ED25519_SCHEME: u64 = 1;
    const SECP256K1_SCHEME: u64 = 2;

    /// Transaction exceeded its allocated max gas
    const EOUT_OF_GAS: u64 = 6;

    /// Prologue errors. These are separated out from the other errors in this
    /// module since they are mapped separately to major VM statuses, and are
    /// important to the semantics of the system.
    const EPrologueInvalidAccountAuthKey: u64 = 1001;
    const EPrologueSequenceNuberTooOld: u64 = 1002;
    const EPrologueSequenceNumberTooNew: u64 = 1003;
    const EPrologueAccountDoesNotExist: u64 = 1004;
    const EPrologueCantPayGasDeposit: u64 = 1005;
    const EPrologueTransactionExpired: u64 = 1006;
    const EPrologueBadChainId: u64 = 1007;
    const EPrologueSequenceNumberTooBig: u64 = 1008;
    const EPrologueSecondaryKeysAddressesCountMismatch: u64 = 1009;

    /// InvalidAuthenticator, incloude invalid signature
    const EInvalidAuthenticator: u64 = 1010;

    #[view]
    /// This function is for Rooch to validate the transaction sender's authenticator.
    /// If the authenticator is invaid, abort this function.
    public fun validate(ctx: &StorageContext, authenticator_info_bytes: vector<u8>){
        let (tx_sequence_number, authenticator) = authenticator::decode_authenticator_info(authenticator_info_bytes);
        authenticator::check_authenticator(&authenticator);
        let scheme = authenticator::scheme(&authenticator);
        if (scheme == ED25519_SCHEME) {
            let ed25519_authenicator = authenticator::decode_ed25519_authenticator(authenticator);
            //FIXME we need to check the public key and address relationship
            //The address is the public key's hash
            //We also need to check the public key via account's auth key, if the user rotate the auth key. 
            assert!(
            ed25519::verify(&authenticator::ed25519_signature(&ed25519_authenicator),
                &authenticator::ed25519_public(&ed25519_authenicator),
                &storage_context::tx_hash(ctx)),
            error::invalid_argument(EInvalidAuthenticator));
        } else if (scheme == SECP256K1_SCHEME) {
            let ecdsa_k1_authenicator = authenticator::decode_secp256k1_authenticator(authenticator);
            assert!(
            ecdsa_k1::verify(
                &authenticator::secp256k1_signature(&ecdsa_k1_authenicator),
                &storage_context::tx_hash(ctx),
                0 // KECCAK256:0, SHA256:1, TODO: The hash type may need to be passed through the authenticator
            ),
            error::invalid_argument(EInvalidAuthenticator));
        };

        assert!(
            (tx_sequence_number as u128) < MAX_U64,
            error::out_of_range(EPrologueSequenceNumberTooBig)
        );

        let account_sequence_number = account::sequence_number_for_sender(ctx);
        assert!(
            tx_sequence_number >= account_sequence_number,
            error::invalid_argument(EPrologueSequenceNuberTooOld)
        );

        // [PCA12]: Check that the transaction's sequence number matches the
        // current sequence number. Otherwise sequence number is too new by [PCA11].
        //assert!(
        //    tx_sequence_number == account_sequence_number,
        //    error::invalid_argument(EPrologueSequenceNumberTooNew)
        //);
    }

    /// Transaction pre_execute function.
    /// Execute before the transaction is executed, automatically called by the MoveOS VM.
    /// This function is for Rooch to auto create account and address maping.
    fun pre_execute(
        ctx: &mut StorageContext,
    ) { 
        let sender = storage_context::sender(ctx);
        //Auto create account if not exist
        if (!account::exists_at(ctx, sender)) {
            account::create_account(ctx, sender); 
        };
        // the transaction validator will put the multi chain address into the context
        let multichain_address = storage_context::get<MultiChainAddress>(ctx);
        if (option::is_some(&multichain_address)){
            let multichain_address = option::extract(&mut multichain_address);
            //Auto create address mapping if not exist
            if (!address_mapping::exists_mapping(ctx, multichain_address)) {
                address_mapping::bind_no_check(ctx, sender, multichain_address); 
            };
        }
    }

    /// Transaction post_execute function.
    /// Execute after the transaction is executed, automatically called by the MoveOS VM.
    /// This function is for Rooch to update the sender's sequence number and pay the gas fee.
    fun post_execute(
        ctx: &mut StorageContext,
    ) { 
        //TODO handle transaction gas fee
        // Increment sequence number
        account::increment_sequence_number(ctx);
    }
}
