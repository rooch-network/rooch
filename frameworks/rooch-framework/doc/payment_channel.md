
<a name="0x3_payment_channel"></a>

# Module `0x3::payment_channel`



-  [Struct `PaymentHubCreatedEvent`](#0x3_payment_channel_PaymentHubCreatedEvent)
-  [Struct `PaymentChannelOpenedEvent`](#0x3_payment_channel_PaymentChannelOpenedEvent)
-  [Struct `ChannelClaimedEvent`](#0x3_payment_channel_ChannelClaimedEvent)
-  [Struct `ChannelClosedEvent`](#0x3_payment_channel_ChannelClosedEvent)
-  [Struct `ChannelCancellationInitiatedEvent`](#0x3_payment_channel_ChannelCancellationInitiatedEvent)
-  [Struct `ChannelDisputeEvent`](#0x3_payment_channel_ChannelDisputeEvent)
-  [Struct `ChannelCancellationFinalizedEvent`](#0x3_payment_channel_ChannelCancellationFinalizedEvent)
-  [Struct `SubChannelAuthorizedEvent`](#0x3_payment_channel_SubChannelAuthorizedEvent)
-  [Struct `PaymentHubWithdrawEvent`](#0x3_payment_channel_PaymentHubWithdrawEvent)
-  [Struct `ChannelKey`](#0x3_payment_channel_ChannelKey)
-  [Resource `PaymentHub`](#0x3_payment_channel_PaymentHub)
-  [Resource `PaymentChannel`](#0x3_payment_channel_PaymentChannel)
-  [Struct `SubChannel`](#0x3_payment_channel_SubChannel)
-  [Struct `CancellationInfo`](#0x3_payment_channel_CancellationInfo)
-  [Struct `CloseProof`](#0x3_payment_channel_CloseProof)
-  [Struct `CloseProofs`](#0x3_payment_channel_CloseProofs)
-  [Struct `CancelProof`](#0x3_payment_channel_CancelProof)
-  [Struct `CancelProofs`](#0x3_payment_channel_CancelProofs)
-  [Struct `SubRAV`](#0x3_payment_channel_SubRAV)
-  [Constants](#@Constants_0)
-  [Function `calc_channel_object_id`](#0x3_payment_channel_calc_channel_object_id)
-  [Function `borrow_or_create_payment_hub`](#0x3_payment_channel_borrow_or_create_payment_hub)
-  [Function `ensure_payment_hub_exists`](#0x3_payment_channel_ensure_payment_hub_exists)
-  [Function `create_payment_hub`](#0x3_payment_channel_create_payment_hub)
-  [Function `deposit_to_hub_entry`](#0x3_payment_channel_deposit_to_hub_entry)
-  [Function `deposit_to_hub`](#0x3_payment_channel_deposit_to_hub)
-  [Function `deposit_to_hub_generic`](#0x3_payment_channel_deposit_to_hub_generic)
-  [Function `withdraw_from_hub`](#0x3_payment_channel_withdraw_from_hub)
-  [Function `withdraw_from_hub_entry`](#0x3_payment_channel_withdraw_from_hub_entry)
-  [Function `open_channel`](#0x3_payment_channel_open_channel)
-  [Function `open_channel_entry`](#0x3_payment_channel_open_channel_entry)
-  [Function `authorize_sub_channel`](#0x3_payment_channel_authorize_sub_channel)
-  [Function `authorize_sub_channel_entry`](#0x3_payment_channel_authorize_sub_channel_entry)
-  [Function `open_channel_with_sub_channel`](#0x3_payment_channel_open_channel_with_sub_channel)
-  [Function `open_channel_with_sub_channel_entry`](#0x3_payment_channel_open_channel_with_sub_channel_entry)
-  [Function `claim_from_channel`](#0x3_payment_channel_claim_from_channel)
-  [Function `claim_from_channel_entry`](#0x3_payment_channel_claim_from_channel_entry)
-  [Function `close_channel`](#0x3_payment_channel_close_channel)
-  [Function `close_channel_entry`](#0x3_payment_channel_close_channel_entry)
-  [Function `initiate_cancellation_entry`](#0x3_payment_channel_initiate_cancellation_entry)
-  [Function `initiate_cancellation`](#0x3_payment_channel_initiate_cancellation)
-  [Function `initiate_cancellation_with_proofs_entry`](#0x3_payment_channel_initiate_cancellation_with_proofs_entry)
-  [Function `dispute_cancellation`](#0x3_payment_channel_dispute_cancellation)
-  [Function `dispute_cancellation_entry`](#0x3_payment_channel_dispute_cancellation_entry)
-  [Function `finalize_cancellation`](#0x3_payment_channel_finalize_cancellation)
-  [Function `finalize_cancellation_entry`](#0x3_payment_channel_finalize_cancellation_entry)
-  [Function `get_payment_hub_id`](#0x3_payment_channel_get_payment_hub_id)
-  [Function `payment_hub_exists`](#0x3_payment_channel_payment_hub_exists)
-  [Function `channel_exists`](#0x3_payment_channel_channel_exists)
-  [Function `get_channel_id`](#0x3_payment_channel_get_channel_id)
-  [Function `get_channel_info`](#0x3_payment_channel_get_channel_info)
-  [Function `get_channel_epoch`](#0x3_payment_channel_get_channel_epoch)
-  [Function `get_sub_channel_state`](#0x3_payment_channel_get_sub_channel_state)
-  [Function `sub_channel_exists`](#0x3_payment_channel_sub_channel_exists)
-  [Function `get_sub_channel_count`](#0x3_payment_channel_get_sub_channel_count)
-  [Function `get_cancellation_info`](#0x3_payment_channel_get_cancellation_info)
-  [Function `get_sub_channel_public_key`](#0x3_payment_channel_get_sub_channel_public_key)
-  [Function `get_sub_channel_method_type`](#0x3_payment_channel_get_sub_channel_method_type)
-  [Function `get_active_channel_count`](#0x3_payment_channel_get_active_channel_count)
-  [Function `can_withdraw_from_hub`](#0x3_payment_channel_can_withdraw_from_hub)
-  [Function `get_balance_in_hub`](#0x3_payment_channel_get_balance_in_hub)
-  [Function `withdraw_from_hub_internal`](#0x3_payment_channel_withdraw_from_hub_internal)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::timestamp</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="account_coin_store.md#0x3_account_coin_store">0x3::account_coin_store</a>;
<b>use</b> <a href="chain_id.md#0x3_chain_id">0x3::chain_id</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="did.md#0x3_did">0x3::did</a>;
<b>use</b> <a href="multi_coin_store.md#0x3_multi_coin_store">0x3::multi_coin_store</a>;
<b>use</b> <a href="payment_revenue.md#0x3_payment_revenue">0x3::payment_revenue</a>;
</code></pre>



<a name="0x3_payment_channel_PaymentHubCreatedEvent"></a>

## Struct `PaymentHubCreatedEvent`

Event emitted when a payment hub is created


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_PaymentHubCreatedEvent">PaymentHubCreatedEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_payment_channel_PaymentChannelOpenedEvent"></a>

## Struct `PaymentChannelOpenedEvent`

Event emitted when a payment channel is opened


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_PaymentChannelOpenedEvent">PaymentChannelOpenedEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_payment_channel_ChannelClaimedEvent"></a>

## Struct `ChannelClaimedEvent`

Event emitted when funds are claimed from a channel


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_ChannelClaimedEvent">ChannelClaimedEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_payment_channel_ChannelClosedEvent"></a>

## Struct `ChannelClosedEvent`

Event emitted when a channel is closed


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_ChannelClosedEvent">ChannelClosedEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_payment_channel_ChannelCancellationInitiatedEvent"></a>

## Struct `ChannelCancellationInitiatedEvent`

Event emitted when channel cancellation is initiated


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_ChannelCancellationInitiatedEvent">ChannelCancellationInitiatedEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_payment_channel_ChannelDisputeEvent"></a>

## Struct `ChannelDisputeEvent`

Event emitted when a dispute is raised during cancellation


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_ChannelDisputeEvent">ChannelDisputeEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_payment_channel_ChannelCancellationFinalizedEvent"></a>

## Struct `ChannelCancellationFinalizedEvent`

Event emitted when channel cancellation is finalized


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_ChannelCancellationFinalizedEvent">ChannelCancellationFinalizedEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_payment_channel_SubChannelAuthorizedEvent"></a>

## Struct `SubChannelAuthorizedEvent`

Event emitted when a sub-channel is authorized


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_SubChannelAuthorizedEvent">SubChannelAuthorizedEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_payment_channel_PaymentHubWithdrawEvent"></a>

## Struct `PaymentHubWithdrawEvent`

Event emitted when funds are withdrawn from a payment hub


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_PaymentHubWithdrawEvent">PaymentHubWithdrawEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_payment_channel_ChannelKey"></a>

## Struct `ChannelKey`

Unique key for identifying a unidirectional payment channel
Used to generate deterministic ObjectID for channels


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_ChannelKey">ChannelKey</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_payment_channel_PaymentHub"></a>

## Resource `PaymentHub`

A central, user-owned object for managing payments.
It contains a MultiCoinStore to support various coin types.
Every account can only have one payment hub, and the hub can not be transferred.


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_PaymentHub">PaymentHub</a> <b>has</b> key
</code></pre>



<a name="0x3_payment_channel_PaymentChannel"></a>

## Resource `PaymentChannel`

A lightweight object representing a payment relationship, linked to a PaymentHub.
The PaymentChannel has no store, it can not be transferred.


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_PaymentChannel">PaymentChannel</a> <b>has</b> key
</code></pre>



<a name="0x3_payment_channel_SubChannel"></a>

## Struct `SubChannel`

The on-chain state for a specific sub-channel, including authorization metadata.


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_SubChannel">SubChannel</a> <b>has</b> store
</code></pre>



<a name="0x3_payment_channel_CancellationInfo"></a>

## Struct `CancellationInfo`

Information stored when a channel cancellation is initiated.


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_CancellationInfo">CancellationInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_payment_channel_CloseProof"></a>

## Struct `CloseProof`

Proof for closing a sub-channel with final state


<pre><code>#[data_struct]
<b>struct</b> <a href="payment_channel.md#0x3_payment_channel_CloseProof">CloseProof</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_payment_channel_CloseProofs"></a>

## Struct `CloseProofs`



<pre><code>#[data_struct]
<b>struct</b> <a href="payment_channel.md#0x3_payment_channel_CloseProofs">CloseProofs</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_payment_channel_CancelProof"></a>

## Struct `CancelProof`

Proof for initiating cancellation of a sub-channel (no signature needed from sender)


<pre><code>#[data_struct]
<b>struct</b> <a href="payment_channel.md#0x3_payment_channel_CancelProof">CancelProof</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_payment_channel_CancelProofs"></a>

## Struct `CancelProofs`



<pre><code>#[data_struct]
<b>struct</b> <a href="payment_channel.md#0x3_payment_channel_CancelProofs">CancelProofs</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_payment_channel_SubRAV"></a>

## Struct `SubRAV`

Structure representing a Sub-RAV (Sub-channel Receipts and Vouchers) for off-chain signature verification


<pre><code>#[data_struct]
<b>struct</b> <a href="payment_channel.md#0x3_payment_channel_SubRAV">SubRAV</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_payment_channel_ErrorInsufficientBalance"></a>

Insufficient balance in the payment hub.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorInsufficientBalance">ErrorInsufficientBalance</a>: u64 = 13;
</code></pre>



<a name="0x3_payment_channel_ErrorInsufficientPermission"></a>

The Verification Method used does not have 'authentication' permission.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorInsufficientPermission">ErrorInsufficientPermission</a>: u64 = 5;
</code></pre>



<a name="0x3_payment_channel_ErrorVerificationMethodAlreadyExists"></a>

The verification method already exists for this sub-channel.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorVerificationMethodAlreadyExists">ErrorVerificationMethodAlreadyExists</a>: u64 = 17;
</code></pre>



<a name="0x3_payment_channel_ErrorVerificationMethodNotFound"></a>

The specified Verification Method was not found in the sender's DID.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorVerificationMethodNotFound">ErrorVerificationMethodNotFound</a>: u64 = 4;
</code></pre>



<a name="0x3_payment_channel_CHALLENGE_PERIOD_MILLISECONDS"></a>



<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_CHALLENGE_PERIOD_MILLISECONDS">CHALLENGE_PERIOD_MILLISECONDS</a>: u64 = 86400000;
</code></pre>



<a name="0x3_payment_channel_ErrorActiveChannelExists"></a>

There are still active channels for this coin type.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorActiveChannelExists">ErrorActiveChannelExists</a>: u64 = 19;
</code></pre>



<a name="0x3_payment_channel_ErrorChallengePeriodNotElapsed"></a>

The challenge period has not elapsed yet.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorChallengePeriodNotElapsed">ErrorChallengePeriodNotElapsed</a>: u64 = 10;
</code></pre>



<a name="0x3_payment_channel_ErrorChannelAlreadyCancelling"></a>

The channel is already in cancelling state.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorChannelAlreadyCancelling">ErrorChannelAlreadyCancelling</a>: u64 = 11;
</code></pre>



<a name="0x3_payment_channel_ErrorChannelAlreadyClosed"></a>

The channel is already closed.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorChannelAlreadyClosed">ErrorChannelAlreadyClosed</a>: u64 = 12;
</code></pre>



<a name="0x3_payment_channel_ErrorChannelAlreadyExists"></a>

A channel between this sender and receiver already exists for this coin type.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorChannelAlreadyExists">ErrorChannelAlreadyExists</a>: u64 = 18;
</code></pre>



<a name="0x3_payment_channel_ErrorChannelNotActive"></a>

The channel is not in an active state.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorChannelNotActive">ErrorChannelNotActive</a>: u64 = 2;
</code></pre>



<a name="0x3_payment_channel_ErrorHubOwnerMismatch"></a>

The owner of the payment hub does not match the sender of the channel.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorHubOwnerMismatch">ErrorHubOwnerMismatch</a>: u64 = 9;
</code></pre>



<a name="0x3_payment_channel_ErrorInvalidAmount"></a>

The claimed amount is less than or equal to the already claimed amount.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorInvalidAmount">ErrorInvalidAmount</a>: u64 = 8;
</code></pre>



<a name="0x3_payment_channel_ErrorInvalidChainId"></a>

The chain_id in the SubRAV does not match the current chain_id.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorInvalidChainId">ErrorInvalidChainId</a>: u64 = 23;
</code></pre>



<a name="0x3_payment_channel_ErrorInvalidChannelEpoch"></a>

The channel epoch in the SubRAV does not match the current channel epoch.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorInvalidChannelEpoch">ErrorInvalidChannelEpoch</a>: u64 = 22;
</code></pre>



<a name="0x3_payment_channel_ErrorInvalidNonce"></a>

The nonce for the sub-channel is not greater than the last confirmed nonce.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorInvalidNonce">ErrorInvalidNonce</a>: u64 = 7;
</code></pre>



<a name="0x3_payment_channel_ErrorInvalidPaymentHub"></a>

The provided payment hub object does not match the one linked in the channel.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorInvalidPaymentHub">ErrorInvalidPaymentHub</a>: u64 = 6;
</code></pre>



<a name="0x3_payment_channel_ErrorInvalidSenderSignature"></a>

The provided signature from the sender is invalid.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorInvalidSenderSignature">ErrorInvalidSenderSignature</a>: u64 = 3;
</code></pre>



<a name="0x3_payment_channel_ErrorMismatchedCoinType"></a>

The coin type provided does not match the channel's coin type.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorMismatchedCoinType">ErrorMismatchedCoinType</a>: u64 = 21;
</code></pre>



<a name="0x3_payment_channel_ErrorNotReceiver"></a>

The signer is not the designated receiver of the channel.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorNotReceiver">ErrorNotReceiver</a>: u64 = 1;
</code></pre>



<a name="0x3_payment_channel_ErrorNotSender"></a>

The signer is not the sender of the channel.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorNotSender">ErrorNotSender</a>: u64 = 14;
</code></pre>



<a name="0x3_payment_channel_ErrorSenderMustIsDID"></a>

The sender must have a DID document to open a channel.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorSenderMustIsDID">ErrorSenderMustIsDID</a>: u64 = 20;
</code></pre>



<a name="0x3_payment_channel_ErrorSubChannelNotAuthorized"></a>

The sub-channel has not been authorized yet.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorSubChannelNotAuthorized">ErrorSubChannelNotAuthorized</a>: u64 = 15;
</code></pre>



<a name="0x3_payment_channel_ErrorUnsupportedVersion"></a>

The SubRAV version is not supported.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorUnsupportedVersion">ErrorUnsupportedVersion</a>: u64 = 24;
</code></pre>



<a name="0x3_payment_channel_ErrorVMAuthorizeOnlySender"></a>

Only the sender can authorize verification methods for the channel.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorVMAuthorizeOnlySender">ErrorVMAuthorizeOnlySender</a>: u64 = 16;
</code></pre>



<a name="0x3_payment_channel_STATUS_ACTIVE"></a>



<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_STATUS_ACTIVE">STATUS_ACTIVE</a>: u8 = 0;
</code></pre>



<a name="0x3_payment_channel_STATUS_CANCELLING"></a>



<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_STATUS_CANCELLING">STATUS_CANCELLING</a>: u8 = 1;
</code></pre>



<a name="0x3_payment_channel_STATUS_CLOSED"></a>



<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_STATUS_CLOSED">STATUS_CLOSED</a>: u8 = 2;
</code></pre>



<a name="0x3_payment_channel_SUB_RAV_VERSION_V1"></a>

Current supported SubRAV version


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_SUB_RAV_VERSION_V1">SUB_RAV_VERSION_V1</a>: u8 = 1;
</code></pre>



<a name="0x3_payment_channel_calc_channel_object_id"></a>

## Function `calc_channel_object_id`

Calculate the deterministic ObjectID for a payment channel
This allows anyone to derive the channel ID from sender, receiver, and coin type


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_calc_channel_object_id">calc_channel_object_id</a>(sender: <b>address</b>, receiver: <b>address</b>, coin_type: <a href="_String">string::String</a>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x3_payment_channel_borrow_or_create_payment_hub"></a>

## Function `borrow_or_create_payment_hub`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_borrow_or_create_payment_hub">borrow_or_create_payment_hub</a>(owner: <b>address</b>): &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="payment_channel.md#0x3_payment_channel_PaymentHub">payment_channel::PaymentHub</a>&gt;
</code></pre>



<a name="0x3_payment_channel_ensure_payment_hub_exists"></a>

## Function `ensure_payment_hub_exists`



<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_ensure_payment_hub_exists">ensure_payment_hub_exists</a>(owner: <b>address</b>)
</code></pre>



<a name="0x3_payment_channel_create_payment_hub"></a>

## Function `create_payment_hub`

Creates and initializes a payment hub for the sender.
This also creates an associated MultiCoinStore.


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_create_payment_hub">create_payment_hub</a>()
</code></pre>



<a name="0x3_payment_channel_deposit_to_hub_entry"></a>

## Function `deposit_to_hub_entry`

Deposits a specific type of coin from the sender's account coin store into the receiver's payment hub


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_deposit_to_hub_entry">deposit_to_hub_entry</a>&lt;CoinType: store, key&gt;(sender: &<a href="">signer</a>, receiver: <b>address</b>, amount: <a href="">u256</a>)
</code></pre>



<a name="0x3_payment_channel_deposit_to_hub"></a>

## Function `deposit_to_hub`

Deposits a specific type of coin into the payment hub of the account


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_deposit_to_hub">deposit_to_hub</a>&lt;CoinType: store, key&gt;(account_addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<a name="0x3_payment_channel_deposit_to_hub_generic"></a>

## Function `deposit_to_hub_generic`

Deposits a generic coin into the payment hub of the account


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_deposit_to_hub_generic">deposit_to_hub_generic</a>(account_addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_GenericCoin">coin::GenericCoin</a>)
</code></pre>



<a name="0x3_payment_channel_withdraw_from_hub"></a>

## Function `withdraw_from_hub`

Withdraws funds from the payment hub to the owner's account coin store
Will fail if there are active channels for this coin type


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_withdraw_from_hub">withdraw_from_hub</a>&lt;CoinType: store, key&gt;(owner: &<a href="">signer</a>, amount: <a href="">u256</a>)
</code></pre>



<a name="0x3_payment_channel_withdraw_from_hub_entry"></a>

## Function `withdraw_from_hub_entry`

Entry function for withdrawing from payment hub


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_withdraw_from_hub_entry">withdraw_from_hub_entry</a>&lt;CoinType: store, key&gt;(owner: &<a href="">signer</a>, amount: <a href="">u256</a>)
</code></pre>



<a name="0x3_payment_channel_open_channel"></a>

## Function `open_channel`

Opens a new payment channel linked to a payment hub.
If a channel already exists and is closed, it will be reactivated.
If a channel already exists and is active, it will return an error.


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_open_channel">open_channel</a>&lt;CoinType: store, key&gt;(channel_sender: &<a href="">signer</a>, channel_receiver: <b>address</b>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x3_payment_channel_open_channel_entry"></a>

## Function `open_channel_entry`

Entry function for opening a channel


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_open_channel_entry">open_channel_entry</a>&lt;CoinType: store, key&gt;(channel_sender: &<a href="">signer</a>, channel_receiver: <b>address</b>)
</code></pre>



<a name="0x3_payment_channel_authorize_sub_channel"></a>

## Function `authorize_sub_channel`

Authorizes a sub-channel by granting a verification method permission for the payment channel.
This function must be called by the sender before using any vm_id_fragment for payments.


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_authorize_sub_channel">authorize_sub_channel</a>(channel_sender: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, vm_id_fragment: <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_payment_channel_authorize_sub_channel_entry"></a>

## Function `authorize_sub_channel_entry`

Entry function for authorizing a sub-channel


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_authorize_sub_channel_entry">authorize_sub_channel_entry</a>(channel_sender: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, vm_id_fragment: <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_payment_channel_open_channel_with_sub_channel"></a>

## Function `open_channel_with_sub_channel`

Convenience function to open a channel and sub-channel in one step.
This function will:
1. Create a new channel if none exists
2. Reactivate a closed channel if one exists
3. Authorize the specified verification method for the channel
Returns the channel ID for reference.


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_open_channel_with_sub_channel">open_channel_with_sub_channel</a>&lt;CoinType: store, key&gt;(channel_sender: &<a href="">signer</a>, channel_receiver: <b>address</b>, vm_id_fragment: <a href="_String">string::String</a>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x3_payment_channel_open_channel_with_sub_channel_entry"></a>

## Function `open_channel_with_sub_channel_entry`

Entry function for opening a channel and sub-channel in one step


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_open_channel_with_sub_channel_entry">open_channel_with_sub_channel_entry</a>&lt;CoinType: store, key&gt;(channel_sender: &<a href="">signer</a>, channel_receiver: <b>address</b>, vm_id_fragment: <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_payment_channel_claim_from_channel"></a>

## Function `claim_from_channel`

Anyone can claim funds from a specific sub-channel on behalf of the receiver.
The funds will always be transferred to the channel receiver regardless of who calls this function.


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_claim_from_channel">claim_from_channel</a>(claimer: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, sender_vm_id_fragment: <a href="_String">string::String</a>, sub_accumulated_amount: <a href="">u256</a>, sub_nonce: u64, sender_signature: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_payment_channel_claim_from_channel_entry"></a>

## Function `claim_from_channel_entry`

Entry function for claiming from channel


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_claim_from_channel_entry">claim_from_channel_entry</a>(claimer: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, sender_vm_id_fragment: <a href="_String">string::String</a>, sub_accumulated_amount: <a href="">u256</a>, sub_nonce: u64, sender_signature: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_payment_channel_close_channel"></a>

## Function `close_channel`

Close the entire channel with final settlement of all sub-channels
Called by receiver with proofs of final state from all sub-channels


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_close_channel">close_channel</a>(channel_receiver: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, proofs: <a href="">vector</a>&lt;<a href="payment_channel.md#0x3_payment_channel_CloseProof">payment_channel::CloseProof</a>&gt;)
</code></pre>



<a name="0x3_payment_channel_close_channel_entry"></a>

## Function `close_channel_entry`

Entry function for closing the entire channel with settlement
Takes serialized closure proofs and deserializes them


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_close_channel_entry">close_channel_entry</a>(channel_receiver: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, serialized_proofs: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_payment_channel_initiate_cancellation_entry"></a>

## Function `initiate_cancellation_entry`

Entry function for initiating cancellation


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_initiate_cancellation_entry">initiate_cancellation_entry</a>(channel_sender: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>)
</code></pre>



<a name="0x3_payment_channel_initiate_cancellation"></a>

## Function `initiate_cancellation`

Sender initiates unilateral channel cancellation with proofs for sub-channels


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_initiate_cancellation">initiate_cancellation</a>(channel_sender: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, proofs: <a href="">vector</a>&lt;<a href="payment_channel.md#0x3_payment_channel_CancelProof">payment_channel::CancelProof</a>&gt;)
</code></pre>



<a name="0x3_payment_channel_initiate_cancellation_with_proofs_entry"></a>

## Function `initiate_cancellation_with_proofs_entry`

Entry function for initiating cancellation with proofs
Takes serialized cancellation proofs and deserializes them


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_initiate_cancellation_with_proofs_entry">initiate_cancellation_with_proofs_entry</a>(channel_sender: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, serialized_proofs: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_payment_channel_dispute_cancellation"></a>

## Function `dispute_cancellation`

Receiver disputes cancellation with newer state


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_dispute_cancellation">dispute_cancellation</a>(channel_receiver: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, sender_vm_id_fragment: <a href="_String">string::String</a>, dispute_accumulated_amount: <a href="">u256</a>, dispute_nonce: u64, sender_signature: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_payment_channel_dispute_cancellation_entry"></a>

## Function `dispute_cancellation_entry`

Entry function for disputing cancellation


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_dispute_cancellation_entry">dispute_cancellation_entry</a>(channel_receiver: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, sender_vm_id_fragment: <a href="_String">string::String</a>, dispute_accumulated_amount: <a href="">u256</a>, dispute_nonce: u64, sender_signature: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_payment_channel_finalize_cancellation"></a>

## Function `finalize_cancellation`

Finalize cancellation after challenge period


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_finalize_cancellation">finalize_cancellation</a>(channel_id: <a href="_ObjectID">object::ObjectID</a>)
</code></pre>



<a name="0x3_payment_channel_finalize_cancellation_entry"></a>

## Function `finalize_cancellation_entry`

Entry function for finalizing cancellation


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_finalize_cancellation_entry">finalize_cancellation_entry</a>(channel_id: <a href="_ObjectID">object::ObjectID</a>)
</code></pre>



<a name="0x3_payment_channel_get_payment_hub_id"></a>

## Function `get_payment_hub_id`

Get payment hub ID for an address


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_payment_hub_id">get_payment_hub_id</a>(owner: <b>address</b>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x3_payment_channel_payment_hub_exists"></a>

## Function `payment_hub_exists`

Check if payment hub exists for an address


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_payment_hub_exists">payment_hub_exists</a>(owner: <b>address</b>): bool
</code></pre>



<a name="0x3_payment_channel_channel_exists"></a>

## Function `channel_exists`

Check if a payment channel exists between sender and receiver for the given coin type


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_channel_exists">channel_exists</a>(sender: <b>address</b>, receiver: <b>address</b>, coin_type: <a href="_String">string::String</a>): bool
</code></pre>



<a name="0x3_payment_channel_get_channel_id"></a>

## Function `get_channel_id`

Get channel ID for a given sender, receiver, and coin type


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_channel_id">get_channel_id</a>(sender: <b>address</b>, receiver: <b>address</b>, coin_type: <a href="_String">string::String</a>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x3_payment_channel_get_channel_info"></a>

## Function `get_channel_info`

Get channel information


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_channel_info">get_channel_info</a>(channel_id: <a href="_ObjectID">object::ObjectID</a>): (<b>address</b>, <b>address</b>, <a href="_String">string::String</a>, u8)
</code></pre>



<a name="0x3_payment_channel_get_channel_epoch"></a>

## Function `get_channel_epoch`

Get channel epoch


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_channel_epoch">get_channel_epoch</a>(channel_id: <a href="_ObjectID">object::ObjectID</a>): u64
</code></pre>



<a name="0x3_payment_channel_get_sub_channel_state"></a>

## Function `get_sub_channel_state`

Get sub-channel state


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_sub_channel_state">get_sub_channel_state</a>(channel_id: <a href="_ObjectID">object::ObjectID</a>, vm_id_fragment: <a href="_String">string::String</a>): (<a href="">u256</a>, u64)
</code></pre>



<a name="0x3_payment_channel_sub_channel_exists"></a>

## Function `sub_channel_exists`

Check if a sub-channel exists


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_sub_channel_exists">sub_channel_exists</a>(channel_id: <a href="_ObjectID">object::ObjectID</a>, vm_id_fragment: <a href="_String">string::String</a>): bool
</code></pre>



<a name="0x3_payment_channel_get_sub_channel_count"></a>

## Function `get_sub_channel_count`

Get the number of sub-channels in a payment channel


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_sub_channel_count">get_sub_channel_count</a>(channel_id: <a href="_ObjectID">object::ObjectID</a>): u64
</code></pre>



<a name="0x3_payment_channel_get_cancellation_info"></a>

## Function `get_cancellation_info`

Get cancellation info


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_cancellation_info">get_cancellation_info</a>(channel_id: <a href="_ObjectID">object::ObjectID</a>): <a href="_Option">option::Option</a>&lt;<a href="payment_channel.md#0x3_payment_channel_CancellationInfo">payment_channel::CancellationInfo</a>&gt;
</code></pre>



<a name="0x3_payment_channel_get_sub_channel_public_key"></a>

## Function `get_sub_channel_public_key`

Get sub-channel public key multibase if exists


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_sub_channel_public_key">get_sub_channel_public_key</a>(channel_id: <a href="_ObjectID">object::ObjectID</a>, vm_id_fragment: <a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x3_payment_channel_get_sub_channel_method_type"></a>

## Function `get_sub_channel_method_type`

Get sub-channel method type if exists


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_sub_channel_method_type">get_sub_channel_method_type</a>(channel_id: <a href="_ObjectID">object::ObjectID</a>, vm_id_fragment: <a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x3_payment_channel_get_active_channel_count"></a>

## Function `get_active_channel_count`

Get the number of active channels for a specific coin type


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_active_channel_count">get_active_channel_count</a>(owner: <b>address</b>, coin_type: <a href="_String">string::String</a>): u64
</code></pre>



<a name="0x3_payment_channel_can_withdraw_from_hub"></a>

## Function `can_withdraw_from_hub`

Check if withdrawal is allowed for a specific coin type


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_can_withdraw_from_hub">can_withdraw_from_hub</a>(owner: <b>address</b>, coin_type: <a href="_String">string::String</a>): bool
</code></pre>



<a name="0x3_payment_channel_get_balance_in_hub"></a>

## Function `get_balance_in_hub`

Get balance of specific coin type in payment hub


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_balance_in_hub">get_balance_in_hub</a>&lt;CoinType: key&gt;(owner: <b>address</b>): <a href="">u256</a>
</code></pre>



<a name="0x3_payment_channel_withdraw_from_hub_internal"></a>

## Function `withdraw_from_hub_internal`

Internal function to withdraw specific coin type from payment hub
(no signer required and does not check for active channels)
Used by system contracts like transaction_gas module


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_withdraw_from_hub_internal">withdraw_from_hub_internal</a>&lt;CoinType: key&gt;(addr: <b>address</b>, amount: <a href="">u256</a>): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>
