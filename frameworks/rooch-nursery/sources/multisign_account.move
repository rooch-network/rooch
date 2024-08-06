// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Bitcoin multisign account module
module rooch_nursery::multisign_account{

    use std::vector;
    use moveos_std::signer;
    use moveos_std::object::{Object};
    use moveos_std::account::{Self, Account};
    use moveos_std::table_vec::{Self, TableVec};
    use moveos_std::table::{Self, Table};
    use moveos_std::bcs;
    use rooch_framework::ecdsa_k1;
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};

    const PROPOSAL_STATUS_PENDING: u8 = 0;
    const PROPOSAL_STATUS_APPROVED: u8 = 1;
    const PROPOSAL_STATUS_REJECTED: u8 = 2;


    const ErrorInvalidThreshold: u64 = 1;
    const ErrorMultisignAccountNotFound: u64 = 2;
    const ErrorInvalidParticipant: u64 = 3;
    const ErrorParticipantMustHasBitcoinAddress: u64 = 4;
    const ErrorParticipantAlreadyJoined: u64 = 5;
    const ErrorInvalidPublicKey: u64 = 6;
    const ErrorInvalidProposal: u64 = 7;
    const ErrorProposalAlreadySigned: u64 = 8;
    const ErrorInvalidProposalStatus: u64 = 9;
    const ErrorInvalidSignature: u64 = 10; 

    struct MultisignAccountInfo has key, store {
        /// The multisign account rooch address
        multisign_address: address,
        /// The multisign account BitcoinAddress
        multisign_bitcoin_address: BitcoinAddress,
        /// The multisign account threshold
        threshold: u64,
        /// The taproot public key of the multisign account
        multisign_public_key: vector<u8>,
        /// The public keys of the multisign account
        participants: Table<address, ParticipantInfo>,
        /// The multisign account proposals on bitcoin
        bitcoin_proposals: TableVec<BitcoinProposal>,
        /// The multisign account proposals on rooch
        rooch_proposals: TableVec<RoochProposal>,
    }

    struct ParticipantInfo has store {
        /// The participant address
        participant_address: address,
        /// The participant BitcoinAddress
        participant_bitcoin_address: BitcoinAddress,
        /// The participant public key
        public_key: vector<u8>,
    }

    struct BitcoinProposal has store {
        /// The multisign account address
        multisign_address: address,
        /// The proposal id
        proposal_id: u64,
        /// The proposal transaction id
        tx_id: address,
        /// The proposal transaction data to be signed
        tx_data: vector<u8>,
        /// The proposal status
        status: u8,
        /// The proposal result
        result: u8,
        /// The proposal participants
        participants: vector<address>,
        /// The proposal signatures
        signatures: vector<vector<u8>>,
    }

    struct RoochProposal has store {
        /// The multisign account address
        multisign_address: address,
        /// The proposal id
        proposal_id: u64,
        /// The proposal transaction id
        tx_id: address,
        /// The proposal transaction data
        /// The signer will sign the sign_message + tx_id
        /// We keep the tx_data here for the verify the tx_id and display to the user
        tx_data: vector<u8>,
        /// The sign message to be signed
        sign_message: vector<u8>,
        /// The proposal status
        status: u8,
        /// The proposal result
        result: u8,
        /// The proposal participants
        participants: vector<address>,
        /// The proposal signatures
        signatures: vector<vector<u8>>,
    }

    /// Initialize a taproot multisign account
    /// If the multisign account already exists, we will init the MultisignAccountInfo into the account
    public entry fun initialize_multisig_account_entry(
        participant_public_keys: vector<vector<u8>>,
        threshold: u64,
    ){
        initialize_multisig_account(participant_public_keys, threshold);
    }
    
    public fun initialize_multisig_account(
        participant_public_keys: vector<vector<u8>>,
        threshold: u64,
    ): address {
        assert!(vector::length(&participant_public_keys) >= threshold, ErrorInvalidThreshold);
        let multisign_public_key = bitcoin_address::derive_multisig_pubkey_from_pubkeys(participant_public_keys, threshold);
        let multisign_bitcoin_address = bitcoin_address::derive_bitcoin_taproot_address_from_pubkey(&multisign_public_key);
        let multisign_address = bitcoin_address::to_rooch_address(&multisign_bitcoin_address);
        let participants = table::new();
        let idx = 0;
        let len = vector::length(&participant_public_keys);
        while(idx < len){
            let public_key = *vector::borrow(&participant_public_keys, idx);
            let participant_bitcoin_address = bitcoin_address::derive_bitcoin_taproot_address_from_pubkey(&public_key);
            let participant_address = bitcoin_address::to_rooch_address(&participant_bitcoin_address);
            let participant_info = ParticipantInfo {
                participant_address,
                participant_bitcoin_address,
                public_key,
            };
            table::add(&mut participants, participant_address, participant_info);
            idx = idx + 1;
        };
        let multisign_account_info = MultisignAccountInfo {
            multisign_bitcoin_address,
            multisign_address,
            threshold,
            multisign_public_key,
            participants,
            bitcoin_proposals: table_vec::new(),
            rooch_proposals: table_vec::new(),
        };
        let account = borrow_mut_or_create_account(multisign_address);
        account::account_move_resource_to(account, multisign_account_info);
        multisign_address
    }

    public fun is_participant(multisign_address: address, participant_address: address) : bool {
        if(!account::exists_at(multisign_address)){
            return false
        };
        let account = borrow_account(multisign_address);
        let multisign_account_info = account::account_borrow_resource<MultisignAccountInfo>(account);
        table::contains(&multisign_account_info.participants, participant_address)
    }

    public fun bitcoin_address(multisign_address: address) : BitcoinAddress {
        let account = borrow_account(multisign_address);
        let multisign_account_info = account::account_borrow_resource<MultisignAccountInfo>(account);
        multisign_account_info.multisign_bitcoin_address
    }

    public fun submit_bitcoin_proposal(
        sender: &signer,
        multisign_address: address,
        tx_id: address,
        tx_data: vector<u8>,
        participants: vector<address>,
    ){
        assert!(account::exists_at(multisign_address), ErrorMultisignAccountNotFound);
        let account = borrow_mut_or_create_account(multisign_address);
        let multisign_account_info = account::account_borrow_mut_resource<MultisignAccountInfo>(account);

        let sender_addr = signer::address_of(sender);
        assert!(table::contains(&multisign_account_info.participants, sender_addr), ErrorInvalidParticipant);

        let proposal_id = table_vec::length(&multisign_account_info.bitcoin_proposals);
        let proposal = BitcoinProposal {
            multisign_address,
            proposal_id,
            tx_id,
            tx_data,
            status: 0,
            result: 0,
            participants,
            signatures: vector::empty(),
        };
        table_vec::push_back(&mut multisign_account_info.bitcoin_proposals, proposal);
    }

    public fun sign_bitcoin_proposal(
        sender: &signer,
        multisign_address: address,
        proposal_id: u64,
        signature: vector<u8>,
    ){
        assert!(account::exists_at(multisign_address), ErrorMultisignAccountNotFound);
        let account = borrow_mut_or_create_account(multisign_address);
        let multisign_account_info = account::account_borrow_mut_resource<MultisignAccountInfo>(account);

        let sender_addr = signer::address_of(sender);
        assert!(table::contains(&multisign_account_info.participants, sender_addr), ErrorInvalidParticipant);

        assert!(table_vec::contains(&multisign_account_info.bitcoin_proposals, proposal_id), ErrorInvalidProposal);
        
        let proposal = table_vec::borrow_mut(&mut multisign_account_info.bitcoin_proposals, proposal_id);
        assert!(proposal.status == PROPOSAL_STATUS_PENDING, ErrorInvalidProposalStatus);
        assert!(!vector::contains(&proposal.participants, &sender_addr), ErrorProposalAlreadySigned);

        
        verify_bitcoin_signature(proposal.tx_id, &signature, &multisign_account_info.multisign_public_key);
        
        vector::push_back(&mut proposal.signatures, signature);
        if(vector::length(&proposal.signatures) >= multisign_account_info.threshold){
            proposal.status = PROPOSAL_STATUS_APPROVED;
        }
    }


    fun verify_bitcoin_signature(tx_id: address, signature: &vector<u8>, public_key: &vector<u8>) {
        //TODO we need verify_schnorr?
        assert!(
            ecdsa_k1::verify(
                signature,
                public_key,
                &bcs::to_bytes(&tx_id),
                ecdsa_k1::sha256()
            ),
            ErrorInvalidSignature
        );
    }

    fun borrow_mut_or_create_account(multisign_address: address) : &mut Object<Account>{
        let module_signer = signer::module_signer<MultisignAccountInfo>();
        let signer = if(!account::exists_at(multisign_address)){
            account::create_account_by_system(&module_signer, multisign_address)
        }else{
            account::create_signer_for_system(&module_signer, multisign_address)
        };
        account::borrow_mut_account(&signer)
    }

    fun borrow_account(multisign_address: address) : &Object<Account>{
        account::borrow_account(multisign_address)
    }

}