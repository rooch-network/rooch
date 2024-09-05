// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Bitcoin multisign account module
module bitcoin_move::multisign_account{

    use std::vector;
    use std::option;
    use moveos_std::signer;
    use moveos_std::object::{Object};
    use moveos_std::account::{Self, Account};
    use moveos_std::bcs;
    use moveos_std::sort;
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::result;
    
    use rooch_framework::ecdsa_k1;
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};
    use rooch_framework::address_mapping;
    
    use bitcoin_move::taproot_builder;
    use bitcoin_move::opcode;
    use bitcoin_move::script_buf::{Self, ScriptBuf};
    
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
        participants: SimpleMap<address, ParticipantInfo>,
    }

    struct ParticipantInfo has store, copy, drop {
        /// The participant address
        participant_address: address,
        /// The participant BitcoinAddress
        participant_bitcoin_address: BitcoinAddress,
        /// The participant public key
        public_key: vector<u8>,
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
        let participants = simple_map::new();
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
            simple_map::add(&mut participants, participant_address, participant_info);
            idx = idx + 1;
        };
        let multisign_account_info = MultisignAccountInfo {
            multisign_bitcoin_address,
            multisign_address,
            threshold,
            participants,
        };
        let account = borrow_mut_or_create_account(multisign_address, multisign_bitcoin_address);
        account::account_move_resource_to(account, multisign_account_info);
        multisign_address
    }

    public fun generate_multisign_address(threshold: u64, public_keys: vector<vector<u8>>): BitcoinAddress{
        let to_x_only_public_keys = to_x_only_public_keys(public_keys);
        //We need to sort the public keys to generate the same multisign address
        //And we sort the x_only_public_keys, not the original public keys
        sort::sort(&mut to_x_only_public_keys);
        let merkle_root = generate_taproot(threshold, &to_x_only_public_keys);
        //Use the sorted first public key as the internal pubkey
        let internal_pubkey = vector::borrow(&to_x_only_public_keys, 0);
        bitcoin_address::p2tr(internal_pubkey, option::some(merkle_root))
    }

    /// Generate a taproot merkle root from x_only_public_keys
    fun generate_taproot(threshold: u64, to_x_only_public_keys: &vector<vector<u8>>): address {
        let multisign_script = create_multisign_script(threshold, to_x_only_public_keys);
        let builder = taproot_builder::new();
        taproot_builder::add_leaf(&mut builder, 0, multisign_script);
        let result = taproot_builder::finalize(builder);
        result::unwrap(result)
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
                vector::slice(&public_key, 1, BITCOIN_COMPRESSED_PUBLIC_KEY_LEN)
            }else if(public_key_len == X_ONLY_PUBLIC_KEY_LEN){
                public_key
            }else{
                //TODO should we support uncompressed public key?
                abort ErrorInvalidPublicKey
            };
            vector::push_back(&mut result, x_only_key);
            idx = idx + 1;
        };
        result
    }

    public fun is_participant(multisign_address: address, participant_address: address) : bool {
        if(!account::exists_at(multisign_address)){
            return false
        };
        let account = borrow_account(multisign_address);
        let multisign_account_info = account::account_borrow_resource<MultisignAccountInfo>(account);
        simple_map::contains_key(&multisign_account_info.participants, &participant_address)
    }

    public fun is_participant_via_public_key(multisign_address: address, public_key: &vector<u8>) : bool {
        if(!account::exists_at(multisign_address)){
            return false
        };
        let account = borrow_account(multisign_address);
        let multisign_account_info = account::account_borrow_resource<MultisignAccountInfo>(account);
        let idx = 0;
        let values = simple_map::values(&multisign_account_info.participants);
        let len = vector::length(&values);
        while(idx < len){
            let participant_info = vector::borrow(&values, idx);
            if(&participant_info.public_key == public_key){
                return true
            };
            idx = idx + 1;
        };
        false
    }

    public fun is_multisign_account(multisign_address: address) : bool {
        account::exists_at(multisign_address) && account::exists_resource<MultisignAccountInfo>(multisign_address)
    }

    public fun bitcoin_address(multisign_address: address) : BitcoinAddress {
        let account = borrow_account(multisign_address);
        let multisign_account_info = account::account_borrow_resource<MultisignAccountInfo>(account);
        multisign_account_info.multisign_bitcoin_address
    }

    public fun threshold(multisign_address: address) : u64 {
        let account = borrow_account(multisign_address);
        let multisign_account_info = account::account_borrow_resource<MultisignAccountInfo>(account);
        multisign_account_info.threshold
    }

    // ======== Participant Info functions ========

    public fun participants(multisign_address: address) : vector<ParticipantInfo> {
        let account = borrow_account(multisign_address);
        let multisign_account_info = account::account_borrow_resource<MultisignAccountInfo>(account);
        simple_map::values(&multisign_account_info.participants)
    }

    public fun participant(multisign_address: address, participant_address: address) : ParticipantInfo {
        let account = borrow_account(multisign_address);
        let multisign_account_info = account::account_borrow_resource<MultisignAccountInfo>(account);
        *simple_map::borrow(&multisign_account_info.participants, &participant_address)
    }

    public fun participant_public_key(participant: &ParticipantInfo) : &vector<u8> {
        &participant.public_key
    }

    public fun participant_bitcoin_address(participant: &ParticipantInfo) : &BitcoinAddress {
        &participant.participant_bitcoin_address
    }

    public fun participant_address(participant: &ParticipantInfo) : address {
        participant.participant_address
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

    fun borrow_mut_or_create_account(multisign_address: address, multisign_bitcoin_address: BitcoinAddress) : &mut Object<Account> {
        // Maybe the multisign account is created by the Bitcoin transaction
        if (account::exists_at(multisign_address)){
            borrow_mut_account(multisign_address)
        }else{
            let module_signer = signer::module_signer<MultisignAccountInfo>();
            let signer = account::create_account_by_system(&module_signer, multisign_address);
            address_mapping::bind_bitcoin_address_by_system(&module_signer, multisign_address, multisign_bitcoin_address);
            account::borrow_mut_account(&signer)
        }
    }

    fun borrow_mut_account(multisign_address: address) : &mut Object<Account>{
        let module_signer = signer::module_signer<MultisignAccountInfo>();
        let signer = account::create_signer_for_system(&module_signer, multisign_address);
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
        sort::sort(&mut x_only_public_keys);
        let buf = create_multisign_script(2, &x_only_public_keys);
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
        sort::sort(&mut x_only_public_keys);
        let merkle_root = generate_taproot(2, &x_only_public_keys);
        //std::debug::print(&merkle_root);
        let expected_root = @0x2dd3a13df28795832b0efbd279ddf0a432f6942ca82172f82abb2e15461c4402;
        assert!(merkle_root == expected_root, 1000);
    }

    #[test]
    fun test_multisign_bitcoin_address_from_less_than_eight_pubkeys(){
        let public_keys = vector::empty();
        vector::push_back(&mut public_keys, x"032d4fb9f88a63f52d8bffd1a46ad40411310150a539913203265c3f46b0397f8c");
        vector::push_back(&mut public_keys, x"039c9f399047d1ca911827c8c9b445ea55e84a68dcfe39641bc1f423c6a7cd99d0");
        vector::push_back(&mut public_keys, x"03ad953cc82a6ed91c8eb3a6400e55965de4735bc5f8a107eabd2e4e7531f64c61");
        vector::push_back(&mut public_keys, x"0346b64846c11f23ccec99811b476aaf68f421f15762287b872fcb896c92caa677");
        vector::push_back(&mut public_keys, x"03730cb693e9a1bc6eaec5537c2e317a75bb6c8107a59fda018810c46c270670be");
        let bitcoin_address = generate_multisign_address(3, public_keys);
        let expected_bitcoin_address = bitcoin_address::from_string(&std::string::utf8(b"bc1pwee7tfs79xapsaamzqnnwn8d5w2z3cfzp2v8nhvsyddlyk4l67gqa0x3w5"));
        assert!(bitcoin_address == expected_bitcoin_address, 1000);
    }

    #[test]
    fun test_multisign_bitcoin_address_from_great_than_or_equal_eight_pubkeys(){
        let public_keys = vector::empty();
        vector::push_back(&mut public_keys, x"032d4fb9f88a63f52d8bffd1a46ad40411310150a539913203265c3f46b0397f8c");
        vector::push_back(&mut public_keys, x"039c9f399047d1ca911827c8c9b445ea55e84a68dcfe39641bc1f423c6a7cd99d0");
        vector::push_back(&mut public_keys, x"03ad953cc82a6ed91c8eb3a6400e55965de4735bc5f8a107eabd2e4e7531f64c61");
        vector::push_back(&mut public_keys, x"0346b64846c11f23ccec99811b476aaf68f421f15762287b872fcb896c92caa677");
        vector::push_back(&mut public_keys, x"03730cb693e9a1bc6eaec5537c2e317a75bb6c8107a59fda018810c46c270670be");
        vector::push_back(&mut public_keys, x"0259a40918150bc16ca1852fb55be383ec0fcf2b6058a73a25f0dfd87394dd92db");
        vector::push_back(&mut public_keys, x"028fd25b727bf77e42d7a99cad4b1fa564d41cdb3bbddaf15219a4529f486a775a");
        vector::push_back(&mut public_keys, x"03786e2d94b8aaac17b2846ea908a245ab8b3c9df7ff34be8c75c27beba8e1f579");
        let bitcoin_address = generate_multisign_address(3, public_keys);
        let expected_bitcoin_address = bitcoin_address::from_string(&std::string::utf8(b"bc1p5pmmc8jmfeqx3fx0he3qgylr8q9cmduf43jclgplpk2kcjyrrmzq5tejw6"));
        assert!(bitcoin_address == expected_bitcoin_address, 1000);
    }
}