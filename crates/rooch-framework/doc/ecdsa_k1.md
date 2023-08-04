
<a name="0x3_ecdsa_k1"></a>

# Module `0x3::ecdsa_k1`



-  [Constants](#@Constants_0)
-  [Function `verify`](#0x3_ecdsa_k1_verify)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ecdsa_k1_EInvalidPubKey"></a>

Error if the public key is invalid.


<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_EInvalidPubKey">EInvalidPubKey</a>: u64 = 1;
</code></pre>



<a name="0x3_ecdsa_k1_EInvalidSignature"></a>

Error if the signature is invalid.


<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_EInvalidSignature">EInvalidSignature</a>: u64 = 0;
</code></pre>



<a name="0x3_ecdsa_k1_KECCAK256"></a>

Hash function name that are valid for ecrecover and verify.


<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_KECCAK256">KECCAK256</a>: u8 = 0;
</code></pre>



<a name="0x3_ecdsa_k1_SHA256"></a>



<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_SHA256">SHA256</a>: u8 = 1;
</code></pre>



<a name="0x3_ecdsa_k1_verify"></a>

## Function `verify`

@param signature: A 64-bytes signature in form (r, s) that is signed using
Ecdsa. This is an non-recoverable signature without recovery id.
@param public_key: A 33-bytes public key that is used to sign messages.
@param msg: The message that the signature is signed against.
@param hash: The hash function used to hash the message when signing.

If the signature is valid to the pubkey and hashed message, return true. Else false.


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_verify">verify</a>(signature: &<a href="">vector</a>&lt;u8&gt;, public_key: &<a href="">vector</a>&lt;u8&gt;, msg: &<a href="">vector</a>&lt;u8&gt;, <a href="../doc/hash.md#0x1_hash">hash</a>: u8): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_verify">verify</a>(
   signature: &<a href="">vector</a>&lt;u8&gt;,
   public_key: &<a href="">vector</a>&lt;u8&gt;,
   msg: &<a href="">vector</a>&lt;u8&gt;,
   <a href="../doc/hash.md#0x1_hash">hash</a>: u8
): bool;
</code></pre>



</details>
