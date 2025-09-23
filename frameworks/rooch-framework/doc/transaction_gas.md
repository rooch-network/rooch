
<a name="0x3_transaction_gas"></a>

# Module `0x3::transaction_gas`

This module handles transaction gas payment with smart fallback between account store and payment hub.
It provides a clean separation of concerns for gas payment logic.


-  [Struct `GasUsageInfo`](#0x3_transaction_gas_GasUsageInfo)
-  [Function `deduct_transaction_gas`](#0x3_transaction_gas_deduct_transaction_gas)
-  [Function `refund_transaction_gas`](#0x3_transaction_gas_refund_transaction_gas)
-  [Function `total_available_gas_balance`](#0x3_transaction_gas_total_available_gas_balance)
-  [Function `store_gas_usage_info`](#0x3_transaction_gas_store_gas_usage_info)
-  [Function `get_gas_usage_info`](#0x3_transaction_gas_get_gas_usage_info)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="gas_coin.md#0x3_gas_coin">0x3::gas_coin</a>;
<b>use</b> <a href="payment_channel.md#0x3_payment_channel">0x3::payment_channel</a>;
</code></pre>



<a name="0x3_transaction_gas_GasUsageInfo"></a>

## Struct `GasUsageInfo`

Gas usage information for proper refund tracking


<pre><code><b>struct</b> <a href="transaction_gas.md#0x3_transaction_gas_GasUsageInfo">GasUsageInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_transaction_gas_deduct_transaction_gas"></a>

## Function `deduct_transaction_gas`

Enhanced gas deduction that tries payment hub first, then account store
Returns: (gas_coin, usage_info)


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_gas.md#0x3_transaction_gas_deduct_transaction_gas">deduct_transaction_gas</a>(addr: <b>address</b>, amount: <a href="">u256</a>): (<a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x3_gas_coin_RGas">gas_coin::RGas</a>&gt;, <a href="transaction_gas.md#0x3_transaction_gas_GasUsageInfo">transaction_gas::GasUsageInfo</a>)
</code></pre>



<a name="0x3_transaction_gas_refund_transaction_gas"></a>

## Function `refund_transaction_gas`

Refund remaining gas proportionally to original sources


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_gas.md#0x3_transaction_gas_refund_transaction_gas">refund_transaction_gas</a>(addr: <b>address</b>, remaining_gas: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x3_gas_coin_RGas">gas_coin::RGas</a>&gt;, usage_info: <a href="transaction_gas.md#0x3_transaction_gas_GasUsageInfo">transaction_gas::GasUsageInfo</a>)
</code></pre>



<a name="0x3_transaction_gas_total_available_gas_balance"></a>

## Function `total_available_gas_balance`

Check total available gas balance across all sources


<pre><code><b>public</b> <b>fun</b> <a href="transaction_gas.md#0x3_transaction_gas_total_available_gas_balance">total_available_gas_balance</a>(addr: <b>address</b>): <a href="">u256</a>
</code></pre>



<a name="0x3_transaction_gas_store_gas_usage_info"></a>

## Function `store_gas_usage_info`

Store gas usage info in transaction context for later retrieval


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_gas.md#0x3_transaction_gas_store_gas_usage_info">store_gas_usage_info</a>(usage_info: <a href="transaction_gas.md#0x3_transaction_gas_GasUsageInfo">transaction_gas::GasUsageInfo</a>)
</code></pre>



<a name="0x3_transaction_gas_get_gas_usage_info"></a>

## Function `get_gas_usage_info`

Retrieve gas usage info from transaction context


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_gas.md#0x3_transaction_gas_get_gas_usage_info">get_gas_usage_info</a>(): <a href="_Option">option::Option</a>&lt;<a href="transaction_gas.md#0x3_transaction_gas_GasUsageInfo">transaction_gas::GasUsageInfo</a>&gt;
</code></pre>
