
<a name="0x2_tx_result"></a>

# Module `0x2::tx_result`



-  [Struct `TxResult`](#0x2_tx_result_TxResult)
-  [Function `is_executed`](#0x2_tx_result_is_executed)
-  [Function `gas_used`](#0x2_tx_result_gas_used)


<pre><code></code></pre>



<a name="0x2_tx_result_TxResult"></a>

## Struct `TxResult`

The result of a transaction.
The VM will put this struct in the TxContext after the transaction execution.
We can get the result in the <code>post_execute</code> function.


<pre><code><b>struct</b> <a href="tx_result.md#0x2_tx_result_TxResult">TxResult</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x2_tx_result_is_executed"></a>

## Function `is_executed`



<pre><code><b>public</b> <b>fun</b> <a href="tx_result.md#0x2_tx_result_is_executed">is_executed</a>(self: &<a href="tx_result.md#0x2_tx_result_TxResult">tx_result::TxResult</a>): bool
</code></pre>



<a name="0x2_tx_result_gas_used"></a>

## Function `gas_used`



<pre><code><b>public</b> <b>fun</b> <a href="tx_result.md#0x2_tx_result_gas_used">gas_used</a>(self: &<a href="tx_result.md#0x2_tx_result_TxResult">tx_result::TxResult</a>): u64
</code></pre>
