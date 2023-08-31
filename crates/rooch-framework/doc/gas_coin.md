
<a name="0x3_gas_coin"></a>

# Module `0x3::gas_coin`

This module defines Rooch Gas Coin.


-  [Resource `GasCoin`](#0x3_gas_coin_GasCoin)
-  [Resource `MintCapStore`](#0x3_gas_coin_MintCapStore)
-  [Struct `DelegatedMintCapability`](#0x3_gas_coin_DelegatedMintCapability)
-  [Resource `Delegations`](#0x3_gas_coin_Delegations)
-  [Constants](#@Constants_0)
-  [Function `balance`](#0x3_gas_coin_balance)
-  [Function `burn`](#0x3_gas_coin_burn)
-  [Function `deduct_gas`](#0x3_gas_coin_deduct_gas)
-  [Function `faucet`](#0x3_gas_coin_faucet)
-  [Function `faucet_entry`](#0x3_gas_coin_faucet_entry)
-  [Function `genesis_init`](#0x3_gas_coin_genesis_init)
-  [Function `has_mint_capability`](#0x3_gas_coin_has_mint_capability)
-  [Function `destroy_mint_cap`](#0x3_gas_coin_destroy_mint_cap)


<pre><code><b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="core_addresses.md#0x3_core_addresses">0x3::core_addresses</a>;
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

<a name="0x3_gas_coin_MintCapStore"></a>

## Resource `MintCapStore`



<pre><code><b>struct</b> <a href="gas_coin.md#0x3_gas_coin_MintCapStore">MintCapStore</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mint_cap: <a href="coin.md#0x3_coin_MintCapability">coin::MintCapability</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_gas_coin_DelegatedMintCapability"></a>

## Struct `DelegatedMintCapability`

Delegation coin created by delegator and can be claimed by the delegatee as MintCapability.


<pre><code><b>struct</b> <a href="gas_coin.md#0x3_gas_coin_DelegatedMintCapability">DelegatedMintCapability</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><b>to</b>: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_gas_coin_Delegations"></a>

## Resource `Delegations`

The container stores the current pending delegations.


<pre><code><b>struct</b> <a href="gas_coin.md#0x3_gas_coin_Delegations">Delegations</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>inner: <a href="">vector</a>&lt;<a href="gas_coin.md#0x3_gas_coin_DelegatedMintCapability">gas_coin::DelegatedMintCapability</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_gas_coin_ErrorNoCapabilities"></a>

Account does not have mint capability


<pre><code><b>const</b> <a href="gas_coin.md#0x3_gas_coin_ErrorNoCapabilities">ErrorNoCapabilities</a>: u64 = 1;
</code></pre>



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


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, genesis_account: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_genesis_init">genesis_init</a>(ctx: &<b>mut</b> StorageContext, genesis_account: &<a href="">signer</a>){
    <b>let</b> (burn_cap, freeze_cap, mint_cap) = <a href="coin.md#0x3_coin_initialize">coin::initialize</a>&lt;<a href="gas_coin.md#0x3_gas_coin_GasCoin">GasCoin</a>&gt;(
        ctx,
        genesis_account,
        <a href="_utf8">string::utf8</a>(b"Rooch Gas Coin"),
        <a href="_utf8">string::utf8</a>(b"RGC"),
        9, // decimals
    );

    // Rooch framework needs mint cap <b>to</b> mint coins <b>to</b> initial validators. This will be revoked once the validators
    // have been initialized.
    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, genesis_account, <a href="gas_coin.md#0x3_gas_coin_MintCapStore">MintCapStore</a> { mint_cap });

    //TODO do we need the cap?
    <a href="coin.md#0x3_coin_destroy_freeze_cap">coin::destroy_freeze_cap</a>(freeze_cap);
    <a href="coin.md#0x3_coin_destroy_mint_cap">coin::destroy_mint_cap</a>(mint_cap);
    <a href="coin.md#0x3_coin_destroy_burn_cap">coin::destroy_burn_cap</a>(burn_cap);
}
</code></pre>



</details>

<a name="0x3_gas_coin_has_mint_capability"></a>

## Function `has_mint_capability`



<pre><code><b>public</b> <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_has_mint_capability">has_mint_capability</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_has_mint_capability">has_mint_capability</a>(ctx: &StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>): bool {
    <a href="_global_exists">account_storage::global_exists</a>&lt;<a href="gas_coin.md#0x3_gas_coin_MintCapStore">MintCapStore</a>&gt;(ctx, <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>))
}
</code></pre>



</details>

<a name="0x3_gas_coin_destroy_mint_cap"></a>

## Function `destroy_mint_cap`

Only called during genesis to destroy the rooch framework account's mint capability once all initial validators
and accounts have been initialized during genesis.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_destroy_mint_cap">destroy_mint_cap</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, rooch_framework: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_coin.md#0x3_gas_coin_destroy_mint_cap">destroy_mint_cap</a>(ctx: &<b>mut</b> StorageContext, rooch_framework: &<a href="">signer</a>) {
    <a href="core_addresses.md#0x3_core_addresses_assert_rooch_framework">core_addresses::assert_rooch_framework</a>(rooch_framework);
    <b>let</b> <a href="gas_coin.md#0x3_gas_coin_MintCapStore">MintCapStore</a> { mint_cap } = <a href="_global_move_from">account_storage::global_move_from</a>&lt;<a href="gas_coin.md#0x3_gas_coin_MintCapStore">MintCapStore</a>&gt;(ctx,@rooch_framework);
    <a href="coin.md#0x3_coin_destroy_mint_cap">coin::destroy_mint_cap</a>(mint_cap);
}
</code></pre>



</details>
