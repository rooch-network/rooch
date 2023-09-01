
<a name="0x3_gas_coin"></a>

# Module `0x3::gas_coin`

This module defines Rooch Gas Coin.


-  [Resource `GasCoin`](#0x3_gas_coin_GasCoin)
-  [Function `balance`](#0x3_gas_coin_balance)
-  [Function `burn`](#0x3_gas_coin_burn)
-  [Function `deduct_gas`](#0x3_gas_coin_deduct_gas)
-  [Function `faucet`](#0x3_gas_coin_faucet)
-  [Function `faucet_entry`](#0x3_gas_coin_faucet_entry)
-  [Function `genesis_init`](#0x3_gas_coin_genesis_init)


<pre><code><b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
</code></pre>



<a name="0x3_gas_coin_GasCoin"></a>

## Resource `GasCoin`



<pre><code><b>struct</b> <a href="gas_coin.md#0x3_gas_coin_GasCoin">GasCoin</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_gas_coin_balance"></a>

## Function `balance`



<pre><code><b>public</b> <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_balance">balance</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_balance">balance</a>(ctx: &StorageContext, addr: <b>address</b>): u256 {
    <a href="coin.md#0x3_coin_balance">coin::balance</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">GasCoin</a>&gt;(ctx, addr)
}
</code></pre>



</details>

<a name="0x3_gas_coin_burn"></a>

## Function `burn`



<pre><code><b>public</b> <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_burn">burn</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_burn">burn</a>(ctx: &<b>mut</b> StorageContext, <a href="coin.md#0x3_coin">coin</a>: Coin&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">GasCoin</a>&gt;) {
    <a href="coin.md#0x3_coin_burn_extend">coin::burn_extend</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">GasCoin</a>&gt;(ctx, <a href="coin.md#0x3_coin">coin</a>);
}
</code></pre>



</details>

<a name="0x3_gas_coin_deduct_gas"></a>

## Function `deduct_gas`

deduct gas coin from the given account.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_deduct_gas">deduct_gas</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_deduct_gas">deduct_gas</a>(ctx: &<b>mut</b> StorageContext, addr: <b>address</b>, amount: u256):Coin&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">GasCoin</a>&gt; {
    <a href="coin.md#0x3_coin_withdraw_extend">coin::withdraw_extend</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">GasCoin</a>&gt;(ctx, addr, amount)
}
</code></pre>



</details>

<a name="0x3_gas_coin_faucet"></a>

## Function `faucet`

Mint gas coin to the given account.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_faucet">faucet</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>, amount: u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_faucet">faucet</a>(ctx: &<b>mut</b> StorageContext, addr: <b>address</b>, amount: u256) {
    <b>let</b> <a href="coin.md#0x3_coin">coin</a> = <a href="gas_coin.md#0x3_gas_coin_mint">mint</a>(ctx, amount);
    <a href="coin.md#0x3_coin_deposit">coin::deposit</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">GasCoin</a>&gt;(ctx, addr, <a href="coin.md#0x3_coin">coin</a>);
}
</code></pre>



</details>

<a name="0x3_gas_coin_faucet_entry"></a>

## Function `faucet_entry`

TODO find a way to protect this function from DOS attack.


<pre><code><b>public</b> entry <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_faucet_entry">faucet_entry</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_faucet_entry">faucet_entry</a>(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <b>let</b> amount = 1_0000_0000u256;
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    <a href="gas_coin.md#0x3_gas_coin_faucet">faucet</a>(ctx, addr, amount);
}
</code></pre>



</details>

<a name="0x3_gas_coin_genesis_init"></a>

## Function `genesis_init`

Can only called during genesis to initialize the Rooch coin.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, _genesis_account: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_genesis_init">genesis_init</a>(ctx: &<b>mut</b> StorageContext, _genesis_account: &<a href="">signer</a>){
    <a href="coin.md#0x3_coin_register_extend">coin::register_extend</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">GasCoin</a>&gt;(
        ctx,
        <a href="_utf8">string::utf8</a>(b"Rooch Gas Coin"),
        <a href="_utf8">string::utf8</a>(b"RGC"),
        9, // decimals
    );
}
</code></pre>



</details>
