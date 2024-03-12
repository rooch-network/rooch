
<a name="0x3_ethereum_light_client"></a>

# Module `0x3::ethereum_light_client`



-  [Struct `BlockHeader`](#0x3_ethereum_light_client_BlockHeader)
-  [Resource `BlockStore`](#0x3_ethereum_light_client_BlockStore)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_ethereum_light_client_genesis_init)
-  [Function `submit_new_block`](#0x3_ethereum_light_client_submit_new_block)
-  [Function `get_block`](#0x3_ethereum_light_client_get_block)


<pre><code><b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="ethereum_address.md#0x3_ethereum_address">0x3::ethereum_address</a>;
<b>use</b> <a href="timestamp.md#0x3_timestamp">0x3::timestamp</a>;
</code></pre>



<a name="0x3_ethereum_light_client_BlockHeader"></a>

## Struct `BlockHeader`



<pre><code>#[data_struct]
<b>struct</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_BlockHeader">BlockHeader</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_ethereum_light_client_BlockStore"></a>

## Resource `BlockStore`



<pre><code><b>struct</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_BlockStore">BlockStore</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ethereum_light_client_ErrorBlockNotFound"></a>



<pre><code><b>const</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_ErrorBlockNotFound">ErrorBlockNotFound</a>: u64 = 1;
</code></pre>



<a name="0x3_ethereum_light_client_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_genesis_init">genesis_init</a>(genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x3_ethereum_light_client_submit_new_block"></a>

## Function `submit_new_block`

The relay server submit a new Ethereum block to the light client.


<pre><code><b>public</b> entry <b>fun</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_submit_new_block">submit_new_block</a>(block_header_bytes: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_ethereum_light_client_get_block"></a>

## Function `get_block`

Get block via block_number


<pre><code><b>public</b> <b>fun</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_get_block">get_block</a>(block_number: u64): &<a href="ethereum_light_client.md#0x3_ethereum_light_client_BlockHeader">ethereum_light_client::BlockHeader</a>
</code></pre>
