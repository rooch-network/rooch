// SPDX-License-Identifier: Apache-2.0
// Copyright (c) Rooch Contributors

// PaymentHub and PaymentChannel implementation for unidirectional payment channel protocol
// See: docs/dev-guide/unidirectional-payment-channel-protocol.md
// All comments must be in English (ASCII only) per Rooch Move guide.

module rooch_framework::payment_channel {
    use std::option::{Self, Option};
    use std::signer;
    use std::vector;
    use moveos_std::bcs;
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::table::{Self, Table};
    use moveos_std::type_info;
    use moveos_std::tx_context;
    use moveos_std::timestamp;
    use moveos_std::event;
    use moveos_std::hash;
    use std::string::{Self, String};
    use rooch_framework::coin::{Coin, GenericCoin};
    use rooch_framework::multi_coin_store::{Self, MultiCoinStore};
    use rooch_framework::did;
    use rooch_framework::account_coin_store;

    // === Error Constants ===
    /// The signer is not the designated receiver of the channel.
    const ErrorNotReceiver: u64 = 1;
    /// The channel is not in an active state.
    const ErrorChannelNotActive: u64 = 2;
    /// The provided signature from the sender is invalid.
    const ErrorInvalidSenderSignature: u64 = 3;
    /// The specified Verification Method was not found in the sender's DID.
    const ErrorVerificationMethodNotFound: u64 = 4;
    /// The Verification Method used does not have 'authentication' permission.
    const ErrorInsufficientPermission: u64 = 5;
    /// The provided payment hub object does not match the one linked in the channel.
    const ErrorInvalidPaymentHub: u64 = 6;
    /// The nonce for the sub-channel is not greater than the last confirmed nonce.
    const ErrorInvalidNonce: u64 = 7;
    /// The claimed amount is less than or equal to the already claimed amount.
    const ErrorInvalidAmount: u64 = 8;
    /// The owner of the payment hub does not match the sender of the channel.
    const ErrorHubOwnerMismatch: u64 = 9;
    /// The challenge period has not elapsed yet.
    const ErrorChallengePeriodNotElapsed: u64 = 10;
    /// The channel is already in cancelling state.
    const ErrorChannelAlreadyCancelling: u64 = 11;
    /// The channel is already closed.
    const ErrorChannelAlreadyClosed: u64 = 12;
    /// Insufficient balance in the payment hub.
    const ErrorInsufficientBalance: u64 = 13;
    /// The signer is not the sender of the channel.
    const ErrorNotSender: u64 = 14;

    // === Constants ===
    const STATUS_ACTIVE: u8 = 0;
    const STATUS_CANCELLING: u8 = 1;
    const STATUS_CLOSED: u8 = 2;
    const CHALLENGE_PERIOD_MILLISECONDS: u64 = 86400000; // 1 day

    // === Events ===
    
    /// Event emitted when a payment hub is created
    struct PaymentHubCreatedEvent has copy, drop {
        hub_id: ObjectID,
        owner: address,
    }

    /// Event emitted when a payment channel is opened
    struct PaymentChannelOpenedEvent has copy, drop {
        channel_id: ObjectID,
        sender: address,
        receiver: address,
        payment_hub_id: ObjectID,
        coin_type: String,
    }

    /// Event emitted when funds are claimed from a channel
    struct ChannelClaimedEvent has copy, drop {
        channel_id: ObjectID,
        receiver: address,
        vm_id_fragment: String,
        amount: u256,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
    }

    /// Event emitted when a channel is closed
    struct ChannelClosedEvent has copy, drop {
        channel_id: ObjectID,
        sender: address,
        receiver: address,
        total_paid: u256,
        sub_channels_count: u64,
    }

    /// Event emitted when channel cancellation is initiated
    struct ChannelCancellationInitiatedEvent has copy, drop {
        channel_id: ObjectID,
        sender: address,
        initiated_time: u64,
        pending_amount: u256,
    }

    /// Event emitted when a dispute is raised during cancellation
    struct ChannelDisputeEvent has copy, drop {
        channel_id: ObjectID,
        receiver: address,
        dispute_amount: u256,
        dispute_nonce: u64,
    }

