
<a name="0x3_payment_channel"></a>

# Module `0x3::payment_channel`



-  [Struct `PaymentHubCreatedEvent`](#0x3_payment_channel_PaymentHubCreatedEvent)
-  [Struct `PaymentChannelOpenedEvent`](#0x3_payment_channel_PaymentChannelOpenedEvent)
-  [Struct `ChannelClaimedEvent`](#0x3_payment_channel_ChannelClaimedEvent)
-  [Struct `ChannelClosedEvent`](#0x3_payment_channel_ChannelClosedEvent)
-  [Struct `ChannelCancellationInitiatedEvent`](#0x3_payment_channel_ChannelCancellationInitiatedEvent)
-  [Struct `ChannelDisputeEvent`](#0x3_payment_channel_ChannelDisputeEvent)
-  [Struct `ChannelCancellationFinalizedEvent`](#0x3_payment_channel_ChannelCancellationFinalizedEvent)
-  [Resource `PaymentHub`](#0x3_payment_channel_PaymentHub)
-  [Resource `PaymentChannel`](#0x3_payment_channel_PaymentChannel)
-  [Struct `SubChannelState`](#0x3_payment_channel_SubChannelState)
-  [Struct `CancellationInfo`](#0x3_payment_channel_CancellationInfo)
-  [Constants](#@Constants_0)
-  [Function `ensure_payment_hub_exists`](#0x3_payment_channel_ensure_payment_hub_exists)
-  [Function `create_payment_hub`](#0x3_payment_channel_create_payment_hub)
-  [Function `deposit_to_hub_entry`](#0x3_payment_channel_deposit_to_hub_entry)
-  [Function `deposit_to_hub`](#0x3_payment_channel_deposit_to_hub)
-  [Function `deposit_to_hub_generic`](#0x3_payment_channel_deposit_to_hub_generic)
-  [Function `open_channel`](#0x3_payment_channel_open_channel)
-  [Function `open_channel_entry`](#0x3_payment_channel_open_channel_entry)
-  [Function `claim_from_channel`](#0x3_payment_channel_claim_from_channel)
-  [Function `claim_from_channel_entry`](#0x3_payment_channel_claim_from_channel_entry)
-  [Function `close_channel`](#0x3_payment_channel_close_channel)
-  [Function `close_channel_entry`](#0x3_payment_channel_close_channel_entry)
-  [Function `initiate_cancellation`](#0x3_payment_channel_initiate_cancellation)
-  [Function `initiate_cancellation_entry`](#0x3_payment_channel_initiate_cancellation_entry)
-  [Function `dispute_cancellation`](#0x3_payment_channel_dispute_cancellation)
-  [Function `dispute_cancellation_entry`](#0x3_payment_channel_dispute_cancellation_entry)
-  [Function `finalize_cancellation`](#0x3_payment_channel_finalize_cancellation)
-  [Function `finalize_cancellation_entry`](#0x3_payment_channel_finalize_cancellation_entry)
-  [Function `get_payment_hub_id`](#0x3_payment_channel_get_payment_hub_id)
-  [Function `payment_hub_exists`](#0x3_payment_channel_payment_hub_exists)
-  [Function `get_channel_info`](#0x3_payment_channel_get_channel_info)
-  [Function `get_sub_channel_state`](#0x3_payment_channel_get_sub_channel_state)
-  [Function `get_cancellation_info`](#0x3_payment_channel_get_cancellation_info)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::hash</a>;
<b>use</b> <a href="">0x2::multibase_codec</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::timestamp</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="account_coin_store.md#0x3_account_coin_store">0x3::account_coin_store</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="did.md#0x3_did">0x3::did</a>;
<b>use</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1">0x3::ecdsa_k1</a>;
<b>use</b> <a href="ecdsa_r1.md#0x3_ecdsa_r1">0x3::ecdsa_r1</a>;
<b>use</b> <a href="ed25519.md#0x3_ed25519">0x3::ed25519</a>;
<b>use</b> <a href="multi_coin_store.md#0x3_multi_coin_store">0x3::multi_coin_store</a>;
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

Event emitted when a channel is cooperatively closed


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


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_PaymentChannel">PaymentChannel</a>&lt;CoinType: store&gt; <b>has</b> store, key
</code></pre>



<a name="0x3_payment_channel_SubChannelState"></a>

## Struct `SubChannelState`

The on-chain state for a specific sub-channel.


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_SubChannelState">SubChannelState</a> <b>has</b> store
</code></pre>



<a name="0x3_payment_channel_CancellationInfo"></a>

## Struct `CancellationInfo`

Information stored when a channel cancellation is initiated.


<pre><code><b>struct</b> <a href="payment_channel.md#0x3_payment_channel_CancellationInfo">CancellationInfo</a> <b>has</b> <b>copy</b>, drop, store
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



<a name="0x3_payment_channel_ErrorVerificationMethodNotFound"></a>

The specified Verification Method was not found in the sender's DID.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorVerificationMethodNotFound">ErrorVerificationMethodNotFound</a>: u64 = 4;
</code></pre>



<a name="0x3_payment_channel_CHALLENGE_PERIOD_MILLISECONDS"></a>



<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_CHALLENGE_PERIOD_MILLISECONDS">CHALLENGE_PERIOD_MILLISECONDS</a>: u64 = 86400000;
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



<a name="0x3_payment_channel_ErrorNotReceiver"></a>

The signer is not the designated receiver of the channel.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorNotReceiver">ErrorNotReceiver</a>: u64 = 1;
</code></pre>



<a name="0x3_payment_channel_ErrorNotSender"></a>

The signer is not the sender of the channel.


<pre><code><b>const</b> <a href="payment_channel.md#0x3_payment_channel_ErrorNotSender">ErrorNotSender</a>: u64 = 14;
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

Deposits a specific type of coin from the account coin store into the payment hub


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_deposit_to_hub_entry">deposit_to_hub_entry</a>&lt;CoinType: store, key&gt;(<a href="">account</a>: &<a href="">signer</a>, amount: <a href="">u256</a>)
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



<a name="0x3_payment_channel_open_channel"></a>

## Function `open_channel`

Opens a new payment channel linked to a payment hub.


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_open_channel">open_channel</a>&lt;CoinType: store, key&gt;(sender: &<a href="">signer</a>, receiver: <b>address</b>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x3_payment_channel_open_channel_entry"></a>

## Function `open_channel_entry`

Entry function for opening a channel


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_open_channel_entry">open_channel_entry</a>&lt;CoinType: store, key&gt;(sender: &<a href="">signer</a>, receiver: <b>address</b>)
</code></pre>



<a name="0x3_payment_channel_claim_from_channel"></a>

## Function `claim_from_channel`

The receiver claims funds from a specific sub-channel.


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_claim_from_channel">claim_from_channel</a>&lt;CoinType: store, key&gt;(<a href="">account</a>: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, sender_vm_id_fragment: <a href="_String">string::String</a>, sub_accumulated_amount: <a href="">u256</a>, sub_nonce: u64, sender_signature: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_payment_channel_claim_from_channel_entry"></a>

## Function `claim_from_channel_entry`

Entry function for claiming from channel


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_claim_from_channel_entry">claim_from_channel_entry</a>&lt;CoinType: store, key&gt;(<a href="">account</a>: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, sender_vm_id_fragment: <a href="_String">string::String</a>, sub_accumulated_amount: <a href="">u256</a>, sub_nonce: u64, sender_signature: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_payment_channel_close_channel"></a>

## Function `close_channel`

Close the channel cooperatively with final state from receiver


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_close_channel">close_channel</a>&lt;CoinType: store, key&gt;(<a href="">account</a>: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, sender_vm_id_fragment: <a href="_String">string::String</a>, final_accumulated_amount: <a href="">u256</a>, final_nonce: u64, sender_signature: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_payment_channel_close_channel_entry"></a>

## Function `close_channel_entry`

Entry function for closing channel


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_close_channel_entry">close_channel_entry</a>&lt;CoinType: store, key&gt;(<a href="">account</a>: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, sender_vm_id_fragment: <a href="_String">string::String</a>, final_accumulated_amount: <a href="">u256</a>, final_nonce: u64, sender_signature: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_payment_channel_initiate_cancellation"></a>

## Function `initiate_cancellation`

Sender initiates unilateral channel cancellation


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_initiate_cancellation">initiate_cancellation</a>&lt;CoinType: store, key&gt;(sender: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, pending_amount: <a href="">u256</a>)
</code></pre>



<a name="0x3_payment_channel_initiate_cancellation_entry"></a>

## Function `initiate_cancellation_entry`

Entry function for initiating cancellation


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_initiate_cancellation_entry">initiate_cancellation_entry</a>&lt;CoinType: store, key&gt;(sender: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, pending_amount: <a href="">u256</a>)
</code></pre>



<a name="0x3_payment_channel_dispute_cancellation"></a>

## Function `dispute_cancellation`

Receiver disputes cancellation with newer state


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_dispute_cancellation">dispute_cancellation</a>&lt;CoinType: store, key&gt;(<a href="">account</a>: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, sender_vm_id_fragment: <a href="_String">string::String</a>, dispute_accumulated_amount: <a href="">u256</a>, dispute_nonce: u64, sender_signature: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_payment_channel_dispute_cancellation_entry"></a>

## Function `dispute_cancellation_entry`

Entry function for disputing cancellation


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_dispute_cancellation_entry">dispute_cancellation_entry</a>&lt;CoinType: store, key&gt;(<a href="">account</a>: &<a href="">signer</a>, channel_id: <a href="_ObjectID">object::ObjectID</a>, sender_vm_id_fragment: <a href="_String">string::String</a>, dispute_accumulated_amount: <a href="">u256</a>, dispute_nonce: u64, sender_signature: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_payment_channel_finalize_cancellation"></a>

## Function `finalize_cancellation`

Finalize cancellation after challenge period


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_finalize_cancellation">finalize_cancellation</a>&lt;CoinType: store, key&gt;(channel_id: <a href="_ObjectID">object::ObjectID</a>)
</code></pre>



<a name="0x3_payment_channel_finalize_cancellation_entry"></a>

## Function `finalize_cancellation_entry`

Entry function for finalizing cancellation


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_finalize_cancellation_entry">finalize_cancellation_entry</a>&lt;CoinType: store, key&gt;(channel_id: <a href="_ObjectID">object::ObjectID</a>)
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



<a name="0x3_payment_channel_get_channel_info"></a>

## Function `get_channel_info`

Get channel information


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_channel_info">get_channel_info</a>&lt;CoinType: store&gt;(channel_id: <a href="_ObjectID">object::ObjectID</a>): (<b>address</b>, <b>address</b>, <a href="_ObjectID">object::ObjectID</a>, u8)
</code></pre>



<a name="0x3_payment_channel_get_sub_channel_state"></a>

## Function `get_sub_channel_state`

Get sub-channel state


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_sub_channel_state">get_sub_channel_state</a>&lt;CoinType: store&gt;(channel_id: <a href="_ObjectID">object::ObjectID</a>, vm_id_fragment: <a href="_String">string::String</a>): (<a href="">u256</a>, u64)
</code></pre>



<a name="0x3_payment_channel_get_cancellation_info"></a>

## Function `get_cancellation_info`

Get cancellation info


<pre><code><b>public</b> <b>fun</b> <a href="payment_channel.md#0x3_payment_channel_get_cancellation_info">get_cancellation_info</a>&lt;CoinType: store&gt;(channel_id: <a href="_ObjectID">object::ObjectID</a>): <a href="_Option">option::Option</a>&lt;<a href="payment_channel.md#0x3_payment_channel_CancellationInfo">payment_channel::CancellationInfo</a>&gt;
</code></pre>
