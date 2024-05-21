
<a name="0x3_ethereum_validator"></a>

# Module `0x3::ethereum_validator`

This module implements Ethereum validator with the ECDSA recoverable signature over Secp256k1.


-  [Struct `EthereumValidator`](#0x3_ethereum_validator_EthereumValidator)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0x3_ethereum_validator_auth_validator_id)
-  [Function `validate_signature`](#0x3_ethereum_validator_validate_signature)
-  [Function `validate`](#0x3_ethereum_validator_validate)


<pre><code><b>use</b> <a href="">0x1::debug</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::features</a>;
<b>use</b> <a href="">0x2::hex</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="auth_payload.md#0x3_auth_payload">0x3::auth_payload</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1">0x3::ecdsa_k1</a>;
<b>use</b> <a href="ethereum_address.md#0x3_ethereum_address">0x3::ethereum_address</a>;
<b>use</b> <a href="multichain_address.md#0x3_multichain_address">0x3::multichain_address</a>;
</code></pre>



<a name="0x3_ethereum_validator_EthereumValidator"></a>

## Struct `EthereumValidator`



<pre><code><b>struct</b> <a href="ethereum_validator.md#0x3_ethereum_validator_EthereumValidator">EthereumValidator</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ethereum_validator_ETHEREUM_AUTH_VALIDATOR_ID"></a>

there defines auth validator id for each blockchain


<pre><code><b>const</b> <a href="ethereum_validator.md#0x3_ethereum_validator_ETHEREUM_AUTH_VALIDATOR_ID">ETHEREUM_AUTH_VALIDATOR_ID</a>: u64 = 1;
</code></pre>



<a name="0x3_ethereum_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="ethereum_validator.md#0x3_ethereum_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0x3_ethereum_validator_validate_signature"></a>

## Function `validate_signature`

Only validate the authenticator's signature.


<pre><code><b>public</b> <b>fun</b> <a href="ethereum_validator.md#0x3_ethereum_validator_validate_signature">validate_signature</a>(payload: <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>, tx_hash: <a href="">vector</a>&lt;u8&gt;): <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ethereum_address::ETHAddress</a>
</code></pre>



<a name="0x3_ethereum_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="ethereum_validator.md#0x3_ethereum_validator_validate">validate</a>(authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>
</code></pre>
