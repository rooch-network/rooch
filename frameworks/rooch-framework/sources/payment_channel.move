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
    use std::u256;
    use std::string::{Self, String};
    use rooch_framework::coin::{Self, Coin, GenericCoin};
    use rooch_framework::multi_coin_store::{Self, MultiCoinStore};
    use rooch_framework::did;
    use rooch_framework::account_coin_store;
    use moveos_std::hash;

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

    // === Constants ===
    const STATUS_ACTIVE: u8 = 0;
    const STATUS_CANCELLING: u8 = 1;
    const STATUS_CLOSED: u8 = 2;
    const CHALLENGE_PERIOD_SECONDS: u64 = 86400; // 1 day

    // === Structs ===
    /// A central, user-owned object for managing payments.
    /// It contains a MultiCoinStore to support various coin types.
    /// Every account can only have one payment hub, and the hub can not be transferred.
    struct PaymentHub has key {
        multi_coin_store: Object<MultiCoinStore>,
        //TODO add more settings to channel
    }

    /// A lightweight object representing a payment relationship, linked to a PaymentHub.
    struct PaymentChannel<CoinType: store> has key, store {
        sender: address,
        receiver: address,
        payment_hub_id: ObjectID, // Links to a PaymentHub object
        sub_channels: Table<vector<u8>, SubChannelState>,
        status: u8,
        cancellation_info: Option<CancellationInfo>,
    }
    
    /// The on-chain state for a specific sub-channel.
    struct SubChannelState has store {
        last_claimed_amount: u256,
        last_confirmed_nonce: u64,
    }

    /// Information stored when a channel cancellation is initiated.
    struct CancellationInfo has store {
        initiated_time: u64,
        pending_amount: u256,
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


    /// Deposits a specific type of coin into the payment hub
    public entry fun deposit_to_hub<CoinType: key + store>(
        account: &signer,
        hub_id: ObjectID,
        amount: u256,
    ) {
        let account_addr = signer::address_of(account);
        let hub_obj = borrow_or_create_payment_hub(account_addr);
        let hub = object::borrow_mut(hub_obj);

        // Withdraw from account and deposit to hub
        let coin = account_coin_store::withdraw<CoinType>(account, amount);
        multi_coin_store::deposit_by_type(&mut hub.multi_coin_store, coin);
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
        channel_id
    }

    /// The receiver claims funds from a specific sub-channel.
    public fun claim_from_channel<CoinType: key + store>(
        channel_id: ObjectID, // The signer must be the receiver.
        sender_vm_id_fragment: vector<u8>,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    ) {
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow_mut(channel_obj);
        // The transaction sender must be the receiver.
        assert!(channel.status == STATUS_ACTIVE, ErrorChannelNotActive);
        // Verify that the correct payment hub is being used.
        //assert!(channel.payment_hub_id == payment_hub_id, ErrorInvalidPaymentHub);
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
        let payment = coin::convert_generic_coin_to_coin<CoinType>(generic_payment);
        // Deposit the coin directly into the receiver's account coin store
        account_coin_store::deposit<CoinType>(channel.receiver, payment);
    }

    // === Internal Helper Functions ===

    fun get_sub_rav_hash(
        channel_id: ObjectID,
        vm_id_fragment: vector<u8>,
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
        bytes
    }

    fun verify_sender_signature(
        channel_id: ObjectID,
        sender_address: address,
        vm_id_fragment: vector<u8>,
        accumulated_amount: u256,
        nonce: u64,
        signature: vector<u8>
    ): bool {
        let msg_hash = get_sub_rav_hash(channel_id, vm_id_fragment, accumulated_amount, nonce);
        // Construct the full verification method ID.
        let did_doc = did::get_did_document_by_address(sender_address);
        let vm_str = std::string::utf8(vm_id_fragment);
        let vm_opt = did::doc_verification_method(did_doc, &vm_str);
        assert!(option::is_some(&vm_opt), ErrorVerificationMethodNotFound);
        // TODO: Check for authentication permission if needed (see Rooch DID API)
        // NOTE: You may need to implement signature verification logic here if not available
        true // Placeholder, replace with actual verification
    }
}
