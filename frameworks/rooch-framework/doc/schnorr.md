
<a name="0x3_schnorr"></a>

# Module `0x3::schnorr`



-  [Constants](#@Constants_0)
-  [Function `verifying_key_length`](#0x3_schnorr_verifying_key_length)
-  [Function `signature_length`](#0x3_schnorr_signature_length)
-  [Function `verify`](#0x3_schnorr_verify)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_schnorr_ErrorInvalidSignature"></a>

Error if the signature is invalid.


<pre><code><b>const</b> <a href="schnorr.md#0x3_schnorr_ErrorInvalidSignature">ErrorInvalidSignature</a>: u64 = 1;
</code></pre>



<a name="0x3_schnorr_ErrorInvalidVerifyingKey"></a>

Error if the verifying key is invalid.


<pre><code><b>const</b> <a href="schnorr.md#0x3_schnorr_ErrorInvalidVerifyingKey">ErrorInvalidVerifyingKey</a>: u64 = 2;
</code></pre>



<a name="0x3_schnorr_SCHNORR_SIGNATURE_LENGTH"></a>



<pre><code><b>const</b> <a href="schnorr.md#0x3_schnorr_SCHNORR_SIGNATURE_LENGTH">SCHNORR_SIGNATURE_LENGTH</a>: u64 = 64;
</code></pre>



<a name="0x3_schnorr_SCHNORR_VERIFYING_KEY_LENGTH"></a>

constant codes


<pre><code><b>const</b> <a href="schnorr.md#0x3_schnorr_SCHNORR_VERIFYING_KEY_LENGTH">SCHNORR_VERIFYING_KEY_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x3_schnorr_verifying_key_length"></a>

## Function `verifying_key_length`

built-in functions


<pre><code><b>public</b> <b>fun</b> <a href="schnorr.md#0x3_schnorr_verifying_key_length">verifying_key_length</a>(): u64
</code></pre>



<a name="0x3_schnorr_signature_length"></a>

## Function `signature_length`



<pre><code><b>public</b> <b>fun</b> <a href="schnorr.md#0x3_schnorr_signature_length">signature_length</a>(): u64
</code></pre>



<a name="0x3_schnorr_verify"></a>

## Function `verify`

@param signature: A 64-bytes signature that is signed using schnorr over secpk256k1 key pairs.
@param verifying_key: A 32-bytes verifying key that is used to verify messages.
@param msg: The message that the signature is signed against.

If the signature and message are valid to the verifying key, return true. Else false.


<pre><code><b>public</b> <b>fun</b> <a href="schnorr.md#0x3_schnorr_verify">verify</a>(signature: &<a href="">vector</a>&lt;u8&gt;, verifying_key: &<a href="">vector</a>&lt;u8&gt;, msg: &<a href="">vector</a>&lt;u8&gt;): bool
</code></pre>
