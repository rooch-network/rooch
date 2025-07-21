// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// Common utilities and helper functions for DID testing
/// This module provides shared test utilities used across all DID test modules
module rooch_framework::did_test_common {
    use rooch_framework::did;
    use rooch_framework::genesis;
    use std::string::{Self, String};
    use std::option::{Self, Option};
    use moveos_std::account;
    use moveos_std::object::ObjectID;
    use moveos_std::did_key;
    use rooch_framework::session_key;
    use rooch_framework::auth_validator;
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};
    use moveos_std::timestamp;
    use moveos_std::tx_context;
    use moveos_std::bcs;
    use moveos_std::multibase_codec;
    use std::vector;

    // ========================================
    // Test Key Generation Functions
    // ========================================

    /// Generate a test Secp256k1 public key in multibase format
    /// This is a valid compressed Secp256k1 public key for testing purposes
    public fun generate_test_secp256k1_multibase(): string::String {
        //let pubkey = x"033e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
        let pk = b"0";
        vector::append(&mut pk, bcs::to_bytes(&tx_context::fresh_address())); 
        multibase_codec::encode_base58btc(&pk)
    }

    /// Generate a test Ed25519 public key in multibase format  
    public fun generate_test_ed25519_multibase(): string::String {
        // This is a test Ed25519 public key (32 bytes) in base58btc multibase format
        //let pk = x"cc62332e34bb2d5cd69f60efbb2a36cb916c7eb458301ea36636c4dbb012bd88";
        let pk = bcs::to_bytes(&tx_context::fresh_address());
        multibase_codec::encode_base58btc(&pk)
    }

    /// Generate a test ECDSA R1 (P-256) public key in multibase format
    public fun generate_test_ecdsa_r1_multibase(): string::String {
        // Generate a test ECDSA R1 public key (33 bytes) in base58btc multibase format
        let pk = b"0";
        vector::append(&mut pk, bcs::to_bytes(&tx_context::fresh_address())); 
        multibase_codec::encode_base58btc(&pk)
    }

    /// Generate a Secp256k1 public key and corresponding Bitcoin address for testing
    public fun generate_secp256k1_public_key_and_bitcoin_address(): (string::String, BitcoinAddress) {
        let pubkey = x"034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14";

        let bitcoin_addr = bitcoin_address::derive_bitcoin_taproot_address_from_pubkey(&pubkey);
        //the address is bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g
        let multibase_key = multibase_codec::encode_base58btc(&pubkey);
        (multibase_key, bitcoin_addr)
    }

    /// Switch to a new account and set up session key authentication
    /// Returns (signer, address, public_key_multibase, bitcoin_address)
    public fun switch_to_new_account(): (signer, address, string::String, BitcoinAddress) {
        let (creator_public_key_multibase, creator_bitcoin_address) = generate_secp256k1_public_key_and_bitcoin_address();
        let creator_address = bitcoin_address::to_rooch_address(&creator_bitcoin_address);
        let creator_signer = account::create_signer_for_testing(creator_address);
        let pk_bytes_opt = multibase_codec::decode(&creator_public_key_multibase);
        assert!(option::is_some(&pk_bytes_opt), 9001);
        let pk_bytes = option::destroy_some(pk_bytes_opt);
        let auth_key = session_key::secp256k1_public_key_to_authentication_key(&pk_bytes);
        auth_validator::set_tx_validate_result_for_testing(
            0, // auth_validator_id
            option::none(), // auth_validator
            option::some(auth_key), // session_key
            creator_bitcoin_address // bitcoin_address
        );
        (creator_signer, creator_address, creator_public_key_multibase, creator_bitcoin_address)
    }

    // ========================================
    // Test Setup Functions
    // ========================================

    /// Unified setup function for DID tests with DID creation
    /// Returns (creator_signer, creator_address, creator_public_key_multibase, did_object_id)
    public fun setup_did_test_with_creation(): (signer, address, string::String, ObjectID) {
        setup_did_test_with_scope_creation(option::none())
    }

    public fun setup_did_test_with_scope_creation(session_scope: Option<vector<String>>): (signer, address, string::String, ObjectID) {
        // Initialize the entire framework including DID registry
        genesis::init_for_test();
        timestamp::fast_forward_milliseconds_for_test(1000);

        let (creator_public_key_multibase, creator_bitcoin_address) = generate_secp256k1_public_key_and_bitcoin_address();
        let creator_address = bitcoin_address::to_rooch_address(&creator_bitcoin_address);
        let creator_signer = account::create_signer_for_testing(creator_address);

        // Setup mock Bitcoin address and session key for testing
        let pk_bytes_opt = multibase_codec::decode(&creator_public_key_multibase);
        assert!(option::is_some(&pk_bytes_opt), 9001);
        let pk_bytes = option::destroy_some(pk_bytes_opt);
        let auth_key = session_key::secp256k1_public_key_to_authentication_key(&pk_bytes);
        
        // Set up mock with the matching Bitcoin address and session key
        auth_validator::set_tx_validate_result_for_testing(
            0, // auth_validator_id
            option::none(), // auth_validator
            option::some(auth_key), // session_key
            creator_bitcoin_address // bitcoin_address
        );
        
        // Create DID
        let did_object_id = did::create_did_object_for_self_with_custom_scopes(&creator_signer, creator_public_key_multibase, session_scope);
        
        (creator_signer, creator_address, creator_public_key_multibase, did_object_id)
    }

    /// Basic setup function for DID tests without creating DID
    /// Returns (creator_signer, creator_address, creator_public_key_multibase)
    public fun setup_did_test_basic(): (signer, address, string::String) {
        // Initialize the entire framework including DID registry
        genesis::init_for_test();
        timestamp::fast_forward_milliseconds_for_test(1000);

        let (creator_public_key_multibase, creator_bitcoin_address) = generate_secp256k1_public_key_and_bitcoin_address();
        let creator_address = bitcoin_address::to_rooch_address(&creator_bitcoin_address);
        let creator_signer = account::create_signer_for_testing(creator_address);

        // Setup mock Bitcoin address and session key for testing
        let pk_bytes_opt = multibase_codec::decode(&creator_public_key_multibase);
        assert!(option::is_some(&pk_bytes_opt), 9002);
        let pk_bytes = option::destroy_some(pk_bytes_opt);
        let auth_key = session_key::secp256k1_public_key_to_authentication_key(&pk_bytes);
        
        // Set up mock with the matching Bitcoin address and session key
        auth_validator::set_tx_validate_result_for_testing(
            0, // auth_validator_id
            option::none(), // auth_validator
            option::some(auth_key), // session_key
            creator_bitcoin_address // bitcoin_address
        );
        
        (creator_signer, creator_address, creator_public_key_multibase)
    }

    /// Create a test account and return the signer
    /// This simulates account creation for testing purposes
    public fun create_test_account_and_signer(): signer {
        // Create a test account using the testing function
        let test_address = @0x42;
        account::create_signer_for_testing(test_address)
    }

    // ========================================
    // Session Key Setup Functions
    // ========================================

    /// Setup session key authentication for Secp256k1 key
    public fun setup_secp256k1_session_key_auth(public_key_multibase: &string::String) {
        let pk_bytes_opt = multibase_codec::decode(public_key_multibase);
        assert!(option::is_some(&pk_bytes_opt), 9003);
        let pk_bytes = option::destroy_some(pk_bytes_opt);
        let auth_key = session_key::secp256k1_public_key_to_authentication_key(&pk_bytes);
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(auth_key));
    }

    /// Setup session key authentication for Ed25519 key
    public fun setup_ed25519_session_key_auth(public_key_multibase: &string::String) {
        let pk_bytes_opt = multibase_codec::decode(public_key_multibase);
        assert!(option::is_some(&pk_bytes_opt), 9004);
        let pk_bytes = option::destroy_some(pk_bytes_opt);
        let auth_key = session_key::ed25519_public_key_to_authentication_key(&pk_bytes);
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(auth_key));
    }

    // ========================================
    // Test Framework Initialization
    // ========================================

    /// Initialize test framework with basic DID registry setup
    public fun init_test_framework() {
        genesis::init_for_test();
        timestamp::fast_forward_milliseconds_for_test(1000);
    }

    // ========================================
    // CADOP Test Setup Functions
    // ========================================

    /// Setup function to create a custodian DID with CADOP service
    /// This allows tests to use the full CADOP API without skipping validations
    /// Returns the custodian signer for further use in tests
    public fun setup_custodian_with_cadop_service(): signer {
        // First create a DID using the standard setup
        let (_creator_signer, _creator_address, creator_public_key, custodian_did_object_id) = setup_did_test_with_creation();
        //std::debug::print(&custodian_did_object_id);
        // Get the DID document and its address
        let custodian_did_document = did::get_did_document_by_object_id(custodian_did_object_id);
        let custodian_did_address = did::get_did_address(custodian_did_document);
        
        // Create a signer for the DID address (this is the custodian)
        let custodian_signer = account::create_signer_for_testing(custodian_did_address);
        
        // Set up session key authentication for the DID address using the creator's key
        // The DID was created with the creator's Secp256k1 key, so we use that for authentication
        setup_secp256k1_session_key_auth(&creator_public_key);
        
        // Add CADOP custodian service to the DID
        let service_fragment = string::utf8(b"cadop-custodian");
        let service_type = string::utf8(b"CadopCustodianService");
        let service_endpoint = string::utf8(b"https://custodian.example.com/cadop");
        
        did::add_service_entry(&custodian_signer, service_fragment, service_type, service_endpoint);
        
        custodian_signer
    }

    /// Setup function for CADOP tests with both custodian and user preparation
    /// Returns (custodian_signer, user_did_key_string, custodian_service_pk, custodian_service_vm_type)
    public fun setup_cadop_test_full(): (signer, string::String, string::String, string::String) {
        
        // For now, let's use the test-only bypass method since setting up proper
        // session key authentication for service addition is complex
        let custodian_signer = setup_custodian_with_cadop_service();
        let user_did_key_string = generate_test_did_key_string();
        let custodian_service_pk_multibase = generate_test_secp256k1_multibase();
        let custodian_service_vm_type = string::utf8(b"EcdsaSecp256k1VerificationKey2019");
        
        (custodian_signer, user_did_key_string, custodian_service_pk_multibase, custodian_service_vm_type)
    }

    /// Generate a valid did:key string for testing with proper multicodec prefix
    public fun generate_test_did_key_string(): string::String {
        
        let pk = b"0";
        vector::append(&mut pk, bcs::to_bytes(&tx_context::fresh_address()));
        
        // Use the did_key module to generate a did:key string
        did_key::generate_secp256r1(&pk)
    }
} 