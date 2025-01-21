
<a name="0x3_transaction_fee"></a>

# Module `0x3::transaction_fee`

The transaction fee module is used to manage the transaction fee pool.
Distribution of Transaction Gas Fees:

1. RoochNetwork 40%
* Before Mainnet launch: Used to repay the debt from Gas airdrops
* After Mainnet launch: Used to buy back Mainnet tokens
2. Sequencer 30%
3. Application Developers 30%
* Goes to the developer of the entry function contract called by the transaction
* If the entry contract is a system Framework contract, this portion goes to the Rooch network


-  [Resource `TransactionFeePool`](#0x3_transaction_fee_TransactionFeePool)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_transaction_fee_genesis_init)
-  [Function `get_gas_factor`](#0x3_transaction_fee_get_gas_factor)
-  [Function `calculate_gas`](#0x3_transaction_fee_calculate_gas)
-  [Function `withdraw_fee`](#0x3_transaction_fee_withdraw_fee)
-  [Function `deposit_fee`](#0x3_transaction_fee_deposit_fee)
-  [Function `distribute_fee`](#0x3_transaction_fee_distribute_fee)
-  [Function `withdraw_gas_revenue`](#0x3_transaction_fee_withdraw_gas_revenue)
-  [Function `withdraw_gas_revenue_entry`](#0x3_transaction_fee_withdraw_gas_revenue_entry)
-  [Function `gas_revenue_balance`](#0x3_transaction_fee_gas_revenue_balance)


<pre><code><b>use</b> <a href="">0x2::core_addresses</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="account_coin_store.md#0x3_account_coin_store">0x3::account_coin_store</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="coin_store.md#0x3_coin_store">0x3::coin_store</a>;
<b>use</b> <a href="gas_coin.md#0x3_gas_coin">0x3::gas_coin</a>;
</code></pre>



<a name="0x3_transaction_fee_TransactionFeePool"></a>

## Resource `TransactionFeePool`



<pre><code><b>struct</b> <a href="transaction_fee.md#0x3_transaction_fee_TransactionFeePool">TransactionFeePool</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_transaction_fee_ErrorInvalidGasUsed"></a>

Error code for invalid gas used in transaction


<pre><code><b>const</b> <a href="transaction_fee.md#0x3_transaction_fee_ErrorInvalidGasUsed">ErrorInvalidGasUsed</a>: u64 = 1;
</code></pre>



<a name="0x3_transaction_fee_SystemFeeAddress"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x3_transaction_fee_SystemFeeAddress">SystemFeeAddress</a>: <b>address</b> = 0x3;
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



<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x3_transaction_fee_calculate_gas">calculate_gas</a>(gas_amount: u64): <a href="">u256</a>
</code></pre>



<a name="0x3_transaction_fee_withdraw_fee"></a>

## Function `withdraw_fee`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x3_transaction_fee_withdraw_fee">withdraw_fee</a>(amount: <a href="">u256</a>): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x3_gas_coin_RGas">gas_coin::RGas</a>&gt;
</code></pre>



<a name="0x3_transaction_fee_deposit_fee"></a>

## Function `deposit_fee`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x3_transaction_fee_deposit_fee">deposit_fee</a>(<a href="gas_coin.md#0x3_gas_coin">gas_coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x3_gas_coin_RGas">gas_coin::RGas</a>&gt;)
</code></pre>



<a name="0x3_transaction_fee_distribute_fee"></a>

## Function `distribute_fee`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x3_transaction_fee_distribute_fee">distribute_fee</a>(total_paid_gas: <a href="">u256</a>, gas_used: <a href="">u256</a>, contract_address: <b>address</b>, sequencer_address: <b>address</b>): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x3_gas_coin_RGas">gas_coin::RGas</a>&gt;
</code></pre>



<a name="0x3_transaction_fee_withdraw_gas_revenue"></a>

## Function `withdraw_gas_revenue`

Withdraw the gas revenue for the sender
The contract address can use <code>moveos_std::signer::module_signer</code> to get the signer


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x3_transaction_fee_withdraw_gas_revenue">withdraw_gas_revenue</a>(sender: &<a href="">signer</a>, amount: <a href="">u256</a>): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x3_gas_coin_RGas">gas_coin::RGas</a>&gt;
</code></pre>



<a name="0x3_transaction_fee_withdraw_gas_revenue_entry"></a>

## Function `withdraw_gas_revenue_entry`

The entry function to withdraw the gas revenue for the sender


<pre><code><b>public</b> entry <b>fun</b> <a href="transaction_fee.md#0x3_transaction_fee_withdraw_gas_revenue_entry">withdraw_gas_revenue_entry</a>(sender: &<a href="">signer</a>, amount: <a href="">u256</a>)
</code></pre>



<a name="0x3_transaction_fee_gas_revenue_balance"></a>

## Function `gas_revenue_balance`

Get the gas revenue balance for the given address


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x3_transaction_fee_gas_revenue_balance">gas_revenue_balance</a>(addr: <b>address</b>): <a href="">u256</a>
</code></pre>
