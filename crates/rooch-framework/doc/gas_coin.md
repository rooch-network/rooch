
<a name="0x3_gas_coin"></a>

# Module `0x3::gas_coin`

This module defines Rooch Gas Coin.


-  [Resource `GasCoin`](#0x3_gas_coin_GasCoin)
-  [Resource `FaucetRecord`](#0x3_gas_coin_FaucetRecord)
-  [Constants](#@Constants_0)
-  [Function `balance`](#0x3_gas_coin_balance)
-  [Function `burn`](#0x3_gas_coin_burn)
-  [Function `deduct_gas`](#0x3_gas_coin_deduct_gas)
-  [Function `faucet`](#0x3_gas_coin_faucet)
-  [Function `faucet_entry`](#0x3_gas_coin_faucet_entry)
-  [Function `genesis_init`](#0x3_gas_coin_genesis_init)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="account_coin_store.md#0x3_account_coin_store">0x3::account_coin_store</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="timestamp.md#0x3_timestamp">0x3::timestamp</a>;
</code></pre>



<a name="0x3_gas_coin_GasCoin"></a>

## Resource `GasCoin`



<pre><code><b>struct</b> <a href="gas_coin.md#0x3_gas_coin_GasCoin">GasCoin</a> <b>has</b> store, key
</code></pre>



<a name="0x3_gas_coin_FaucetRecord"></a>

## Resource `FaucetRecord`

Record the last time when faucet is called for each address.


<pre><code><b>struct</b> <a href="gas_coin.md#0x3_gas_coin_FaucetRecord">FaucetRecord</a> <b>has</b> store, key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_gas_coin_ErrorFaucetTooFrequently"></a>

Faucet too frequently


<pre><code><b>const</b> <a href="gas_coin.md#0x3_gas_coin_ErrorFaucetTooFrequently">ErrorFaucetTooFrequently</a>: u64 = 1;
</code></pre>



<a name="0x3_gas_coin_FAUCET_INTERVAL"></a>

Faucet interval in seconds


<pre><code><b>const</b> <a href="gas_coin.md#0x3_gas_coin_FAUCET_INTERVAL">FAUCET_INTERVAL</a>: u64 = 86400;
</code></pre>



<a name="0x3_gas_coin_balance"></a>

## Function `balance`



<pre><code><b>public</b> <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_balance">balance</a>(addr: <b>address</b>): u256
</code></pre>



<a name="0x3_gas_coin_burn"></a>

## Function `burn`



<pre><code><b>public</b> <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_burn">burn</a>(<a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;)
</code></pre>



<a name="0x3_gas_coin_deduct_gas"></a>

## Function `deduct_gas`

deduct gas coin from the given account.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_deduct_gas">deduct_gas</a>(addr: <b>address</b>, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;
</code></pre>



<a name="0x3_gas_coin_faucet"></a>

## Function `faucet`

Mint gas coin to the given account.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_faucet">faucet</a>(addr: <b>address</b>, amount: u256)
</code></pre>



<a name="0x3_gas_coin_faucet_entry"></a>

## Function `faucet_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_faucet_entry">faucet_entry</a>(<a href="">account</a>: &<a href="">signer</a>)
</code></pre>



<a name="0x3_gas_coin_genesis_init"></a>

## Function `genesis_init`

Can only be called during genesis to initialize the Rooch coin.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_genesis_init">genesis_init</a>(_genesis_account: &<a href="">signer</a>)
</code></pre>
