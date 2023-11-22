
<a name="0x3_bitcoin_light_client"></a>

# Module `0x3::bitcoin_light_client`



-  [Resource `BitcoinStore`](#0x3_bitcoin_light_client_BitcoinStore)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_bitcoin_light_client_genesis_init)
-  [Function `submit_new_block`](#0x3_bitcoin_light_client_submit_new_block)
-  [Function `get_block`](#0x3_bitcoin_light_client_get_block)
-  [Function `get_block_height`](#0x3_bitcoin_light_client_get_block_height)
-  [Function `get_block_by_height`](#0x3_bitcoin_light_client_get_block_by_height)
-  [Function `get_latest_block_height`](#0x3_bitcoin_light_client_get_latest_block_height)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="bitcoin_types.md#0x3_bitcoin_types">0x3::bitcoin_types</a>;
<b>use</b> <a href="timestamp.md#0x3_timestamp">0x3::timestamp</a>;
</code></pre>



<a name="0x3_bitcoin_light_client_BitcoinStore"></a>

## Resource `BitcoinStore`



<pre><code><b>struct</b> <a href="bitcoin_light_client.md#0x3_bitcoin_light_client_BitcoinStore">BitcoinStore</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_bitcoin_light_client_ErrorBlockAlreadyProcessed"></a>



<pre><code><b>const</b> <a href="bitcoin_light_client.md#0x3_bitcoin_light_client_ErrorBlockAlreadyProcessed">ErrorBlockAlreadyProcessed</a>: u64 = 2;
</code></pre>



<a name="0x3_bitcoin_light_client_ErrorBlockNotFound"></a>



<pre><code><b>const</b> <a href="bitcoin_light_client.md#0x3_bitcoin_light_client_ErrorBlockNotFound">ErrorBlockNotFound</a>: u64 = 1;
</code></pre>



<a name="0x3_bitcoin_light_client_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bitcoin_light_client.md#0x3_bitcoin_light_client_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, _genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x3_bitcoin_light_client_submit_new_block"></a>

## Function `submit_new_block`

The relay server submit a new Bitcoin block to the light client.


<pre><code>entry <b>fun</b> <a href="bitcoin_light_client.md#0x3_bitcoin_light_client_submit_new_block">submit_new_block</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, btc_store_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="bitcoin_light_client.md#0x3_bitcoin_light_client_BitcoinStore">bitcoin_light_client::BitcoinStore</a>&gt;, block_height: u64, block_hash: <b>address</b>, block_bytes: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_bitcoin_light_client_get_block"></a>

## Function `get_block`

Get block via block_hash


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_light_client.md#0x3_bitcoin_light_client_get_block">get_block</a>(btc_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitcoin_light_client.md#0x3_bitcoin_light_client_BitcoinStore">bitcoin_light_client::BitcoinStore</a>&gt;, block_hash: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="bitcoin_types.md#0x3_bitcoin_types_Header">bitcoin_types::Header</a>&gt;
</code></pre>



<a name="0x3_bitcoin_light_client_get_block_height"></a>

## Function `get_block_height`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_light_client.md#0x3_bitcoin_light_client_get_block_height">get_block_height</a>(btc_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitcoin_light_client.md#0x3_bitcoin_light_client_BitcoinStore">bitcoin_light_client::BitcoinStore</a>&gt;, block_hash: <b>address</b>): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<a name="0x3_bitcoin_light_client_get_block_by_height"></a>

## Function `get_block_by_height`

Get block via block_height


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_light_client.md#0x3_bitcoin_light_client_get_block_by_height">get_block_by_height</a>(btc_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitcoin_light_client.md#0x3_bitcoin_light_client_BitcoinStore">bitcoin_light_client::BitcoinStore</a>&gt;, block_height: u64): <a href="_Option">option::Option</a>&lt;<a href="bitcoin_types.md#0x3_bitcoin_types_Header">bitcoin_types::Header</a>&gt;
</code></pre>



<a name="0x3_bitcoin_light_client_get_latest_block_height"></a>

## Function `get_latest_block_height`

Get block via block_height


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_light_client.md#0x3_bitcoin_light_client_get_latest_block_height">get_latest_block_height</a>(btc_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitcoin_light_client.md#0x3_bitcoin_light_client_BitcoinStore">bitcoin_light_client::BitcoinStore</a>&gt;): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>
