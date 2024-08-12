// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Bitcoin multisign account module
module rooch_nursery::multisign_account{

    use std::vector;
    use std::option;
    use moveos_std::signer;
    use moveos_std::object::{Object};
    use moveos_std::account::{Self, Account};
    use moveos_std::table_vec::{Self, TableVec};
    use moveos_std::table::{Self, Table};
    use moveos_std::bcs;
    use moveos_std::compare;
    use bitcoin_move::opcode;
    use bitcoin_move::script_buf::{Self, ScriptBuf};
    use rooch_framework::ecdsa_k1;
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};
    use rooch_nursery::taproot_builder;

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

    struct MultisignAccountInfo has key, store {
        /// The multisign account rooch address
        multisign_address: address,
        /// The multisign account BitcoinAddress
        multisign_bitcoin_address: BitcoinAddress,
        /// The multisign account threshold
        threshold: u64,
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
        threshold: u64,
        participant_public_keys: vector<vector<u8>>,
    ){
        initialize_multisig_account(threshold, participant_public_keys);
    }
    
    public fun initialize_multisig_account(
        threshold: u64,
        participant_public_keys: vector<vector<u8>>,
    ): address {
        assert!(vector::length(&participant_public_keys) >= threshold, ErrorInvalidThreshold);
        check_public_keys(&participant_public_keys);
        let multisign_bitcoin_address = generate_multisign_address(threshold, participant_public_keys);
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
            participants,
            bitcoin_proposals: table_vec::new(),
            rooch_proposals: table_vec::new(),
        };
        let account = borrow_mut_or_create_account(multisign_address);
        account::account_move_resource_to(account, multisign_account_info);
        multisign_address
    }

    public fun generate_multisign_address(threshold: u64, public_keys: vector<vector<u8>>): BitcoinAddress{
        let to_x_only_public_keys = to_x_only_public_keys(public_keys);
        //We need to sort the public keys to generate the same multisign address
        //And we sort the x_only_public_keys, not the original public keys
        let sorted_public_keys = quick_sort(to_x_only_public_keys);
        let merkle_root = generate_taproot(threshold, &sorted_public_keys);
        //Use the sorted first public key as the internal pubkey
        let internal_pubkey = vector::borrow(&sorted_public_keys, 0);
        bitcoin_address::p2tr(internal_pubkey, option::some(merkle_root))
    }

    /// Generate a taproot merkle root from x_only_public_keys
    fun generate_taproot(threshold: u64, to_x_only_public_keys: &vector<vector<u8>>): address {
        let multisign_script = create_multisign_script(threshold, to_x_only_public_keys);
        let builder = taproot_builder::new();
        taproot_builder::add_leaf(&mut builder, 0, multisign_script);
        taproot_builder::finalize(builder)
    }

    fun create_multisign_script(threshold: u64, to_x_only_public_keys: &vector<vector<u8>>) : ScriptBuf {
        let buf = script_buf::empty();
        let idx = 0;
        let len = vector::length(to_x_only_public_keys);
        while(idx < len){
            let x_only_key = *vector::borrow(to_x_only_public_keys, idx);
            if(script_buf::is_empty(&buf)){
                script_buf::push_x_only_key(&mut buf, x_only_key);
                script_buf::push_opcode(&mut buf, opcode::op_checksig());
            }else{
                script_buf::push_x_only_key(&mut buf, x_only_key);
                script_buf::push_opcode(&mut buf, opcode::op_checksigadd());
            };
            idx = idx + 1;
        };
        script_buf::push_int(&mut buf, threshold);
        script_buf::push_opcode(&mut buf, opcode::op_greaterthanorequal());
        buf
    }

    fun to_x_only_public_keys(public_keys: vector<vector<u8>>) : vector<vector<u8>> {
        let result = vector::empty();
        let idx = 0;
        let len = vector::length(&public_keys);
        while(idx < len){
            let public_key = *vector::borrow(&public_keys, idx);
            let public_key_len = vector::length(&public_key);
            let x_only_key = if (public_key_len == BITCOIN_COMPRESSED_PUBLIC_KEY_LEN){
                sub_vector(&public_key, 1, BITCOIN_COMPRESSED_PUBLIC_KEY_LEN)
            }else{
                //TODO should we support uncompressed public key?
                abort ErrorInvalidPublicKey
            };
            sub_vector(&public_key, 1, 33);
            vector::push_back(&mut result, x_only_key);
            idx = idx + 1;
        };
        result
    }
    
    //TODO put this function in a more general module
    fun sub_vector(bytes: &vector<u8>, start: u64, end: u64): vector<u8>{
        let result = vector::empty();
        let i = start;
        while(i < end) {
            vector::push_back(&mut result, *vector::borrow(bytes, i));
            i = i + 1;
        };
        result
    }

    //TODO migrate this function to a suitable module
    fun quick_sort(data: vector<vector<u8>>): vector<vector<u8>> {
        if (vector::length(&data) <= 1) {
            return data
        };

        let pivot = *vector::borrow(&data, 0);
        let less = vector::empty();
        let equal = vector::empty();
        let greater = vector::empty();

        while (vector::length(&data) > 0) {
            let value = vector::remove(&mut data, 0);
            let cmp = compare::compare_vector_u8(&value, &pivot);
            if (cmp == compare::result_less_than()) {
                vector::push_back(&mut less, value);
            } else if (cmp == 0) {
                vector::push_back(&mut equal, value);
            } else {
                vector::push_back(&mut greater, value);
            };
        };

        let sortedData = vector::empty();
        vector::append(&mut sortedData, quick_sort(less));
        vector::append(&mut sortedData, equal);
        vector::append(&mut sortedData, quick_sort(greater));
        sortedData
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
        
        let participant = table::borrow(&multisign_account_info.participants, sender_addr);
        let proposal = table_vec::borrow_mut(&mut multisign_account_info.bitcoin_proposals, proposal_id);
        assert!(proposal.status == PROPOSAL_STATUS_PENDING, ErrorInvalidProposalStatus);
        assert!(!vector::contains(&proposal.participants, &sender_addr), ErrorProposalAlreadySigned);

        
        verify_bitcoin_signature(sender_addr, proposal.tx_id, &signature, &participant.public_key);
        
        vector::push_back(&mut proposal.signatures, signature);
        if(vector::length(&proposal.signatures) >= multisign_account_info.threshold){
            proposal.status = PROPOSAL_STATUS_APPROVED;
        }
    }


    fun verify_bitcoin_signature(_sender_addr: address, tx_id: address, signature: &vector<u8>, public_key: &vector<u8>) {
        //TODO verify the public key with sender_addr's BitcoinAddress
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
        assert!(account::exists_at(multisign_address), ErrorMultisignAccountNotFound);
        account::borrow_account(multisign_address)
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

    #[test]
    fun test_create_multisign_script(){
        let public_keys = vector::empty();
        vector::push_back(&mut public_keys, x"0308839c624d3da34ae240086f60196409d619f285365cc3498fdd3a90b72599e4");
        vector::push_back(&mut public_keys, x"0338121decf4ea2dbfd2ad1fe05a32a67448e78bf97a18bc107b4da177c27af752");
        vector::push_back(&mut public_keys, x"03786e2d94b8aaac17b2846ea908a245ab8b3c9df7ff34be8c75c27beba8e1f579");
        let x_only_public_keys = to_x_only_public_keys(public_keys);
        let sorted_public_keys = quick_sort(x_only_public_keys);
        let buf = create_multisign_script(2, &sorted_public_keys);
        std::debug::print(&buf);
        let expect_result = x"2008839c624d3da34ae240086f60196409d619f285365cc3498fdd3a90b72599e4ac2038121decf4ea2dbfd2ad1fe05a32a67448e78bf97a18bc107b4da177c27af752ba20786e2d94b8aaac17b2846ea908a245ab8b3c9df7ff34be8c75c27beba8e1f579ba52a2";
        assert!(script_buf::into_bytes(buf) == expect_result, 1000);
    }

    #[test]
    fun test_multisign_taproot(){
        let public_keys = vector::empty();
        vector::push_back(&mut public_keys, x"0308839c624d3da34ae240086f60196409d619f285365cc3498fdd3a90b72599e4");
        vector::push_back(&mut public_keys, x"0338121decf4ea2dbfd2ad1fe05a32a67448e78bf97a18bc107b4da177c27af752");
        vector::push_back(&mut public_keys, x"03786e2d94b8aaac17b2846ea908a245ab8b3c9df7ff34be8c75c27beba8e1f579");
        let x_only_public_keys = to_x_only_public_keys(public_keys);
        let sorted_public_keys = quick_sort(x_only_public_keys);
        let merkle_root = generate_taproot(2, &sorted_public_keys);
        //std::debug::print(&merkle_root);
        let expected_root = @0x2dd3a13df28795832b0efbd279ddf0a432f6942ca82172f82abb2e15461c4402;
        assert!(merkle_root == expected_root, 1000);
    }

    #[test]
    fun test_multisign_bitcoin_address(){
        let public_keys = vector::empty();
        vector::push_back(&mut public_keys, x"0308839c624d3da34ae240086f60196409d619f285365cc3498fdd3a90b72599e4");
        vector::push_back(&mut public_keys, x"0338121decf4ea2dbfd2ad1fe05a32a67448e78bf97a18bc107b4da177c27af752");
        vector::push_back(&mut public_keys, x"03786e2d94b8aaac17b2846ea908a245ab8b3c9df7ff34be8c75c27beba8e1f579");
        let bitcoin_address = generate_multisign_address(2, public_keys);
        let expected_bitcoin_address = bitcoin_address::from_string(&std::string::utf8(b"tb1phldgaz7jzshk4zw60hvveeac498jt57dst25kuhuut96dkl6kvcskvg57y"));
        assert!(bitcoin_address == expected_bitcoin_address, 1000);
    }
}