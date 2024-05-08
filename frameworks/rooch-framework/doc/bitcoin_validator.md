
<a name="0x3_bitcoin_validator"></a>

# Module `0x3::bitcoin_validator`

This module implements Bitcoin validator with the ECDSA recoverable signature over Secp256k1.


-  [Struct `BitcoinValidator`](#0x3_bitcoin_validator_BitcoinValidator)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0x3_bitcoin_validator_auth_validator_id)
-  [Function `rotate_authentication_key_entry`](#0x3_bitcoin_validator_rotate_authentication_key_entry)
-  [Function `remove_authentication_key_entry`](#0x3_bitcoin_validator_remove_authentication_key_entry)
-  [Function `public_key_to_authentication_key`](#0x3_bitcoin_validator_public_key_to_authentication_key)
-  [Function `get_authentication_key_option_from_account`](#0x3_bitcoin_validator_get_authentication_key_option_from_account)
-  [Function `is_authentication_key_in_account`](#0x3_bitcoin_validator_is_authentication_key_in_account)
-  [Function `get_authentication_key_from_account`](#0x3_bitcoin_validator_get_authentication_key_from_account)
-  [Function `validate_signature`](#0x3_bitcoin_validator_validate_signature)
-  [Function `validate`](#0x3_bitcoin_validator_validate)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::features</a>;
<b>use</b> <a href="">0x2::hash</a>;
<b>use</b> <a href="">0x2::hex</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="account_authentication.md#0x3_account_authentication">0x3::account_authentication</a>;
<b>use</b> <a href="auth_payload.md#0x3_auth_payload">0x3::auth_payload</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1">0x3::ecdsa_k1</a>;
<b>use</b> <a href="multichain_address.md#0x3_multichain_address">0x3::multichain_address</a>;
</code></pre>



<a name="0x3_bitcoin_validator_BitcoinValidator"></a>

## Struct `BitcoinValidator`



<pre><code><b>struct</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_BitcoinValidator">BitcoinValidator</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_bitcoin_validator_BITCOIN_AUTH_VALIDATOR_ID"></a>

there defines auth validator id for each blockchain


<pre><code><b>const</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_BITCOIN_AUTH_VALIDATOR_ID">BITCOIN_AUTH_VALIDATOR_ID</a>: u64 = 2;
</code></pre>



<a name="0x3_bitcoin_validator_ErrorInvalidPublicKeyLength"></a>



<pre><code><b>const</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_ErrorInvalidPublicKeyLength">ErrorInvalidPublicKeyLength</a>: u64 = 1;
</code></pre>



<a name="0x3_bitcoin_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0x3_bitcoin_validator_rotate_authentication_key_entry"></a>

## Function `rotate_authentication_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_rotate_authentication_key_entry">rotate_authentication_key_entry</a>(<a href="">account</a>: &<a href="">signer</a>, public_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_bitcoin_validator_remove_authentication_key_entry"></a>

## Function `remove_authentication_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_remove_authentication_key_entry">remove_authentication_key_entry</a>(<a href="">account</a>: &<a href="">signer</a>)
</code></pre>



<a name="0x3_bitcoin_validator_public_key_to_authentication_key"></a>

## Function `public_key_to_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_validator_get_authentication_key_option_from_account"></a>

## Function `get_authentication_key_option_from_account`

Get the authentication key option of the given account.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_get_authentication_key_option_from_account">get_authentication_key_option_from_account</a>(addr: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_bitcoin_validator_is_authentication_key_in_account"></a>

## Function `is_authentication_key_in_account`

The authentication key exists in account or not.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_is_authentication_key_in_account">is_authentication_key_in_account</a>(addr: <b>address</b>): bool
</code></pre>



<a name="0x3_bitcoin_validator_get_authentication_key_from_account"></a>

## Function `get_authentication_key_from_account`

Extract the authentication key of the authentication key option.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_get_authentication_key_from_account">get_authentication_key_from_account</a>(addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_validator_validate_signature"></a>

## Function `validate_signature`

Only validate the authenticator's signature.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_validate_signature">validate_signature</a>(payload: <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>, tx_hash: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_bitcoin_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_validate">validate</a>(authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>
</code></pre>