    /// Event emitted when channel cancellation is finalized
    struct ChannelCancellationFinalizedEvent has copy, drop {
        channel_id: ObjectID,
        sender: address,
        final_amount: u256,
    }

    // === Structs ===
    /// A central, user-owned object for managing payments.
    /// It contains a MultiCoinStore to support various coin types.
    /// Every account can only have one payment hub, and the hub can not be transferred.
    struct PaymentHub has key {
        multi_coin_store: Object<MultiCoinStore>,
        //TODO add more settings to channel
    }

    /// A lightweight object representing a payment relationship, linked to a PaymentHub.
    /// The PaymentChannel has no store, it can not be transferred.
    struct PaymentChannel<phantom CoinType: store> has key {
        sender: address,
        receiver: address,
        payment_hub_id: ObjectID, // Links to a PaymentHub object
        sub_channels: Table<String, SubChannelState>,
        status: u8,
        cancellation_info: Option<CancellationInfo>,
    }
    
    /// The on-chain state for a specific sub-channel.
    struct SubChannelState has store {
        last_claimed_amount: u256,
        last_confirmed_nonce: u64,
    }

    /// Information stored when a channel cancellation is initiated.
    struct CancellationInfo has copy, drop, store {
        initiated_time: u64,
        pending_amount: u256,
    }

    
    #[data_struct]
    /// Proof for closing a sub-channel with final state
    struct CloseProof has copy, drop, store {
        vm_id_fragment: String,
        accumulated_amount: u256,
        nonce: u64,
        sender_signature: vector<u8>,
    }

    #[data_struct]
    /// Proof for initiating cancellation of a sub-channel (no signature needed from sender)
    struct CancelProof has copy, drop, store {
        vm_id_fragment: String,
        accumulated_amount: u256,
        nonce: u64,
    }

    // === Public Functions ===

    fun borrow_or_create_payment_hub(owner: address) : &mut Object<PaymentHub> {
        let hub_obj_id = object::account_named_object_id<PaymentHub>(owner);
        if (!object::exists_object_with_type<PaymentHub>(hub_obj_id)) {
            let multi_coin_store = multi_coin_store::create();
            let hub = PaymentHub {
                multi_coin_store,
            };
            // Every account can only have one payment hub
            let hub_obj = object::new_account_named_object(
                owner,
                hub
            );
            object::transfer_extend(hub_obj, owner);
            
            // Emit event for hub creation
            event::emit(PaymentHubCreatedEvent {
                hub_id: hub_obj_id,
                owner,
            });
        };
        object::borrow_mut_object_extend<PaymentHub>(hub_obj_id)
    }

    public fun ensure_payment_hub_exists(owner: address){
        let _hub_obj = borrow_or_create_payment_hub(owner);
    }

    /// Creates and initializes a payment hub for the sender.
    /// This also creates an associated MultiCoinStore.
    public entry fun create_payment_hub() {
        let sender = tx_context::sender();
        ensure_payment_hub_exists(sender);
    }

    /// Deposits a specific type of coin from the account coin store into the payment hub
    public entry fun deposit_to_hub_entry<CoinType: key + store>(
        account: &signer,
        amount: u256,
    ) {
        let account_addr = signer::address_of(account);
        let coin = account_coin_store::withdraw<CoinType>(account, amount);
        deposit_to_hub<CoinType>(account_addr, coin);
    }

    /// Deposits a specific type of coin into the payment hub of the account
    public fun deposit_to_hub<CoinType: key + store>(
        account_addr: address,
        coin: Coin<CoinType>,
    ) {
        let hub_obj = borrow_or_create_payment_hub(account_addr);
        let hub = object::borrow_mut(hub_obj);
        multi_coin_store::deposit_by_type(&mut hub.multi_coin_store, coin);
    }

    /// Deposits a generic coin into the payment hub of the account
    public fun deposit_to_hub_generic(
        account_addr: address,
        coin: GenericCoin,
    ) {
        let hub_obj = borrow_or_create_payment_hub(account_addr);
        let hub = object::borrow_mut(hub_obj);
        multi_coin_store::deposit(&mut hub.multi_coin_store, coin);
    }

