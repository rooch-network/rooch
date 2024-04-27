
<a name="0x3_chain_id"></a>

# Module `0x3::chain_id`



-  [Resource `ChainID`](#0x3_chain_id_ChainID)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_chain_id_genesis_init)
-  [Function `id`](#0x3_chain_id_id)
-  [Function `borrow`](#0x3_chain_id_borrow)
-  [Function `chain_id`](#0x3_chain_id_chain_id)
-  [Function `is_local`](#0x3_chain_id_is_local)
-  [Function `is_dev`](#0x3_chain_id_is_dev)
-  [Function `is_test`](#0x3_chain_id_is_test)
-  [Function `is_main`](#0x3_chain_id_is_main)


<pre><code><b>use</b> <a href="">0x2::object</a>;
</code></pre>



<a name="0x3_chain_id_ChainID"></a>

## Resource `ChainID`

The ChainID in the global storage


<pre><code><b>struct</b> <a href="chain_id.md#0x3_chain_id_ChainID">ChainID</a> <b>has</b> store, key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_chain_id_CHAIN_ID_DEV"></a>



<pre><code><b>const</b> <a href="chain_id.md#0x3_chain_id_CHAIN_ID_DEV">CHAIN_ID_DEV</a>: u64 = 3;
</code></pre>



<a name="0x3_chain_id_CHAIN_ID_LOCAL"></a>



<pre><code><b>const</b> <a href="chain_id.md#0x3_chain_id_CHAIN_ID_LOCAL">CHAIN_ID_LOCAL</a>: u64 = 4;
</code></pre>



<a name="0x3_chain_id_CHAIN_ID_MAIN"></a>



<pre><code><b>const</b> <a href="chain_id.md#0x3_chain_id_CHAIN_ID_MAIN">CHAIN_ID_MAIN</a>: u64 = 1;
</code></pre>



<a name="0x3_chain_id_CHAIN_ID_TEST"></a>



<pre><code><b>const</b> <a href="chain_id.md#0x3_chain_id_CHAIN_ID_TEST">CHAIN_ID_TEST</a>: u64 = 2;
</code></pre>



<a name="0x3_chain_id_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="chain_id.md#0x3_chain_id_genesis_init">genesis_init</a>(_genesis_account: &<a href="">signer</a>, <a href="chain_id.md#0x3_chain_id">chain_id</a>: u64)
</code></pre>



<a name="0x3_chain_id_id"></a>

## Function `id`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_id">id</a>(self: &<a href="chain_id.md#0x3_chain_id_ChainID">chain_id::ChainID</a>): u64
</code></pre>



<a name="0x3_chain_id_borrow"></a>

## Function `borrow`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_borrow">borrow</a>(): &<a href="chain_id.md#0x3_chain_id_ChainID">chain_id::ChainID</a>
</code></pre>



<a name="0x3_chain_id_chain_id"></a>

## Function `chain_id`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id">chain_id</a>(): u64
</code></pre>



<a name="0x3_chain_id_is_local"></a>

## Function `is_local`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_is_local">is_local</a>(): bool
</code></pre>



<a name="0x3_chain_id_is_dev"></a>

## Function `is_dev`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_is_dev">is_dev</a>(): bool
</code></pre>



<a name="0x3_chain_id_is_test"></a>

## Function `is_test`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_is_test">is_test</a>(): bool
</code></pre>



<a name="0x3_chain_id_is_main"></a>

## Function `is_main`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_is_main">is_main</a>(): bool
</code></pre>
