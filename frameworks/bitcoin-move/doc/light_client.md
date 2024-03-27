
<a name="0x4_light_client"></a>

# Module `0x4::light_client`



-  [Struct `TxProgressErrorLogEvent`](#0x4_light_client_TxProgressErrorLogEvent)
-  [Resource `BitcoinBlockStore`](#0x4_light_client_BitcoinBlockStore)
-  [Resource `BitcoinUTXOStore`](#0x4_light_client_BitcoinUTXOStore)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_light_client_genesis_init)
-  [Function `submit_new_block`](#0x4_light_client_submit_new_block)
-  [Function `remaining_tx_count`](#0x4_light_client_remaining_tx_count)
-  [Function `process_utxos`](#0x4_light_client_process_utxos)
-  [Function `txs`](#0x4_light_client_txs)
-  [Function `tx_ids`](#0x4_light_client_tx_ids)
-  [Function `get_tx`](#0x4_light_client_get_tx)
-  [Function `get_block`](#0x4_light_client_get_block)
-  [Function `get_block_height`](#0x4_light_client_get_block_height)
-  [Function `get_block_by_height`](#0x4_light_client_get_block_by_height)
-  [Function `get_latest_block_height`](#0x4_light_client_get_latest_block_height)
-  [Function `get_utxo`](#0x4_light_client_get_utxo)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::simple_multimap</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::table_vec</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="">0x3::bitcoin_address</a>;
<b>use</b> <a href="">0x3::timestamp</a>;
<b>use</b> <a href="ord.md#0x4_ord">0x4::ord</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
<b>use</b> <a href="utxo.md#0x4_utxo">0x4::utxo</a>;
</code></pre>



<a name="0x4_light_client_TxProgressErrorLogEvent"></a>

## Struct `TxProgressErrorLogEvent`



<pre><code><b>struct</b> <a href="light_client.md#0x4_light_client_TxProgressErrorLogEvent">TxProgressErrorLogEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x4_light_client_BitcoinBlockStore"></a>

## Resource `BitcoinBlockStore`



<pre><code><b>struct</b> <a href="light_client.md#0x4_light_client_BitcoinBlockStore">BitcoinBlockStore</a> <b>has</b> key
</code></pre>



<a name="0x4_light_client_BitcoinUTXOStore"></a>

## Resource `BitcoinUTXOStore`



<pre><code><b>struct</b> <a href="light_client.md#0x4_light_client_BitcoinUTXOStore">BitcoinUTXOStore</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_light_client_ErrorBlockNotFound"></a>



<pre><code><b>const</b> <a href="light_client.md#0x4_light_client_ErrorBlockNotFound">ErrorBlockNotFound</a>: u64 = 1;
</code></pre>



<a name="0x4_light_client_ErrorBlockAlreadyProcessed"></a>



<pre><code><b>const</b> <a href="light_client.md#0x4_light_client_ErrorBlockAlreadyProcessed">ErrorBlockAlreadyProcessed</a>: u64 = 2;
</code></pre>



<a name="0x4_light_client_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="light_client.md#0x4_light_client_genesis_init">genesis_init</a>(_genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x4_light_client_submit_new_block"></a>

## Function `submit_new_block`

The relay server submit a new Bitcoin block to the light client.


<pre><code>entry <b>fun</b> <a href="light_client.md#0x4_light_client_submit_new_block">submit_new_block</a>(btc_block_store_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinBlockStore">light_client::BitcoinBlockStore</a>&gt;, block_height: u64, block_hash: <b>address</b>, block_bytes: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x4_light_client_remaining_tx_count"></a>

## Function `remaining_tx_count`



<pre><code><b>public</b> <b>fun</b> <a href="light_client.md#0x4_light_client_remaining_tx_count">remaining_tx_count</a>(btc_block_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinBlockStore">light_client::BitcoinBlockStore</a>&gt;, btc_utxo_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinUTXOStore">light_client::BitcoinUTXOStore</a>&gt;): u64
</code></pre>



<a name="0x4_light_client_process_utxos"></a>

## Function `process_utxos`



<pre><code>entry <b>fun</b> <a href="light_client.md#0x4_light_client_process_utxos">process_utxos</a>(btc_block_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinBlockStore">light_client::BitcoinBlockStore</a>&gt;, btc_utxo_store_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinUTXOStore">light_client::BitcoinUTXOStore</a>&gt;, batch_size: u64)
</code></pre>



<a name="0x4_light_client_txs"></a>

## Function `txs`



<pre><code><b>public</b> <b>fun</b> <a href="light_client.md#0x4_light_client_txs">txs</a>(btc_block_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinBlockStore">light_client::BitcoinBlockStore</a>&gt;): &<a href="_Table">table::Table</a>&lt;<b>address</b>, <a href="types.md#0x4_types_Transaction">types::Transaction</a>&gt;
</code></pre>



<a name="0x4_light_client_tx_ids"></a>

## Function `tx_ids`



<pre><code><b>public</b> <b>fun</b> <a href="light_client.md#0x4_light_client_tx_ids">tx_ids</a>(btc_block_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinBlockStore">light_client::BitcoinBlockStore</a>&gt;): &<a href="_TableVec">table_vec::TableVec</a>&lt;<b>address</b>&gt;
</code></pre>



<a name="0x4_light_client_get_tx"></a>

## Function `get_tx`



<pre><code><b>public</b> <b>fun</b> <a href="light_client.md#0x4_light_client_get_tx">get_tx</a>(btc_block_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinBlockStore">light_client::BitcoinBlockStore</a>&gt;, txid: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="types.md#0x4_types_Transaction">types::Transaction</a>&gt;
</code></pre>



<a name="0x4_light_client_get_block"></a>

## Function `get_block`

Get block via block_hash


<pre><code><b>public</b> <b>fun</b> <a href="light_client.md#0x4_light_client_get_block">get_block</a>(btc_block_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinBlockStore">light_client::BitcoinBlockStore</a>&gt;, block_hash: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="types.md#0x4_types_Header">types::Header</a>&gt;
</code></pre>



<a name="0x4_light_client_get_block_height"></a>

## Function `get_block_height`



<pre><code><b>public</b> <b>fun</b> <a href="light_client.md#0x4_light_client_get_block_height">get_block_height</a>(btc_block_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinBlockStore">light_client::BitcoinBlockStore</a>&gt;, block_hash: <b>address</b>): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<a name="0x4_light_client_get_block_by_height"></a>

## Function `get_block_by_height`

Get block via block_height


<pre><code><b>public</b> <b>fun</b> <a href="light_client.md#0x4_light_client_get_block_by_height">get_block_by_height</a>(btc_block_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinBlockStore">light_client::BitcoinBlockStore</a>&gt;, block_height: u64): <a href="_Option">option::Option</a>&lt;<a href="types.md#0x4_types_Header">types::Header</a>&gt;
</code></pre>



<a name="0x4_light_client_get_latest_block_height"></a>

## Function `get_latest_block_height`

Get latest block height


<pre><code><b>public</b> <b>fun</b> <a href="light_client.md#0x4_light_client_get_latest_block_height">get_latest_block_height</a>(btc_block_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinBlockStore">light_client::BitcoinBlockStore</a>&gt;): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<a name="0x4_light_client_get_utxo"></a>

## Function `get_utxo`

Get UTXO via txid and vout


<pre><code><b>public</b> <b>fun</b> <a href="light_client.md#0x4_light_client_get_utxo">get_utxo</a>(btc_utxo_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="light_client.md#0x4_light_client_BitcoinUTXOStore">light_client::BitcoinUTXOStore</a>&gt;, txid: <b>address</b>, vout: u32): <a href="_Option">option::Option</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>