    /// Opens a new payment channel linked to a payment hub.
    public fun open_channel<CoinType: key + store>(
        sender: &signer,
        receiver: address,
    ) : ObjectID {
        let sender_addr = signer::address_of(sender);
        let payment_hub_obj = borrow_or_create_payment_hub(sender_addr);
        let payment_hub_id = object::id(payment_hub_obj);
        let channel_obj = object::new<PaymentChannel<CoinType>>(PaymentChannel<CoinType> {
            sender: sender_addr,
            receiver,
            payment_hub_id,
            sub_channels: table::new(),
            status: STATUS_ACTIVE,
            cancellation_info: option::none(),
        });
        let channel_id = object::id(&channel_obj);
        object::transfer_extend(channel_obj, sender_addr);
        
        // Emit event for channel opening
        event::emit(PaymentChannelOpenedEvent {
            channel_id,
            sender: sender_addr,
            receiver,
            payment_hub_id,
            coin_type: type_info::type_name<CoinType>(),
        });
        
        channel_id
    }

    /// Entry function for opening a channel
    public entry fun open_channel_entry<CoinType: key + store>(
        sender: &signer,
        receiver: address,
    ) {
        let _channel_id = open_channel<CoinType>(sender, receiver);
    }

    /// The receiver claims funds from a specific sub-channel.
    public fun claim_from_channel<CoinType: key + store>(
        account: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    ) {
        let receiver = signer::address_of(account);
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow_mut(channel_obj);
        
        // Verify the transaction sender is the receiver
        assert!(channel.receiver == receiver, ErrorNotReceiver);
        assert!(channel.status == STATUS_ACTIVE, ErrorChannelNotActive);
        
        // Verify the sender's signature on the off-chain proof (SubRAV).
        assert!(
            verify_sender_signature(
                channel_id,
                channel.sender,
                sender_vm_id_fragment,
                sub_accumulated_amount,
                sub_nonce,
                sender_signature
            ),
            ErrorInvalidSenderSignature
        );
        
        // Get or create the sub-channel state.
        let sub_channel_state = if (table::contains(&channel.sub_channels, sender_vm_id_fragment)) {
            table::borrow_mut(&mut channel.sub_channels, sender_vm_id_fragment)
        } else {
            table::add(&mut channel.sub_channels, sender_vm_id_fragment, SubChannelState {
                last_claimed_amount: 0u256,
                last_confirmed_nonce: 0,
            });
            table::borrow_mut(&mut channel.sub_channels, sender_vm_id_fragment)
        };
        
        // Validate amount and nonce are strictly increasing.
        assert!(sub_accumulated_amount > sub_channel_state.last_claimed_amount, ErrorInvalidAmount);
        assert!(sub_nonce > sub_channel_state.last_confirmed_nonce, ErrorInvalidNonce);
        
        let incremental_amount = sub_accumulated_amount - sub_channel_state.last_claimed_amount;
        
        // Update the sub-channel state on-chain.
        sub_channel_state.last_claimed_amount = sub_accumulated_amount;
        sub_channel_state.last_confirmed_nonce = sub_nonce;
        
        // Withdraw funds from the payment hub and transfer to the receiver.
        let hub_obj = borrow_or_create_payment_hub(channel.sender);
        let hub = object::borrow_mut(hub_obj);
        let coin_type_name = type_info::type_name<CoinType>();
        let generic_payment = multi_coin_store::withdraw(&mut hub.multi_coin_store, coin_type_name, incremental_amount);

        // Deposit the coin directly into the receiver's payment hub
        deposit_to_hub_generic(channel.receiver, generic_payment);
        
        // Emit claim event
        event::emit(ChannelClaimedEvent {
            channel_id,
            receiver,
            vm_id_fragment: sender_vm_id_fragment,
            amount: incremental_amount,
            sub_accumulated_amount,
            sub_nonce,
        });
    }

