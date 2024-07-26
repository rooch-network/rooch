
<a name="0x3_transaction_fee"></a>

# Module `0x3::transaction_fee`



-  [Resource `TransactionFeePool`](#0x3_transaction_fee_TransactionFeePool)
-  [Function `genesis_init`](#0x3_transaction_fee_genesis_init)
-  [Function `get_gas_factor`](#0x3_transaction_fee_get_gas_factor)
-  [Function `calculate_gas`](#0x3_transaction_fee_calculate_gas)
-  [Function `withdraw_fee`](#0x3_transaction_fee_withdraw_fee)
-  [Function `deposit_fee`](#0x3_transaction_fee_deposit_fee)


<pre><code><b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="coin_store.md#0x3_coin_store">0x3::coin_store</a>;
<b>use</b> <a href="gas_coin.md#0x3_gas_coin">0x3::gas_coin</a>;
</code></pre>



<a name="0x3_transaction_fee_TransactionFeePool"></a>

## Resource `TransactionFeePool`



<pre><code><b>struct</b> <a href="transaction_fee.md#0x3_transaction_fee_TransactionFeePool">TransactionFeePool</a> <b>has</b> key
</code></pre>



<a name="0x3_transaction_fee_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x3_transaction_fee_genesis_init">genesis_init</a>(_genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x3_transaction_fee_get_gas_factor"></a>

## Function `get_gas_factor`

Returns the gas factor of gas.


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x3_transaction_fee_get_gas_factor">get_gas_factor</a>(): u64
</code></pre>



<a name="0x3_transaction_fee_calculate_gas"></a>

## Function `calculate_gas`



<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x3_transaction_fee_calculate_gas">calculate_gas</a>(gas_amount: u64): u256
</code></pre>



<a name="0x3_transaction_fee_withdraw_fee"></a>

## Function `withdraw_fee`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x3_transaction_fee_withdraw_fee">withdraw_fee</a>(amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;
</code></pre>



<a name="0x3_transaction_fee_deposit_fee"></a>

## Function `deposit_fee`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x3_transaction_fee_deposit_fee">deposit_fee</a>(<a href="gas_coin.md#0x3_gas_coin">gas_coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;)
</code></pre>
