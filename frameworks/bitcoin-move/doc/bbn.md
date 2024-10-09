
<a name="0x4_bbn"></a>

# Module `0x4::bbn`



-  [Resource `BBNGlobalParam`](#0x4_bbn_BBNGlobalParam)
-  [Resource `BBNGlobalParams`](#0x4_bbn_BBNGlobalParams)
-  [Struct `BBNOpReturnData`](#0x4_bbn_BBNOpReturnData)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_bbn_genesis_init)
-  [Function `try_get_bbn_op_return_data`](#0x4_bbn_try_get_bbn_op_return_data)
-  [Function `try_get_staking_output`](#0x4_bbn_try_get_staking_output)
-  [Function `derive_bbn_utxo`](#0x4_bbn_derive_bbn_utxo)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x3::bitcoin_address</a>;
<b>use</b> <a href="bitcoin.md#0x4_bitcoin">0x4::bitcoin</a>;
<b>use</b> <a href="script_buf.md#0x4_script_buf">0x4::script_buf</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
<b>use</b> <a href="utxo.md#0x4_utxo">0x4::utxo</a>;
</code></pre>



<a name="0x4_bbn_BBNGlobalParam"></a>

## Resource `BBNGlobalParam`



<pre><code><b>struct</b> <a href="bbn.md#0x4_bbn_BBNGlobalParam">BBNGlobalParam</a> <b>has</b> store, key
</code></pre>



<a name="0x4_bbn_BBNGlobalParams"></a>

## Resource `BBNGlobalParams`



<pre><code><b>struct</b> <a href="bbn.md#0x4_bbn_BBNGlobalParams">BBNGlobalParams</a> <b>has</b> key
</code></pre>



<a name="0x4_bbn_BBNOpReturnData"></a>

## Struct `BBNOpReturnData`



<pre><code><b>struct</b> <a href="bbn.md#0x4_bbn_BBNOpReturnData">BBNOpReturnData</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_bbn_ErrorNotBabylonOpReturn"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorNotBabylonOpReturn">ErrorNotBabylonOpReturn</a>: u64 = 2;
</code></pre>



<a name="0x4_bbn_ErrorNotBabylonUTXO"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorNotBabylonUTXO">ErrorNotBabylonUTXO</a>: u64 = 0;
</code></pre>



<a name="0x4_bbn_ErrorNotTransaction"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorNotTransaction">ErrorNotTransaction</a>: u64 = 1;
</code></pre>



<a name="0x4_bbn_ErrorTransactionLockTime"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorTransactionLockTime">ErrorTransactionLockTime</a>: u64 = 3;
</code></pre>



<a name="0x4_bbn_UNSPENDABLEKEYPATHKEY"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_UNSPENDABLEKEYPATHKEY">UNSPENDABLEKEYPATHKEY</a>: <a href="">vector</a>&lt;u8&gt; = [48, 50, 53, 48, 57, 50, 57, 98, 55, 52, 99, 49, 97, 48, 52, 57, 53, 52, 98, 55, 56, 98, 52, 98, 54, 48, 51, 53, 101, 57, 55, 97, 53, 101, 48, 55, 56, 97, 53, 97, 48, 102, 50, 56, 101, 99, 57, 54, 100, 53, 52, 55, 98, 102, 101, 101, 57, 97, 99, 101, 56, 48, 51, 97, 99, 48];
</code></pre>



<a name="0x4_bbn_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bbn.md#0x4_bbn_genesis_init">genesis_init</a>()
</code></pre>



<a name="0x4_bbn_try_get_bbn_op_return_data"></a>

## Function `try_get_bbn_op_return_data`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_try_get_bbn_op_return_data">try_get_bbn_op_return_data</a>(<a href="">transaction</a>: <a href="types.md#0x4_types_Transaction">types::Transaction</a>): (bool, u64, <a href="bbn.md#0x4_bbn_BBNOpReturnData">bbn::BBNOpReturnData</a>)
</code></pre>



<a name="0x4_bbn_try_get_staking_output"></a>

## Function `try_get_staking_output`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_try_get_staking_output">try_get_staking_output</a>(<a href="">transaction</a>: <a href="types.md#0x4_types_Transaction">types::Transaction</a>, staking_output_script: &<a href="">vector</a>&lt;u8&gt;): (bool, u64, <a href="_Option">option::Option</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;)
</code></pre>



<a name="0x4_bbn_derive_bbn_utxo"></a>

## Function `derive_bbn_utxo`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_derive_bbn_utxo">derive_bbn_utxo</a>(utxo_obj: &<a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;)
</code></pre>
