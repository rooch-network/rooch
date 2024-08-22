
<a name="0x4_bitcoin"></a>

# Module `0x4::bitcoin`



-  [Struct `TxProgressErrorLogEvent`](#0x4_bitcoin_TxProgressErrorLogEvent)
-  [Struct `RepeatCoinbaseTxEvent`](#0x4_bitcoin_RepeatCoinbaseTxEvent)
-  [Resource `BitcoinBlockStore`](#0x4_bitcoin_BitcoinBlockStore)
-  [Struct `TransferUTXOEvent`](#0x4_bitcoin_TransferUTXOEvent)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_bitcoin_genesis_init)
-  [Function `get_tx`](#0x4_bitcoin_get_tx)
-  [Function `get_tx_height`](#0x4_bitcoin_get_tx_height)
-  [Function `get_block`](#0x4_bitcoin_get_block)
-  [Function `get_block_height`](#0x4_bitcoin_get_block_height)
-  [Function `get_block_hash_by_height`](#0x4_bitcoin_get_block_hash_by_height)
-  [Function `get_block_by_height`](#0x4_bitcoin_get_block_by_height)
-  [Function `get_genesis_block`](#0x4_bitcoin_get_genesis_block)
-  [Function `get_latest_block`](#0x4_bitcoin_get_latest_block)
-  [Function `get_bitcoin_time`](#0x4_bitcoin_get_bitcoin_time)
-  [Function `contains_header`](#0x4_bitcoin_contains_header)


<pre><code><b>use</b> <a href="">0x1::debug</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::address</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::event_queue</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::simple_multimap</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::table_vec</a>;
<b>use</b> <a href="">0x2::timestamp</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="">0x3::address_mapping</a>;
<b>use</b> <a href="">0x3::bitcoin_address</a>;
<b>use</b> <a href="">0x3::chain_id</a>;
<b>use</b> <a href="network.md#0x4_network">0x4::network</a>;
<b>use</b> <a href="ord.md#0x4_ord">0x4::ord</a>;
<b>use</b> <a href="pending_block.md#0x4_pending_block">0x4::pending_block</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
<b>use</b> <a href="utxo.md#0x4_utxo">0x4::utxo</a>;
</code></pre>



<a name="0x4_bitcoin_TxProgressErrorLogEvent"></a>

## Struct `TxProgressErrorLogEvent`



<pre><code><b>struct</b> <a href="bitcoin.md#0x4_bitcoin_TxProgressErrorLogEvent">TxProgressErrorLogEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x4_bitcoin_RepeatCoinbaseTxEvent"></a>

## Struct `RepeatCoinbaseTxEvent`



<pre><code><b>struct</b> <a href="bitcoin.md#0x4_bitcoin_RepeatCoinbaseTxEvent">RepeatCoinbaseTxEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x4_bitcoin_BitcoinBlockStore"></a>

## Resource `BitcoinBlockStore`



<pre><code><b>struct</b> <a href="bitcoin.md#0x4_bitcoin_BitcoinBlockStore">BitcoinBlockStore</a> <b>has</b> key
</code></pre>



<a name="0x4_bitcoin_TransferUTXOEvent"></a>

## Struct `TransferUTXOEvent`



<pre><code><b>struct</b> <a href="bitcoin.md#0x4_bitcoin_TransferUTXOEvent">TransferUTXOEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_bitcoin_ErrorBlockAlreadyProcessed"></a>



<pre><code><b>const</b> <a href="bitcoin.md#0x4_bitcoin_ErrorBlockAlreadyProcessed">ErrorBlockAlreadyProcessed</a>: u64 = 2;
</code></pre>



<a name="0x4_bitcoin_BIP_34_HEIGHT"></a>

https://github.com/bitcoin/bips/blob/master/bip-0034.mediawiki


<pre><code><b>const</b> <a href="bitcoin.md#0x4_bitcoin_BIP_34_HEIGHT">BIP_34_HEIGHT</a>: u64 = 227835;
</code></pre>



<a name="0x4_bitcoin_ErrorBlockProcessError"></a>

If the process block failed, we need to stop the system and fix the issue


<pre><code><b>const</b> <a href="bitcoin.md#0x4_bitcoin_ErrorBlockProcessError">ErrorBlockProcessError</a>: u64 = 1;
</code></pre>



<a name="0x4_bitcoin_ErrorReorgTooDeep"></a>

The reorg is too deep, we need to stop the system and fix the issue


<pre><code><b>const</b> <a href="bitcoin.md#0x4_bitcoin_ErrorReorgTooDeep">ErrorReorgTooDeep</a>: u64 = 3;
</code></pre>



<a name="0x4_bitcoin_ORDINAL_GENESIS_HEIGHT"></a>



<pre><code><b>const</b> <a href="bitcoin.md#0x4_bitcoin_ORDINAL_GENESIS_HEIGHT">ORDINAL_GENESIS_HEIGHT</a>: u64 = 767430;
</code></pre>



<a name="0x4_bitcoin_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bitcoin.md#0x4_bitcoin_genesis_init">genesis_init</a>(_genesis_account: &<a href="">signer</a>, genesis_block_height: u64, genesis_block_hash: <b>address</b>)
</code></pre>



<a name="0x4_bitcoin_get_tx"></a>

## Function `get_tx`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin.md#0x4_bitcoin_get_tx">get_tx</a>(txid: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="types.md#0x4_types_Transaction">types::Transaction</a>&gt;
</code></pre>



<a name="0x4_bitcoin_get_tx_height"></a>

## Function `get_tx_height`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin.md#0x4_bitcoin_get_tx_height">get_tx_height</a>(txid: <b>address</b>): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<a name="0x4_bitcoin_get_block"></a>

## Function `get_block`

Get block via block_hash


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin.md#0x4_bitcoin_get_block">get_block</a>(block_hash: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="types.md#0x4_types_Header">types::Header</a>&gt;
</code></pre>



<a name="0x4_bitcoin_get_block_height"></a>

## Function `get_block_height`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin.md#0x4_bitcoin_get_block_height">get_block_height</a>(block_hash: <b>address</b>): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<a name="0x4_bitcoin_get_block_hash_by_height"></a>

## Function `get_block_hash_by_height`

Get block hash via block_height


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin.md#0x4_bitcoin_get_block_hash_by_height">get_block_hash_by_height</a>(block_height: u64): <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;
</code></pre>



<a name="0x4_bitcoin_get_block_by_height"></a>

## Function `get_block_by_height`

Get block via block_height


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin.md#0x4_bitcoin_get_block_by_height">get_block_by_height</a>(block_height: u64): <a href="_Option">option::Option</a>&lt;<a href="types.md#0x4_types_Header">types::Header</a>&gt;
</code></pre>



<a name="0x4_bitcoin_get_genesis_block"></a>

## Function `get_genesis_block`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin.md#0x4_bitcoin_get_genesis_block">get_genesis_block</a>(): <a href="types.md#0x4_types_BlockHeightHash">types::BlockHeightHash</a>
</code></pre>



<a name="0x4_bitcoin_get_latest_block"></a>

## Function `get_latest_block`

Get latest block height


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin.md#0x4_bitcoin_get_latest_block">get_latest_block</a>(): <a href="_Option">option::Option</a>&lt;<a href="types.md#0x4_types_BlockHeightHash">types::BlockHeightHash</a>&gt;
</code></pre>



<a name="0x4_bitcoin_get_bitcoin_time"></a>

## Function `get_bitcoin_time`

Get the bitcoin time in seconds


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin.md#0x4_bitcoin_get_bitcoin_time">get_bitcoin_time</a>(): u32
</code></pre>



<a name="0x4_bitcoin_contains_header"></a>

## Function `contains_header`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin.md#0x4_bitcoin_contains_header">contains_header</a>(block_header: &<a href="types.md#0x4_types_Header">types::Header</a>): bool
</code></pre>
