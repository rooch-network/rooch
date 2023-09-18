
<a name="0x3_coin_entry"></a>

# Module `0x3::coin_entry`



-  [Function `transfer_entry`](#0x3_coin_entry_transfer_entry)


<pre><code><b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="account.md#0x3_account">0x3::account</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
</code></pre>



<a name="0x3_coin_entry_transfer_entry"></a>

## Function `transfer_entry`

Transfer <code>amount</code> of coins <code>CoinType</code> from <code>from</code> to <code><b>to</b></code>.
This public entry function requires the <code>CoinType</code> to have <code>key</code> and <code>store</code> abilities.


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_transfer_entry">transfer_entry</a>&lt;CoinType: store, key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, from: &<a href="">signer</a>, <b>to</b>: <b>address</b>, amount: u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_entry.md#0x3_coin_entry_transfer_entry">transfer_entry</a>&lt;CoinType: key + store&gt;(
    ctx: &<b>mut</b> StorageContext,
    from: &<a href="">signer</a>,
    <b>to</b>: <b>address</b>,
    amount: u256,
) {
    <b>if</b>(!<a href="account.md#0x3_account_exists_at">account::exists_at</a>(ctx, <b>to</b>)) {
        <a href="account.md#0x3_account_create_account">account::create_account</a>(ctx, <b>to</b>);
    };

    <a href="coin.md#0x3_coin_transfer">coin::transfer</a>&lt;CoinType&gt;(ctx, from, <b>to</b>, amount)
}
</code></pre>



</details>