    /// Entry function for claiming from channel
    public entry fun claim_from_channel_entry<CoinType: key + store>(
        account: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    ) {
        claim_from_channel<CoinType>(
            account,
            channel_id,
            sender_vm_id_fragment,
            sub_accumulated_amount,
            sub_nonce,
            sender_signature
        );
    }

    /// Close a specific sub-channel with final state from receiver
    public fun close_sub_channel<CoinType: key + store>(
        account: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        final_accumulated_amount: u256,
        final_nonce: u64,
        sender_signature: vector<u8>
    ) {
        let receiver = signer::address_of(account);
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow_mut(channel_obj);
        
        // Verify the transaction sender is the receiver
        assert!(channel.receiver == receiver, ErrorNotReceiver);
        assert!(channel.status == STATUS_ACTIVE, ErrorChannelNotActive);
        
        // Verify the sender's signature on the final SubRAV
        assert!(
            verify_sender_signature(
                channel_id,
                channel.sender,
                sender_vm_id_fragment,
                final_accumulated_amount,
                final_nonce,
                sender_signature
            ),
            ErrorInvalidSenderSignature
        );
        
        // Get the sub-channel state
        let sub_channel_state = if (table::contains(&channel.sub_channels, sender_vm_id_fragment)) {
            table::borrow_mut(&mut channel.sub_channels, sender_vm_id_fragment)
        } else {
            table::add(&mut channel.sub_channels, sender_vm_id_fragment, SubChannelState {
                last_claimed_amount: 0u256,
                last_confirmed_nonce: 0,
            });
            table::borrow_mut(&mut channel.sub_channels, sender_vm_id_fragment)
        };
        
        // Validate final amounts
        assert!(final_accumulated_amount >= sub_channel_state.last_claimed_amount, ErrorInvalidAmount);
        assert!(final_nonce >= sub_channel_state.last_confirmed_nonce, ErrorInvalidNonce);
        
        let final_payment_amount = final_accumulated_amount - sub_channel_state.last_claimed_amount;
        
        if (final_payment_amount > 0) {
            // Transfer final payment
            let hub_obj = borrow_or_create_payment_hub(channel.sender);
            let hub = object::borrow_mut(hub_obj);
            let coin_type_name = type_info::type_name<CoinType>();
            let generic_payment = multi_coin_store::withdraw(&mut hub.multi_coin_store, coin_type_name, final_payment_amount);
            deposit_to_hub_generic(channel.receiver, generic_payment);
        };
        
        // Update final state for this sub-channel
        sub_channel_state.last_claimed_amount = final_accumulated_amount;
        sub_channel_state.last_confirmed_nonce = final_nonce;
        
        // Emit sub-channel close event
        event::emit(ChannelClaimedEvent {
            channel_id,
            receiver,
            vm_id_fragment: sender_vm_id_fragment,
            amount: final_payment_amount,
            sub_accumulated_amount: final_accumulated_amount,
            sub_nonce: final_nonce,
        });
    }

    /// Entry function for closing a sub-channel
    public entry fun close_sub_channel_entry<CoinType: key + store>(
        account: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        final_accumulated_amount: u256,
        final_nonce: u64,
        sender_signature: vector<u8>
    ) {
        close_sub_channel<CoinType>(
            account,
            channel_id,
            sender_vm_id_fragment,
            final_accumulated_amount,
            final_nonce,
            sender_signature
        );
    }

