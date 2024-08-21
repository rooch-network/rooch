// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Bitcoin multisign account wallet to manage the multisign tx on Bitcoin and Rooch
module rooch_nursery::multisign_wallet{

    use std::vector;
    use moveos_std::signer;
    use moveos_std::object;
    use moveos_std::table_vec::{Self, TableVec};
    use moveos_std::bcs;
    use bitcoin_move::multisign_account;
    use rooch_framework::ecdsa_k1;

    const PROPOSAL_STATUS_PENDING: u8 = 0;
    const PROPOSAL_STATUS_APPROVED: u8 = 1;
    const PROPOSAL_STATUS_REJECTED: u8 = 2;

    const X_ONLY_PUBLIC_KEY_LEN: u64 = 32;
    const BITCOIN_COMPRESSED_PUBLIC_KEY_LEN: u64 = 33;

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

    struct MultisignWallet has key {
        bitcoin_proposals: TableVec<BitcoinProposal>,
        rooch_proposals: TableVec<RoochProposal>,
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

    fun create_or_borrow_mut_wallet(multisign_address: address) : &mut MultisignWallet {
        let wallet_id = object::account_named_object_id<MultisignWallet>(multisign_address);
        if (!object::exists_object(wallet_id)){
            let wallet = MultisignWallet {
                bitcoin_proposals: table_vec::new(),
                rooch_proposals: table_vec::new(),
            };
            let wallet_obj = object::new_account_named_object(multisign_address, wallet);
            object::transfer_extend(wallet_obj, multisign_address);
        };
        let wallet_obj = object::borrow_mut_object_extend<MultisignWallet>(wallet_id);
        object::borrow_mut(wallet_obj)
    }

    public fun submit_bitcoin_proposal(
        sender: &signer,
        multisign_address: address,
        tx_id: address,
        tx_data: vector<u8>,
    ){
        assert!(multisign_account::is_multisign_account(multisign_address), ErrorMultisignAccountNotFound);
        
        let sender_addr = signer::address_of(sender);
        assert!(multisign_account::is_participant(multisign_address, sender_addr), ErrorInvalidParticipant);

        let wallet = create_or_borrow_mut_wallet(multisign_address);

        let proposal_id = table_vec::length(&wallet.bitcoin_proposals);
        let proposal = BitcoinProposal {
            multisign_address,
            proposal_id,
            tx_id,
            tx_data,
            status: 0,
            result: 0,
            participants: vector::empty(),
            signatures: vector::empty(),
        };
        table_vec::push_back(&mut wallet.bitcoin_proposals, proposal);
    }

    public fun sign_bitcoin_proposal(
        sender: &signer,
        multisign_address: address,
        proposal_id: u64,
        signature: vector<u8>,
    ){
        assert!(multisign_account::is_multisign_account(multisign_address), ErrorMultisignAccountNotFound);
        
        let sender_addr = signer::address_of(sender);
        assert!(multisign_account::is_participant(multisign_address, sender_addr), ErrorInvalidParticipant);

        let wallet = create_or_borrow_mut_wallet(multisign_address);

        assert!(table_vec::contains(&wallet.bitcoin_proposals, proposal_id), ErrorInvalidProposal);
        
        let proposal = table_vec::borrow_mut(&mut wallet.bitcoin_proposals, proposal_id);
        assert!(proposal.status == PROPOSAL_STATUS_PENDING, ErrorInvalidProposalStatus);
        assert!(!vector::contains(&proposal.participants, &sender_addr), ErrorProposalAlreadySigned);

        let participant_public_key = multisign_account::participant_public_key(multisign_address, sender_addr);
        verify_bitcoin_signature(proposal.tx_id, &signature, &participant_public_key);
        let threshold = multisign_account::threshold(multisign_address);
        vector::push_back(&mut proposal.signatures, signature);
        if(vector::length(&proposal.signatures) >= threshold){
            proposal.status = PROPOSAL_STATUS_APPROVED;
        }
    }


    fun verify_bitcoin_signature(tx_id: address, signature: &vector<u8>, public_key: &vector<u8>) {
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

    fun check_public_keys(public_keys: &vector<vector<u8>>) {
        let idx = 0;
        let len = vector::length(public_keys);
        while(idx < len){
            let public_key = vector::borrow(public_keys, idx);
            check_public_key(public_key);
            idx = idx + 1;
        };
    }

    fun check_public_key(public_key: &vector<u8>) {
        let public_key_len = vector::length(public_key);
        assert!(public_key_len == BITCOIN_COMPRESSED_PUBLIC_KEY_LEN, ErrorInvalidPublicKey);
    }

}