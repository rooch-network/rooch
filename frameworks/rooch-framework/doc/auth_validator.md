
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
-  [Function `error_invalid_account_auth_key`](#0x3_auth_validator_error_invalid_account_auth_key)
-  [Function `error_invalid_authenticator`](#0x3_auth_validator_error_invalid_authenticator)
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


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
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



<a name="0x3_auth_validator_ErrorValidateInvalidAccountAuthKey"></a>

The AuthKey in transaction's authenticator do not match with the sender's account auth key


<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateInvalidAccountAuthKey">ErrorValidateInvalidAccountAuthKey</a>: u64 = 1001;
</code></pre>



<a name="0x3_auth_validator_ErrorValidateInvalidAuthenticator"></a>

InvalidAuthenticator, include invalid signature


<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_ErrorValidateInvalidAuthenticator">ErrorValidateInvalidAuthenticator</a>: u64 = 1002;
</code></pre>



<a name="0x3_auth_validator_error_invalid_account_auth_key"></a>

## Function `error_invalid_account_auth_key`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_invalid_account_auth_key">error_invalid_account_auth_key</a>(): u64
</code></pre>



<a name="0x3_auth_validator_error_invalid_authenticator"></a>

## Function `error_invalid_authenticator`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_invalid_authenticator">error_invalid_authenticator</a>(): u64
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



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_new_tx_validate_result">new_tx_validate_result</a>(auth_validator_id: u64, <a href="auth_validator.md#0x3_auth_validator">auth_validator</a>: <a href="_Option">option::Option</a>&lt;<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>&gt;, <a href="session_key.md#0x3_session_key">session_key</a>: <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <a href="auth_validator.md#0x3_auth_validator_TxValidateResult">auth_validator::TxValidateResult</a>
</code></pre>



<a name="0x3_auth_validator_get_validate_result_from_ctx"></a>

## Function `get_validate_result_from_ctx`

Get the TxValidateResult from the TxContext, Only can be called after the transaction is validated


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_get_validate_result_from_ctx">get_validate_result_from_ctx</a>(): <a href="auth_validator.md#0x3_auth_validator_TxValidateResult">auth_validator::TxValidateResult</a>
</code></pre>



<a name="0x3_auth_validator_get_validator_id_from_ctx"></a>

## Function `get_validator_id_from_ctx`

Get the auth validator's id from the TxValidateResult in the TxContext


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_get_validator_id_from_ctx">get_validator_id_from_ctx</a>(): u64
</code></pre>



<a name="0x3_auth_validator_get_session_key_from_ctx_option"></a>

## Function `get_session_key_from_ctx_option`

Get the session key from the TxValidateResult in the TxContext
If the TxValidateResult is None or SessionKey is None, return None


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_get_session_key_from_ctx_option">get_session_key_from_ctx_option</a>(): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_auth_validator_is_validate_via_session_key"></a>

## Function `is_validate_via_session_key`

The current tx is validate via the session key or not


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_is_validate_via_session_key">is_validate_via_session_key</a>(): bool
</code></pre>



<a name="0x3_auth_validator_get_session_key_from_ctx"></a>

## Function `get_session_key_from_ctx`

Get the session key from the TxValidateResult in the TxContext
Only can be called after the transaction is validated


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_get_session_key_from_ctx">get_session_key_from_ctx</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>