    /// Close the entire channel with final settlement of all sub-channels
    /// Called by receiver with proofs of final state from all sub-channels
    public fun close_channel<CoinType: key + store>(
        receiver: &signer,
        channel_id: ObjectID,
        proofs: vector<CloseProof>,
    ) {
        let receiver_addr = signer::address_of(receiver);
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow_mut(channel_obj);
        
        // Verify receiver is the channel receiver and channel is active
        assert!(channel.receiver == receiver_addr, ErrorNotReceiver);
        assert!(channel.status == STATUS_ACTIVE, ErrorChannelNotActive);
        
        // Get sub-channels count for processing
        let sub_channels_count = table::length(&channel.sub_channels);
        
        let total_incremental_amount = 0u256;
        
        // Process each closure proof (if any)
        if (sub_channels_count > 0) {
            let i = 0;
            let proofs_len = vector::length(&proofs);
            
            while (i < proofs_len) {
            let proof = vector::borrow(&proofs, i);
            let vm_id_fragment = proof.vm_id_fragment;
            
            // Verify the sender's signature on this final SubRAV
            assert!(
                verify_sender_signature(
                    channel_id,
                    channel.sender,
                    vm_id_fragment,
                    proof.accumulated_amount,
                    proof.nonce,
                    proof.sender_signature
                ),
                ErrorInvalidSenderSignature
            );
            
            // Get the existing sub-channel state (must exist)
            assert!(table::contains(&channel.sub_channels, vm_id_fragment), ErrorInvalidAmount);
            let sub_channel_state = table::borrow_mut(&mut channel.sub_channels, vm_id_fragment);
            
            // Validate amount and nonce progression
            assert!(proof.accumulated_amount >= sub_channel_state.last_claimed_amount, ErrorInvalidAmount);
            assert!(proof.nonce >= sub_channel_state.last_confirmed_nonce, ErrorInvalidNonce);
            
            // Calculate incremental amount for this sub-channel
            let incremental_amount = proof.accumulated_amount - sub_channel_state.last_claimed_amount;
            total_incremental_amount = total_incremental_amount + incremental_amount;
            
            // Update sub-channel state to final values
            sub_channel_state.last_claimed_amount = proof.accumulated_amount;
            sub_channel_state.last_confirmed_nonce = proof.nonce;
            
                i = i + 1;
            };
        };
        
        // Transfer total incremental amount from sender's hub to receiver's hub
        if (total_incremental_amount > 0) {
            let hub_obj = borrow_or_create_payment_hub(channel.sender);
            let hub = object::borrow_mut(hub_obj);
            let coin_type_name = type_info::type_name<CoinType>();
            let generic_payment = multi_coin_store::withdraw(&mut hub.multi_coin_store, coin_type_name, total_incremental_amount);
            deposit_to_hub_generic(channel.receiver, generic_payment);
        };
        
        // Mark channel as closed
        channel.status = STATUS_CLOSED;
        
        // Emit channel closed event
        event::emit(ChannelClosedEvent {
            channel_id,
            sender: channel.sender,
            receiver: receiver_addr,
            total_paid: total_incremental_amount,
            sub_channels_count,
        });
    }

    /// Entry function for closing the entire channel with settlement
    /// Takes serialized closure proofs and deserializes them
    public entry fun close_channel_entry<CoinType: key + store>(
        receiver: &signer,
        channel_id: ObjectID,
        serialized_proofs: vector<u8>,
    ) {
        let proofs = bcs::from_bytes<vector<CloseProof>>(serialized_proofs);
        close_channel<CoinType>(receiver, channel_id, proofs);
    } 

    /// Entry function for initiating cancellation
    public entry fun initiate_cancellation_entry<CoinType: key + store>(
        sender: &signer,
        channel_id: ObjectID,
    ) {
        initiate_cancellation<CoinType>(sender, channel_id, vector::empty());
    }

