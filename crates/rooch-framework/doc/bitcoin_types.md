
<a name="0x3_bitcoin_types"></a>

# Module `0x3::bitcoin_types`



-  [Struct `Block`](#0x3_bitcoin_types_Block)
-  [Struct `Header`](#0x3_bitcoin_types_Header)
-  [Struct `Transaction`](#0x3_bitcoin_types_Transaction)
-  [Struct `TxIn`](#0x3_bitcoin_types_TxIn)
-  [Struct `Witness`](#0x3_bitcoin_types_Witness)
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
-  [Function `witness_nth`](#0x3_bitcoin_types_witness_nth)
-  [Function `witness_len`](#0x3_bitcoin_types_witness_len)
-  [Function `witness_tapscript`](#0x3_bitcoin_types_witness_tapscript)
-  [Function `new_outpoint`](#0x3_bitcoin_types_new_outpoint)
-  [Function `outpoint_txid`](#0x3_bitcoin_types_outpoint_txid)
-  [Function `outpoint_vout`](#0x3_bitcoin_types_outpoint_vout)
-  [Function `unpack_outpoint`](#0x3_bitcoin_types_unpack_outpoint)
-  [Function `txout_value`](#0x3_bitcoin_types_txout_value)
-  [Function `txout_script_pubkey`](#0x3_bitcoin_types_txout_script_pubkey)
-  [Function `txout_address`](#0x3_bitcoin_types_txout_address)
-  [Function `txout_object_address`](#0x3_bitcoin_types_txout_object_address)
-  [Function `unpack_txout`](#0x3_bitcoin_types_unpack_txout)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="bitcoin_address.md#0x3_bitcoin_address">0x3::bitcoin_address</a>;
<b>use</b> <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf">0x3::bitcoin_script_buf</a>;
<b>use</b> <a href="multichain_address.md#0x3_multichain_address">0x3::multichain_address</a>;
</code></pre>



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



<a name="0x3_bitcoin_types_Witness"></a>

## Struct `Witness`



<pre><code>#[data_struct]
<b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_Witness">Witness</a> <b>has</b> <b>copy</b>, drop, store
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



<a name="0x3_bitcoin_types_TAPROOT_ANNEX_PREFIX"></a>



<pre><code><b>const</b> <a href="bitcoin_types.md#0x3_bitcoin_types_TAPROOT_ANNEX_PREFIX">TAPROOT_ANNEX_PREFIX</a>: u8 = 80;
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



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_txin_witness">txin_witness</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_TxIn">bitcoin_types::TxIn</a>): &<a href="bitcoin_types.md#0x3_bitcoin_types_Witness">bitcoin_types::Witness</a>
</code></pre>



<a name="0x3_bitcoin_types_witness_nth"></a>

## Function `witness_nth`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_witness_nth">witness_nth</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Witness">bitcoin_types::Witness</a>, nth: u64): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_types_witness_len"></a>

## Function `witness_len`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_witness_len">witness_len</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Witness">bitcoin_types::Witness</a>): u64
</code></pre>



<a name="0x3_bitcoin_types_witness_tapscript"></a>

## Function `witness_tapscript`

Get Tapscript following BIP341 rules regarding accounting for an annex.

This does not guarantee that this represents a P2TR [<code><a href="bitcoin_types.md#0x3_bitcoin_types_Witness">Witness</a></code>]. It
merely gets the second to last or third to last element depending on
the first byte of the last element being equal to 0x50. See
bitcoin_script::is_v1_p2tr to check whether this is actually a Taproot witness.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_witness_tapscript">witness_tapscript</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Witness">bitcoin_types::Witness</a>): <a href="_Option">option::Option</a>&lt;<a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">bitcoin_script_buf::ScriptBuf</a>&gt;
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



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_txout_script_pubkey">txout_script_pubkey</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_TxOut">bitcoin_types::TxOut</a>): &<a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">bitcoin_script_buf::ScriptBuf</a>
</code></pre>



<a name="0x3_bitcoin_types_txout_address"></a>

## Function `txout_address`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_txout_address">txout_address</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_TxOut">bitcoin_types::TxOut</a>): <a href="_Option">option::Option</a>&lt;<a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>&gt;
</code></pre>



<a name="0x3_bitcoin_types_txout_object_address"></a>

## Function `txout_object_address`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_txout_object_address">txout_object_address</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_TxOut">bitcoin_types::TxOut</a>): <b>address</b>
</code></pre>



<a name="0x3_bitcoin_types_unpack_txout"></a>

## Function `unpack_txout`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_unpack_txout">unpack_txout</a>(self: <a href="bitcoin_types.md#0x3_bitcoin_types_TxOut">bitcoin_types::TxOut</a>): (u64, <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">bitcoin_script_buf::ScriptBuf</a>)
</code></pre>
