
<a name="0xa_bitcoin_multisign_validator"></a>

# Module `0xa::bitcoin_multisign_validator`



-  [Struct `BitcoinMultisignValidator`](#0xa_bitcoin_multisign_validator_BitcoinMultisignValidator)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0xa_bitcoin_multisign_validator_auth_validator_id)
-  [Function `genesis_init`](#0xa_bitcoin_multisign_validator_genesis_init)
-  [Function `validate`](#0xa_bitcoin_multisign_validator_validate)


<pre><code><b>use</b> <a href="">0x2::hash</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x3::auth_payload</a>;
<b>use</b> <a href="">0x3::auth_validator</a>;
<b>use</b> <a href="">0x3::auth_validator_registry</a>;
<b>use</b> <a href="">0x3::ecdsa_k1</a>;
<b>use</b> <a href="multisign_account.md#0xa_multisign_account">0xa::multisign_account</a>;
</code></pre>



<a name="0xa_bitcoin_multisign_validator_BitcoinMultisignValidator"></a>

## Struct `BitcoinMultisignValidator`



<pre><code><b>struct</b> <a href="bitcoin_multisign_validator.md#0xa_bitcoin_multisign_validator_BitcoinMultisignValidator">BitcoinMultisignValidator</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_bitcoin_multisign_validator_BITCOIN_MULTISIGN_VALIDATOR_ID"></a>

there defines auth validator id for each auth validator


<pre><code><b>const</b> <a href="bitcoin_multisign_validator.md#0xa_bitcoin_multisign_validator_BITCOIN_MULTISIGN_VALIDATOR_ID">BITCOIN_MULTISIGN_VALIDATOR_ID</a>: u64 = 2;
</code></pre>



<a name="0xa_bitcoin_multisign_validator_ErrorGenesisInitError"></a>



<pre><code><b>const</b> <a href="bitcoin_multisign_validator.md#0xa_bitcoin_multisign_validator_ErrorGenesisInitError">ErrorGenesisInitError</a>: u64 = 1;
</code></pre>



<a name="0xa_bitcoin_multisign_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_multisign_validator.md#0xa_bitcoin_multisign_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0xa_bitcoin_multisign_validator_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bitcoin_multisign_validator.md#0xa_bitcoin_multisign_validator_genesis_init">genesis_init</a>()
</code></pre>



<a name="0xa_bitcoin_multisign_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_multisign_validator.md#0xa_bitcoin_multisign_validator_validate">validate</a>(authenticator_payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>
