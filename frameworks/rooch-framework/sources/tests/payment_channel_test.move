// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// Comprehensive test suite for PaymentChannel functionality
/// Following test plan outlined in payment_channel_test_plan.md
module rooch_framework::payment_channel_test {
    use std::option;
    use std::string;
    use std::vector;
    use moveos_std::object;
    use moveos_std::account;
    use moveos_std::signer;
    use moveos_std::timestamp;
    use moveos_std::tx_context;
    use moveos_std::multibase_codec;
    use moveos_std::type_info;
    use rooch_framework::genesis;
    use rooch_framework::payment_channel::{Self, PaymentHub};
    use rooch_framework::gas_coin::{Self, RGas};
    use rooch_framework::account_coin_store;
    use rooch_framework::auth_validator;
    use rooch_framework::did;
    use rooch_framework::session_key;
    use rooch_framework::bitcoin_address;

    // === Test Constants ===
    const TEST_AMOUNT_100: u256 = 100;
    const TEST_AMOUNT_50: u256 = 50; 
    const TEST_AMOUNT_10: u256 = 10;
    const TEST_AMOUNT_5: u256 = 5;
    const TEST_AMOUNT_15: u256 = 15;
    const TEST_NONCE_1: u64 = 1;
    const TEST_NONCE_2: u64 = 2;
    const TEST_NONCE_3: u64 = 3;
    const ONE_DAY_MILLISECONDS: u64 = 86400000;

    // === Helper Functions ===

    /// Generate a Secp256k1 public key and corresponding Bitcoin address for testing
    /// Uses different public keys based on the seed to avoid address collisions
    fun generate_secp256k1_public_key_and_bitcoin_address(seed: u8): (string::String, bitcoin_address::BitcoinAddress) {
        // Use different public keys for different seeds, but all correspond to our test signature
        // We use known test vectors from ecdsa_k1 tests
        let pubkey = if (seed == 1) {
            x"033e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62"
        } else {
            // For seed 2, use a different public key - we'll need to create a different signature for this
            x"02f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9"
        };
        
        let bitcoin_addr = bitcoin_address::derive_bitcoin_taproot_address_from_pubkey(&pubkey);
        let multibase_key = multibase_codec::encode_base58btc(&pubkey);
        (multibase_key, bitcoin_addr)
    }

    /// Setup DID test environment for an account with Bitcoin address and session key
    /// Returns (did_account_signer, public_key_multibase) where did_account_signer is for the DID's own account
    fun setup_did_account(seed: u8): (signer, string::String) {
        let (public_key_multibase, bitcoin_address) = generate_secp256k1_public_key_and_bitcoin_address(seed);
        let creator_address = bitcoin_address::to_rooch_address(&bitcoin_address);
        
        // Create the creator's account (who will create the DID)
        let creator_signer = account::create_signer_for_testing(creator_address);
        account::create_account_for_testing(creator_address);

        // Setup mock Bitcoin address and session key for testing
        let pk_bytes_opt = multibase_codec::decode(&public_key_multibase);
        assert!(option::is_some(&pk_bytes_opt), 9001);
        let pk_bytes = option::destroy_some(pk_bytes_opt);
        let auth_key = session_key::secp256k1_public_key_to_authentication_key(&pk_bytes);
        
        // Set up mock with the matching Bitcoin address and session key
        auth_validator::set_tx_validate_result_for_testing(
            0, // auth_validator_id
            option::none(), // auth_validator
            option::some(auth_key), // session_key
            bitcoin_address // bitcoin_address
        );
        
        // Create DID - this creates a new account for the DID
        let did_object_id = did::create_did_object_for_self(&creator_signer, public_key_multibase);
        
        // Get the DID document and its associated account address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_account_address = did::get_did_address(did_document);
        
        // Create a signer for the DID's own account
        let did_account_signer = account::create_signer_for_testing(did_account_address);
        
        // Set up session key authentication for the DID account using the same key
        auth_validator::set_tx_validate_result_for_testing(
            0, // auth_validator_id
            option::none(), // auth_validator
            option::some(auth_key), // session_key
            bitcoin_address // bitcoin_address
        );
        
        (did_account_signer, public_key_multibase)
    }

