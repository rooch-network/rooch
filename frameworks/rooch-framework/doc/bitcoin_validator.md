
<a name="0x3_bitcoin_validator"></a>

# Module `0x3::bitcoin_validator`

This module implements Bitcoin validator with the ECDSA recoverable signature over Secp256k1.


-  [Struct `BitcoinValidator`](#0x3_bitcoin_validator_BitcoinValidator)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0x3_bitcoin_validator_auth_validator_id)
-  [Function `validate`](#0x3_bitcoin_validator_validate)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::features</a>;
<b>use</b> <a href="">0x2::hash</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="auth_payload.md#0x3_auth_payload">0x3::auth_payload</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="bitcoin_address.md#0x3_bitcoin_address">0x3::bitcoin_address</a>;
<b>use</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1">0x3::ecdsa_k1</a>;
</code></pre>



<a name="0x3_bitcoin_validator_BitcoinValidator"></a>

## Struct `BitcoinValidator`



<pre><code><b>struct</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_BitcoinValidator">BitcoinValidator</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_bitcoin_validator_BITCOIN_AUTH_VALIDATOR_ID"></a>

there defines auth validator id for each auth validator


<pre><code><b>const</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_BITCOIN_AUTH_VALIDATOR_ID">BITCOIN_AUTH_VALIDATOR_ID</a>: u64 = 1;
</code></pre>



<a name="0x3_bitcoin_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0x3_bitcoin_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_validate">validate</a>(authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>
</code></pre>
