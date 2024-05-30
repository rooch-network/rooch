
<a name="0x3_transfer"></a>

# Module `0x3::transfer`



-  [Constants](#@Constants_0)
-  [Function `transfer_coin`](#0x3_transfer_transfer_coin)
-  [Function `transfer_coin_to_multichain_address`](#0x3_transfer_transfer_coin_to_multichain_address)
-  [Function `transfer_object`](#0x3_transfer_transfer_object)


<pre><code><b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="account_coin_store.md#0x3_account_coin_store">0x3::account_coin_store</a>;
<b>use</b> <a href="address_mapping.md#0x3_address_mapping">0x3::address_mapping</a>;
<b>use</b> <a href="multichain_address.md#0x3_multichain_address">0x3::multichain_address</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_transfer_ErrorAccountNotExists"></a>



<pre><code><b>const</b> <a href="transfer.md#0x3_transfer_ErrorAccountNotExists">ErrorAccountNotExists</a>: u64 = 1;
</code></pre>



<a name="0x3_transfer_transfer_coin"></a>

## Function `transfer_coin`

Transfer <code>amount</code> of coins <code>CoinType</code> from <code>from</code> to <code><b>to</b></code>.
This public entry function requires the <code>CoinType</code> to have <code>key</code> and <code>store</code> abilities.


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer.md#0x3_transfer_transfer_coin">transfer_coin</a>&lt;CoinType: store, key&gt;(from: &<a href="">signer</a>, <b>to</b>: <b>address</b>, amount: u256)
</code></pre>



<a name="0x3_transfer_transfer_coin_to_multichain_address"></a>

## Function `transfer_coin_to_multichain_address`

Transfer <code>amount</code> of coins <code>CoinType</code> from <code>from</code> to a MultiChainAddress.
The MultiChainAddress is represented by <code>multichain_id</code> and <code>raw_address</code>.
This public entry function requires the <code>CoinType</code> to have <code>key</code> and <code>store</code> abilities.


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer.md#0x3_transfer_transfer_coin_to_multichain_address">transfer_coin_to_multichain_address</a>&lt;CoinType: store, key&gt;(from: &<a href="">signer</a>, multichain_id: u64, raw_address: <a href="">vector</a>&lt;u8&gt;, amount: u256)
</code></pre>



<a name="0x3_transfer_transfer_object"></a>

## Function `transfer_object`

Transfer <code>from</code> owned <code>Object&lt;T&gt;</code> to <code><b>to</b></code> account.
TODO: Currently, we can not pass the <code>Object&lt;T&gt;</code> argument by value, so, we use <code>ObjectID</code> instead.
After the <code>Object&lt;T&gt;</code> argument can be passed by value, we should change the argument type to <code>Object&lt;T&gt;</code>.


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer.md#0x3_transfer_transfer_object">transfer_object</a>&lt;T: store, key&gt;(from: &<a href="">signer</a>, <b>to</b>: <b>address</b>, object_id: <a href="_ObjectID">object::ObjectID</a>)
</code></pre>
