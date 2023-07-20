module rooch_framework::transaction_validator {
    use std::error;
    use std::option;
    use moveos_std::storage_context::{Self,StorageContext};
    use rooch_framework::account;
    use rooch_framework::address_mapping::{Self,MultiChainAddress};
    use rooch_framework::account_authentication;
    use rooch_framework::auth_validator_registry::{Self, AuthValidator};

    const MAX_U64: u128 = 18446744073709551615;


    /// Transaction exceeded its allocated max gas
    const EOUT_OF_GAS: u64 = 6;

    /// Validate errors. These are separated out from the other errors in this
    /// module since they are mapped separately to major VM statuses, and are
    /// important to the semantics of the system.


    const EValidateSequenceNuberTooOld: u64 = 1001;
    const EValidateSequenceNumberTooNew: u64 = 1002;
    const EValidateAccountDoesNotExist: u64 = 1003;
    const EValidateCantPayGasDeposit: u64 = 1004;
    const EValidateTransactionExpired: u64 = 1005;
    const EValidateBadChainId: u64 = 1006;
    const EValidateSequenceNumberTooBig: u64 = 1007;

    /// The authenticator's scheme is not installed to the sender's account
    const EValidateNotInstalledAuthValidator: u64 = 1010;

   
    #[view]
    /// This function is for Rooch to validate the transaction sender's authenticator.
    /// If the authenticator is invaid, abort this function.
    //TODO should we need the authenticator_payload?
    public fun validate(ctx: &StorageContext, tx_sequence_number: u64, scheme: u64, _authenticator_payload: vector<u8>): &AuthValidator {
        // === validate the sequence number ===
        
        assert!(
            (tx_sequence_number as u128) < MAX_U64,
            error::out_of_range(EValidateSequenceNumberTooBig)
        );

        let account_sequence_number = account::sequence_number_for_sender(ctx);
        assert!(
            tx_sequence_number >= account_sequence_number,
            error::invalid_argument(EValidateSequenceNuberTooOld)
        );

        // [PCA12]: Check that the transaction's sequence number matches the
        // current sequence number. Otherwise sequence number is too new by [PCA11].
        assert!(
            tx_sequence_number == account_sequence_number,
            error::invalid_argument(EValidateSequenceNumberTooNew)
        );

        // === validate the authenticator ===

        let sender = storage_context::sender(ctx);
        let auth_validator = auth_validator_registry::borrow_validator(ctx, scheme);
        let validator_id = auth_validator_registry::validator_id(auth_validator);
        // builtin scheme do not need to install
        if(!rooch_framework::builtin_validators::is_builtin_scheme(scheme)){
            assert!(account_authentication::is_auth_validator_installed(ctx, sender, validator_id), error::invalid_state(EValidateNotInstalledAuthValidator));
        };
        auth_validator
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