    /// Sender initiates unilateral channel cancellation with proofs for sub-channels
    public fun initiate_cancellation<CoinType: key + store>(
        sender: &signer,
        channel_id: ObjectID,
        proofs: vector<CancelProof>,
    ) {
        let sender_addr = signer::address_of(sender);
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow_mut(channel_obj);
        
        // Verify sender is the channel sender and channel is active
        assert!(channel.sender == sender_addr, ErrorNotSender);
        assert!(channel.status == STATUS_ACTIVE, ErrorChannelNotActive);
        
        // Get sub-channels count for processing
        let sub_channels_count = table::length(&channel.sub_channels);
        // Sender can provide partial proofs, receiver can dispute missing ones during challenge period
        
        let total_pending_amount = 0u256;
        
        // Process each cancellation proof (if any)
        if (sub_channels_count > 0) {
            let i = 0;
            let proofs_len = vector::length(&proofs);
            
            while (i < proofs_len) {
                let proof = vector::borrow(&proofs, i);
                let vm_id_fragment = proof.vm_id_fragment;
                
                // Get the existing sub-channel state (must exist)
                assert!(table::contains(&channel.sub_channels, vm_id_fragment), ErrorInvalidAmount);
                let sub_channel_state = table::borrow_mut(&mut channel.sub_channels, vm_id_fragment);
                
                // Validate amount and nonce progression
                assert!(proof.accumulated_amount >= sub_channel_state.last_claimed_amount, ErrorInvalidAmount);
                assert!(proof.nonce >= sub_channel_state.last_confirmed_nonce, ErrorInvalidNonce);
                
                // Calculate incremental amount for this sub-channel
                let incremental_amount = proof.accumulated_amount - sub_channel_state.last_claimed_amount;
                total_pending_amount = total_pending_amount + incremental_amount;
                
                // Update sub-channel state to new baseline to prevent double counting in disputes
                sub_channel_state.last_claimed_amount = proof.accumulated_amount;
                sub_channel_state.last_confirmed_nonce = proof.nonce;
                
                i = i + 1;
            };
        };
        
        // Set channel to cancelling state
        channel.status = STATUS_CANCELLING;
        let current_time = timestamp::now_milliseconds();
        channel.cancellation_info = option::some(CancellationInfo {
            initiated_time: current_time,
            pending_amount: total_pending_amount,
        });
        
        // Emit cancellation event
        event::emit(ChannelCancellationInitiatedEvent {
            channel_id,
            sender: sender_addr,
            initiated_time: current_time,
            pending_amount: total_pending_amount,
        });
    }

    /// Entry function for initiating cancellation with proofs
    /// Takes serialized cancellation proofs and deserializes them
    public entry fun initiate_cancellation_with_proofs_entry<CoinType: key + store>(
        sender: &signer,
        channel_id: ObjectID,
        serialized_proofs: vector<u8>,
    ) {
        let proofs = bcs::from_bytes<vector<CancelProof>>(serialized_proofs);
        initiate_cancellation<CoinType>(sender, channel_id, proofs);
    }

    /// Receiver disputes cancellation with newer state
    public fun dispute_cancellation<CoinType: key + store>(
        account: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        dispute_accumulated_amount: u256,
        dispute_nonce: u64,
        sender_signature: vector<u8>
    ) {
        let receiver = signer::address_of(account);
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow_mut(channel_obj);
        
        // Verify receiver and channel state
        assert!(channel.receiver == receiver, ErrorNotReceiver);
        assert!(channel.status == STATUS_CANCELLING, ErrorChannelNotActive);
        
        // Verify signature
        assert!(
            verify_sender_signature(
                channel_id,
                channel.sender,
                sender_vm_id_fragment,
                dispute_accumulated_amount,
                dispute_nonce,
                sender_signature
            ),
            ErrorInvalidSenderSignature
        );
        
        // Get or create the sub-channel state
        let sub_channel_state = if (table::contains(&channel.sub_channels, sender_vm_id_fragment)) {
            table::borrow_mut(&mut channel.sub_channels, sender_vm_id_fragment)
        } else {
            table::add(&mut channel.sub_channels, sender_vm_id_fragment, SubChannelState {
                last_claimed_amount: 0u256,
                last_confirmed_nonce: 0,
            });
            table::borrow_mut(&mut channel.sub_channels, sender_vm_id_fragment)
        };
        
        // Validate dispute amount and nonce
        assert!(dispute_accumulated_amount >= sub_channel_state.last_claimed_amount, ErrorInvalidAmount);
        assert!(dispute_nonce >= sub_channel_state.last_confirmed_nonce, ErrorInvalidNonce);
        
        // Calculate the additional incremental amount from this dispute
        let old_claimed_amount = sub_channel_state.last_claimed_amount;
        let new_claimed_amount = dispute_accumulated_amount;
        let additional_amount = new_claimed_amount - old_claimed_amount;
        
        // Update the pending amount if this dispute increases the total
        if (additional_amount > 0) {
            let cancellation_info = option::borrow_mut(&mut channel.cancellation_info);
            cancellation_info.pending_amount = cancellation_info.pending_amount + additional_amount;
            
            // Update the sub-channel state to reflect this dispute
            sub_channel_state.last_claimed_amount = dispute_accumulated_amount;
            sub_channel_state.last_confirmed_nonce = dispute_nonce;
        };
        
        // Emit dispute event
        event::emit(ChannelDisputeEvent {
            channel_id,
            receiver,
            dispute_amount: dispute_accumulated_amount,
            dispute_nonce,
        });
    }

