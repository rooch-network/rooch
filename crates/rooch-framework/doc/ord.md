
<a name="0x3_ord"></a>

# Module `0x3::ord`



-  [Struct `Inscription`](#0x3_ord_Inscription)
-  [Function `from_transaction`](#0x3_ord_from_transaction)
-  [Function `from_transaction_bytes`](#0x3_ord_from_transaction_bytes)
-  [Function `from_witness`](#0x3_ord_from_witness)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="bitcoin_types.md#0x3_bitcoin_types">0x3::bitcoin_types</a>;
</code></pre>



<a name="0x3_ord_Inscription"></a>

## Struct `Inscription`



<pre><code><b>struct</b> <a href="ord.md#0x3_ord_Inscription">Inscription</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_ord_from_transaction"></a>

## Function `from_transaction`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_from_transaction">from_transaction</a>(transaction: &<a href="bitcoin_types.md#0x3_bitcoin_types_Transaction">bitcoin_types::Transaction</a>): <a href="">vector</a>&lt;<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x3_ord_from_transaction_bytes"></a>

## Function `from_transaction_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_from_transaction_bytes">from_transaction_bytes</a>(transaction_bytes: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x3_ord_from_witness"></a>

## Function `from_witness`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_from_witness">from_witness</a>(witness: &<a href="bitcoin_types.md#0x3_bitcoin_types_Witness">bitcoin_types::Witness</a>): <a href="">vector</a>&lt;<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>
