
<a name="0x3_payment_revenue"></a>

# Module `0x3::payment_revenue`



-  [Struct `RevenueSource`](#0x3_payment_revenue_RevenueSource)
-  [Resource `PaymentRevenueHub`](#0x3_payment_revenue_PaymentRevenueHub)
-  [Struct `RevenueHubCreatedEvent`](#0x3_payment_revenue_RevenueHubCreatedEvent)
-  [Struct `RevenueDepositedEvent`](#0x3_payment_revenue_RevenueDepositedEvent)
-  [Struct `RevenueWithdrawnEvent`](#0x3_payment_revenue_RevenueWithdrawnEvent)
-  [Constants](#@Constants_0)
-  [Function `get_revenue_hub_id`](#0x3_payment_revenue_get_revenue_hub_id)
-  [Function `revenue_hub_exists`](#0x3_payment_revenue_revenue_hub_exists)
-  [Function `get_revenue_balance`](#0x3_payment_revenue_get_revenue_balance)
-  [Function `get_revenue_by_source`](#0x3_payment_revenue_get_revenue_by_source)
-  [Function `create_revenue_hub`](#0x3_payment_revenue_create_revenue_hub)
-  [Function `withdraw_revenue`](#0x3_payment_revenue_withdraw_revenue)
-  [Function `withdraw_revenue_entry`](#0x3_payment_revenue_withdraw_revenue_entry)
-  [Function `deposit_revenue_generic`](#0x3_payment_revenue_deposit_revenue_generic)
-  [Function `withdraw_revenue_internal`](#0x3_payment_revenue_withdraw_revenue_internal)
-  [Function `preview_withdrawal_fee`](#0x3_payment_revenue_preview_withdrawal_fee)
-  [Function `create_revenue_source`](#0x3_payment_revenue_create_revenue_source)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="account_coin_store.md#0x3_account_coin_store">0x3::account_coin_store</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="multi_coin_store.md#0x3_multi_coin_store">0x3::multi_coin_store</a>;
</code></pre>



<a name="0x3_payment_revenue_RevenueSource"></a>

## Struct `RevenueSource`

Revenue source information for tracking and events


<pre><code><b>struct</b> <a href="payment_revenue.md#0x3_payment_revenue_RevenueSource">RevenueSource</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_payment_revenue_PaymentRevenueHub"></a>

## Resource `PaymentRevenueHub`

Revenue management hub for each account
Stores revenue earned through various sources separately from principal funds


<pre><code><b>struct</b> <a href="payment_revenue.md#0x3_payment_revenue_PaymentRevenueHub">PaymentRevenueHub</a> <b>has</b> key
</code></pre>



<a name="0x3_payment_revenue_RevenueHubCreatedEvent"></a>

## Struct `RevenueHubCreatedEvent`

Event emitted when a revenue hub is created


<pre><code><b>struct</b> <a href="payment_revenue.md#0x3_payment_revenue_RevenueHubCreatedEvent">RevenueHubCreatedEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_payment_revenue_RevenueDepositedEvent"></a>

## Struct `RevenueDepositedEvent`

Event emitted when revenue is deposited


<pre><code><b>struct</b> <a href="payment_revenue.md#0x3_payment_revenue_RevenueDepositedEvent">RevenueDepositedEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_payment_revenue_RevenueWithdrawnEvent"></a>

## Struct `RevenueWithdrawnEvent`

Event emitted when revenue is withdrawn


<pre><code><b>struct</b> <a href="payment_revenue.md#0x3_payment_revenue_RevenueWithdrawnEvent">RevenueWithdrawnEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_payment_revenue_ErrorInsufficientBalance"></a>

Insufficient revenue balance for withdrawal.


<pre><code><b>const</b> <a href="payment_revenue.md#0x3_payment_revenue_ErrorInsufficientBalance">ErrorInsufficientBalance</a>: u64 = 1;
</code></pre>



<a name="0x3_payment_revenue_ErrorInvalidRevenueSource"></a>

Invalid revenue source information.


<pre><code><b>const</b> <a href="payment_revenue.md#0x3_payment_revenue_ErrorInvalidRevenueSource">ErrorInvalidRevenueSource</a>: u64 = 3;
</code></pre>



<a name="0x3_payment_revenue_ErrorInvalidWithdrawalAmount"></a>

Withdrawal amount must be greater than zero.


<pre><code><b>const</b> <a href="payment_revenue.md#0x3_payment_revenue_ErrorInvalidWithdrawalAmount">ErrorInvalidWithdrawalAmount</a>: u64 = 4;
</code></pre>



<a name="0x3_payment_revenue_ErrorRevenueHubNotExists"></a>

The revenue hub does not exist for this account.


<pre><code><b>const</b> <a href="payment_revenue.md#0x3_payment_revenue_ErrorRevenueHubNotExists">ErrorRevenueHubNotExists</a>: u64 = 2;
</code></pre>



<a name="0x3_payment_revenue_get_revenue_hub_id"></a>

## Function `get_revenue_hub_id`

Get the revenue hub ID for an address


<pre><code><b>public</b> <b>fun</b> <a href="payment_revenue.md#0x3_payment_revenue_get_revenue_hub_id">get_revenue_hub_id</a>(owner: <b>address</b>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x3_payment_revenue_revenue_hub_exists"></a>

## Function `revenue_hub_exists`

Check if revenue hub exists for an address


<pre><code><b>public</b> <b>fun</b> <a href="payment_revenue.md#0x3_payment_revenue_revenue_hub_exists">revenue_hub_exists</a>(owner: <b>address</b>): bool
</code></pre>



<a name="0x3_payment_revenue_get_revenue_balance"></a>

## Function `get_revenue_balance`

Get revenue balance for a specific coin type


<pre><code><b>public</b> <b>fun</b> <a href="payment_revenue.md#0x3_payment_revenue_get_revenue_balance">get_revenue_balance</a>&lt;CoinType: key&gt;(owner: <b>address</b>): <a href="">u256</a>
</code></pre>



<a name="0x3_payment_revenue_get_revenue_by_source"></a>

## Function `get_revenue_by_source`

Get revenue balance by source type and coin type


<pre><code><b>public</b> <b>fun</b> <a href="payment_revenue.md#0x3_payment_revenue_get_revenue_by_source">get_revenue_by_source</a>(owner: <b>address</b>, source_type: <a href="_String">string::String</a>, coin_type: <a href="_String">string::String</a>): <a href="">u256</a>
</code></pre>



<a name="0x3_payment_revenue_create_revenue_hub"></a>

## Function `create_revenue_hub`

Create a revenue hub for the sender


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_revenue.md#0x3_payment_revenue_create_revenue_hub">create_revenue_hub</a>()
</code></pre>



<a name="0x3_payment_revenue_withdraw_revenue"></a>

## Function `withdraw_revenue`

Withdraw revenue to account coin store
Future: This will support fee deduction


<pre><code><b>public</b> <b>fun</b> <a href="payment_revenue.md#0x3_payment_revenue_withdraw_revenue">withdraw_revenue</a>&lt;CoinType: store, key&gt;(owner: &<a href="">signer</a>, amount: <a href="">u256</a>)
</code></pre>



<a name="0x3_payment_revenue_withdraw_revenue_entry"></a>

## Function `withdraw_revenue_entry`

Entry function for withdrawing revenue


<pre><code><b>public</b> entry <b>fun</b> <a href="payment_revenue.md#0x3_payment_revenue_withdraw_revenue_entry">withdraw_revenue_entry</a>&lt;CoinType: store, key&gt;(owner: &<a href="">signer</a>, amount: <a href="">u256</a>)
</code></pre>



<a name="0x3_payment_revenue_deposit_revenue_generic"></a>

## Function `deposit_revenue_generic`

Deposit revenue from trusted modules (friend only)


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="payment_revenue.md#0x3_payment_revenue_deposit_revenue_generic">deposit_revenue_generic</a>(<a href="">account</a>: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_GenericCoin">coin::GenericCoin</a>, source: <a href="payment_revenue.md#0x3_payment_revenue_RevenueSource">payment_revenue::RevenueSource</a>)
</code></pre>



<a name="0x3_payment_revenue_withdraw_revenue_internal"></a>

## Function `withdraw_revenue_internal`

Internal withdrawal for system modules (friend only)
Future: This will be used by gas fee deduction or other system operations


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="payment_revenue.md#0x3_payment_revenue_withdraw_revenue_internal">withdraw_revenue_internal</a>&lt;CoinType: key&gt;(<a href="">account</a>: <b>address</b>, amount: <a href="">u256</a>): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<a name="0x3_payment_revenue_preview_withdrawal_fee"></a>

## Function `preview_withdrawal_fee`

Preview withdrawal fee (placeholder for future fee mechanism)
Currently returns zero fee, will be implemented when RevenueConfig is added


<pre><code><b>public</b> <b>fun</b> <a href="payment_revenue.md#0x3_payment_revenue_preview_withdrawal_fee">preview_withdrawal_fee</a>&lt;CoinType: key&gt;(_owner: <b>address</b>, amount: <a href="">u256</a>): (<a href="">u256</a>, <a href="">u256</a>, <a href="">u256</a>)
</code></pre>



<a name="0x3_payment_revenue_create_revenue_source"></a>

## Function `create_revenue_source`

Create a revenue source for tracking purposes


<pre><code><b>public</b> <b>fun</b> <a href="payment_revenue.md#0x3_payment_revenue_create_revenue_source">create_revenue_source</a>(source_type: <a href="_String">string::String</a>, source_id: <a href="_Option">option::Option</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;, description: <a href="_String">string::String</a>): <a href="payment_revenue.md#0x3_payment_revenue_RevenueSource">payment_revenue::RevenueSource</a>
</code></pre>
