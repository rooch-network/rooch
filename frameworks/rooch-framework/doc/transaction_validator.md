
<a name="0x3_transaction_validator"></a>

# Module `0x3::transaction_validator`



-  [Struct `TransactionValidatorPlaceholder`](#0x3_transaction_validator_TransactionValidatorPlaceholder)
-  [Constants](#@Constants_0)
-  [Function `validate`](#0x3_transaction_validator_validate)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::gas_schedule</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::timestamp</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x2::tx_result</a>;
<b>use</b> <a href="account.md#0x3_account">0x3::account</a>;
<b>use</b> <a href="account_authentication.md#0x3_account_authentication">0x3::account_authentication</a>;
<b>use</b> <a href="address_mapping.md#0x3_address_mapping">0x3::address_mapping</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry">0x3::auth_validator_registry</a>;
<b>use</b> <a href="bitcoin_address.md#0x3_bitcoin_address">0x3::bitcoin_address</a>;
<b>use</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator">0x3::bitcoin_validator</a>;
<b>use</b> <a href="chain_id.md#0x3_chain_id">0x3::chain_id</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="gas_coin.md#0x3_gas_coin">0x3::gas_coin</a>;
<b>use</b> <a href="session_key.md#0x3_session_key">0x3::session_key</a>;
<b>use</b> <a href="session_validator.md#0x3_session_validator">0x3::session_validator</a>;
<b>use</b> <a href="transaction.md#0x3_transaction">0x3::transaction</a>;
<b>use</b> <a href="transaction_fee.md#0x3_transaction_fee">0x3::transaction_fee</a>;
</code></pre>



<a name="0x3_transaction_validator_TransactionValidatorPlaceholder"></a>

## Struct `TransactionValidatorPlaceholder`

Just using to get module signer


<pre><code><b>struct</b> <a href="transaction_validator.md#0x3_transaction_validator_TransactionValidatorPlaceholder">TransactionValidatorPlaceholder</a>
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_transaction_validator_MAX_U64"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x3_transaction_validator_ErrorMaxGasAmountExceeded"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_ErrorMaxGasAmountExceeded">ErrorMaxGasAmountExceeded</a>: u64 = 1008;
</code></pre>



<a name="0x3_transaction_validator_ErrorValidateAccountDoesNotExist"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_ErrorValidateAccountDoesNotExist">ErrorValidateAccountDoesNotExist</a>: u64 = 1003;
</code></pre>



<a name="0x3_transaction_validator_ErrorValidateBadChainId"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_ErrorValidateBadChainId">ErrorValidateBadChainId</a>: u64 = 1006;
</code></pre>



<a name="0x3_transaction_validator_ErrorValidateCantPayGasDeposit"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_ErrorValidateCantPayGasDeposit">ErrorValidateCantPayGasDeposit</a>: u64 = 1004;
</code></pre>



<a name="0x3_transaction_validator_ErrorValidateNotInstalledAuthValidator"></a>

The authenticator's auth validator id is not installed to the sender's account


<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_ErrorValidateNotInstalledAuthValidator">ErrorValidateNotInstalledAuthValidator</a>: u64 = 1010;
</code></pre>



<a name="0x3_transaction_validator_ErrorValidateSequenceNuberTooOld"></a>

Validate errors. These are separated out from the other errors in this
module since they are mapped separately to major VM statuses, and are
important to the semantics of the system.


<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_ErrorValidateSequenceNuberTooOld">ErrorValidateSequenceNuberTooOld</a>: u64 = 1001;
</code></pre>



<a name="0x3_transaction_validator_ErrorValidateSequenceNumberTooBig"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_ErrorValidateSequenceNumberTooBig">ErrorValidateSequenceNumberTooBig</a>: u64 = 1007;
</code></pre>



<a name="0x3_transaction_validator_ErrorValidateSequenceNumberTooNew"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_ErrorValidateSequenceNumberTooNew">ErrorValidateSequenceNumberTooNew</a>: u64 = 1002;
</code></pre>



<a name="0x3_transaction_validator_ErrorValidateTransactionExpired"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_ErrorValidateTransactionExpired">ErrorValidateTransactionExpired</a>: u64 = 1005;
</code></pre>



<a name="0x3_transaction_validator_validate"></a>

## Function `validate`

This function is for Rooch to validate the transaction sender's authenticator.
If the authenticator is invaid, abort this function.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_validator.md#0x3_transaction_validator_validate">validate</a>(<a href="chain_id.md#0x3_chain_id">chain_id</a>: u64, auth_validator_id: u64, authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="auth_validator.md#0x3_auth_validator_TxValidateResult">auth_validator::TxValidateResult</a>
</code></pre>
