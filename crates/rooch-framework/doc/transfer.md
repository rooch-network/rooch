
<a name="0x3_transfer"></a>

# Module `0x3::transfer`



-  [Function `transfer_coin`](#0x3_transfer_transfer_coin)
-  [Function `transfer_coin_to_multichain_address`](#0x3_transfer_transfer_coin_to_multichain_address)


<pre><code><b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="account.md#0x3_account">0x3::account</a>;
<b>use</b> <a href="account_coin_store.md#0x3_account_coin_store">0x3::account_coin_store</a>;
<b>use</b> <a href="address_mapping.md#0x3_address_mapping">0x3::address_mapping</a>;
<b>use</b> <a href="multichain_address.md#0x3_multichain_address">0x3::multichain_address</a>;
</code></pre>



<a name="0x3_transfer_transfer_coin"></a>

## Function `transfer_coin`

Transfer <code>amount</code> of coins <code>CoinType</code> from <code>from</code> to <code><b>to</b></code>.
This public entry function requires the <code>CoinType</code> to have <code>key</code> and <code>store</code> abilities.


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer.md#0x3_transfer_transfer_coin">transfer_coin</a>&lt;CoinType: store, key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, from: &<a href="">signer</a>, <b>to</b>: <b>address</b>, amount: u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer.md#0x3_transfer_transfer_coin">transfer_coin</a>&lt;CoinType: key + store&gt;(
    ctx: &<b>mut</b> Context,
    from: &<a href="">signer</a>,
    <b>to</b>: <b>address</b>,
    amount: u256,
) {
    <b>if</b>(!<a href="account.md#0x3_account_exists_at">account::exists_at</a>(ctx, <b>to</b>)) {
        <a href="account.md#0x3_account_create_account">account::create_account</a>(ctx, <b>to</b>);
    };

    <a href="account_coin_store.md#0x3_account_coin_store_transfer">account_coin_store::transfer</a>&lt;CoinType&gt;(ctx, from, <b>to</b>, amount)
}
</code></pre>



</details>

<a name="0x3_transfer_transfer_coin_to_multichain_address"></a>

## Function `transfer_coin_to_multichain_address`

Transfer <code>amount</code> of coins <code>CoinType</code> from <code>from</code> to a MultiChainAddress.
The MultiChainAddress is represented by <code>multichain_id</code> and <code>raw_address</code>.
This public entry function requires the <code>CoinType</code> to have <code>key</code> and <code>store</code> abilities.


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer.md#0x3_transfer_transfer_coin_to_multichain_address">transfer_coin_to_multichain_address</a>&lt;CoinType: store, key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, from: &<a href="">signer</a>, multichain_id: u64, raw_address: <a href="">vector</a>&lt;u8&gt;, amount: u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer.md#0x3_transfer_transfer_coin_to_multichain_address">transfer_coin_to_multichain_address</a>&lt;CoinType: key + store&gt;(
    ctx: &<b>mut</b> Context,
    from: &<a href="">signer</a>,
    multichain_id: u64,
    raw_address: <a href="">vector</a>&lt;u8&gt;,
    amount: u256,
) {
    <b>let</b> maddress = <a href="multichain_address.md#0x3_multichain_address_new">multichain_address::new</a>(multichain_id, raw_address);
    <b>let</b> <b>to</b> = <a href="address_mapping.md#0x3_address_mapping_resolve_or_generate">address_mapping::resolve_or_generate</a>(ctx, maddress);
    <b>if</b>(!<a href="account.md#0x3_account_exists_at">account::exists_at</a>(ctx, <b>to</b>)) {
        <a href="account.md#0x3_account_create_account">account::create_account</a>(ctx, <b>to</b>);
        <a href="address_mapping.md#0x3_address_mapping_bind_no_check">address_mapping::bind_no_check</a>(ctx, <b>to</b>, maddress);
    };
    <a href="account_coin_store.md#0x3_account_coin_store_transfer">account_coin_store::transfer</a>&lt;CoinType&gt;(ctx, from, <b>to</b>, amount)
}
</code></pre>



</details>
