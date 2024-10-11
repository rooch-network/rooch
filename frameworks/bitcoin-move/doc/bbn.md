
<a name="0x4_bbn"></a>

# Module `0x4::bbn`



-  [Resource `BBNGlobalParam`](#0x4_bbn_BBNGlobalParam)
-  [Resource `BBNGlobalParams`](#0x4_bbn_BBNGlobalParams)
-  [Struct `BBNOpReturnData`](#0x4_bbn_BBNOpReturnData)
-  [Resource `BBNStakeSeal`](#0x4_bbn_BBNStakeSeal)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_bbn_genesis_init)
-  [Function `init_for_upgrade`](#0x4_bbn_init_for_upgrade)
-  [Function `is_bbn_tx`](#0x4_bbn_is_bbn_tx)
-  [Function `process_bbn_tx_entry`](#0x4_bbn_process_bbn_tx_entry)
-  [Function `add_temp_state`](#0x4_bbn_add_temp_state)
-  [Function `contains_temp_state`](#0x4_bbn_contains_temp_state)
-  [Function `borrow_temp_state`](#0x4_bbn_borrow_temp_state)
-  [Function `borrow_mut_temp_state`](#0x4_bbn_borrow_mut_temp_state)
-  [Function `remove_temp_state`](#0x4_bbn_remove_temp_state)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="">0x3::bitcoin_address</a>;
<b>use</b> <a href="bitcoin.md#0x4_bitcoin">0x4::bitcoin</a>;
<b>use</b> <a href="opcode.md#0x4_opcode">0x4::opcode</a>;
<b>use</b> <a href="script_buf.md#0x4_script_buf">0x4::script_buf</a>;
<b>use</b> <a href="temp_state.md#0x4_temp_state">0x4::temp_state</a>;
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



<a name="0x4_bbn_BBNStakeSeal"></a>

## Resource `BBNStakeSeal`



<pre><code><b>struct</b> <a href="bbn.md#0x4_bbn_BBNStakeSeal">BBNStakeSeal</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_bbn_TEMPORARY_AREA"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_TEMPORARY_AREA">TEMPORARY_AREA</a>: <a href="">vector</a>&lt;u8&gt; = [116, 101, 109, 112, 111, 114, 97, 114, 121, 95, 97, 114, 101, 97];
</code></pre>



<a name="0x4_bbn_ErrorAlreadyInit"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorAlreadyInit">ErrorAlreadyInit</a>: u64 = 1;
</code></pre>



<a name="0x4_bbn_ErrorInvalidBytesLen"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorInvalidBytesLen">ErrorInvalidBytesLen</a>: u64 = 6;
</code></pre>



<a name="0x4_bbn_ErrorNotBabylonOpReturn"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorNotBabylonOpReturn">ErrorNotBabylonOpReturn</a>: u64 = 4;
</code></pre>



<a name="0x4_bbn_ErrorNotBabylonUTXO"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorNotBabylonUTXO">ErrorNotBabylonUTXO</a>: u64 = 2;
</code></pre>



<a name="0x4_bbn_ErrorTransactionLockTime"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorTransactionLockTime">ErrorTransactionLockTime</a>: u64 = 5;
</code></pre>



<a name="0x4_bbn_ErrorTransactionNotFound"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorTransactionNotFound">ErrorTransactionNotFound</a>: u64 = 3;
</code></pre>



<a name="0x4_bbn_UNSPENDABLEKEYPATHKEY"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_UNSPENDABLEKEYPATHKEY">UNSPENDABLEKEYPATHKEY</a>: <a href="">vector</a>&lt;u8&gt; = [48, 50, 53, 48, 57, 50, 57, 98, 55, 52, 99, 49, 97, 48, 52, 57, 53, 52, 98, 55, 56, 98, 52, 98, 54, 48, 51, 53, 101, 57, 55, 97, 53, 101, 48, 55, 56, 97, 53, 97, 48, 102, 50, 56, 101, 99, 57, 54, 100, 53, 52, 55, 98, 102, 101, 101, 57, 97, 99, 101, 56, 48, 51, 97, 99, 48];
</code></pre>



<a name="0x4_bbn_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bbn.md#0x4_bbn_genesis_init">genesis_init</a>()
</code></pre>



<a name="0x4_bbn_init_for_upgrade"></a>

## Function `init_for_upgrade`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_init_for_upgrade">init_for_upgrade</a>()
</code></pre>



<a name="0x4_bbn_is_bbn_tx"></a>

## Function `is_bbn_tx`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_is_bbn_tx">is_bbn_tx</a>(txid: <b>address</b>): bool
</code></pre>



<a name="0x4_bbn_process_bbn_tx_entry"></a>

## Function `process_bbn_tx_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="bbn.md#0x4_bbn_process_bbn_tx_entry">process_bbn_tx_entry</a>(txid: <b>address</b>)
</code></pre>



<a name="0x4_bbn_add_temp_state"></a>

## Function `add_temp_state`



<pre><code>#[private_generics(#[S])]
<b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_add_temp_state">add_temp_state</a>&lt;S: drop, store&gt;(stake: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>&gt;, state: S)
</code></pre>



<a name="0x4_bbn_contains_temp_state"></a>

## Function `contains_temp_state`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_contains_temp_state">contains_temp_state</a>&lt;S: drop, store&gt;(stake: &<a href="_Object">object::Object</a>&lt;<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>&gt;): bool
</code></pre>



<a name="0x4_bbn_borrow_temp_state"></a>

## Function `borrow_temp_state`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_borrow_temp_state">borrow_temp_state</a>&lt;S: drop, store&gt;(stake: &<a href="_Object">object::Object</a>&lt;<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>&gt;): &S
</code></pre>



<a name="0x4_bbn_borrow_mut_temp_state"></a>

## Function `borrow_mut_temp_state`



<pre><code>#[private_generics(#[S])]
<b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_borrow_mut_temp_state">borrow_mut_temp_state</a>&lt;S: drop, store&gt;(stake: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>&gt;): &<b>mut</b> S
</code></pre>



<a name="0x4_bbn_remove_temp_state"></a>

## Function `remove_temp_state`



<pre><code>#[private_generics(#[S])]
<b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_remove_temp_state">remove_temp_state</a>&lt;S: drop, store&gt;(stake: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>&gt;): S
</code></pre>
