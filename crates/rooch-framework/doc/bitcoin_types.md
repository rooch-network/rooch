
<a name="0x3_bitcoin_types"></a>

# Module `0x3::bitcoin_types`



-  [Struct `Block`](#0x3_bitcoin_types_Block)
-  [Struct `Header`](#0x3_bitcoin_types_Header)
-  [Struct `Transaction`](#0x3_bitcoin_types_Transaction)
-  [Struct `TxIn`](#0x3_bitcoin_types_TxIn)
-  [Struct `OutPoint`](#0x3_bitcoin_types_OutPoint)
-  [Struct `TxOut`](#0x3_bitcoin_types_TxOut)
-  [Constants](#@Constants_0)
-  [Function `header`](#0x3_bitcoin_types_header)
-  [Function `txdata`](#0x3_bitcoin_types_txdata)
-  [Function `version`](#0x3_bitcoin_types_version)
-  [Function `prev_blockhash`](#0x3_bitcoin_types_prev_blockhash)
-  [Function `merkle_root`](#0x3_bitcoin_types_merkle_root)
-  [Function `time`](#0x3_bitcoin_types_time)
-  [Function `bits`](#0x3_bitcoin_types_bits)
-  [Function `nonce`](#0x3_bitcoin_types_nonce)
-  [Function `tx_id`](#0x3_bitcoin_types_tx_id)
-  [Function `tx_version`](#0x3_bitcoin_types_tx_version)
-  [Function `tx_lock_time`](#0x3_bitcoin_types_tx_lock_time)
-  [Function `tx_input`](#0x3_bitcoin_types_tx_input)
-  [Function `tx_output`](#0x3_bitcoin_types_tx_output)
-  [Function `txin_previous_output`](#0x3_bitcoin_types_txin_previous_output)
-  [Function `txin_script_sig`](#0x3_bitcoin_types_txin_script_sig)
-  [Function `txin_sequence`](#0x3_bitcoin_types_txin_sequence)
-  [Function `txin_witness`](#0x3_bitcoin_types_txin_witness)
-  [Function `new_outpoint`](#0x3_bitcoin_types_new_outpoint)
-  [Function `outpoint_txid`](#0x3_bitcoin_types_outpoint_txid)
-  [Function `outpoint_vout`](#0x3_bitcoin_types_outpoint_vout)
-  [Function `unpack_outpoint`](#0x3_bitcoin_types_unpack_outpoint)
-  [Function `txout_value`](#0x3_bitcoin_types_txout_value)
-  [Function `txout_script_pubkey`](#0x3_bitcoin_types_txout_script_pubkey)
-  [Function `unpack_txout`](#0x3_bitcoin_types_unpack_txout)


<pre><code></code></pre>



<a name="0x3_bitcoin_types_Block"></a>

## Struct `Block`



<pre><code>#[data_struct]
<b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_Block">Block</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_bitcoin_types_Header"></a>

## Struct `Header`



<pre><code>#[data_struct]
<b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_Header">Header</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_bitcoin_types_Transaction"></a>

## Struct `Transaction`



<pre><code>#[data_struct]
<b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_Transaction">Transaction</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_bitcoin_types_TxIn"></a>

## Struct `TxIn`



<pre><code>#[data_struct]
<b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_TxIn">TxIn</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_bitcoin_types_OutPoint"></a>

## Struct `OutPoint`



<pre><code>#[data_struct]
<b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_OutPoint">OutPoint</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_bitcoin_types_TxOut"></a>

## Struct `TxOut`



<pre><code>#[data_struct]
<b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_TxOut">TxOut</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_bitcoin_types_LOCK_TIME_THRESHOLD"></a>



<pre><code><b>const</b> <a href="bitcoin_types.md#0x3_bitcoin_types_LOCK_TIME_THRESHOLD">LOCK_TIME_THRESHOLD</a>: u32 = 500000000;
</code></pre>



<a name="0x3_bitcoin_types_header"></a>

## Function `header`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_header">header</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Block">bitcoin_types::Block</a>): &<a href="bitcoin_types.md#0x3_bitcoin_types_Header">bitcoin_types::Header</a>
</code></pre>



<a name="0x3_bitcoin_types_txdata"></a>

## Function `txdata`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_txdata">txdata</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Block">bitcoin_types::Block</a>): &<a href="">vector</a>&lt;<a href="bitcoin_types.md#0x3_bitcoin_types_Transaction">bitcoin_types::Transaction</a>&gt;
</code></pre>



<a name="0x3_bitcoin_types_version"></a>

## Function `version`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_version">version</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Header">bitcoin_types::Header</a>): u32
</code></pre>



<a name="0x3_bitcoin_types_prev_blockhash"></a>

## Function `prev_blockhash`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_prev_blockhash">prev_blockhash</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Header">bitcoin_types::Header</a>): <b>address</b>
</code></pre>



<a name="0x3_bitcoin_types_merkle_root"></a>

## Function `merkle_root`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_merkle_root">merkle_root</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Header">bitcoin_types::Header</a>): <b>address</b>
</code></pre>



<a name="0x3_bitcoin_types_time"></a>

## Function `time`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_time">time</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Header">bitcoin_types::Header</a>): u32
</code></pre>



<a name="0x3_bitcoin_types_bits"></a>

## Function `bits`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_bits">bits</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Header">bitcoin_types::Header</a>): u32
</code></pre>



<a name="0x3_bitcoin_types_nonce"></a>

## Function `nonce`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_nonce">nonce</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Header">bitcoin_types::Header</a>): u32
</code></pre>



<a name="0x3_bitcoin_types_tx_id"></a>

## Function `tx_id`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_tx_id">tx_id</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Transaction">bitcoin_types::Transaction</a>): <b>address</b>
</code></pre>



<a name="0x3_bitcoin_types_tx_version"></a>

## Function `tx_version`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_tx_version">tx_version</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Transaction">bitcoin_types::Transaction</a>): u32
</code></pre>



<a name="0x3_bitcoin_types_tx_lock_time"></a>

## Function `tx_lock_time`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_tx_lock_time">tx_lock_time</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Transaction">bitcoin_types::Transaction</a>): u32
</code></pre>



<a name="0x3_bitcoin_types_tx_input"></a>

## Function `tx_input`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_tx_input">tx_input</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Transaction">bitcoin_types::Transaction</a>): &<a href="">vector</a>&lt;<a href="bitcoin_types.md#0x3_bitcoin_types_TxIn">bitcoin_types::TxIn</a>&gt;
</code></pre>



<a name="0x3_bitcoin_types_tx_output"></a>

## Function `tx_output`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_tx_output">tx_output</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Transaction">bitcoin_types::Transaction</a>): &<a href="">vector</a>&lt;<a href="bitcoin_types.md#0x3_bitcoin_types_TxOut">bitcoin_types::TxOut</a>&gt;
</code></pre>



<a name="0x3_bitcoin_types_txin_previous_output"></a>

## Function `txin_previous_output`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_txin_previous_output">txin_previous_output</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_TxIn">bitcoin_types::TxIn</a>): &<a href="bitcoin_types.md#0x3_bitcoin_types_OutPoint">bitcoin_types::OutPoint</a>
</code></pre>



<a name="0x3_bitcoin_types_txin_script_sig"></a>

## Function `txin_script_sig`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_txin_script_sig">txin_script_sig</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_TxIn">bitcoin_types::TxIn</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_types_txin_sequence"></a>

## Function `txin_sequence`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_txin_sequence">txin_sequence</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_TxIn">bitcoin_types::TxIn</a>): u32
</code></pre>



<a name="0x3_bitcoin_types_txin_witness"></a>

## Function `txin_witness`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_txin_witness">txin_witness</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_TxIn">bitcoin_types::TxIn</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_types_new_outpoint"></a>

## Function `new_outpoint`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_new_outpoint">new_outpoint</a>(txid: <b>address</b>, vout: u32): <a href="bitcoin_types.md#0x3_bitcoin_types_OutPoint">bitcoin_types::OutPoint</a>
</code></pre>



<a name="0x3_bitcoin_types_outpoint_txid"></a>

## Function `outpoint_txid`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_outpoint_txid">outpoint_txid</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_OutPoint">bitcoin_types::OutPoint</a>): <b>address</b>
</code></pre>



<a name="0x3_bitcoin_types_outpoint_vout"></a>

## Function `outpoint_vout`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_outpoint_vout">outpoint_vout</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_OutPoint">bitcoin_types::OutPoint</a>): u32
</code></pre>



<a name="0x3_bitcoin_types_unpack_outpoint"></a>

## Function `unpack_outpoint`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_unpack_outpoint">unpack_outpoint</a>(self: <a href="bitcoin_types.md#0x3_bitcoin_types_OutPoint">bitcoin_types::OutPoint</a>): (<b>address</b>, u32)
</code></pre>



<a name="0x3_bitcoin_types_txout_value"></a>

## Function `txout_value`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_txout_value">txout_value</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_TxOut">bitcoin_types::TxOut</a>): u64
</code></pre>



<a name="0x3_bitcoin_types_txout_script_pubkey"></a>

## Function `txout_script_pubkey`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_txout_script_pubkey">txout_script_pubkey</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_TxOut">bitcoin_types::TxOut</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_types_unpack_txout"></a>

## Function `unpack_txout`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_unpack_txout">unpack_txout</a>(self: <a href="bitcoin_types.md#0x3_bitcoin_types_TxOut">bitcoin_types::TxOut</a>): (u64, <a href="">vector</a>&lt;u8&gt;)
</code></pre>