    /// Entry function for disputing cancellation
    public entry fun dispute_cancellation_entry<CoinType: key + store>(
        account: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        dispute_accumulated_amount: u256,
        dispute_nonce: u64,
        sender_signature: vector<u8>
    ) {
        dispute_cancellation<CoinType>(
            account,
            channel_id,
            sender_vm_id_fragment,
            dispute_accumulated_amount,
            dispute_nonce,
            sender_signature
        );
    }

    /// Finalize cancellation after challenge period
    public fun finalize_cancellation<CoinType: key + store>(
        channel_id: ObjectID,
    ) {
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow_mut(channel_obj);
        
        // Verify channel is in cancelling state
        assert!(channel.status == STATUS_CANCELLING, ErrorChannelNotActive);
        assert!(option::is_some(&channel.cancellation_info), ErrorInvalidAmount);
        
        let cancellation_info = option::borrow(&channel.cancellation_info);
        let current_time = timestamp::now_milliseconds();
        
        // Verify challenge period has elapsed
        assert!(
            current_time >= cancellation_info.initiated_time + CHALLENGE_PERIOD_MILLISECONDS,
            ErrorChallengePeriodNotElapsed
        );
        
        let final_amount = cancellation_info.pending_amount;
        
        if (final_amount > 0) {
            // Transfer final payment to receiver
            let hub_obj = borrow_or_create_payment_hub(channel.sender);
            let hub = object::borrow_mut(hub_obj);
            let coin_type_name = type_info::type_name<CoinType>();
            let generic_payment = multi_coin_store::withdraw(&mut hub.multi_coin_store, coin_type_name, final_amount);
            deposit_to_hub_generic(channel.receiver, generic_payment);
        };
        
        // Mark channel as closed
        channel.status = STATUS_CLOSED;
        
        // Emit finalization event
        event::emit(ChannelCancellationFinalizedEvent {
            channel_id,
            sender: channel.sender,
            final_amount,
        });
    }

    /// Entry function for finalizing cancellation
    public entry fun finalize_cancellation_entry<CoinType: key + store>(
        channel_id: ObjectID,
    ) {
        finalize_cancellation<CoinType>(channel_id);
    }

    // === View Functions ===

    /// Get payment hub ID for an address
    public fun get_payment_hub_id(owner: address): ObjectID {
        object::account_named_object_id<PaymentHub>(owner)
    }

    /// Check if payment hub exists for an address
    public fun payment_hub_exists(owner: address): bool {
        let hub_id = get_payment_hub_id(owner);
        object::exists_object_with_type<PaymentHub>(hub_id)
    }

    /// Get channel information
    public fun get_channel_info<CoinType: store>(channel_id: ObjectID): (address, address, ObjectID, u8) {
        let channel_obj = object::borrow_object<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow(channel_obj);
        (channel.sender, channel.receiver, channel.payment_hub_id, channel.status)
    }

    /// Get sub-channel state
    public fun get_sub_channel_state<CoinType: store>(channel_id: ObjectID, vm_id_fragment: String): (u256, u64) {
        let channel_obj = object::borrow_object<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow(channel_obj);
        
        if (table::contains(&channel.sub_channels, vm_id_fragment)) {
            let state = table::borrow(&channel.sub_channels, vm_id_fragment);
            (state.last_claimed_amount, state.last_confirmed_nonce)
        } else {
            (0u256, 0u64)
        }
    }

