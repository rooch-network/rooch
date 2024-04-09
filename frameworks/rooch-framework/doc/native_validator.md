
<a name="0x3_native_validator"></a>

# Module `0x3::native_validator`

This module implements the native validator.


-  [Struct `NativeValidator`](#0x3_native_validator_NativeValidator)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0x3_native_validator_auth_validator_id)
-  [Function `rotate_authentication_key_entry`](#0x3_native_validator_rotate_authentication_key_entry)
-  [Function `remove_authentication_key_entry`](#0x3_native_validator_remove_authentication_key_entry)
-  [Function `get_public_key_from_authenticator_payload`](#0x3_native_validator_get_public_key_from_authenticator_payload)
-  [Function `get_signature_from_authenticator_payload`](#0x3_native_validator_get_signature_from_authenticator_payload)
-  [Function `get_authentication_key_from_authenticator_payload`](#0x3_native_validator_get_authentication_key_from_authenticator_payload)
-  [Function `public_key_to_address`](#0x3_native_validator_public_key_to_address)
-  [Function `public_key_to_authentication_key`](#0x3_native_validator_public_key_to_authentication_key)
-  [Function `get_authentication_key_with_default`](#0x3_native_validator_get_authentication_key_with_default)
-  [Function `default_authentication_key`](#0x3_native_validator_default_authentication_key)
-  [Function `validate_signature`](#0x3_native_validator_validate_signature)
-  [Function `validate`](#0x3_native_validator_validate)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="account_authentication.md#0x3_account_authentication">0x3::account_authentication</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="ed25519.md#0x3_ed25519">0x3::ed25519</a>;
<b>use</b> <a href="hash.md#0x3_hash">0x3::hash</a>;
</code></pre>



<a name="0x3_native_validator_NativeValidator"></a>

## Struct `NativeValidator`



<pre><code><b>struct</b> <a href="native_validator.md#0x3_native_validator_NativeValidator">NativeValidator</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_native_validator_ErrorInvalidPublicKeyLength"></a>



<pre><code><b>const</b> <a href="native_validator.md#0x3_native_validator_ErrorInvalidPublicKeyLength">ErrorInvalidPublicKeyLength</a>: u64 = 1;
</code></pre>



<a name="0x3_native_validator_NATIVE_VALIDATOR_ID"></a>

there defines auth validator id for each blockchain


<pre><code><b>const</b> <a href="native_validator.md#0x3_native_validator_NATIVE_VALIDATOR_ID">NATIVE_VALIDATOR_ID</a>: u64 = 0;
</code></pre>



<a name="0x3_native_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0x3_native_validator_rotate_authentication_key_entry"></a>

## Function `rotate_authentication_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="native_validator.md#0x3_native_validator_rotate_authentication_key_entry">rotate_authentication_key_entry</a>(<a href="">account</a>: &<a href="">signer</a>, public_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_native_validator_remove_authentication_key_entry"></a>

## Function `remove_authentication_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="native_validator.md#0x3_native_validator_remove_authentication_key_entry">remove_authentication_key_entry</a>(<a href="">account</a>: &<a href="">signer</a>)
</code></pre>



<a name="0x3_native_validator_get_public_key_from_authenticator_payload"></a>

## Function `get_public_key_from_authenticator_payload`



<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_native_validator_get_signature_from_authenticator_payload"></a>

## Function `get_signature_from_authenticator_payload`



<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_get_signature_from_authenticator_payload">get_signature_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_native_validator_get_authentication_key_from_authenticator_payload"></a>

## Function `get_authentication_key_from_authenticator_payload`

Get the authentication key of the given authenticator from authenticator_payload.


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_get_authentication_key_from_authenticator_payload">get_authentication_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_native_validator_public_key_to_address"></a>

## Function `public_key_to_address`



<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_public_key_to_address">public_key_to_address</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<a name="0x3_native_validator_public_key_to_authentication_key"></a>

## Function `public_key_to_authentication_key`

Get the authentication key of the given public key.


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_native_validator_get_authentication_key_with_default"></a>

## Function `get_authentication_key_with_default`

Get the authentication key of the given account, if it not exist, return the account address as authentication key.


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_get_authentication_key_with_default">get_authentication_key_with_default</a>(addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_native_validator_default_authentication_key"></a>

## Function `default_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_default_authentication_key">default_authentication_key</a>(addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_native_validator_validate_signature"></a>

## Function `validate_signature`

Only validate the authenticator's signature.


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_validate_signature">validate_signature</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;, tx_hash: &<a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_native_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_validate">validate</a>(authenticator_payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>