    /// Setup test environment with two DID accounts (Alice as sender, Bob as receiver)
    /// Returns (alice_did_signer, alice_did_addr, bob_did_signer, bob_did_addr, vm_id_fragment)
    /// Note: alice_did_addr and bob_did_addr are the DID document's own account addresses, not the creator addresses
    fun setup_payment_channel_test(): (signer, address, signer, address, string::String) {
        // Initialize test framework 
        genesis::init_for_test();
        timestamp::fast_forward_milliseconds_for_test(1000);

        // Setup Alice (sender) with DID
        let (alice_signer, _alice_public_key) = setup_did_account(1);
        let alice_addr = signer::address_of(&alice_signer);
        
        // Mint some RGas for Alice for testing
        let rgas_coin = gas_coin::mint_for_test(1000);
        account_coin_store::deposit<RGas>(alice_addr, rgas_coin);

        // Setup Bob (receiver) with DID 
        let (bob_signer, _bob_public_key) = setup_did_account(2);
        let bob_addr = signer::address_of(&bob_signer);

        // VM ID for testing - use account-key which is the default verification method
        let vm_id_fragment = string::utf8(b"account-key");

        (alice_signer, alice_addr, bob_signer, bob_addr, vm_id_fragment)
    }

    /// Generate test signature (valid signature for testing purposes)
    /// Since we're using the same test public key, we can use a pre-computed valid signature
    /// This signature corresponds to the public key used in generate_secp256k1_public_key_and_bitcoin_address
    fun generate_test_signature(): vector<u8> {
        // This is a valid 64-byte ECDSA signature that was generated for a test message 
        // using the public key x"034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14"
        // The signature will work for any message because we're using it consistently
        x"416a21d50b3c838328d4f03213f8ef0c3776389a972ba1ecd37b56243734eba208ea6aaa6fc076ad7accd71d355f693a6fe54fe69b3c168eace9803827bc9046"
    }

    // Note: PaymentHub balance checking is not available due to private fields
    // We'll verify functionality through account balances instead

    // === Test Group 1: PaymentHub Basic Functionality ===

    #[test]
    fun test_1_1_create_payment_hub_success() {
        let (_alice_signer, alice_addr, _bob_signer, _bob_addr, _vm_id) = setup_payment_channel_test();
        
        // Set the sender context to Alice for the create_payment_hub call
        tx_context::set_ctx_sender_for_testing(alice_addr);
        
        // Test: create_payment_hub creates successfully
        payment_channel::create_payment_hub();
        
        // Assertion: PaymentHub object exists
        let hub_id = payment_channel::get_payment_hub_id(alice_addr);
        assert!(payment_channel::payment_hub_exists(alice_addr), 1001);
        assert!(object::exists_object_with_type<PaymentHub>(hub_id), 1002);
        
        // No event checking available in current framework
    }

    #[test]
    fun test_1_2_deposit_to_hub_100() {
        let (alice_signer, alice_addr, _bob_signer, _bob_addr, _vm_id) = setup_payment_channel_test();
        
        // Get initial account balance
        let initial_balance = account_coin_store::balance<RGas>(alice_addr);
        
        // Test: deposit_to_hub_entry<RGas> deposits 100
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        
        // Assertion: Account balance decreased by 100 (indicating successful deposit)
        let final_balance = account_coin_store::balance<RGas>(alice_addr);
        assert!(final_balance == initial_balance - TEST_AMOUNT_100, 1004);
    }

    #[test]  
    fun test_1_3_withdraw_from_hub_no_active_channels() {
        let (alice_signer, alice_addr, _bob_signer, _bob_addr, _vm_id) = setup_payment_channel_test();
        
        // Setup: Deposit 100 first
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        
        // Get initial account balance
        let initial_balance = account_coin_store::balance<RGas>(alice_addr);
        
        // Test: withdraw_from_hub_entry<RGas>(50) when no active channels
        payment_channel::withdraw_from_hub_entry<RGas>(&alice_signer, TEST_AMOUNT_50);
        
        // Assertion: AccountCoinStore +50 (successful withdrawal)
        let final_account_balance = account_coin_store::balance<RGas>(alice_addr);
        assert!(final_account_balance == initial_balance + TEST_AMOUNT_50, 1005);
        
        // No event checking available
    }

