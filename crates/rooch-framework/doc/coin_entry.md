
<a name="0x3_coin_entry"></a>

# Module `0x3::coin_entry`

CoinEntry is built to make a simple walkthrough of the Coins module.
It contains scripts you will need to initialize, mint, burn, transfer coins.
By utilizing this current module, a developer can create his own coin and care less about mint and burn capabilities,


-  [Resource `Capabilities`](#0x3_coin_entry_Capabilities)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x3_coin_entry_initialize)
-  [Function `mint`](#0x3_coin_entry_mint)
-  [Function `burn`](#0x3_coin_entry_burn)
-  [Function `accept_coin`](#0x3_coin_entry_accept_coin)
-  [Function `enable_auto_accept_coin`](#0x3_coin_entry_enable_auto_accept_coin)
-  [Function `disable_auto_accept_coin`](#0x3_coin_entry_disable_auto_accept_coin)
-  [Function `transfer`](#0x3_coin_entry_transfer)
-  [Function `freeze_coin_store`](#0x3_coin_entry_freeze_coin_store)
-  [Function `unfreeze_coin_store`](#0x3_coin_entry_unfreeze_coin_store)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="account.md#0x3_account">0x3::account</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
</code></pre>



<a name="0x3_coin_entry_Capabilities"></a>

## Resource `Capabilities`

Capabilities resource storing mint and burn capabilities.
The resource is stored on the account that initialized coin <code>CoinType</code>.


<pre><code><b>struct</b> <a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>burn_cap: <a href="coin.md#0x3_coin_BurnCapability">coin::BurnCapability</a>&lt;CoinType&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>freeze_cap: <a href="coin.md#0x3_coin_FreezeCapability">coin::FreezeCapability</a>&lt;CoinType&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>mint_cap: <a href="coin.md#0x3_coin_MintCapability">coin::MintCapability</a>&lt;CoinType&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_coin_entry_ENoCapabilities"></a>

account has no capabilities (burn/mint).


<pre><code><b>const</b> <a href="coin_entry.md#0x3_coin_entry_ENoCapabilities">ENoCapabilities</a>: u64 = 1;
</code></pre>



<a name="0x3_coin_entry_initialize"></a>

## Function `initialize`

Initialize new coin <code>CoinType</code> in Rooch Blockchain.
Mint and Burn Capabilities will be stored under <code><a href="account.md#0x3_account">account</a></code> in <code><a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a></code> resource.


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_initialize">initialize</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, name: <a href="">vector</a>&lt;u8&gt;, symbol: <a href="">vector</a>&lt;u8&gt;, decimals: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_initialize">initialize</a>&lt;CoinType&gt;(
    ctx: &<b>mut</b> StorageContext,
    <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>,
    name: <a href="">vector</a>&lt;u8&gt;,
    symbol: <a href="">vector</a>&lt;u8&gt;,
    decimals: u8,
) {
    <b>let</b> (burn_cap, freeze_cap, mint_cap) = <a href="coin.md#0x3_coin_initialize">coin::initialize</a>&lt;CoinType&gt;(
        ctx,
        <a href="account.md#0x3_account">account</a>,
        <a href="_utf8">string::utf8</a>(name),
        <a href="_utf8">string::utf8</a>(symbol),
        decimals,
    );

    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, <a href="account.md#0x3_account">account</a>, <a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt; {
        burn_cap,
        freeze_cap,
        mint_cap
    });
}
</code></pre>



</details>

<a name="0x3_coin_entry_mint"></a>

## Function `mint`

Create new coins <code>CoinType</code> and deposit them into dst_addr's account.


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_mint">mint</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, dst_addr: <b>address</b>, amount: u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_mint">mint</a>&lt;CoinType&gt;(
    ctx: &<b>mut</b> StorageContext,
    <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>,
    dst_addr: <b>address</b>,
    amount: u256,
) {
    <b>let</b> account_addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);

    <b>assert</b>!(
        // <b>exists</b>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(account_addr),
        <a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, account_addr),
        <a href="_not_found">error::not_found</a>(<a href="coin_entry.md#0x3_coin_entry_ENoCapabilities">ENoCapabilities</a>),
    );

    <b>let</b> cap = <a href="_global_move_from">account_storage::global_move_from</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, account_addr);
    // <b>let</b> cap = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, account_addr);
    <b>let</b> coins_minted = <a href="coin.md#0x3_coin_mint">coin::mint</a>(ctx, amount, &cap.mint_cap);
    <a href="account.md#0x3_account_deposit">account::deposit</a>(ctx, dst_addr, coins_minted);
    <a href="_global_move_to">account_storage::global_move_to</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, <a href="account.md#0x3_account">account</a>, cap)
}
</code></pre>



</details>

<a name="0x3_coin_entry_burn"></a>

## Function `burn`

Withdraw an <code>amount</code> of coin <code>CoinType</code> from <code><a href="account.md#0x3_account">account</a></code> and burn it.


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_burn">burn</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, amount: u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_burn">burn</a>&lt;CoinType&gt;(
    ctx: &<b>mut</b> StorageContext,
    <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>,
    amount: u256,
) {
    <b>let</b> account_addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);

    <b>assert</b>!(
        <a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, account_addr),
        <a href="_not_found">error::not_found</a>(<a href="coin_entry.md#0x3_coin_entry_ENoCapabilities">ENoCapabilities</a>),
    );

    // <b>let</b> cap = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, account_addr);
    <b>let</b> cap = <a href="_global_move_from">account_storage::global_move_from</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, account_addr);
    <b>let</b> to_burn = <a href="account.md#0x3_account_withdraw">account::withdraw</a>&lt;CoinType&gt;(ctx, <a href="account.md#0x3_account">account</a>, amount);
    // <b>let</b> burn_cap = borrow_burn_cap&lt;CoinType&gt;(ctx, account_addr);
    <a href="coin.md#0x3_coin_burn">coin::burn</a>&lt;CoinType&gt;(ctx, to_burn, &cap.burn_cap);
    <a href="_global_move_to">account_storage::global_move_to</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, <a href="account.md#0x3_account">account</a>, cap);
}
</code></pre>



