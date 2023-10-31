
<a name="0x3_ecdsa_k1_recoverable"></a>

# Module `0x3::ecdsa_k1_recoverable`



-  [Constants](#@Constants_0)
-  [Function `public_key_length`](#0x3_ecdsa_k1_recoverable_public_key_length)
-  [Function `uncompressed_public_key_length`](#0x3_ecdsa_k1_recoverable_uncompressed_public_key_length)
-  [Function `signature_length`](#0x3_ecdsa_k1_recoverable_signature_length)
-  [Function `keccak256`](#0x3_ecdsa_k1_recoverable_keccak256)
-  [Function `ecrecover`](#0x3_ecdsa_k1_recoverable_ecrecover)
-  [Function `decompress_pubkey`](#0x3_ecdsa_k1_recoverable_decompress_pubkey)
-  [Function `verify`](#0x3_ecdsa_k1_recoverable_verify)


<pre><code><b>use</b> <a href="hash.md#0x3_hash">0x3::hash</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ecdsa_k1_recoverable_ErrorInvalidPubKey"></a>

Error if the public key is invalid.


<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_ErrorInvalidPubKey">ErrorInvalidPubKey</a>: u64 = 3;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_ErrorInvalidSignature"></a>

Error if the signature is invalid.


<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_ErrorInvalidSignature">ErrorInvalidSignature</a>: u64 = 2;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_ECDSA_K1_RECOVERABLE_COMPRESSED_PUBKEY_LENGTH"></a>

constant codes


<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_ECDSA_K1_RECOVERABLE_COMPRESSED_PUBKEY_LENGTH">ECDSA_K1_RECOVERABLE_COMPRESSED_PUBKEY_LENGTH</a>: u64 = 33;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_ECDSA_K1_RECOVERABLE_SIG_LENGTH"></a>



<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_ECDSA_K1_RECOVERABLE_SIG_LENGTH">ECDSA_K1_RECOVERABLE_SIG_LENGTH</a>: u64 = 65;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_ECDSA_K1_RECOVERABLE_UNCOMPRESSED_PUBKEY_LENGTH"></a>



<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_ECDSA_K1_RECOVERABLE_UNCOMPRESSED_PUBKEY_LENGTH">ECDSA_K1_RECOVERABLE_UNCOMPRESSED_PUBKEY_LENGTH</a>: u64 = 65;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_ErrorFailToRecoverPubKey"></a>

Error if the public key cannot be recovered from the signature.


<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_ErrorFailToRecoverPubKey">ErrorFailToRecoverPubKey</a>: u64 = 1;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_ErrorInvalidHashType"></a>

Invalid hash function


<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_ErrorInvalidHashType">ErrorInvalidHashType</a>: u64 = 4;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_KECCAK256"></a>

Hash function name that are valid for ecrecover and verify.


<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_KECCAK256">KECCAK256</a>: u8 = 0;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_public_key_length"></a>

## Function `public_key_length`

built-in functions


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_public_key_length">public_key_length</a>(): u64
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_uncompressed_public_key_length"></a>

## Function `uncompressed_public_key_length`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_uncompressed_public_key_length">uncompressed_public_key_length</a>(): u64
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_signature_length"></a>

## Function `signature_length`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_signature_length">signature_length</a>(): u64
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_keccak256"></a>

## Function `keccak256`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_keccak256">keccak256</a>(): u8
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_ecrecover"></a>

## Function `ecrecover`

@param signature: A 65-bytes signature in form (r, s, v) that is signed using
The accepted v values are {0, 1, 2, 3}.
@param msg: The message that the signature is signed against, this is raw message without hashing.
@param hash: The hash function used to hash the message when signing.

If the signature is valid, return the corresponding recovered Secpk256k1 public
key, otherwise throw error. This is similar to ecrecover in Ethereum, can only be
applied to Ecdsa signatures.


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_ecrecover">ecrecover</a>(signature: &<a href="">vector</a>&lt;u8&gt;, msg: &<a href="">vector</a>&lt;u8&gt;, <a href="">hash</a>: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_decompress_pubkey"></a>

## Function `decompress_pubkey`

@param pubkey: A 33-bytes compressed public key, a prefix either 0x02 or 0x03 and a 256-bit integer.

If the compressed public key is valid, return the 65-bytes uncompressed public key,
otherwise throw error.


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_decompress_pubkey">decompress_pubkey</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_verify"></a>

## Function `verify`

@param signature: A 65-bytes signature in form (r, s, v) that is signed using
Ecdsa. This is a recoverable signature with a recovery id.
@param msg: The message that the signature is signed against.
@param hash: The hash function used to hash the message when signing.

If the signature is valid to the pubkey and hashed message, return true. Else false.


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_verify">verify</a>(signature: &<a href="">vector</a>&lt;u8&gt;, msg: &<a href="">vector</a>&lt;u8&gt;, <a href="">hash</a>: u8): bool
</code></pre>
