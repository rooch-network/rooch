// SPDX-License-Identifier: Apache-2.0
// Copyright (c) Rooch Contributors

// PaymentHub and PaymentChannel implementation for unidirectional payment channel protocol
// See: docs/dev-guide/unidirectional-payment-channel-protocol.md

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
    use std::string::String;
    use rooch_framework::coin::{Self, Coin, GenericCoin};
    use rooch_framework::multi_coin_store::{Self, MultiCoinStore};
    use rooch_framework::did;
    use rooch_framework::account_coin_store;
    use rooch_framework::chain_id;

    friend rooch_framework::transaction_gas;

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
    /// The sub-channel has not been authorized yet.
    const ErrorSubChannelNotAuthorized: u64 = 15;
    /// Only the sender can authorize verification methods for the channel.
    const ErrorVMAuthorizeOnlySender: u64 = 16;
    /// The verification method already exists for this sub-channel.
    const ErrorVerificationMethodAlreadyExists: u64 = 17;
    /// A channel between this sender and receiver already exists for this coin type.
    const ErrorChannelAlreadyExists: u64 = 18;
    /// There are still active channels for this coin type.
    const ErrorActiveChannelExists: u64 = 19;
    /// The sender must have a DID document to open a channel.
    const ErrorSenderMustIsDID: u64 = 20;
    /// The coin type provided does not match the channel's coin type.
    const ErrorMismatchedCoinType: u64 = 21;
    /// The channel epoch in the SubRAV does not match the current channel epoch.
    const ErrorInvalidChannelEpoch: u64 = 22;
    /// The chain_id in the SubRAV does not match the current chain_id.
    const ErrorInvalidChainId: u64 = 23;
    /// The SubRAV version is not supported.
    const ErrorUnsupportedVersion: u64 = 24;

    // === Constants ===
    const STATUS_ACTIVE: u8 = 0;
    const STATUS_CANCELLING: u8 = 1;
    const STATUS_CLOSED: u8 = 2;

    const CHALLENGE_PERIOD_MILLISECONDS: u64 = 86400000; // 1 day

    /// Current supported SubRAV version
    const SUB_RAV_VERSION_V1: u8 = 1;

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

    /// Event emitted when a sub-channel is authorized
    struct SubChannelAuthorizedEvent has copy, drop {
        channel_id: ObjectID,
        sender: address,
        vm_id_fragment: String,
        pk_multibase: String,
        method_type: String,
    }

    /// Event emitted when funds are withdrawn from a payment hub
    struct PaymentHubWithdrawEvent has copy, drop {
        hub_id: ObjectID,
        owner: address,
        coin_type: String,
        amount: u256,
    }

    // === Structs ===
    /// Unique key for identifying a unidirectional payment channel
    /// Used to generate deterministic ObjectID for channels
    struct ChannelKey has copy, drop, store {
        sender: address,
        receiver: address,
        coin_type: String,
    }

    /// A central, user-owned object for managing payments.
    /// It contains a MultiCoinStore to support various coin types.
    /// Every account can only have one payment hub, and the hub can not be transferred.
    struct PaymentHub has key {
        multi_coin_store: Object<MultiCoinStore>,
        // Record the number of active channels for each coin type
        active_channels: Table<String, u64>,
        //TODO add more settings to channel
    }

    /// A lightweight object representing a payment relationship, linked to a PaymentHub.
    /// The PaymentChannel has no store, it can not be transferred.
    struct PaymentChannel has key {
        sender: address,
        receiver: address,
        coin_type: String, // The type of coin used in this channel
        sub_channels: Table<String, SubChannel>,
        status: u8,
        channel_epoch: u64, // Incremented each time the channel is closed and reopened
        cancellation_info: Option<CancellationInfo>,
    }
    
    /// The on-chain state for a specific sub-channel, including authorization metadata.
    struct SubChannel has store {
        // --- Authorization metadata (set once during authorize_sub_channel) ---
        // We store the public key and method type to avoid the sender removing the verification method after the sub-channel is authorized
        pk_multibase: String,
        method_type: String,
        
        // --- State data (evolves with claim/close operations) ---
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
    struct CloseProofs has copy, drop, store {
        proofs: vector<CloseProof>,
    }

    #[data_struct]
    /// Proof for initiating cancellation of a sub-channel (no signature needed from sender)
    struct CancelProof has copy, drop, store {
        vm_id_fragment: String,
        accumulated_amount: u256,
        nonce: u64,
    }

    #[data_struct]
    struct CancelProofs has copy, drop, store {
        proofs: vector<CancelProof>,
    }

    #[data_struct]
    /// Structure representing a Sub-RAV (Sub-channel Receipts and Vouchers) for off-chain signature verification
    struct SubRAV has copy, drop, store {
        version: u8,
        chain_id: u64,
        channel_id: ObjectID,
        channel_epoch: u64,
        vm_id_fragment: String,
        accumulated_amount: u256,
        nonce: u64,
    }

    // === Public Functions ===

    /// Calculate the deterministic ObjectID for a payment channel
    /// This allows anyone to derive the channel ID from sender, receiver, and coin type
    public fun calc_channel_object_id(sender: address, receiver: address, coin_type: String): ObjectID {
        let key = ChannelKey { sender, receiver, coin_type };
        object::custom_object_id<ChannelKey, PaymentChannel>(key)
    }

    public(friend) fun borrow_or_create_payment_hub(owner: address) : &mut Object<PaymentHub> {
        let hub_obj_id = object::account_named_object_id<PaymentHub>(owner);
        if (!object::exists_object_with_type<PaymentHub>(hub_obj_id)) {
            let multi_coin_store = multi_coin_store::create();
            let hub = PaymentHub {
                multi_coin_store,
                active_channels: table::new(),
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

    /// Deposits a specific type of coin from the sender's account coin store into the receiver's payment hub
    public entry fun deposit_to_hub_entry<CoinType: key + store>(
        sender: &signer,
        receiver: address,
        amount: u256,
    ) {
        let coin = account_coin_store::withdraw<CoinType>(sender, amount);
        deposit_to_hub<CoinType>(receiver, coin);
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

    /// Withdraws funds from the payment hub to the owner's account coin store
    /// Will fail if there are active channels for this coin type
    public fun withdraw_from_hub<CoinType: key + store>(
        owner: &signer,
        amount: u256,
    ) {
        let owner_addr = signer::address_of(owner);
        let hub_obj = borrow_or_create_payment_hub(owner_addr);
        let hub = object::borrow_mut(hub_obj);
        
        // Check if there are active channels for this coin type
        let coin_type_name = type_info::type_name<CoinType>();
        if (table::contains(&hub.active_channels, coin_type_name)) {
            let count = table::borrow(&hub.active_channels, coin_type_name);
            assert!(*count == 0, ErrorActiveChannelExists);
        };
        
        // Withdraw from multi_coin_store and deposit to account coin store
        let coin = multi_coin_store::withdraw_by_type<CoinType>(&mut hub.multi_coin_store, amount);
        account_coin_store::deposit<CoinType>(owner_addr, coin);
        
        // Emit withdrawal event
        let hub_id = object::id(hub_obj);
        event::emit(PaymentHubWithdrawEvent {
            hub_id,
            owner: owner_addr,
            coin_type: coin_type_name,
            amount,
        });
    }

    /// Entry function for withdrawing from payment hub
    public entry fun withdraw_from_hub_entry<CoinType: key + store>(
        owner: &signer,
        amount: u256,
    ) {
        withdraw_from_hub<CoinType>(owner, amount);
    }

    /// Opens a new payment channel linked to a payment hub.
    /// If a channel already exists and is closed, it will be reactivated.
    /// If a channel already exists and is active, it will return an error.
    public fun open_channel<CoinType: key + store>(
        channel_sender: &signer,
        channel_receiver: address,
    ) : ObjectID {
        let sender_addr = signer::address_of(channel_sender);
        assert!(sender_addr != channel_receiver, ErrorNotReceiver);
        assert!(did::exists_did_for_address(sender_addr), ErrorSenderMustIsDID);
        let coin_type = type_info::type_name<CoinType>();
        let channel_id = calc_channel_object_id(sender_addr, channel_receiver, coin_type);
        
        // Check if channel already exists
        if (object::exists_object_with_type<PaymentChannel>(channel_id)) {
            // Channel exists, check if it can be reused
            let channel_obj = object::borrow_mut_object_extend<PaymentChannel>(channel_id);
            let channel = object::borrow_mut(channel_obj);
            
            // Only allow reuse if channel is closed
            assert!(channel.status == STATUS_CLOSED, ErrorChannelAlreadyExists);
            
            // Verify coin type matches
            assert!(channel.coin_type == coin_type, ErrorMismatchedCoinType);
            
            // Reactivate the channel
            channel.status = STATUS_ACTIVE;
            channel.cancellation_info = option::none();
            // Note: sub_channels table is preserved, so previously authorized VMs remain valid
            
            // Increment active channel count for this coin type
            let payment_hub_obj = borrow_or_create_payment_hub(sender_addr);
            let payment_hub = object::borrow_mut(payment_hub_obj);
            if (table::contains(&payment_hub.active_channels, coin_type)) {
                let count = table::borrow_mut(&mut payment_hub.active_channels, coin_type);
                *count = *count + 1;
            } else {
                table::add(&mut payment_hub.active_channels, coin_type, 1);
            };
            
            // Emit event for channel reactivation
            event::emit(PaymentChannelOpenedEvent {
                channel_id,
                sender: sender_addr,
                receiver: channel_receiver,
                coin_type,
            });
            
            return channel_id
        };
        
        // Create new channel
        let payment_hub_obj = borrow_or_create_payment_hub(sender_addr);
        
        // Increment active channel count for this coin type
        let payment_hub = object::borrow_mut(payment_hub_obj);
        if (table::contains(&payment_hub.active_channels, coin_type)) {
            let count = table::borrow_mut(&mut payment_hub.active_channels, coin_type);
            *count = *count + 1;
        } else {
            table::add(&mut payment_hub.active_channels, coin_type, 1);
        };
        
        let key = ChannelKey { sender: sender_addr, receiver: channel_receiver, coin_type };
        let channel_obj = object::new_with_id(key, PaymentChannel {
            sender: sender_addr,
            receiver: channel_receiver,
            coin_type,
            sub_channels: table::new(),
            status: STATUS_ACTIVE,
            channel_epoch: 0,
            cancellation_info: option::none(),
        });
        object::transfer_extend(channel_obj, sender_addr);
        
        // Emit event for new channel creation
        event::emit(PaymentChannelOpenedEvent {
            channel_id,
            sender: sender_addr,
            receiver: channel_receiver,
            coin_type,
        });
        
        channel_id
    }

    /// Entry function for opening a channel
    public entry fun open_channel_entry<CoinType: key + store>(
        channel_sender: &signer,
        channel_receiver: address,
    ) {
        let _channel_id = open_channel<CoinType>(channel_sender, channel_receiver);
    }

    /// Authorizes a sub-channel by granting a verification method permission for the payment channel.
    /// This function must be called by the sender before using any vm_id_fragment for payments.
    public fun authorize_sub_channel(
        channel_sender: &signer,
        channel_id: ObjectID,
        vm_id_fragment: String,
    ) {
        let sender_addr = signer::address_of(channel_sender);
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel>(channel_id);
        let channel = object::borrow_mut(channel_obj);
        
        // Verify the transaction sender is the channel sender
        assert!(channel.sender == sender_addr, ErrorVMAuthorizeOnlySender);
        assert!(channel.status == STATUS_ACTIVE, ErrorChannelNotActive);
        
        // Get DID document and verify the verification method
        let did_doc = did::get_did_document_by_address(sender_addr);
        let vm_opt = did::doc_verification_method(did_doc, &vm_id_fragment);
        assert!(option::is_some(&vm_opt), ErrorVerificationMethodNotFound);
        
        let vm = option::extract(&mut vm_opt);
        
        // Check if verification method has authentication permission
        assert!(
            did::has_verification_relationship_in_doc(did_doc, &vm_id_fragment, did::verification_relationship_authentication()),
            ErrorInsufficientPermission
        );
        
        // Extract metadata from verification method
        let pk_multibase = *did::verification_method_public_key_multibase(&vm);
        let method_type = *did::verification_method_type(&vm);
        
        // A sub-channel can only be authorized once.
        assert!(!table::contains(&channel.sub_channels, vm_id_fragment), ErrorVerificationMethodAlreadyExists);
        
        // Create and store the sub-channel with authorization metadata
        table::add(&mut channel.sub_channels, vm_id_fragment, SubChannel {
            pk_multibase,
            method_type,
            last_claimed_amount: 0u256,
            last_confirmed_nonce: 0,
        });
 
        // Emit sub-channel authorized event
        event::emit(SubChannelAuthorizedEvent {
            channel_id,
            sender: sender_addr,
            vm_id_fragment,
            pk_multibase,
            method_type,
        });
    }

    /// Entry function for authorizing a sub-channel
    public entry fun authorize_sub_channel_entry(
        channel_sender: &signer,
        channel_id: ObjectID,
        vm_id_fragment: String,
    ) {
        authorize_sub_channel(channel_sender, channel_id, vm_id_fragment);
    }

    /// Convenience function to open a channel and sub-channel in one step.
    /// This function will:
    /// 1. Create a new channel if none exists
    /// 2. Reactivate a closed channel if one exists
    /// 3. Authorize the specified verification method for the channel
    /// Returns the channel ID for reference.
    public fun open_channel_with_sub_channel<CoinType: key + store>(
        channel_sender: &signer,
        channel_receiver: address,
        vm_id_fragment: String,
    ): ObjectID {
        let sender_addr = signer::address_of(channel_sender);
        let coin_type = type_info::type_name<CoinType>();
        let channel_id = calc_channel_object_id(sender_addr, channel_receiver, coin_type);
        
        // Step 1: Ensure channel exists and is active
        if (!object::exists_object_with_type<PaymentChannel>(channel_id)) {
            // Channel doesn't exist, create it
            let _ = open_channel<CoinType>(channel_sender, channel_receiver);
        } else {
            // Channel exists, check if it needs reactivation
            let channel_obj = object::borrow_object<PaymentChannel>(channel_id);
            let channel = object::borrow(channel_obj);
            if (channel.status == STATUS_CLOSED) {
                // Reactivate closed channel
                let _ = open_channel<CoinType>(channel_sender, channel_receiver);
            };
            // If already active, do nothing for the channel itself
        };
        
        // Step 2: Ensure sub-channel is authorized (authorize VM if not already done)
        let channel_obj = object::borrow_object<PaymentChannel>(channel_id);
        let channel = object::borrow(channel_obj);
        if (!table::contains(&channel.sub_channels, vm_id_fragment)) {
            // Sub-channel not authorized yet, authorize it
            authorize_sub_channel(channel_sender, channel_id, vm_id_fragment);
        };
        // If sub-channel already exists, it means VM was already authorized
        
        channel_id
    }

    /// Entry function for opening a channel and sub-channel in one step
    public entry fun open_channel_with_sub_channel_entry<CoinType: key + store>(
        channel_sender: &signer,
        channel_receiver: address,
        vm_id_fragment: String,
    ) {
        let _channel_id = open_channel_with_sub_channel<CoinType>(channel_sender, channel_receiver, vm_id_fragment);
    }

    /// Anyone can claim funds from a specific sub-channel on behalf of the receiver.
    /// The funds will always be transferred to the channel receiver regardless of who calls this function.
    public fun claim_from_channel(
        claimer: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    ) {
        internal_claim_from_channel(
            claimer,
            channel_id,
            sender_vm_id_fragment,
            sub_accumulated_amount,
            sub_nonce,
            sender_signature,
            false
        );
    }

    /// The receiver claims funds from a specific sub-channel.
    fun internal_claim_from_channel(
        _claimer: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>,
        skip_signature_verification: bool
    ) {
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel>(channel_id);
        let channel = object::borrow_mut(channel_obj);
        
        // Note: Anyone can execute claim on behalf of the receiver
        // The funds will always go to the channel.receiver regardless of who calls this function
        assert!(channel.status == STATUS_ACTIVE, ErrorChannelNotActive);
        
        // Verify the sub-channel has been authorized
        assert!(table::contains(&channel.sub_channels, sender_vm_id_fragment), ErrorSubChannelNotAuthorized);
        
        // Verify the sender's signature on the off-chain proof (SubRAV).
        if (!skip_signature_verification) {
            let sub_rav = SubRAV {
                version: SUB_RAV_VERSION_V1,
                chain_id: chain_id::chain_id(),
                channel_id,
                channel_epoch: channel.channel_epoch,
                vm_id_fragment: sender_vm_id_fragment,
                accumulated_amount: sub_accumulated_amount,
                nonce: sub_nonce,
            };

            assert!(
            verify_sender_signature(
                channel,
                sub_rav,
                sender_signature
                ),
                ErrorInvalidSenderSignature
            );
        };
        
        // Get the sub-channel state.
        let sub_channel = table::borrow_mut(&mut channel.sub_channels, sender_vm_id_fragment);
        
        // Validate amount and nonce are >= (allow equal amounts for idempotent calls)
        assert!(sub_accumulated_amount >= sub_channel.last_claimed_amount, ErrorInvalidAmount);
        assert!(sub_nonce >= sub_channel.last_confirmed_nonce, ErrorInvalidNonce);
        
        let incremental_amount = sub_accumulated_amount - sub_channel.last_claimed_amount;
        
        // Update the sub-channel state on-chain.
        sub_channel.last_claimed_amount = sub_accumulated_amount;
        sub_channel.last_confirmed_nonce = sub_nonce;
        
        // Only transfer funds if there's an incremental amount
        if (incremental_amount > 0) {
            // Withdraw funds from the payment hub and transfer to the receiver.
            let hub_obj = borrow_or_create_payment_hub(channel.sender);
            let hub = object::borrow_mut(hub_obj);
            let generic_payment = multi_coin_store::withdraw(&mut hub.multi_coin_store, channel.coin_type, incremental_amount);

            // Deposit the coin directly into the receiver's payment hub
            deposit_to_hub_generic(channel.receiver, generic_payment);
        };
        
        // Emit claim event
        event::emit(ChannelClaimedEvent {
            channel_id,
            receiver: channel.receiver,
            vm_id_fragment: sender_vm_id_fragment,
            amount: incremental_amount,
            sub_accumulated_amount,
            sub_nonce,
        });
    }

    /// Entry function for claiming from channel
    public entry fun claim_from_channel_entry(
        claimer: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    ) {
        claim_from_channel(
            claimer,
            channel_id,
            sender_vm_id_fragment,
            sub_accumulated_amount,
            sub_nonce,
            sender_signature
        );
    } 

    /// Close the entire channel with final settlement of all sub-channels
    /// Called by receiver with proofs of final state from all sub-channels
    public fun close_channel(
        channel_receiver: &signer,
        channel_id: ObjectID,
        proofs: vector<CloseProof>,
    ) {
        let receiver_addr = signer::address_of(channel_receiver);
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel>(channel_id);
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
            let sub_rav = SubRAV {
                version: SUB_RAV_VERSION_V1,
                chain_id: chain_id::chain_id(),
                channel_id,
                channel_epoch: channel.channel_epoch,
                vm_id_fragment,
                accumulated_amount: proof.accumulated_amount,
                nonce: proof.nonce,
            };

            assert!(
                verify_sender_signature(
                    channel,
                    sub_rav,
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
            let coin_type_name = channel.coin_type;
            let generic_payment = multi_coin_store::withdraw(&mut hub.multi_coin_store, coin_type_name, total_incremental_amount);
            deposit_to_hub_generic(channel.receiver, generic_payment);
        };
        
        // Mark channel as closed and increment epoch
        channel.status = STATUS_CLOSED;
        channel.channel_epoch = channel.channel_epoch + 1;
        
        // Note: We don't need to clear sub-channels table since channel_epoch increment
        // will invalidate all old signatures. The table will be preserved for reactivation.
        
        // Decrease active channel count
        decrease_active_channel_count(channel.sender, channel.coin_type);
        
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
    public entry fun close_channel_entry(
        channel_receiver: &signer,
        channel_id: ObjectID,
        serialized_proofs: vector<u8>,
    ) {
        let proofs = bcs::from_bytes<CloseProofs>(serialized_proofs);
        close_channel(channel_receiver, channel_id, proofs.proofs);
    } 

    /// Entry function for initiating cancellation
    public entry fun initiate_cancellation_entry(
        channel_sender: &signer,
        channel_id: ObjectID,
    ) {
        initiate_cancellation(channel_sender, channel_id, vector::empty());
    }

    /// Sender initiates unilateral channel cancellation with proofs for sub-channels
    public fun initiate_cancellation(
        channel_sender: &signer,
        channel_id: ObjectID,
        proofs: vector<CancelProof>,
    ) {
        let sender_addr = signer::address_of(channel_sender);
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel>(channel_id);
        let channel = object::borrow_mut(channel_obj);
        
        // Verify sender is the channel sender and channel is active
        assert!(channel.sender == sender_addr, ErrorNotSender);
        assert!(channel.status == STATUS_ACTIVE, ErrorChannelNotActive);
        
        // Get sub-channels count for processing
        let sub_channels_count = table::length(&channel.sub_channels);
        
        // If no sub-channels exist, close the channel immediately
        if (sub_channels_count == 0) {
            // No active sub-channels means no pending amounts or disputes possible
            channel.status = STATUS_CLOSED;
            channel.channel_epoch = channel.channel_epoch + 1;
            
            // Note: We don't need to clear sub-channels table since channel_epoch increment
            // will invalidate all old signatures. The table will be preserved for reactivation.
            
            // Decrease active channel count
            decrease_active_channel_count(sender_addr, channel.coin_type);
            
            // Emit immediate closure event (no funds to transfer)
            event::emit(ChannelCancellationFinalizedEvent {
                channel_id,
                sender: sender_addr,
                final_amount: 0u256,
            });
            
            return
        };
        
        // Process sub-channels that require challenge period
        let total_pending_amount = 0u256;
        let i = 0;
        let proofs_len = vector::length(&proofs);
        
        while (i < proofs_len) {
            let proof = vector::borrow(&proofs, i);
            let vm_id_fragment = proof.vm_id_fragment;
            
            // Get the existing sub-channel state (must exist)
            assert!(table::contains(&channel.sub_channels, vm_id_fragment), ErrorSubChannelNotAuthorized);
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
        
        // Set channel to cancelling state (requires challenge period)
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
    public entry fun initiate_cancellation_with_proofs_entry(
        channel_sender: &signer,
        channel_id: ObjectID,
        serialized_proofs: vector<u8>,
    ) {
        let proofs = bcs::from_bytes<CancelProofs>(serialized_proofs);
        initiate_cancellation(channel_sender, channel_id, proofs.proofs);
    }

    /// Receiver disputes cancellation with newer state
    public fun dispute_cancellation(
        channel_receiver: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        dispute_accumulated_amount: u256,
        dispute_nonce: u64,
        sender_signature: vector<u8>
    ) {
        internal_dispute_cancellation(
            channel_receiver,
            channel_id,
            sender_vm_id_fragment,
            dispute_accumulated_amount,
            dispute_nonce,
            sender_signature,
            false
        );
    }

    fun internal_dispute_cancellation(
        channel_receiver: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        dispute_accumulated_amount: u256,
        dispute_nonce: u64,
        sender_signature: vector<u8>,
        skip_signature_verification: bool
    ) {
        let receiver = signer::address_of(channel_receiver);
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel>(channel_id);
        let channel = object::borrow_mut(channel_obj);
        
        // Verify receiver and channel state
        assert!(channel.receiver == receiver, ErrorNotReceiver);
        assert!(channel.status == STATUS_CANCELLING, ErrorChannelNotActive);
        
        // Verify the sub-channel has been authorized
        assert!(table::contains(&channel.sub_channels, sender_vm_id_fragment), ErrorSubChannelNotAuthorized);
        
        if (!skip_signature_verification) {
            let sub_rav = SubRAV {
                version: SUB_RAV_VERSION_V1,
                chain_id: chain_id::chain_id(),
                channel_id,
                channel_epoch: channel.channel_epoch,
                vm_id_fragment: sender_vm_id_fragment,
                accumulated_amount: dispute_accumulated_amount,
                nonce: dispute_nonce,
            };

            // Verify signature
            assert!(
                verify_sender_signature(
                    channel,
                    sub_rav,
                    sender_signature
                    ),
                ErrorInvalidSenderSignature
            );
        };
        
        // Get the sub-channel state
        let sub_channel = table::borrow_mut(&mut channel.sub_channels, sender_vm_id_fragment);
        
        // Validate dispute amount and nonce
        assert!(dispute_accumulated_amount >= sub_channel.last_claimed_amount, ErrorInvalidAmount);
        assert!(dispute_nonce >= sub_channel.last_confirmed_nonce, ErrorInvalidNonce);
        
        // Calculate the additional incremental amount from this dispute
        let old_claimed_amount = sub_channel.last_claimed_amount;
        let new_claimed_amount = dispute_accumulated_amount;
        let additional_amount = new_claimed_amount - old_claimed_amount;
        
        // Update the pending amount if this dispute increases the total
        if (additional_amount > 0) {
            let cancellation_info = option::borrow_mut(&mut channel.cancellation_info);
            cancellation_info.pending_amount = cancellation_info.pending_amount + additional_amount;
            
            // Update the sub-channel state to reflect this dispute
            sub_channel.last_claimed_amount = dispute_accumulated_amount;
            sub_channel.last_confirmed_nonce = dispute_nonce;
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
    public entry fun dispute_cancellation_entry(
        channel_receiver: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        dispute_accumulated_amount: u256,
        dispute_nonce: u64,
        sender_signature: vector<u8>
    ) {
        dispute_cancellation(
            channel_receiver,
            channel_id,
            sender_vm_id_fragment,
            dispute_accumulated_amount,
            dispute_nonce,
            sender_signature
        );
    }

    /// Finalize cancellation after challenge period
    public fun finalize_cancellation(
        channel_id: ObjectID,
    ) {
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel>(channel_id);
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
            let coin_type_name = channel.coin_type;
            let generic_payment = multi_coin_store::withdraw(&mut hub.multi_coin_store, coin_type_name, final_amount);
            deposit_to_hub_generic(channel.receiver, generic_payment);
        };
        
        // Mark channel as closed and increment epoch
        channel.status = STATUS_CLOSED;
        channel.channel_epoch = channel.channel_epoch + 1;
        
        // Note: We don't need to clear sub-channels table since channel_epoch increment
        // will invalidate all old signatures. The table will be preserved for reactivation.
        
        // Decrease active channel count
        decrease_active_channel_count(channel.sender, channel.coin_type);
        
        // Emit finalization event
        event::emit(ChannelCancellationFinalizedEvent {
            channel_id,
            sender: channel.sender,
            final_amount,
        });
    }

    /// Entry function for finalizing cancellation
    public entry fun finalize_cancellation_entry(
        channel_id: ObjectID,
    ) {
        finalize_cancellation(channel_id);
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

    /// Check if a payment channel exists between sender and receiver for the given coin type
    public fun channel_exists(sender: address, receiver: address, coin_type: String): bool {
        let channel_id = calc_channel_object_id(sender, receiver, coin_type);
        object::exists_object_with_type<PaymentChannel>(channel_id)
    }

    /// Get channel ID for a given sender, receiver, and coin type
    public fun get_channel_id(sender: address, receiver: address, coin_type: String): ObjectID {
        calc_channel_object_id(sender, receiver, coin_type)
    }

    /// Get channel information
    public fun get_channel_info(channel_id: ObjectID): (address, address, String, u8) {
        let channel_obj = object::borrow_object<PaymentChannel>(channel_id);
        let channel = object::borrow(channel_obj);
        (channel.sender, channel.receiver, channel.coin_type, channel.status)
    }

    /// Get channel epoch
    public fun get_channel_epoch(channel_id: ObjectID): u64 {
        let channel_obj = object::borrow_object<PaymentChannel>(channel_id);
        let channel = object::borrow(channel_obj);
        channel.channel_epoch
    }

    /// Get sub-channel state
    public fun get_sub_channel_state(channel_id: ObjectID, vm_id_fragment: String): (u256, u64) {
        let channel_obj = object::borrow_object<PaymentChannel>(channel_id);
        let channel = object::borrow(channel_obj);
        
        if (table::contains(&channel.sub_channels, vm_id_fragment)) {
            let state = table::borrow(&channel.sub_channels, vm_id_fragment);
            (state.last_claimed_amount, state.last_confirmed_nonce)
        } else {
            (0u256, 0u64)
        }
    }

    /// Check if a sub-channel exists
    public fun sub_channel_exists(channel_id: ObjectID, vm_id_fragment: String): bool {
        let channel_obj = object::borrow_object<PaymentChannel>(channel_id);
        let channel = object::borrow(channel_obj);
        table::contains(&channel.sub_channels, vm_id_fragment)
    }

    /// Get the number of sub-channels in a payment channel
    public fun get_sub_channel_count(channel_id: ObjectID): u64 {
        let channel_obj = object::borrow_object<PaymentChannel>(channel_id);
        let channel = object::borrow(channel_obj);
        table::length(&channel.sub_channels)
    }

    /// Get cancellation info
    public fun get_cancellation_info(channel_id: ObjectID): Option<CancellationInfo> {
        let channel_obj = object::borrow_object<PaymentChannel>(channel_id);
        let channel = object::borrow(channel_obj);
        
        if (option::is_some(&channel.cancellation_info)) {
            let info = option::borrow(&channel.cancellation_info);
            option::some(*info)
        } else {
            option::none()
        }
    }



    /// Get sub-channel public key multibase if exists
    public fun get_sub_channel_public_key(channel_id: ObjectID, vm_id_fragment: String): Option<String> {
        let channel_obj = object::borrow_object<PaymentChannel>(channel_id);
        let channel = object::borrow(channel_obj);
        
        if (table::contains(&channel.sub_channels, vm_id_fragment)) {
            let sub_channel = table::borrow(&channel.sub_channels, vm_id_fragment);
            option::some(sub_channel.pk_multibase)
        } else {
            option::none()
        }
    }

    /// Get sub-channel method type if exists
    public fun get_sub_channel_method_type(channel_id: ObjectID, vm_id_fragment: String): Option<String> {
        let channel_obj = object::borrow_object<PaymentChannel>(channel_id);
        let channel = object::borrow(channel_obj);
        
        if (table::contains(&channel.sub_channels, vm_id_fragment)) {
            let sub_channel = table::borrow(&channel.sub_channels, vm_id_fragment);
            option::some(sub_channel.method_type)
        } else {
            option::none()
        }
    }

    /// Get the number of active channels for a specific coin type
    public fun get_active_channel_count(owner: address, coin_type: String): u64 {
        let hub_id = get_payment_hub_id(owner);
        if (!object::exists_object_with_type<PaymentHub>(hub_id)) {
            return 0
        };
        
        let hub_obj = object::borrow_object<PaymentHub>(hub_id);
        let hub = object::borrow(hub_obj);
        
        if (table::contains(&hub.active_channels, coin_type)) {
            *table::borrow(&hub.active_channels, coin_type)
        } else {
            0
        }
    }

    /// Check if withdrawal is allowed for a specific coin type
    public fun can_withdraw_from_hub(owner: address, coin_type: String): bool {
        get_active_channel_count(owner, coin_type) == 0
    }

    // === Internal Helper Functions ===

    /// Decrease active channel count for a specific coin type
    fun decrease_active_channel_count(sender_addr: address, coin_type_name: String) {
        let payment_hub_obj = borrow_or_create_payment_hub(sender_addr);
        let payment_hub = object::borrow_mut(payment_hub_obj);
        
        if (table::contains(&payment_hub.active_channels, coin_type_name)) {
            let count = table::borrow_mut(&mut payment_hub.active_channels, coin_type_name);
            *count = *count - 1;
            
            // If count reaches zero, remove the entry to save gas
            if (*count == 0) {
                table::remove(&mut payment_hub.active_channels, coin_type_name);
            };
        };
    }

    fun verify_sender_signature(
        channel: &PaymentChannel,
        sub_rav: SubRAV,
        signature: vector<u8>
    ): bool {
        // First validate the version
        if (sub_rav.version != SUB_RAV_VERSION_V1) {
            return false
        };

        // Verify chain_id matches current chain
        if (sub_rav.chain_id != chain_id::chain_id()) {
            return false
        };
        
        // Verify channel epoch matches
        if (sub_rav.channel_epoch != channel.channel_epoch) {
            return false
        };
        
        // Get the sub-channel to access stored public key information
        let sub_channel = table::borrow(&channel.sub_channels, sub_rav.vm_id_fragment);
        
        verify_rav_signature(sub_rav, signature, sub_channel.pk_multibase, sub_channel.method_type)
    }

    fun verify_rav_signature(
        sub_rav: SubRAV,
        signature: vector<u8>,
        pk_multibase: String,
        method_type: String
    ): bool {
        // First validate the version
        assert!(sub_rav.version == SUB_RAV_VERSION_V1, ErrorUnsupportedVersion);
        
        let msg = bcs::to_bytes(&sub_rav);
        // Verify signature using the stored public key and method type
        did::verify_signature_by_type(msg, signature, &pk_multibase, &method_type)
    }

    #[test_only]
    /// Test-only version of claim_from_channel that uses the test signature verification
    public fun claim_from_channel_for_test(
        claimer: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    ) {
        internal_claim_from_channel(
            claimer,
            channel_id,
            sender_vm_id_fragment,
            sub_accumulated_amount,
            sub_nonce,
            sender_signature,
            true
        );
    }

    #[test_only]
    public fun dispute_cancellation_for_test(
        channel_receiver: &signer,
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        dispute_accumulated_amount: u256,
        dispute_nonce: u64,
        sender_signature: vector<u8>
    ) {
        internal_dispute_cancellation(
            channel_receiver,
            channel_id,
            sender_vm_id_fragment,
            dispute_accumulated_amount,
            dispute_nonce,
            sender_signature,
            true
        );
    }

    #[test]
    fun test_sub_rav_hash() {
        let sub_rav = SubRAV {
            version: SUB_RAV_VERSION_V1,
            chain_id: 4, // CHAIN_ID_LOCAL for test
            channel_id: object::from_string(&std::string::utf8(b"0x35df6e58502089ed640382c477e4b6f99e5e90d881678d37ed774a737fd3797c")),
            channel_epoch: 0,
            vm_id_fragment: std::string::utf8(b"account-key"),
            accumulated_amount: 10000,
            nonce: 1,
        };
        assert!(sub_rav.channel_id == object::from_string(&std::string::utf8(b"0x35df6e58502089ed640382c477e4b6f99e5e90d881678d37ed774a737fd3797c")), 1);
        assert!(sub_rav.channel_epoch == 0, 2);
        assert!(sub_rav.vm_id_fragment == std::string::utf8(b"account-key"), 3);
        assert!(sub_rav.accumulated_amount == 10000, 4);
        assert!(sub_rav.nonce == 1, 5);
    }

    #[test]
    fun test_sub_rav_signature() {
        let sub_rav = SubRAV {
            version: SUB_RAV_VERSION_V1,
            chain_id: 4, // CHAIN_ID_LOCAL for test
            channel_id: object::from_string(&std::string::utf8(b"0x35df6e58502089ed640382c477e4b6f99e5e90d881678d37ed774a737fd3797c")),
            channel_epoch: 0,
            vm_id_fragment: std::string::utf8(b"account-key"),
            accumulated_amount: 10000,
            nonce: 1,
        };
        let signature = x"178a4171000c67db0be16cef70ae0ba4d43e779a1fa25ee901dd2683ccc8966a7c6b2c2b95b17a0fd77db5ee3099c5d660f4e9811f7257824a731f9eb269d360";
        let pk_multibase = std::string::utf8(b"zwvRask8Xx7oi3Aw6PvvmmBvdYbHqsJPkvCZYxDFZMwZa");
        let method_type = std::string::utf8(b"EcdsaSecp256k1VerificationKey2019");
        assert!(verify_rav_signature(sub_rav, signature, pk_multibase, method_type), 3);    
    }

    // === Generic Coin Functions for Payment Hub ===

    /// Get balance of specific coin type in payment hub
    public fun get_balance_in_hub<CoinType: key>(owner: address): u256 {
        if (!payment_hub_exists(owner)) {
            return 0u256
        };
        
        let hub_id = get_payment_hub_id(owner);
        let hub_obj = object::borrow_object<PaymentHub>(hub_id);
        let hub = object::borrow(hub_obj);
        let coin_type = type_info::type_name<CoinType>();
        multi_coin_store::balance(&hub.multi_coin_store, coin_type)
    }


    /// Internal function to withdraw specific coin type from payment hub 
    ///(no signer required and does not check for active channels)
    /// Used by system contracts like transaction_gas module
    public(friend) fun withdraw_from_hub_internal<CoinType: key>(addr: address, amount: u256): Coin<CoinType> {
        let hub_obj = borrow_or_create_payment_hub(addr);
        let hub = object::borrow_mut(hub_obj);
        let coin_type = type_info::type_name<CoinType>();
        let generic_coin = multi_coin_store::withdraw(&mut hub.multi_coin_store, coin_type, amount);
        coin::convert_generic_coin_to_coin<CoinType>(generic_coin)
    }
}
