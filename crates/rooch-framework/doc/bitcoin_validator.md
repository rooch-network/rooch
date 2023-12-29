
<a name="0x3_bitcoin_validator"></a>

# Module `0x3::bitcoin_validator`

This module implements Bitcoin validator with the ECDSA recoverable signature over Secp256k1.


-  [Struct `BitcoinValidator`](#0x3_bitcoin_validator_BitcoinValidator)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0x3_bitcoin_validator_auth_validator_id)
-  [Function `get_address_from_authenticator_payload`](#0x3_bitcoin_validator_get_address_from_authenticator_payload)
-  [Function `get_signature_from_authenticator_payload`](#0x3_bitcoin_validator_get_signature_from_authenticator_payload)
-  [Function `validate_signature`](#0x3_bitcoin_validator_validate_signature)
-  [Function `validate`](#0x3_bitcoin_validator_validate)


<pre><code><b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable">0x3::ecdsa_k1_recoverable</a>;
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



<a name="0x3_bitcoin_validator_get_address_from_authenticator_payload"></a>

## Function `get_address_from_authenticator_payload`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_get_address_from_authenticator_payload">get_address_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_validator_get_signature_from_authenticator_payload"></a>

## Function `get_signature_from_authenticator_payload`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_get_signature_from_authenticator_payload">get_signature_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_validator_validate_signature"></a>

## Function `validate_signature`

Only validate the authenticator's signature.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_validate_signature">validate_signature</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;, tx_hash: &<a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_bitcoin_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_validate">validate</a>(ctx: &<a href="_Context">context::Context</a>, authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>
</code></pre>
