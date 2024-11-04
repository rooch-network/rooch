
<a name="0x3_ton_validator"></a>

# Module `0x3::ton_validator`

This module implements Ton blockchain auth validator.


-  [Struct `TonValidator`](#0x3_ton_validator_TonValidator)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0x3_ton_validator_auth_validator_id)
-  [Function `genesis_init`](#0x3_ton_validator_genesis_init)
-  [Function `validate`](#0x3_ton_validator_validate)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::hex</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="address_mapping.md#0x3_address_mapping">0x3::address_mapping</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry">0x3::auth_validator_registry</a>;
<b>use</b> <a href="ton_address.md#0x3_ton_address">0x3::ton_address</a>;
<b>use</b> <a href="ton_proof.md#0x3_ton_proof">0x3::ton_proof</a>;
</code></pre>



<a name="0x3_ton_validator_TonValidator"></a>

## Struct `TonValidator`



<pre><code><b>struct</b> <a href="ton_validator.md#0x3_ton_validator_TonValidator">TonValidator</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ton_validator_ErrorAddressMappingRecordNotFound"></a>



<pre><code><b>const</b> <a href="ton_validator.md#0x3_ton_validator_ErrorAddressMappingRecordNotFound">ErrorAddressMappingRecordNotFound</a>: u64 = 2;
</code></pre>



<a name="0x3_ton_validator_ErrorGenesisInitError"></a>



<pre><code><b>const</b> <a href="ton_validator.md#0x3_ton_validator_ErrorGenesisInitError">ErrorGenesisInitError</a>: u64 = 1;
</code></pre>



<a name="0x3_ton_validator_TON_AUTH_VALIDATOR_ID"></a>

there defines auth validator id for each blockchain


<pre><code><b>const</b> <a href="ton_validator.md#0x3_ton_validator_TON_AUTH_VALIDATOR_ID">TON_AUTH_VALIDATOR_ID</a>: u64 = 4;
</code></pre>



<a name="0x3_ton_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="ton_validator.md#0x3_ton_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0x3_ton_validator_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ton_validator.md#0x3_ton_validator_genesis_init">genesis_init</a>()
</code></pre>



<a name="0x3_ton_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="ton_validator.md#0x3_ton_validator_validate">validate</a>(authenticator_payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>
