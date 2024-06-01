
<a name="0x3_auth_validator"></a>

# Module `0x3::auth_validator`

This module contains the error code for auth_validator module
The auth_validator implementation should contain the following functions
public fun validate(authenticator_payload: vector<u8>)
fun pre_execute()
fun post_execute()


-  [Struct `AuthValidator`](#0x3_auth_validator_AuthValidator)
-  [Struct `TxValidateResult`](#0x3_auth_validator_TxValidateResult)
-  [Constants](#@Constants_0)
-  [Function `error_validate_sequence_number_too_old`](#0x3_auth_validator_error_validate_sequence_number_too_old)
-  [Function `error_validate_sequence_number_too_new`](#0x3_auth_validator_error_validate_sequence_number_too_new)
-  [Function `error_validate_account_does_not_exist`](#0x3_auth_validator_error_validate_account_does_not_exist)
-  [Function `error_validate_cant_pay_gas_deposit`](#0x3_auth_validator_error_validate_cant_pay_gas_deposit)
-  [Function `error_validate_transaction_expired`](#0x3_auth_validator_error_validate_transaction_expired)
-  [Function `error_validate_bad_chain_id`](#0x3_auth_validator_error_validate_bad_chain_id)
-  [Function `error_validate_sequence_number_too_big`](#0x3_auth_validator_error_validate_sequence_number_too_big)
-  [Function `error_validate_max_gas_amount_exceeded`](#0x3_auth_validator_error_validate_max_gas_amount_exceeded)
-  [Function `error_validate_invalid_account_auth_key`](#0x3_auth_validator_error_validate_invalid_account_auth_key)
-  [Function `error_validate_invalid_authenticator`](#0x3_auth_validator_error_validate_invalid_authenticator)
-  [Function `error_validate_not_installed_auth_validator`](#0x3_auth_validator_error_validate_not_installed_auth_validator)
-  [Function `error_validate_session_is_expired`](#0x3_auth_validator_error_validate_session_is_expired)
-  [Function `error_validate_function_call_beyond_session_scope`](#0x3_auth_validator_error_validate_function_call_beyond_session_scope)
-  [Function `new_auth_validator`](#0x3_auth_validator_new_auth_validator)
-  [Function `validator_id`](#0x3_auth_validator_validator_id)
-  [Function `validator_module_address`](#0x3_auth_validator_validator_module_address)
-  [Function `validator_module_name`](#0x3_auth_validator_validator_module_name)
-  [Function `new_tx_validate_result`](#0x3_auth_validator_new_tx_validate_result)
-  [Function `get_validate_result_from_ctx`](#0x3_auth_validator_get_validate_result_from_ctx)
-  [Function `get_validator_id_from_ctx`](#0x3_auth_validator_get_validator_id_from_ctx)
-  [Function `get_session_key_from_ctx_option`](#0x3_auth_validator_get_session_key_from_ctx_option)
-  [Function `is_validate_via_session_key`](#0x3_auth_validator_is_validate_via_session_key)
-  [Function `get_session_key_from_ctx`](#0x3_auth_validator_get_session_key_from_ctx)
-  [Function `get_bitcoin_address_from_ctx`](#0x3_auth_validator_get_bitcoin_address_from_ctx)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="bitcoin_address.md#0x3_bitcoin_address">0x3::bitcoin_address</a>;
</code></pre>



<a name="0x3_auth_validator_AuthValidator"></a>

## Struct `AuthValidator`

The Authentication Validator


<pre><code><b>struct</b> <a href="auth_validator.md#0x3_auth_validator_AuthValidator">AuthValidator</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_auth_validator_TxValidateResult"></a>

## Struct `TxValidateResult`

The Transaction Validate Result
this result will be stored in the TxContext


<pre><code><b>struct</b> <a href="auth_validator.md#0x3_auth_validator_TxValidateResult">TxValidateResult</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_auth_validator_ErrorMustExecuteAfterValidate"></a>

The function must be executed after the transaction is validated


<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorMustExecuteAfterValidate">ErrorMustExecuteAfterValidate</a>: u64 = 1;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateAccountDoesNotExist"></a>



<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateAccountDoesNotExist">ErrorValidateAccountDoesNotExist</a>: u64 = 1003;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateBadChainId"></a>



<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateBadChainId">ErrorValidateBadChainId</a>: u64 = 1006;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateCantPayGasDeposit"></a>



<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateCantPayGasDeposit">ErrorValidateCantPayGasDeposit</a>: u64 = 1004;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateFunctionCallBeyondSessionScope"></a>

The function call is beyond the session's scope


<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateFunctionCallBeyondSessionScope">ErrorValidateFunctionCallBeyondSessionScope</a>: u64 = 1013;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateInvalidAccountAuthKey"></a>

The AuthKey in transaction's authenticator do not match with the sender's account auth key


<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateInvalidAccountAuthKey">ErrorValidateInvalidAccountAuthKey</a>: u64 = 1009;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateInvalidAuthenticator"></a>

InvalidAuthenticator, include invalid signature


<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateInvalidAuthenticator">ErrorValidateInvalidAuthenticator</a>: u64 = 1010;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateMaxGasAmountExceeded"></a>



<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateMaxGasAmountExceeded">ErrorValidateMaxGasAmountExceeded</a>: u64 = 1008;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateNotInstalledAuthValidator"></a>

The authenticator's auth validator id is not installed to the sender's account


<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateNotInstalledAuthValidator">ErrorValidateNotInstalledAuthValidator</a>: u64 = 1011;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateSequenceNuberTooOld"></a>

Validate errors. These are separated out from the other errors in this
module since they are mapped separately to major VM statuses, and are
important to the semantics of the system.


<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateSequenceNuberTooOld">ErrorValidateSequenceNuberTooOld</a>: u64 = 1001;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateSequenceNumberTooBig"></a>



<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateSequenceNumberTooBig">ErrorValidateSequenceNumberTooBig</a>: u64 = 1007;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateSequenceNumberTooNew"></a>



<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateSequenceNumberTooNew">ErrorValidateSequenceNumberTooNew</a>: u64 = 1002;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateSessionIsExpired"></a>

The session is expired


<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateSessionIsExpired">ErrorValidateSessionIsExpired</a>: u64 = 1012;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateTransactionExpired"></a>



<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateTransactionExpired">ErrorValidateTransactionExpired</a>: u64 = 1005;
</code></pre>



<a name="0x3_auth_validator_error_validate_sequence_number_too_old"></a>

## Function `error_validate_sequence_number_too_old`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_sequence_number_too_old">error_validate_sequence_number_too_old</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_validate_sequence_number_too_new"></a>

## Function `error_validate_sequence_number_too_new`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_sequence_number_too_new">error_validate_sequence_number_too_new</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_validate_account_does_not_exist"></a>

## Function `error_validate_account_does_not_exist`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_account_does_not_exist">error_validate_account_does_not_exist</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_validate_cant_pay_gas_deposit"></a>

## Function `error_validate_cant_pay_gas_deposit`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_cant_pay_gas_deposit">error_validate_cant_pay_gas_deposit</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_validate_transaction_expired"></a>

## Function `error_validate_transaction_expired`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_transaction_expired">error_validate_transaction_expired</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_validate_bad_chain_id"></a>

## Function `error_validate_bad_chain_id`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_bad_chain_id">error_validate_bad_chain_id</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_validate_sequence_number_too_big"></a>

## Function `error_validate_sequence_number_too_big`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_sequence_number_too_big">error_validate_sequence_number_too_big</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_validate_max_gas_amount_exceeded"></a>

## Function `error_validate_max_gas_amount_exceeded`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_max_gas_amount_exceeded">error_validate_max_gas_amount_exceeded</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_validate_invalid_account_auth_key"></a>

## Function `error_validate_invalid_account_auth_key`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_invalid_account_auth_key">error_validate_invalid_account_auth_key</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_validate_invalid_authenticator"></a>

## Function `error_validate_invalid_authenticator`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_invalid_authenticator">error_validate_invalid_authenticator</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_validate_not_installed_auth_validator"></a>

## Function `error_validate_not_installed_auth_validator`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_not_installed_auth_validator">error_validate_not_installed_auth_validator</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_validate_session_is_expired"></a>

## Function `error_validate_session_is_expired`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_session_is_expired">error_validate_session_is_expired</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_validate_function_call_beyond_session_scope"></a>

## Function `error_validate_function_call_beyond_session_scope`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_validate_function_call_beyond_session_scope">error_validate_function_call_beyond_session_scope</a>(): u64
</code></pre>



<a name="0x3_auth_validator_new_auth_validator"></a>

## Function `new_auth_validator`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_new_auth_validator">new_auth_validator</a>(id: u64, module_address: <b>address</b>, module_name: <a href="_String">string::String</a>): <a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>
</code></pre>



<a name="0x3_auth_validator_validator_id"></a>

## Function `validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_validator_id">validator_id</a>(validator: &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>): u64
</code></pre>



<a name="0x3_auth_validator_validator_module_address"></a>

## Function `validator_module_address`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_validator_module_address">validator_module_address</a>(validator: &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>): <b>address</b>
</code></pre>



<a name="0x3_auth_validator_validator_module_name"></a>

## Function `validator_module_name`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_validator_module_name">validator_module_name</a>(validator: &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_auth_validator_new_tx_validate_result"></a>

## Function `new_tx_validate_result`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_new_tx_validate_result">new_tx_validate_result</a>(auth_validator_id: u64, <a href="auth_validator.md#0x3_auth_validator">auth_validator</a>: <a href="_Option">option::Option</a>&lt;<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>&gt;, <a href="session_key.md#0x3_session_key">session_key</a>: <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, <a href="bitcoin_address.md#0x3_bitcoin_address">bitcoin_address</a>: <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>): <a href="auth_validator.md#0x3_auth_validator_TxValidateResult">auth_validator::TxValidateResult</a>
</code></pre>



<a name="0x3_auth_validator_get_validate_result_from_ctx"></a>

## Function `get_validate_result_from_ctx`

Get the TxValidateResult from the TxContext, Only can be called after the transaction is validated


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_get_validate_result_from_ctx">get_validate_result_from_ctx</a>(): <a href="auth_validator.md#0x3_auth_validator_TxValidateResult">auth_validator::TxValidateResult</a>
</code></pre>



<a name="0x3_auth_validator_get_validator_id_from_ctx"></a>

## Function `get_validator_id_from_ctx`

Get the auth validator's id from the TxValidateResult in the TxContext


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_get_validator_id_from_ctx">get_validator_id_from_ctx</a>(): u64
</code></pre>



<a name="0x3_auth_validator_get_session_key_from_ctx_option"></a>

## Function `get_session_key_from_ctx_option`

Get the session key from the TxValidateResult in the TxContext
If the TxValidateResult is None or SessionKey is None, return None


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_get_session_key_from_ctx_option">get_session_key_from_ctx_option</a>(): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_auth_validator_is_validate_via_session_key"></a>

## Function `is_validate_via_session_key`

The current tx is validate via the session key or not


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_is_validate_via_session_key">is_validate_via_session_key</a>(): bool
</code></pre>



<a name="0x3_auth_validator_get_session_key_from_ctx"></a>

## Function `get_session_key_from_ctx`

Get the session key from the TxValidateResult in the TxContext
Only can be called after the transaction is validated


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_get_session_key_from_ctx">get_session_key_from_ctx</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_validator_get_bitcoin_address_from_ctx"></a>

## Function `get_bitcoin_address_from_ctx`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_get_bitcoin_address_from_ctx">get_bitcoin_address_from_ctx</a>(): <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>
</code></pre>