    /// Check if a sub-channel exists
    public fun sub_channel_exists<CoinType: store>(channel_id: ObjectID, vm_id_fragment: String): bool {
        let channel_obj = object::borrow_object<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow(channel_obj);
        table::contains(&channel.sub_channels, vm_id_fragment)
    }

    /// Get the number of sub-channels in a payment channel
    public fun get_sub_channel_count<CoinType: store>(channel_id: ObjectID): u64 {
        let channel_obj = object::borrow_object<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow(channel_obj);
        table::length(&channel.sub_channels)
    }

    /// Get cancellation info
    public fun get_cancellation_info<CoinType: store>(channel_id: ObjectID): Option<CancellationInfo> {
        let channel_obj = object::borrow_object<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow(channel_obj);
        
        if (option::is_some(&channel.cancellation_info)) {
            let info = option::borrow(&channel.cancellation_info);
            option::some(*info)
        } else {
            option::none()
        }
    }

    // === Internal Helper Functions ===

    fun get_sub_rav_hash(
        channel_id: ObjectID,
        vm_id_fragment: String,
        accumulated_amount: u256,
        nonce: u64
    ): vector<u8> {
        // Serialize each field and concatenate for hash
        let bytes = vector::empty<u8>();
        let id_bytes = bcs::to_bytes(&channel_id);
        let frag_bytes = bcs::to_bytes(&vm_id_fragment);
        let amt_bytes = bcs::to_bytes(&accumulated_amount);
        let nonce_bytes = bcs::to_bytes(&nonce);
        vector::append(&mut bytes, id_bytes);
        vector::append(&mut bytes, frag_bytes);
        vector::append(&mut bytes, amt_bytes);
        vector::append(&mut bytes, nonce_bytes);
        hash::sha3_256(bytes)
    }

    fun verify_sender_signature(
        channel_id: ObjectID,
        sender_address: address,
        vm_id_fragment: String,
        accumulated_amount: u256,
        nonce: u64,
        signature: vector<u8>
    ): bool {
        let msg_hash = get_sub_rav_hash(channel_id, vm_id_fragment, accumulated_amount, nonce);
        
        // Get DID document for sender
        let did_doc = did::get_did_document_by_address(sender_address);
        let vm_opt = did::doc_verification_method(did_doc, &vm_id_fragment);
        assert!(option::is_some(&vm_opt), ErrorVerificationMethodNotFound);
        
        let vm = option::extract(&mut vm_opt);
        
        // Check if verification method has authentication permission
        assert!(
            did::has_verification_relationship_in_doc(did_doc, &vm_id_fragment, did::verification_relationship_authentication()),
            ErrorInsufficientPermission
        );
        
        // Verify signature based on verification method type
        let public_key_multibase = did::verification_method_public_key_multibase(&vm);
        let method_type = did::verification_method_type(&vm);
        
        verify_signature_by_type(msg_hash, signature, public_key_multibase, method_type)
    }

    fun verify_signature_by_type(
        message: vector<u8>,
        signature: vector<u8>,
        public_key_multibase: &String,
        method_type: &String
    ): bool {
        use moveos_std::multibase_codec;
        use rooch_framework::ed25519;
        use rooch_framework::ecdsa_k1;
        use rooch_framework::ecdsa_r1;
        
        // Decode public key from multibase
        let pk_bytes_opt = multibase_codec::decode(public_key_multibase);
        assert!(option::is_some(&pk_bytes_opt), ErrorInvalidSenderSignature);
        let pk_bytes = option::destroy_some(pk_bytes_opt);
        
        // Verify signature based on method type
        if (*method_type == string::utf8(b"Ed25519VerificationKey2020")) {
            ed25519::verify(&signature, &pk_bytes, &message)
        } else if (*method_type == string::utf8(b"EcdsaSecp256k1VerificationKey2019")) {
            ecdsa_k1::verify(&signature, &pk_bytes, &message, ecdsa_k1::sha256())
        } else if (*method_type == string::utf8(b"EcdsaSecp256r1VerificationKey2019")) {
            ecdsa_r1::verify(&signature, &pk_bytes, &message)
        } else {
            false
        }
    }
}
