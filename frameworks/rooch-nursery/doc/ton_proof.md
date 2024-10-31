
<a name="0xa_ton_proof"></a>

# Module `0xa::ton_proof`



-  [Struct `TonDomain`](#0xa_ton_proof_TonDomain)
-  [Struct `TonProof`](#0xa_ton_proof_TonProof)
-  [Function `decode_proof`](#0xa_ton_proof_decode_proof)
-  [Function `verify_proof`](#0xa_ton_proof_verify_proof)
-  [Function `domain`](#0xa_ton_proof_domain)
-  [Function `payload`](#0xa_ton_proof_payload)
-  [Function `signature`](#0xa_ton_proof_signature)
-  [Function `state_init`](#0xa_ton_proof_state_init)
-  [Function `timestamp`](#0xa_ton_proof_timestamp)
-  [Function `domain_length_bytes`](#0xa_ton_proof_domain_length_bytes)
-  [Function `domain_value`](#0xa_ton_proof_domain_value)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="ton_address.md#0xa_ton_address">0xa::ton_address</a>;
</code></pre>



<a name="0xa_ton_proof_TonDomain"></a>

## Struct `TonDomain`



<pre><code>#[data_struct]
<b>struct</b> <a href="ton_proof.md#0xa_ton_proof_TonDomain">TonDomain</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_ton_proof_TonProof"></a>

## Struct `TonProof`



<pre><code>#[data_struct]
<b>struct</b> <a href="ton_proof.md#0xa_ton_proof_TonProof">TonProof</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_ton_proof_decode_proof"></a>

## Function `decode_proof`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0xa_ton_proof_decode_proof">decode_proof</a>(ton_proof_bytes: <a href="">vector</a>&lt;u8&gt;): <a href="ton_proof.md#0xa_ton_proof_TonProof">ton_proof::TonProof</a>
</code></pre>



<a name="0xa_ton_proof_verify_proof"></a>

## Function `verify_proof`

verify the proof


<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0xa_ton_proof_verify_proof">verify_proof</a>(_ton_addr: &<a href="ton_address.md#0xa_ton_address_TonAddress">ton_address::TonAddress</a>, _ton_proof: &<a href="ton_proof.md#0xa_ton_proof_TonProof">ton_proof::TonProof</a>): bool
</code></pre>



<a name="0xa_ton_proof_domain"></a>

## Function `domain`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0xa_ton_proof_domain">domain</a>(<a href="ton_proof.md#0xa_ton_proof">ton_proof</a>: &<a href="ton_proof.md#0xa_ton_proof_TonProof">ton_proof::TonProof</a>): &<a href="ton_proof.md#0xa_ton_proof_TonDomain">ton_proof::TonDomain</a>
</code></pre>



<a name="0xa_ton_proof_payload"></a>

## Function `payload`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0xa_ton_proof_payload">payload</a>(<a href="ton_proof.md#0xa_ton_proof">ton_proof</a>: &<a href="ton_proof.md#0xa_ton_proof_TonProof">ton_proof::TonProof</a>): &<a href="_String">string::String</a>
</code></pre>



<a name="0xa_ton_proof_signature"></a>

## Function `signature`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0xa_ton_proof_signature">signature</a>(<a href="ton_proof.md#0xa_ton_proof">ton_proof</a>: &<a href="ton_proof.md#0xa_ton_proof_TonProof">ton_proof::TonProof</a>): &<a href="_String">string::String</a>
</code></pre>



<a name="0xa_ton_proof_state_init"></a>

## Function `state_init`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0xa_ton_proof_state_init">state_init</a>(<a href="ton_proof.md#0xa_ton_proof">ton_proof</a>: &<a href="ton_proof.md#0xa_ton_proof_TonProof">ton_proof::TonProof</a>): &<a href="_String">string::String</a>
</code></pre>



<a name="0xa_ton_proof_timestamp"></a>

## Function `timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="">timestamp</a>(<a href="ton_proof.md#0xa_ton_proof">ton_proof</a>: &<a href="ton_proof.md#0xa_ton_proof_TonProof">ton_proof::TonProof</a>): u64
</code></pre>



<a name="0xa_ton_proof_domain_length_bytes"></a>

## Function `domain_length_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0xa_ton_proof_domain_length_bytes">domain_length_bytes</a>(ton_domain: &<a href="ton_proof.md#0xa_ton_proof_TonDomain">ton_proof::TonDomain</a>): u64
</code></pre>



<a name="0xa_ton_proof_domain_value"></a>

## Function `domain_value`



<pre><code><b>public</b> <b>fun</b> <a href="ton_proof.md#0xa_ton_proof_domain_value">domain_value</a>(ton_domain: &<a href="ton_proof.md#0xa_ton_proof_TonDomain">ton_proof::TonDomain</a>): &<a href="_String">string::String</a>
</code></pre>
