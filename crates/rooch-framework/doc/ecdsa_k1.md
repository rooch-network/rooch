
<a name="0x3_ecdsa_k1"></a>

# Module `0x3::ecdsa_k1`



-  [Constants](#@Constants_0)
-  [Function `ecrecover`](#0x3_ecdsa_k1_ecrecover)
-  [Function `decompress_pubkey`](#0x3_ecdsa_k1_decompress_pubkey)
-  [Function `verify`](#0x3_ecdsa_k1_verify)


<pre><code><b>use</b> <a href="hash.md#0x3_hash">0x3::hash</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ecdsa_k1_EFailToRecoverPubKey"></a>

Error if the public key cannot be recovered from the signature.


<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_EFailToRecoverPubKey">EFailToRecoverPubKey</a>: u64 = 0;
</code></pre>



<a name="0x3_ecdsa_k1_EInvalidSignature"></a>

Error if the signature is invalid.


<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_EInvalidSignature">EInvalidSignature</a>: u64 = 1;
</code></pre>



<a name="0x3_ecdsa_k1_KECCAK256"></a>

Hash function name that are valid for ecrecover and verify.


<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_KECCAK256">KECCAK256</a>: u8 = 0;
</code></pre>



<a name="0x3_ecdsa_k1_SHA256"></a>



<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_SHA256">SHA256</a>: u8 = 1;
</code></pre>



<a name="0x3_ecdsa_k1_ecrecover"></a>

## Function `ecrecover`

@param signature: A 65-bytes signature in form (r, s, v) that is signed using
The accepted v values are {0, 1, 2, 3}.
@param msg: The message that the signature is signed against, this is raw message without hashing.
@param hash: The hash function used to hash the message when signing.

If the signature is valid, return the corresponding recovered Secpk256k1 public
key, otherwise throw error. This is similar to ecrecover in Ethereum, can only be
applied to Ecdsa signatures.


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_ecrecover">ecrecover</a>(signature: &<a href="">vector</a>&lt;u8&gt;, msg: &<a href="">vector</a>&lt;u8&gt;, <a href="../doc/hash.md#0x1_hash">hash</a>: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_ecrecover">ecrecover</a>(signature: &<a href="">vector</a>&lt;u8&gt;, msg: &<a href="">vector</a>&lt;u8&gt;, <a href="../doc/hash.md#0x1_hash">hash</a>: u8): <a href="">vector</a>&lt;u8&gt;;
</code></pre>



</details>

<a name="0x3_ecdsa_k1_decompress_pubkey"></a>

## Function `decompress_pubkey`

@param pubkey: A 33-bytes compressed public key, a prefix either 0x02 or 0x03 and a 256-bit integer.

If the compressed public key is valid, return the 65-bytes uncompressed public key,
otherwise throw error.


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_decompress_pubkey">decompress_pubkey</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_decompress_pubkey">decompress_pubkey</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;;
</code></pre>



</details>

<a name="0x3_ecdsa_k1_verify"></a>

## Function `verify`

@param signature: A 65-bytes signature in form (r, s, v) that is signed using
Ecdsa. This is an non-recoverable signature without recovery id.
@param msg: The message that the signature is signed against.
@param hash: The hash function used to hash the message when signing.

If the signature is valid to the pubkey and hashed message, return true. Else false.


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_verify">verify</a>(signature: &<a href="">vector</a>&lt;u8&gt;, msg: &<a href="">vector</a>&lt;u8&gt;, <a href="../doc/hash.md#0x1_hash">hash</a>: u8): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_verify">verify</a>(
   signature: &<a href="">vector</a>&lt;u8&gt;,
   msg: &<a href="">vector</a>&lt;u8&gt;,
   <a href="../doc/hash.md#0x1_hash">hash</a>: u8
): bool;
</code></pre>



</details>
