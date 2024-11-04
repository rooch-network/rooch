
<a name="0x3_ethereum_validator"></a>

# Module `0x3::ethereum_validator`

This module implements Ethereum validator with the ECDSA recoverable signature over Secp256k1.


-  [Struct `EthereumValidator`](#0x3_ethereum_validator_EthereumValidator)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0x3_ethereum_validator_auth_validator_id)
-  [Function `genesis_init`](#0x3_ethereum_validator_genesis_init)
-  [Function `validate`](#0x3_ethereum_validator_validate)


<pre><code><b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry">0x3::auth_validator_registry</a>;
</code></pre>



<a name="0x3_ethereum_validator_EthereumValidator"></a>

## Struct `EthereumValidator`



<pre><code><b>struct</b> <a href="ethereum_validator.md#0x3_ethereum_validator_EthereumValidator">EthereumValidator</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ethereum_validator_ETHEREUM_AUTH_VALIDATOR_ID"></a>

there defines auth validator id for each blockchain


<pre><code><b>const</b> <a href="ethereum_validator.md#0x3_ethereum_validator_ETHEREUM_AUTH_VALIDATOR_ID">ETHEREUM_AUTH_VALIDATOR_ID</a>: u64 = 3;
</code></pre>



<a name="0x3_ethereum_validator_ErrorAddressMappingRecordNotFound"></a>



<pre><code><b>const</b> <a href="ethereum_validator.md#0x3_ethereum_validator_ErrorAddressMappingRecordNotFound">ErrorAddressMappingRecordNotFound</a>: u64 = 2;
</code></pre>



<a name="0x3_ethereum_validator_ErrorGenesisInitError"></a>



<pre><code><b>const</b> <a href="ethereum_validator.md#0x3_ethereum_validator_ErrorGenesisInitError">ErrorGenesisInitError</a>: u64 = 1;
</code></pre>



<a name="0x3_ethereum_validator_ErrorNotImplemented"></a>



<pre><code><b>const</b> <a href="ethereum_validator.md#0x3_ethereum_validator_ErrorNotImplemented">ErrorNotImplemented</a>: u64 = 3;
</code></pre>



<a name="0x3_ethereum_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="ethereum_validator.md#0x3_ethereum_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0x3_ethereum_validator_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ethereum_validator.md#0x3_ethereum_validator_genesis_init">genesis_init</a>()
</code></pre>



<a name="0x3_ethereum_validator_validate"></a>

## Function `validate`

We need to redesign the Ethereum auth validator
This module is just for placeholder the AUTH_VALIDATOR_ID


<pre><code><b>public</b> <b>fun</b> <a href="ethereum_validator.md#0x3_ethereum_validator_validate">validate</a>(_authenticator_payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>
