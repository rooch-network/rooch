
<a name="0x3_auth_validator"></a>

# Module `0x3::auth_validator`

This module contains the error code for auth_validator module
The auth_validator implementation should contain the following functions
public fun validate(ctx: &StorageContext, payload: vector<u8>)
fun pre_execute(ctx: &mut StorageContext)
fun post_execute(ctx: &mut StorageContext)


-  [Constants](#@Constants_0)
-  [Function `error_invalid_account_auth_key`](#0x3_auth_validator_error_invalid_account_auth_key)
-  [Function `error_invalid_authenticator`](#0x3_auth_validator_error_invalid_authenticator)


<pre><code><b>use</b> <a href="">0x1::error</a>;
</code></pre>



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