</details>

<a name="0x3_coin_entry_accept_coin"></a>

## Function `accept_coin`

Creating a resource that stores balance of <code>CoinType</code> on user's account.
Required if user wants to start accepting deposits of <code>CoinType</code> in his account.


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_accept_coin">accept_coin</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_accept_coin">accept_coin</a>&lt;CoinType&gt;(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="account.md#0x3_account_do_accept_coin">account::do_accept_coin</a>&lt;CoinType&gt;(ctx, <a href="account.md#0x3_account">account</a>)
}
</code></pre>



</details>

<a name="0x3_coin_entry_enable_auto_accept_coin"></a>

## Function `enable_auto_accept_coin`

Enable account's auto-accept-coin feature.
The script function is reenterable.


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_enable_auto_accept_coin">enable_auto_accept_coin</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_enable_auto_accept_coin">enable_auto_accept_coin</a>(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="account.md#0x3_account_set_auto_accept_coin">account::set_auto_accept_coin</a>(ctx, <a href="account.md#0x3_account">account</a>, <b>true</b>)
}
</code></pre>



</details>

<a name="0x3_coin_entry_disable_auto_accept_coin"></a>

## Function `disable_auto_accept_coin`

Disable account's auto-accept-coin feature.
The script function is reenterable.


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_disable_auto_accept_coin">disable_auto_accept_coin</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_disable_auto_accept_coin">disable_auto_accept_coin</a>(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="account.md#0x3_account_set_auto_accept_coin">account::set_auto_accept_coin</a>(ctx, <a href="account.md#0x3_account">account</a>, <b>false</b>);
}
</code></pre>



</details>

<a name="0x3_coin_entry_transfer"></a>

## Function `transfer`

Transfer <code>amount</code> of coins <code>CoinType</code> from <code>from</code> to <code><b>to</b></code>.


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_transfer">transfer</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, from: &<a href="">signer</a>, <b>to</b>: <b>address</b>, amount: u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_transfer">transfer</a>&lt;CoinType&gt;(
    ctx: &<b>mut</b> StorageContext,
    from: &<a href="">signer</a>,
    <b>to</b>: <b>address</b>,
    amount: u256,
) {
    <a href="account.md#0x3_account_transfer">account::transfer</a>&lt;CoinType&gt;(ctx, from, <b>to</b>, amount)
}
</code></pre>



</details>

<a name="0x3_coin_entry_freeze_coin_store"></a>

## Function `freeze_coin_store`

Freeze a CoinStore to prevent transfers


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_freeze_coin_store">freeze_coin_store</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_freeze_coin_store">freeze_coin_store</a>&lt;CoinType&gt;(
    ctx: &<b>mut</b> StorageContext,
    <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>
) {
    <b>let</b> account_addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    <b>assert</b>!(
        <a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, account_addr),
        <a href="_not_found">error::not_found</a>(<a href="coin_entry.md#0x3_coin_entry_ENoCapabilities">ENoCapabilities</a>),
    );
    // <b>let</b> cap = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, account_addr);
    <b>let</b> cap = <a href="_global_move_from">account_storage::global_move_from</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, account_addr);
    <a href="coin.md#0x3_coin_freeze_coin_store">coin::freeze_coin_store</a>(ctx, account_addr, &cap.freeze_cap);
    <a href="_global_move_to">account_storage::global_move_to</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, <a href="account.md#0x3_account">account</a>, cap)
}
</code></pre>



</details>

<a name="0x3_coin_entry_unfreeze_coin_store"></a>

## Function `unfreeze_coin_store`

Unfreeze a CoinStore to allow transfers


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_unfreeze_coin_store">unfreeze_coin_store</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_unfreeze_coin_store">unfreeze_coin_store</a>&lt;CoinType&gt;(
    ctx: &<b>mut</b> StorageContext,
    <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>
) {
    <b>let</b> account_addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    <b>assert</b>!(
        <a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, account_addr),
        <a href="_not_found">error::not_found</a>(<a href="coin_entry.md#0x3_coin_entry_ENoCapabilities">ENoCapabilities</a>),
    );
    <b>let</b> cap = <a href="_global_move_from">account_storage::global_move_from</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, account_addr);
    // <b>let</b> cap = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, account_addr);
    <a href="coin.md#0x3_coin_unfreeze_coin_store">coin::unfreeze_coin_store</a>(ctx, account_addr, &cap.freeze_cap);
    <a href="_global_move_to">account_storage::global_move_to</a>&lt;<a href="coin_entry.md#0x3_coin_entry_Capabilities">Capabilities</a>&lt;CoinType&gt;&gt;(ctx, <a href="account.md#0x3_account">account</a>, cap)
}
</code></pre>



</details>