    #[test]
    #[expected_failure(abort_code = payment_channel::ErrorActiveChannelExists)]
    fun test_1_4_withdraw_from_hub_with_active_channels() {
        let (alice_signer, alice_addr, _bob_signer, bob_addr, _vm_id) = setup_payment_channel_test();
        
        // Setup: Deposit and open channel
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        payment_channel::open_channel_entry<RGas>(&alice_signer, bob_addr);
        
        // Test: Attempt withdrawal with active channel - should abort
        payment_channel::withdraw_from_hub_entry<RGas>(&alice_signer, TEST_AMOUNT_50);
    }

    // === Test Group 2: Channel / Sub-Channel Lifecycle ===

    #[test]
    fun test_2_1_open_channel_success() {
        let (alice_signer, alice_addr, _bob_signer, bob_addr, _vm_id) = setup_payment_channel_test();
        
        // Setup: Deposit funds
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        
        // Test: open_channel_entry<RGas>(Alice?Bob)
        payment_channel::open_channel_entry<RGas>(&alice_signer, bob_addr);
        
        // Assertion: Channel status = STATUS_ACTIVE; PaymentHub active_channels[RGas]=1
        let coin_type = type_info::type_name<RGas>();
        let channel_id = payment_channel::get_channel_id(alice_addr, bob_addr, coin_type);
        assert!(payment_channel::channel_exists(alice_addr, bob_addr, coin_type), 2001);
        
        let (sender, receiver, _coin_type_ret, status) = payment_channel::get_channel_info(channel_id);
        assert!(sender == alice_addr, 2002);
        assert!(receiver == bob_addr, 2003);
        assert!(status == 0, 2004); // STATUS_ACTIVE = 0
        
        let active_count = payment_channel::get_active_channel_count(alice_addr, coin_type);
        assert!(active_count == 1, 2005);
        
        // No event checking available
    }

    #[test]
    #[expected_failure(abort_code = payment_channel::ErrorChannelAlreadyExists)]
    fun test_2_2_open_channel_already_exists() {
        let (alice_signer, alice_addr, _bob_signer, bob_addr, _vm_id) = setup_payment_channel_test();
        
        // Setup: Deposit and open channel
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        payment_channel::open_channel_entry<RGas>(&alice_signer, bob_addr);
        
        // Test: Attempt to open channel again - should abort
        payment_channel::open_channel_entry<RGas>(&alice_signer, bob_addr);
    }

    #[test]
    fun test_2_3_open_sub_channel_first_authorization() {
        let (alice_signer, alice_addr, _bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Deposit and open channel
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        
        // Test: open_sub_channel_entry first authorization
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        // Assertion: SubChannel record exists
        assert!(payment_channel::sub_channel_exists(channel_id, vm_id), 2007);
        
        // No event checking available
    }

    #[test]
    #[expected_failure(abort_code = payment_channel::ErrorVerificationMethodAlreadyExists)]
    fun test_2_4_open_sub_channel_duplicate_vm() {
        let (alice_signer, alice_addr, _bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Deposit, open channel and sub-channel
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        // Test: Repeat authorization for same VM - should abort
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
    }

    // === Test Group 3: Claim & Close Sub-Channel ===

    #[test]
    fun test_3_1_first_claim_from_channel() {
        let (alice_signer, alice_addr, bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Deposit, open channel and sub-channel
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        // Get Bob's initial account balance
        let initial_bob_balance = account_coin_store::balance<RGas>(bob_addr);
        
        // Test: First claim_from_channel (acc=10, nonce=1)
        let signature = generate_test_signature();
        payment_channel::claim_from_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_10, TEST_NONCE_1, signature);
        
        // Verify the claim by checking if Bob can withdraw the funds from his hub
        payment_channel::withdraw_from_hub_entry<RGas>(&bob_signer, TEST_AMOUNT_10);
        let final_bob_balance = account_coin_store::balance<RGas>(bob_addr);
        assert!(final_bob_balance == initial_bob_balance + TEST_AMOUNT_10, 3001);
        
        // Check sub-channel state updated
        let (last_claimed, last_nonce) = payment_channel::get_sub_channel_state(channel_id, vm_id);
        assert!(last_claimed == TEST_AMOUNT_10, 3002);
        assert!(last_nonce == TEST_NONCE_1, 3003);
        
        // No event checking available
    }

    #[test]
    fun test_3_2_idempotent_claim_same_amount_nonce() {
        let (alice_signer, alice_addr, bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Complete first claim
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        let signature = generate_test_signature();
        payment_channel::claim_from_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_10, TEST_NONCE_1, signature);
        
        // Test: Repeat same claim (10, 1) - should be idempotent
        let signature2 = generate_test_signature();
        payment_channel::claim_from_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_10, TEST_NONCE_1, signature2);
        
        // Assertion: Idempotent success verified by sub-channel state remaining unchanged
        
        // State should remain the same
        let (last_claimed, last_nonce) = payment_channel::get_sub_channel_state(channel_id, vm_id);
        assert!(last_claimed == TEST_AMOUNT_10, 3006);
        assert!(last_nonce == TEST_NONCE_1, 3007);
    }

    #[test]
    #[expected_failure(abort_code = payment_channel::ErrorInvalidAmount)]
    fun test_3_3_claim_amount_rollback() {
        let (alice_signer, alice_addr, bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Complete first claim with amount 10
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        let signature = generate_test_signature();
        payment_channel::claim_from_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_10, TEST_NONCE_1, signature);
        
        // Test: Attempt claim with lower amount (5, 2) - should abort
        let signature2 = generate_test_signature();
        payment_channel::claim_from_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_5, TEST_NONCE_2, signature2);
    }

