
<a id="0x4_transaction_validator"></a>

# Module `0x4::transaction_validator`



-  [Struct `TransactionValidatorPlaceholder`](#0x4_transaction_validator_TransactionValidatorPlaceholder)
-  [Function `validate_l1_tx`](#0x4_transaction_validator_validate_l1_tx)


<pre><code><b>use</b> <a href="bitcoin.md#0x4_bitcoin">0x4::bitcoin</a>;
</code></pre>



<a id="0x4_transaction_validator_TransactionValidatorPlaceholder"></a>

## Struct `TransactionValidatorPlaceholder`

The l1 tx already execute
Just using to get module signer


<pre><code><b>struct</b> <a href="transaction_validator.md#0x4_transaction_validator_TransactionValidatorPlaceholder">TransactionValidatorPlaceholder</a>
</code></pre>



<a id="0x4_transaction_validator_validate_l1_tx"></a>

## Function `validate_l1_tx`

This function is for Rooch to validate the l1 transaction.
If validate fails, abort this function.


<pre><code><b>public</b> <b>fun</b> <a href="transaction_validator.md#0x4_transaction_validator_validate_l1_tx">validate_l1_tx</a>(tx_hash: <b>address</b>, _payload: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>
