
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
-  [Function `time`](#0x3_bitcoin_types_time)


<pre><code></code></pre>



<a name="0x3_bitcoin_types_Block"></a>

## Struct `Block`



<pre><code><b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_Block">Block</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_bitcoin_types_Header"></a>

## Struct `Header`



<pre><code><b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_Header">Header</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_bitcoin_types_Transaction"></a>

## Struct `Transaction`



<pre><code><b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_Transaction">Transaction</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_bitcoin_types_TxIn"></a>

## Struct `TxIn`



<pre><code><b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_TxIn">TxIn</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_bitcoin_types_OutPoint"></a>

## Struct `OutPoint`



<pre><code><b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_OutPoint">OutPoint</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_bitcoin_types_TxOut"></a>

## Struct `TxOut`



<pre><code><b>struct</b> <a href="bitcoin_types.md#0x3_bitcoin_types_TxOut">TxOut</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_bitcoin_types_LOCK_TIME_THRESHOLD"></a>



<pre><code><b>const</b> <a href="bitcoin_types.md#0x3_bitcoin_types_LOCK_TIME_THRESHOLD">LOCK_TIME_THRESHOLD</a>: u32 = 500000000;
</code></pre>



<a name="0x3_bitcoin_types_header"></a>

## Function `header`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_header">header</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Block">bitcoin_types::Block</a>): <a href="bitcoin_types.md#0x3_bitcoin_types_Header">bitcoin_types::Header</a>
</code></pre>



<a name="0x3_bitcoin_types_time"></a>

## Function `time`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_types.md#0x3_bitcoin_types_time">time</a>(self: &<a href="bitcoin_types.md#0x3_bitcoin_types_Header">bitcoin_types::Header</a>): u32
</code></pre>
