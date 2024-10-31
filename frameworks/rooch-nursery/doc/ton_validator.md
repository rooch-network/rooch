
<a name="0xa_ton_validator"></a>

# Module `0xa::ton_validator`

This module implements Ton blockchain auth validator.


-  [Struct `TonValidator`](#0xa_ton_validator_TonValidator)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0xa_ton_validator_auth_validator_id)
-  [Function `validate_signature`](#0xa_ton_validator_validate_signature)
-  [Function `validate`](#0xa_ton_validator_validate)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::hex</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x3::auth_validator</a>;
<b>use</b> <a href="ton_address.md#0xa_ton_address">0xa::ton_address</a>;
<b>use</b> <a href="ton_address_mapping.md#0xa_ton_address_mapping">0xa::ton_address_mapping</a>;
<b>use</b> <a href="ton_proof.md#0xa_ton_proof">0xa::ton_proof</a>;
</code></pre>



<a name="0xa_ton_validator_TonValidator"></a>

## Struct `TonValidator`



<pre><code><b>struct</b> <a href="ton_validator.md#0xa_ton_validator_TonValidator">TonValidator</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_ton_validator_ErrorAddressMappingRecordNotFound"></a>



<pre><code><b>const</b> <a href="ton_validator.md#0xa_ton_validator_ErrorAddressMappingRecordNotFound">ErrorAddressMappingRecordNotFound</a>: u64 = 1;
</code></pre>



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



<pre><code><b>public</b> <b>fun</b> <a href="ton_validator.md#0xa_ton_validator_validate_signature">validate_signature</a>(<a href="ton_address.md#0xa_ton_address">ton_address</a>: &<a href="ton_address.md#0xa_ton_address_TonAddress">ton_address::TonAddress</a>, proof: &<a href="ton_proof.md#0xa_ton_proof_TonProof">ton_proof::TonProof</a>, tx_hash: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0xa_ton_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="ton_validator.md#0xa_ton_validator_validate">validate</a>(authenticator_payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>
