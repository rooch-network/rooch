
<a name="0x4_bbn"></a>

# Module `0x4::bbn`



-  [Struct `BBNGlobalParam`](#0x4_bbn_BBNGlobalParam)
-  [Resource `BBNGlobalParams`](#0x4_bbn_BBNGlobalParams)
-  [Struct `BBNOpReturnOutput`](#0x4_bbn_BBNOpReturnOutput)
-  [Struct `BBNV0OpReturnData`](#0x4_bbn_BBNV0OpReturnData)
-  [Resource `BBNStakeSeal`](#0x4_bbn_BBNStakeSeal)
-  [Struct `BBNScriptPaths`](#0x4_bbn_BBNScriptPaths)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_bbn_genesis_init)
-  [Function `init_for_upgrade`](#0x4_bbn_init_for_upgrade)
-  [Function `is_possible_bbn_tx`](#0x4_bbn_is_possible_bbn_tx)
-  [Function `process_bbn_tx_entry`](#0x4_bbn_process_bbn_tx_entry)
-  [Function `add_temp_state`](#0x4_bbn_add_temp_state)
-  [Function `contains_temp_state`](#0x4_bbn_contains_temp_state)
-  [Function `borrow_temp_state`](#0x4_bbn_borrow_temp_state)
-  [Function `borrow_mut_temp_state`](#0x4_bbn_borrow_mut_temp_state)
-  [Function `remove_temp_state`](#0x4_bbn_remove_temp_state)
-  [Function `block_height`](#0x4_bbn_block_height)
-  [Function `txid`](#0x4_bbn_txid)
-  [Function `vout`](#0x4_bbn_vout)
-  [Function `outpoint`](#0x4_bbn_outpoint)
-  [Function `tag`](#0x4_bbn_tag)
-  [Function `staker_pub_key`](#0x4_bbn_staker_pub_key)
-  [Function `finality_provider_pub_key`](#0x4_bbn_finality_provider_pub_key)
-  [Function `staking_time`](#0x4_bbn_staking_time)
-  [Function `staking_amount`](#0x4_bbn_staking_amount)
-  [Function `is_expired`](#0x4_bbn_is_expired)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::result</a>;
<b>use</b> <a href="">0x2::sort</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="">0x3::bitcoin_address</a>;
<b>use</b> <a href="bitcoin.md#0x4_bitcoin">0x4::bitcoin</a>;
<b>use</b> <a href="opcode.md#0x4_opcode">0x4::opcode</a>;
<b>use</b> <a href="script_buf.md#0x4_script_buf">0x4::script_buf</a>;
<b>use</b> <a href="taproot_builder.md#0x4_taproot_builder">0x4::taproot_builder</a>;
<b>use</b> <a href="temp_state.md#0x4_temp_state">0x4::temp_state</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
<b>use</b> <a href="utxo.md#0x4_utxo">0x4::utxo</a>;
</code></pre>



<a name="0x4_bbn_BBNGlobalParam"></a>

## Struct `BBNGlobalParam`



<pre><code><b>struct</b> <a href="bbn.md#0x4_bbn_BBNGlobalParam">BBNGlobalParam</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_bbn_BBNGlobalParams"></a>

## Resource `BBNGlobalParams`



<pre><code><b>struct</b> <a href="bbn.md#0x4_bbn_BBNGlobalParams">BBNGlobalParams</a> <b>has</b> key
</code></pre>



<a name="0x4_bbn_BBNOpReturnOutput"></a>

## Struct `BBNOpReturnOutput`



<pre><code><b>struct</b> <a href="bbn.md#0x4_bbn_BBNOpReturnOutput">BBNOpReturnOutput</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_bbn_BBNV0OpReturnData"></a>

## Struct `BBNV0OpReturnData`



<pre><code><b>struct</b> <a href="bbn.md#0x4_bbn_BBNV0OpReturnData">BBNV0OpReturnData</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_bbn_BBNStakeSeal"></a>

## Resource `BBNStakeSeal`



<pre><code><b>struct</b> <a href="bbn.md#0x4_bbn_BBNStakeSeal">BBNStakeSeal</a> <b>has</b> key
</code></pre>



<a name="0x4_bbn_BBNScriptPaths"></a>

## Struct `BBNScriptPaths`



<pre><code><b>struct</b> <a href="bbn.md#0x4_bbn_BBNScriptPaths">BBNScriptPaths</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_bbn_ErrorInvalidThreshold"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorInvalidThreshold">ErrorInvalidThreshold</a>: u64 = 11;
</code></pre>



<a name="0x4_bbn_TEMPORARY_AREA"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_TEMPORARY_AREA">TEMPORARY_AREA</a>: <a href="">vector</a>&lt;u8&gt; = [116, 101, 109, 112, 111, 114, 97, 114, 121, 95, 97, 114, 101, 97];
</code></pre>



<a name="0x4_bbn_ErrorAlreadyInit"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorAlreadyInit">ErrorAlreadyInit</a>: u64 = 1;
</code></pre>



<a name="0x4_bbn_ErrorFailedToFinalizeTaproot"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorFailedToFinalizeTaproot">ErrorFailedToFinalizeTaproot</a>: u64 = 12;
</code></pre>



<a name="0x4_bbn_ErrorInvalidBabylonOpReturn"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorInvalidBabylonOpReturn">ErrorInvalidBabylonOpReturn</a>: u64 = 5;
</code></pre>



<a name="0x4_bbn_ErrorInvalidBytesLen"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorInvalidBytesLen">ErrorInvalidBytesLen</a>: u64 = 7;
</code></pre>



<a name="0x4_bbn_ErrorInvalidKeysLen"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorInvalidKeysLen">ErrorInvalidKeysLen</a>: u64 = 10;
</code></pre>



<a name="0x4_bbn_ErrorNoBabylonOpReturn"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorNoBabylonOpReturn">ErrorNoBabylonOpReturn</a>: u64 = 4;
</code></pre>



<a name="0x4_bbn_ErrorNoBabylonStakingOutput"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorNoBabylonStakingOutput">ErrorNoBabylonStakingOutput</a>: u64 = 14;
</code></pre>



<a name="0x4_bbn_ErrorNoBabylonUTXO"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorNoBabylonUTXO">ErrorNoBabylonUTXO</a>: u64 = 2;
</code></pre>



<a name="0x4_bbn_ErrorNoKeysProvided"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorNoKeysProvided">ErrorNoKeysProvided</a>: u64 = 9;
</code></pre>



<a name="0x4_bbn_ErrorNotBabylonTx"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorNotBabylonTx">ErrorNotBabylonTx</a>: u64 = 8;
</code></pre>



<a name="0x4_bbn_ErrorTransactionLockTime"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorTransactionLockTime">ErrorTransactionLockTime</a>: u64 = 6;
</code></pre>



<a name="0x4_bbn_ErrorTransactionNotFound"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorTransactionNotFound">ErrorTransactionNotFound</a>: u64 = 3;
</code></pre>



<a name="0x4_bbn_ErrorUTXOAlreadySealed"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_ErrorUTXOAlreadySealed">ErrorUTXOAlreadySealed</a>: u64 = 13;
</code></pre>



<a name="0x4_bbn_UNSPENDABLEKEYPATHKEY"></a>



<pre><code><b>const</b> <a href="bbn.md#0x4_bbn_UNSPENDABLEKEYPATHKEY">UNSPENDABLEKEYPATHKEY</a>: <a href="">vector</a>&lt;u8&gt; = [80, 146, 155, 116, 193, 160, 73, 84, 183, 139, 75, 96, 53, 233, 122, 94, 7, 138, 90, 15, 40, 236, 150, 213, 71, 191, 238, 154, 206, 128, 58, 192];
</code></pre>



<a name="0x4_bbn_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bbn.md#0x4_bbn_genesis_init">genesis_init</a>()
</code></pre>



<a name="0x4_bbn_init_for_upgrade"></a>

## Function `init_for_upgrade`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_init_for_upgrade">init_for_upgrade</a>()
</code></pre>



<a name="0x4_bbn_is_possible_bbn_tx"></a>

## Function `is_possible_bbn_tx`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_is_possible_bbn_tx">is_possible_bbn_tx</a>(txid: <b>address</b>): bool
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



<a name="0x4_bbn_block_height"></a>

## Function `block_height`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_block_height">block_height</a>(stake: &<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>): u64
</code></pre>



<a name="0x4_bbn_txid"></a>

## Function `txid`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_txid">txid</a>(stake: &<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>): <b>address</b>
</code></pre>



<a name="0x4_bbn_vout"></a>

## Function `vout`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_vout">vout</a>(stake: &<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>): u32
</code></pre>



<a name="0x4_bbn_outpoint"></a>

## Function `outpoint`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_outpoint">outpoint</a>(stake: &<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>): <a href="types.md#0x4_types_OutPoint">types::OutPoint</a>
</code></pre>



<a name="0x4_bbn_tag"></a>

## Function `tag`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_tag">tag</a>(stake: &<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_bbn_staker_pub_key"></a>

## Function `staker_pub_key`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_staker_pub_key">staker_pub_key</a>(stake: &<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_bbn_finality_provider_pub_key"></a>

## Function `finality_provider_pub_key`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_finality_provider_pub_key">finality_provider_pub_key</a>(stake: &<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_bbn_staking_time"></a>

## Function `staking_time`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_staking_time">staking_time</a>(stake: &<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>): u16
</code></pre>



<a name="0x4_bbn_staking_amount"></a>

## Function `staking_amount`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_staking_amount">staking_amount</a>(stake: &<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>): u64
</code></pre>



<a name="0x4_bbn_is_expired"></a>

## Function `is_expired`



<pre><code><b>public</b> <b>fun</b> <a href="bbn.md#0x4_bbn_is_expired">is_expired</a>(stake: &<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>): bool
</code></pre>