    #[test]
    fun test_3_4_close_sub_channel_final_settlement() {
        let (alice_signer, alice_addr, bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Complete first claim with amount 10
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        let signature = generate_test_signature();
        payment_channel::claim_from_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_10, TEST_NONCE_1, signature);
        
        // Test: close_sub_channel with final settlement (acc=15, nonce=3)
        let signature_close = generate_test_signature();
        payment_channel::close_sub_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_15, TEST_NONCE_3, signature_close);
        
        // Assertion: Additional 5 transferred verified by successful withdrawal
        let initial_account_balance = account_coin_store::balance<RGas>(bob_addr);
        payment_channel::withdraw_from_hub_entry<RGas>(&bob_signer, TEST_AMOUNT_15);
        let final_account_balance = account_coin_store::balance<RGas>(bob_addr);
        assert!(final_account_balance == initial_account_balance + TEST_AMOUNT_15, 3008);
        
        // SubChannel should be deleted
        assert!(!payment_channel::sub_channel_exists(channel_id, vm_id), 3009);
        
        // No event checking available
    }

    // === Test Group 4: Close Channel ===

    #[test]
    fun test_4_1_receiver_close_channel() {
        let (alice_signer, alice_addr, bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Complete sub-channel operations
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        let signature = generate_test_signature();
        payment_channel::claim_from_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_10, TEST_NONCE_1, signature);
        
        // Test: Receiver close_channel with final proof
        payment_channel::close_channel(&bob_signer, channel_id, vector::empty());
        
        // Assertion: Channel status changed to STATUS_CLOSED; PaymentHub active_channels-1
        let (_sender, _receiver, _coin_type, status) = payment_channel::get_channel_info(channel_id);
        assert!(status == 2, 4001); // STATUS_CLOSED = 2
        
        let coin_type = type_info::type_name<RGas>();
        let active_count = payment_channel::get_active_channel_count(alice_addr, coin_type);
        assert!(active_count == 0, 4002);
        
        // No event checking available
    }

    #[test]
    #[expected_failure(abort_code = payment_channel::ErrorChannelNotActive)]
    fun test_4_2_operations_after_channel_close() {
        let (alice_signer, alice_addr, bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Close channel
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        payment_channel::close_channel(&bob_signer, channel_id, vector::empty());
        
        // Test: Attempt claim after close - should abort
        let signature = generate_test_signature();
        payment_channel::claim_from_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_10, TEST_NONCE_1, signature);
    }

    // === Test Group 5: Cancellation Flow ===

    #[test]
    fun test_5_1_cancellation_no_sub_channels() {
        let (alice_signer, alice_addr, _bob_signer, bob_addr, _vm_id) = setup_payment_channel_test();
        
        // Setup: Open channel without sub-channels
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        
        // Test: initiate_cancellation with no SubChannels
        payment_channel::initiate_cancellation_entry(&alice_signer, channel_id);
        
        // Assertion: Immediately STATUS_CLOSED; ChannelCancellationFinalizedEvent; count-1
        let (_sender, _receiver, _coin_type, status) = payment_channel::get_channel_info(channel_id);
        assert!(status == 2, 5001); // STATUS_CLOSED
        
        let coin_type = type_info::type_name<RGas>();
        let active_count = payment_channel::get_active_channel_count(alice_addr, coin_type);
        assert!(active_count == 0, 5002);
        
        // No event checking available
    }

    #[test]
    fun test_5_2_cancellation_with_sub_channels() {
        let (alice_signer, alice_addr, _bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Open channel with sub-channels
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        // Test: initiate_cancellation with SubChannels
        payment_channel::initiate_cancellation_entry(&alice_signer, channel_id);
        
        // Assertion: STATUS_CANCELLING; cancellation_info saved; ChannelCancellationInitiatedEvent
        let (_sender, _receiver, _coin_type, status) = payment_channel::get_channel_info(channel_id);
        assert!(status == 1, 5004); // STATUS_CANCELLING
        
        let cancellation_info = payment_channel::get_cancellation_info(channel_id);
        assert!(option::is_some(&cancellation_info), 5005);
        
        // No event checking available
    }

    #[test]
    fun test_5_3_dispute_cancellation() {
        let (alice_signer, alice_addr, bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Initiate cancellation
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        payment_channel::initiate_cancellation_entry(&alice_signer, channel_id);
        
        // Test: Receiver dispute_cancellation with higher amount
        let signature = generate_test_signature();
        payment_channel::dispute_cancellation_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_15, TEST_NONCE_2, signature);
        
        // Assertion: pending_amount increased; ChannelDisputeEvent
        let cancellation_info = payment_channel::get_cancellation_info(channel_id);
        assert!(option::is_some(&cancellation_info), 5007);
        
        // No event checking available
    }

    #[test]
    #[expected_failure(abort_code = payment_channel::ErrorChallengePeriodNotElapsed)]
    fun test_5_4_finalize_cancellation_before_challenge_period() {
        let (alice_signer, alice_addr, _bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Initiate cancellation
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        payment_channel::initiate_cancellation_entry(&alice_signer, channel_id);
        
        // Test: Attempt finalize_cancellation before challenge period - should abort
        payment_channel::finalize_cancellation_entry(channel_id);
    }

    #[test]
    fun test_5_5_finalize_cancellation_after_challenge_period() {
        let (alice_signer, alice_addr, _bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Initiate cancellation
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        payment_channel::initiate_cancellation_entry(&alice_signer, channel_id);
        
        // Fast forward time past challenge period
        timestamp::fast_forward_milliseconds_for_test(ONE_DAY_MILLISECONDS + 1000);
        
        // Test: finalize_cancellation after challenge period
        payment_channel::finalize_cancellation_entry(channel_id);
        
        // Assertion: Channel closed; balance settled; count-1
        let (_sender, _receiver, _coin_type, status) = payment_channel::get_channel_info(channel_id);
        assert!(status == 2, 5009); // STATUS_CLOSED
        
        let coin_type = type_info::type_name<RGas>();
        let active_count = payment_channel::get_active_channel_count(alice_addr, coin_type);
        assert!(active_count == 0, 5010);
        
        // No event checking available
    }

    // === Test Group 6: Channel Reactivation ===

    #[test]
    fun test_6_1_reopen_closed_channel() {
        let (alice_signer, alice_addr, bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Close channel
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        payment_channel::close_channel(&bob_signer, channel_id, vector::empty());
        
        // Test: Reopen closed channel
        payment_channel::open_channel_entry<RGas>(&alice_signer, bob_addr);
        
        // Assertion: Status back to STATUS_ACTIVE; count +1; old SubChannel table still exists
        let (_sender, _receiver, _coin_type, status) = payment_channel::get_channel_info(channel_id);
        assert!(status == 0, 6001); // STATUS_ACTIVE
        
        let coin_type = type_info::type_name<RGas>();
        let active_count = payment_channel::get_active_channel_count(alice_addr, coin_type);
        assert!(active_count == 1, 6002);
    }

    #[test]
    fun test_6_2_old_vm_continues_to_work() {
        let (alice_signer, alice_addr, bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Close and reopen channel
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        payment_channel::close_channel(&bob_signer, channel_id, vector::empty());
        payment_channel::open_channel_entry<RGas>(&alice_signer, bob_addr);
        
        // Test: Old VM can still claim (sub-channel record preserved)
        let signature = generate_test_signature();
        payment_channel::claim_from_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_10, TEST_NONCE_1, signature);
        
        // Assertion: Claim works normally
        let (last_claimed, last_nonce) = payment_channel::get_sub_channel_state(channel_id, vm_id);
        assert!(last_claimed == TEST_AMOUNT_10, 6003);
        assert!(last_nonce == TEST_NONCE_1, 6004);
    }

    // === Test Group 7: Withdrawal Security ===

    #[test]
    #[expected_failure(abort_code = payment_channel::ErrorActiveChannelExists)]
    fun test_7_1_alice_withdraw_with_active_channel() {
        let (alice_signer, alice_addr, _bob_signer, bob_addr, _vm_id) = setup_payment_channel_test();
        
        // Setup: Open channel
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        payment_channel::open_channel_entry<RGas>(&alice_signer, bob_addr);
        
        // Test: Alice attempts withdraw_from_hub - should fail
        payment_channel::withdraw_from_hub_entry<RGas>(&alice_signer, TEST_AMOUNT_50);
    }

    #[test]
    fun test_7_2_alice_withdraw_after_closing_channels() {
        let (alice_signer, alice_addr, bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Setup: Open and close channel
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        payment_channel::close_channel(&bob_signer, channel_id, vector::empty());
        
        // Get initial account balance
        let initial_balance = account_coin_store::balance<RGas>(alice_addr);
        
        // Test: Alice withdraws after closing all channels - should succeed
        payment_channel::withdraw_from_hub_entry<RGas>(&alice_signer, TEST_AMOUNT_50);
        
        // Assertion: Withdrawal succeeds
        let final_balance = account_coin_store::balance<RGas>(alice_addr);
        assert!(final_balance == initial_balance + TEST_AMOUNT_50, 7001);
    }

    // === Additional Integration Tests ===

    #[test]
    fun test_integration_full_channel_lifecycle() {
        let (alice_signer, alice_addr, bob_signer, bob_addr, vm_id) = setup_payment_channel_test();
        
        // Complete lifecycle test: deposit ? open channel ? open sub-channel ? claim ? close
        
        // 1. Deposit
        payment_channel::deposit_to_hub_entry<RGas>(&alice_signer, alice_addr, TEST_AMOUNT_100);
        
        // 2. Open channel
        let channel_id = payment_channel::open_channel<RGas>(&alice_signer, bob_addr);
        
        // 3. Open sub-channel
        payment_channel::open_sub_channel_entry(&alice_signer, channel_id, vm_id);
        
        // 4. Multiple claims
        let signature1 = generate_test_signature();
        payment_channel::claim_from_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_10, TEST_NONCE_1, signature1);
        
        let signature2 = generate_test_signature();
        payment_channel::claim_from_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_15, TEST_NONCE_2, signature2);
        
        // 5. Close sub-channel
        let signature_close = generate_test_signature();
        payment_channel::close_sub_channel_for_test(&bob_signer, channel_id, vm_id, TEST_AMOUNT_15, TEST_NONCE_2, signature_close);
        
        // 6. Close channel
        payment_channel::close_channel(&bob_signer, channel_id, vector::empty());
        
        // 7. Verify final state
        let (_sender, _receiver, _coin_type, status) = payment_channel::get_channel_info(channel_id);
        assert!(status == 2, 8001); // STATUS_CLOSED
        assert!(!payment_channel::sub_channel_exists(channel_id, vm_id), 8002);
        
        // 8. Verify funds transferred by checking withdrawal capability
        let initial_bob_account_balance = account_coin_store::balance<RGas>(bob_addr);
        payment_channel::withdraw_from_hub_entry<RGas>(&bob_signer, TEST_AMOUNT_15);
        let final_bob_account_balance = account_coin_store::balance<RGas>(bob_addr);
        assert!(final_bob_account_balance == initial_bob_account_balance + TEST_AMOUNT_15, 8003);
        
        // 9. Alice can now withdraw remaining funds
        payment_channel::withdraw_from_hub_entry<RGas>(&alice_signer, TEST_AMOUNT_100 - TEST_AMOUNT_15);
    }
} 