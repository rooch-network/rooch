
<a name="0x3_auth_validator"></a>

# Module `0x3::auth_validator`

This module contains the error code for auth_validator module
The auth_validator implementation should contain the following functions
public fun validate(ctx: &StorageContext, payload: vector<u8>)
fun pre_execute(ctx: &mut StorageContext)
fun post_execute(ctx: &mut StorageContext)


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
-  [Function `is_validate_via_session_key`](#0x3_auth_validator_is_validate_via_session_key)
-  [Function `get_session_key`](#0x3_auth_validator_get_session_key)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
</code></pre>



<a name="0x3_auth_validator_AuthValidator"></a>

## Struct `AuthValidator`

The Authentication Validator


<pre><code><b>struct</b> <a href="auth_validator.md#0x3_auth_validator_AuthValidator">AuthValidator</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>module_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>module_name: <a href="_String">ascii::String</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_auth_validator_TxValidateResult"></a>

## Struct `TxValidateResult`

The Transaction Validate Result
this result will be stored in the TxContext


<pre><code><b>struct</b> <a href="auth_validator.md#0x3_auth_validator_TxValidateResult">TxValidateResult</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="auth_validator.md#0x3_auth_validator">auth_validator</a>: <a href="_Option">option::Option</a>&lt;<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code><a href="session_key.md#0x3_session_key">session_key</a>: <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_auth_validator_EValidateInvalidAccountAuthKey"></a>

The AuthKey in transaction's authenticator do not match with the sender's account auth key


<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_EValidateInvalidAccountAuthKey">EValidateInvalidAccountAuthKey</a>: u64 = 1001;
</code></pre>



<a name="0x3_auth_validator_EValidateInvalidAuthenticator"></a>

InvalidAuthenticator, include invalid signature


<pre><code><b>const</b> <a href="auth_validator.md#0x3_auth_validator_EValidateInvalidAuthenticator">EValidateInvalidAuthenticator</a>: u64 = 1002;
</code></pre>



<a name="0x3_auth_validator_error_invalid_account_auth_key"></a>

## Function `error_invalid_account_auth_key`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_invalid_account_auth_key">error_invalid_account_auth_key</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_invalid_account_auth_key">error_invalid_account_auth_key</a>(): u64 {
   <a href="_invalid_argument">error::invalid_argument</a>(<a href="auth_validator.md#0x3_auth_validator_EValidateInvalidAccountAuthKey">EValidateInvalidAccountAuthKey</a>)
}
</code></pre>



</details>

<a name="0x3_auth_validator_error_invalid_authenticator"></a>

## Function `error_invalid_authenticator`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_invalid_authenticator">error_invalid_authenticator</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_error_invalid_authenticator">error_invalid_authenticator</a>(): u64 {
   <a href="_invalid_argument">error::invalid_argument</a>(<a href="auth_validator.md#0x3_auth_validator_EValidateInvalidAuthenticator">EValidateInvalidAuthenticator</a>)
}
</code></pre>



</details>

<a name="0x3_auth_validator_new_auth_validator"></a>

## Function `new_auth_validator`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_new_auth_validator">new_auth_validator</a>(id: u64, module_address: <b>address</b>, module_name: <a href="_String">ascii::String</a>): <a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_new_auth_validator">new_auth_validator</a>(id: u64, module_address: <b>address</b>, module_name: std::ascii::String): <a href="auth_validator.md#0x3_auth_validator_AuthValidator">AuthValidator</a> {
   <a href="auth_validator.md#0x3_auth_validator_AuthValidator">AuthValidator</a> {
      id: id,
      module_address: module_address,
      module_name: module_name,
   }
}
</code></pre>



</details>

<a name="0x3_auth_validator_validator_id"></a>

## Function `validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_validator_id">validator_id</a>(validator: &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_validator_id">validator_id</a>(validator: &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">AuthValidator</a>): u64 {
     validator.id
}
</code></pre>



</details>

<a name="0x3_auth_validator_validator_module_address"></a>

## Function `validator_module_address`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_validator_module_address">validator_module_address</a>(validator: &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_validator_module_address">validator_module_address</a>(validator: &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">AuthValidator</a>): <b>address</b> {
   validator.module_address
}
</code></pre>



</details>

<a name="0x3_auth_validator_validator_module_name"></a>

## Function `validator_module_name`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_validator_module_name">validator_module_name</a>(validator: &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>): <a href="_String">ascii::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_validator_module_name">validator_module_name</a>(validator: &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">AuthValidator</a>): std::ascii::String {
   validator.module_name
}
</code></pre>



</details>

<a name="0x3_auth_validator_new_tx_validate_result"></a>

## Function `new_tx_validate_result`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_new_tx_validate_result">new_tx_validate_result</a>(<a href="auth_validator.md#0x3_auth_validator">auth_validator</a>: <a href="_Option">option::Option</a>&lt;<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>&gt;, <a href="session_key.md#0x3_session_key">session_key</a>: <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <a href="auth_validator.md#0x3_auth_validator_TxValidateResult">auth_validator::TxValidateResult</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_new_tx_validate_result">new_tx_validate_result</a>(<a href="auth_validator.md#0x3_auth_validator">auth_validator</a>: Option&lt;<a href="auth_validator.md#0x3_auth_validator_AuthValidator">AuthValidator</a>&gt;, <a href="session_key.md#0x3_session_key">session_key</a>: Option&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <a href="auth_validator.md#0x3_auth_validator_TxValidateResult">TxValidateResult</a> {
   <a href="auth_validator.md#0x3_auth_validator_TxValidateResult">TxValidateResult</a> {
      <a href="auth_validator.md#0x3_auth_validator">auth_validator</a>: <a href="auth_validator.md#0x3_auth_validator">auth_validator</a>,
      <a href="session_key.md#0x3_session_key">session_key</a>: <a href="session_key.md#0x3_session_key">session_key</a>,
   }
}
</code></pre>



</details>

<a name="0x3_auth_validator_is_validate_via_session_key"></a>

## Function `is_validate_via_session_key`

The current tx is validate via the session key or not


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_is_validate_via_session_key">is_validate_via_session_key</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_is_validate_via_session_key">is_validate_via_session_key</a>(ctx: &StorageContext) : bool {
   <a href="_is_some">option::is_some</a>(&<a href="auth_validator.md#0x3_auth_validator_get_session_key">get_session_key</a>(ctx))
}
</code></pre>



</details>

<a name="0x3_auth_validator_get_session_key"></a>

## Function `get_session_key`

Get the session key from the TxValidateResult in the TxContext


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_get_session_key">get_session_key</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator.md#0x3_auth_validator_get_session_key">get_session_key</a>(ctx: &StorageContext): Option&lt;<a href="">vector</a>&lt;u8&gt;&gt;{
   <b>let</b> validate_result_opt = <a href="_get">storage_context::get</a>&lt;<a href="auth_validator.md#0x3_auth_validator_TxValidateResult">TxValidateResult</a>&gt;(ctx);
   <b>if</b>(<a href="_is_some">option::is_some</a>(&validate_result_opt)){
      <b>let</b> validate_result = <a href="_extract">option::extract</a>(&<b>mut</b> validate_result_opt);
      validate_result.<a href="session_key.md#0x3_session_key">session_key</a>
   }<b>else</b>{
      <a href="_none">option::none</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;()
   }
}
</code></pre>



</details>
