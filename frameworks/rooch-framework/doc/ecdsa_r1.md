
<a name="0x3_ecdsa_r1"></a>

# Module `0x3::ecdsa_r1`



-  [Constants](#@Constants_0)
-  [Function `verify`](#0x3_ecdsa_r1_verify)
-  [Function `public_key_length`](#0x3_ecdsa_r1_public_key_length)
-  [Function `raw_signature_length`](#0x3_ecdsa_r1_raw_signature_length)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ecdsa_r1_ErrorInvalidPubKey"></a>



<pre><code><b>const</b> <a href="ecdsa_r1.md#0x3_ecdsa_r1_ErrorInvalidPubKey">ErrorInvalidPubKey</a>: u64 = 2;
</code></pre>



<a name="0x3_ecdsa_r1_ErrorInvalidSignature"></a>



<pre><code><b>const</b> <a href="ecdsa_r1.md#0x3_ecdsa_r1_ErrorInvalidSignature">ErrorInvalidSignature</a>: u64 = 1;
</code></pre>



<a name="0x3_ecdsa_r1_ECDSA_R1_COMPRESSED_PUBKEY_LENGTH"></a>

Compressed public key length for P-256


<pre><code><b>const</b> <a href="ecdsa_r1.md#0x3_ecdsa_r1_ECDSA_R1_COMPRESSED_PUBKEY_LENGTH">ECDSA_R1_COMPRESSED_PUBKEY_LENGTH</a>: u64 = 33;
</code></pre>



<a name="0x3_ecdsa_r1_ECDSA_R1_RAW_SIGNATURE_LENGTH"></a>

Signature length (r, s)


<pre><code><b>const</b> <a href="ecdsa_r1.md#0x3_ecdsa_r1_ECDSA_R1_RAW_SIGNATURE_LENGTH">ECDSA_R1_RAW_SIGNATURE_LENGTH</a>: u64 = 64;
</code></pre>



<a name="0x3_ecdsa_r1_verify"></a>

## Function `verify`

Verifies an ECDSA signature over the secp256r1 (P-256) curve.
The message is hashed with SHA256 before verification.


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_r1.md#0x3_ecdsa_r1_verify">verify</a>(signature: &<a href="">vector</a>&lt;u8&gt;, public_key: &<a href="">vector</a>&lt;u8&gt;, msg: &<a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<a name="0x3_ecdsa_r1_public_key_length"></a>

## Function `public_key_length`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_r1.md#0x3_ecdsa_r1_public_key_length">public_key_length</a>(): u64
</code></pre>



<a name="0x3_ecdsa_r1_raw_signature_length"></a>

## Function `raw_signature_length`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_r1.md#0x3_ecdsa_r1_raw_signature_length">raw_signature_length</a>(): u64
</code></pre>
