
<a name="0xa_ton_validator"></a>

# Module `0xa::ton_validator`

This module implements Ton blockchain auth validator.


-  [Struct `TonValidator`](#0xa_ton_validator_TonValidator)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0xa_ton_validator_auth_validator_id)
-  [Function `validate_signature`](#0xa_ton_validator_validate_signature)
-  [Function `validate`](#0xa_ton_validator_validate)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::features</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x3::auth_payload</a>;
<b>use</b> <a href="">0x3::auth_validator</a>;
<b>use</b> <a href="">0x3::ecdsa_k1</a>;
<b>use</b> <a href="">0x3::ethereum_address</a>;
<b>use</b> <a href="">0x3::multichain_address</a>;
</code></pre>



<a name="0xa_ton_validator_TonValidator"></a>

## Struct `TonValidator`



<pre><code><b>struct</b> <a href="ton_validator.md#0xa_ton_validator_TonValidator">TonValidator</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_ton_validator_TON_AUTH_VALIDATOR_ID"></a>

there defines auth validator id for each blockchain


<pre><code><b>const</b> <a href="ton_validator.md#0xa_ton_validator_TON_AUTH_VALIDATOR_ID">TON_AUTH_VALIDATOR_ID</a>: u64 = 3;
</code></pre>



<a name="0xa_ton_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="ton_validator.md#0xa_ton_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0xa_ton_validator_validate_signature"></a>

## Function `validate_signature`

Only validate the authenticator's signature.


<pre><code><b>public</b> <b>fun</b> <a href="ton_validator.md#0xa_ton_validator_validate_signature">validate_signature</a>(payload: &<a href="_AuthPayload">auth_payload::AuthPayload</a>, tx_hash: <a href="">vector</a>&lt;u8&gt;): <a href="_ETHAddress">ethereum_address::ETHAddress</a>
</code></pre>



<a name="0xa_ton_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="ton_validator.md#0xa_ton_validator_validate">validate</a>(authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="_MultiChainAddress">multichain_address::MultiChainAddress</a>
</code></pre>
