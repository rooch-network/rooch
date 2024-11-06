
<a name="0x3_ton_proof"></a>

# Module `0x3::ton_proof`



-  [Struct `TonDomain`](#0x3_ton_proof_TonDomain)
-  [Struct `TonProof`](#0x3_ton_proof_TonProof)
-  [Struct `TonProofData`](#0x3_ton_proof_TonProofData)
-  [Struct `RawCell`](#0x3_ton_proof_RawCell)
-  [Struct `RawBagOfCells`](#0x3_ton_proof_RawBagOfCells)
-  [Constants](#@Constants_0)
-  [Function `decode_proof_data`](#0x3_ton_proof_decode_proof_data)
-  [Function `verify_proof`](#0x3_ton_proof_verify_proof)
-  [Function `name`](#0x3_ton_proof_name)
-  [Function `proof`](#0x3_ton_proof_proof)
-  [Function `state_init`](#0x3_ton_proof_state_init)
-  [Function `domain`](#0x3_ton_proof_domain)
-  [Function `payload`](#0x3_ton_proof_payload)
-  [Function `payload_message`](#0x3_ton_proof_payload_message)
-  [Function `payload_bitcoin_address`](#0x3_ton_proof_payload_bitcoin_address)
-  [Function `payload_tx_hash`](#0x3_ton_proof_payload_tx_hash)
-  [Function `signature`](#0x3_ton_proof_signature)
-  [Function `timestamp`](#0x3_ton_proof_timestamp)
-  [Function `domain_length_bytes`](#0x3_ton_proof_domain_length_bytes)
-  [Function `domain_value`](#0x3_ton_proof_domain_value)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="ton_address.md#0x3_ton_address">0x3::ton_address</a>;
</code></pre>



<a name="0x3_ton_proof_TonDomain"></a>

## Struct `TonDomain`



<pre><code>#[data_struct]
<b>struct</b> <a href="ton_proof.md#0x3_ton_proof_TonDomain">TonDomain</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_ton_proof_TonProof"></a>

## Struct `TonProof`



<pre><code>#[data_struct]
<b>struct</b> <a href="ton_proof.md#0x3_ton_proof_TonProof">TonProof</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_ton_proof_TonProofData"></a>

## Struct `TonProofData`



<pre><code>#[data_struct]
<b>struct</b> <a href="ton_proof.md#0x3_ton_proof_TonProofData">TonProofData</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_ton_proof_RawCell"></a>

## Struct `RawCell`



<pre><code>#[data_struct]
<b>struct</b> <a href="ton_proof.md#0x3_ton_proof_RawCell">RawCell</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_ton_proof_RawBagOfCells"></a>

## Struct `RawBagOfCells`



<pre><code>#[data_struct]
<b>struct</b> <a href="ton_proof.md#0x3_ton_proof_RawBagOfCells">RawBagOfCells</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ton_proof_PAYLOAD_BITCOIN_ADDRESS_IDX"></a>



<pre><code><b>const</b> <a href="ton_proof.md#0x3_ton_proof_PAYLOAD_BITCOIN_ADDRESS_IDX">PAYLOAD_BITCOIN_ADDRESS_IDX</a>: u64 = 1;
</code></pre>



<a name="0x3_ton_proof_PAYLOAD_MESSAGE_IDX"></a>



<pre><code><b>const</b> <a href="ton_proof.md#0x3_ton_proof_PAYLOAD_MESSAGE_IDX">PAYLOAD_MESSAGE_IDX</a>: u64 = 0;
</code></pre>



<a name="0x3_ton_proof_PAYLOAD_TX_HASH_IDX"></a>



<pre><code><b>const</b> <a href="ton_proof.md#0x3_ton_proof_PAYLOAD_TX_HASH_IDX">PAYLOAD_TX_HASH_IDX</a>: u64 = 2;
</code></pre>



<a name="0x3_ton_proof_decode_proof_data"></a>

## Function `decode_proof_data`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_decode_proof_data">decode_proof_data</a>(proof_data_bytes: <a href="">vector</a>&lt;u8&gt;): <a href="ton_proof.md#0x3_ton_proof_TonProofData">ton_proof::TonProofData</a>
</code></pre>



<a name="0x3_ton_proof_verify_proof"></a>

## Function `verify_proof`

verify the proof


<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_verify_proof">verify_proof</a>(_ton_addr: &<a href="ton_address.md#0x3_ton_address_TonAddress">ton_address::TonAddress</a>, _ton_proof_data: &<a href="ton_proof.md#0x3_ton_proof_TonProofData">ton_proof::TonProofData</a>): bool
</code></pre>



<a name="0x3_ton_proof_name"></a>

## Function `name`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_name">name</a>(ton_proof_data: &<a href="ton_proof.md#0x3_ton_proof_TonProofData">ton_proof::TonProofData</a>): &<a href="_String">string::String</a>
</code></pre>



<a name="0x3_ton_proof_proof"></a>

## Function `proof`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_proof">proof</a>(ton_proof_data: &<a href="ton_proof.md#0x3_ton_proof_TonProofData">ton_proof::TonProofData</a>): &<a href="ton_proof.md#0x3_ton_proof_TonProof">ton_proof::TonProof</a>
</code></pre>



<a name="0x3_ton_proof_state_init"></a>

## Function `state_init`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_state_init">state_init</a>(ton_proof_data: &<a href="ton_proof.md#0x3_ton_proof_TonProofData">ton_proof::TonProofData</a>): &<a href="_String">string::String</a>
</code></pre>



<a name="0x3_ton_proof_domain"></a>

## Function `domain`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_domain">domain</a>(<a href="ton_proof.md#0x3_ton_proof">ton_proof</a>: &<a href="ton_proof.md#0x3_ton_proof_TonProof">ton_proof::TonProof</a>): &<a href="ton_proof.md#0x3_ton_proof_TonDomain">ton_proof::TonDomain</a>
</code></pre>



<a name="0x3_ton_proof_payload"></a>

## Function `payload`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_payload">payload</a>(<a href="ton_proof.md#0x3_ton_proof">ton_proof</a>: &<a href="ton_proof.md#0x3_ton_proof_TonProof">ton_proof::TonProof</a>): &<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x3_ton_proof_payload_message"></a>

## Function `payload_message`

Get the message from the payload, if the payload is not long enough, return an empty string


<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_payload_message">payload_message</a>(<a href="ton_proof.md#0x3_ton_proof">ton_proof</a>: &<a href="ton_proof.md#0x3_ton_proof_TonProof">ton_proof::TonProof</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_ton_proof_payload_bitcoin_address"></a>

## Function `payload_bitcoin_address`

Get the bitcoin address from the payload, if the payload is not long enough, return an empty string


<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_payload_bitcoin_address">payload_bitcoin_address</a>(<a href="ton_proof.md#0x3_ton_proof">ton_proof</a>: &<a href="ton_proof.md#0x3_ton_proof_TonProof">ton_proof::TonProof</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_ton_proof_payload_tx_hash"></a>

## Function `payload_tx_hash`

Get the tx hash from the payload, if the payload is not long enough, return an empty string


<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_payload_tx_hash">payload_tx_hash</a>(<a href="ton_proof.md#0x3_ton_proof">ton_proof</a>: &<a href="ton_proof.md#0x3_ton_proof_TonProof">ton_proof::TonProof</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_ton_proof_signature"></a>

## Function `signature`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_signature">signature</a>(<a href="ton_proof.md#0x3_ton_proof">ton_proof</a>: &<a href="ton_proof.md#0x3_ton_proof_TonProof">ton_proof::TonProof</a>): &<a href="_String">string::String</a>
</code></pre>



<a name="0x3_ton_proof_timestamp"></a>

## Function `timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="">timestamp</a>(<a href="ton_proof.md#0x3_ton_proof">ton_proof</a>: &<a href="ton_proof.md#0x3_ton_proof_TonProof">ton_proof::TonProof</a>): u64
</code></pre>



<a name="0x3_ton_proof_domain_length_bytes"></a>

## Function `domain_length_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_domain_length_bytes">domain_length_bytes</a>(ton_domain: &<a href="ton_proof.md#0x3_ton_proof_TonDomain">ton_proof::TonDomain</a>): u64
</code></pre>



<a name="0x3_ton_proof_domain_value"></a>

## Function `domain_value`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0x3_ton_proof_domain_value">domain_value</a>(ton_domain: &<a href="ton_proof.md#0x3_ton_proof_TonDomain">ton_proof::TonDomain</a>): &<a href="_String">string::String</a>
</code></pre>
