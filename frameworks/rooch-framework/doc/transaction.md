
<a name="0x3_transaction"></a>

# Module `0x3::transaction`



-  [Struct `TransactionSequenceInfo`](#0x3_transaction_TransactionSequenceInfo)
-  [Function `tx_order`](#0x3_transaction_tx_order)
-  [Function `tx_order_signature`](#0x3_transaction_tx_order_signature)
-  [Function `tx_accumulator_root`](#0x3_transaction_tx_accumulator_root)
-  [Function `tx_timestamp`](#0x3_transaction_tx_timestamp)


<pre><code></code></pre>



<a name="0x3_transaction_TransactionSequenceInfo"></a>

## Struct `TransactionSequenceInfo`



<pre><code>#[data_struct]
<b>struct</b> <a href="transaction.md#0x3_transaction_TransactionSequenceInfo">TransactionSequenceInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_transaction_tx_order"></a>

## Function `tx_order`



<pre><code><b>public</b> <b>fun</b> <a href="transaction.md#0x3_transaction_tx_order">tx_order</a>(self: &<a href="transaction.md#0x3_transaction_TransactionSequenceInfo">transaction::TransactionSequenceInfo</a>): u64
</code></pre>



<a name="0x3_transaction_tx_order_signature"></a>

## Function `tx_order_signature`



<pre><code><b>public</b> <b>fun</b> <a href="transaction.md#0x3_transaction_tx_order_signature">tx_order_signature</a>(self: &<a href="transaction.md#0x3_transaction_TransactionSequenceInfo">transaction::TransactionSequenceInfo</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_transaction_tx_accumulator_root"></a>

## Function `tx_accumulator_root`



<pre><code><b>public</b> <b>fun</b> <a href="transaction.md#0x3_transaction_tx_accumulator_root">tx_accumulator_root</a>(self: &<a href="transaction.md#0x3_transaction_TransactionSequenceInfo">transaction::TransactionSequenceInfo</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_transaction_tx_timestamp"></a>

## Function `tx_timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="transaction.md#0x3_transaction_tx_timestamp">tx_timestamp</a>(self: &<a href="transaction.md#0x3_transaction_TransactionSequenceInfo">transaction::TransactionSequenceInfo</a>): u64
</code></pre>
