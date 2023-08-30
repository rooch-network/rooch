
<a name="0x3_ecdsa_k1"></a>

# Module `0x3::ecdsa_k1`



-  [Constants](#@Constants_0)
-  [Function `scheme_length`](#0x3_ecdsa_k1_scheme_length)
-  [Function `public_key_length`](#0x3_ecdsa_k1_public_key_length)
-  [Function `signature_length`](#0x3_ecdsa_k1_signature_length)
-  [Function `keccak256`](#0x3_ecdsa_k1_keccak256)
-  [Function `sha256`](#0x3_ecdsa_k1_sha256)
-  [Function `ripemd160`](#0x3_ecdsa_k1_ripemd160)
-  [Function `get_public_key_from_authenticator_payload`](#0x3_ecdsa_k1_get_public_key_from_authenticator_payload)
-  [Function `get_signature_from_authenticator_payload`](#0x3_ecdsa_k1_get_signature_from_authenticator_payload)
-  [Function `verify`](#0x3_ecdsa_k1_verify)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ecdsa_k1_ErrorInvalidPubKey"></a>

Error if the public key is invalid.


<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_ErrorInvalidPubKey">ErrorInvalidPubKey</a>: u64 = 1;
</code></pre>



<a name="0x3_ecdsa_k1_ErrorInvalidSignature"></a>

Error if the signature is invalid.


<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_ErrorInvalidSignature">ErrorInvalidSignature</a>: u64 = 0;
</code></pre>



<a name="0x3_ecdsa_k1_KECCAK256"></a>

Hash function name that are valid for ecrecover and verify.


<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_KECCAK256">KECCAK256</a>: u8 = 0;
</code></pre>



<a name="0x3_ecdsa_k1_RIPEMD160"></a>



<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_RIPEMD160">RIPEMD160</a>: u8 = 2;
</code></pre>



<a name="0x3_ecdsa_k1_SHA256"></a>



<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_SHA256">SHA256</a>: u8 = 1;
</code></pre>



<a name="0x3_ecdsa_k1_VALID_ECDSA_K1_COMPRESSED_PUBKEY_LENGTH"></a>



<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_VALID_ECDSA_K1_COMPRESSED_PUBKEY_LENGTH">VALID_ECDSA_K1_COMPRESSED_PUBKEY_LENGTH</a>: u64 = 33;
</code></pre>



<a name="0x3_ecdsa_k1_VALID_ECDSA_K1_SIG_LENGTH"></a>



<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_VALID_ECDSA_K1_SIG_LENGTH">VALID_ECDSA_K1_SIG_LENGTH</a>: u64 = 64;
</code></pre>



<a name="0x3_ecdsa_k1_VALID_ECDSA_K1_TO_SCHEME_BITCOIN_LENGTH"></a>

constant codes


<pre><code><b>const</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_VALID_ECDSA_K1_TO_SCHEME_BITCOIN_LENGTH">VALID_ECDSA_K1_TO_SCHEME_BITCOIN_LENGTH</a>: u64 = 1;
</code></pre>



<a name="0x3_ecdsa_k1_scheme_length"></a>

## Function `scheme_length`

built-in functions


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_scheme_length">scheme_length</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_scheme_length">scheme_length</a>(): u64 {
    <a href="ecdsa_k1.md#0x3_ecdsa_k1_VALID_ECDSA_K1_TO_SCHEME_BITCOIN_LENGTH">VALID_ECDSA_K1_TO_SCHEME_BITCOIN_LENGTH</a>
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_public_key_length"></a>

## Function `public_key_length`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_public_key_length">public_key_length</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_public_key_length">public_key_length</a>(): u64 {
    <a href="ecdsa_k1.md#0x3_ecdsa_k1_VALID_ECDSA_K1_COMPRESSED_PUBKEY_LENGTH">VALID_ECDSA_K1_COMPRESSED_PUBKEY_LENGTH</a>
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_signature_length"></a>

## Function `signature_length`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_signature_length">signature_length</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_signature_length">signature_length</a>(): u64 {
    <a href="ecdsa_k1.md#0x3_ecdsa_k1_VALID_ECDSA_K1_SIG_LENGTH">VALID_ECDSA_K1_SIG_LENGTH</a>
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_keccak256"></a>

## Function `keccak256`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_keccak256">keccak256</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_keccak256">keccak256</a>(): u8 {
    <a href="ecdsa_k1.md#0x3_ecdsa_k1_KECCAK256">KECCAK256</a>
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_sha256"></a>

## Function `sha256`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_sha256">sha256</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_sha256">sha256</a>(): u8 {
    <a href="ecdsa_k1.md#0x3_ecdsa_k1_SHA256">SHA256</a>
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_ripemd160"></a>

## Function `ripemd160`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_ripemd160">ripemd160</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_ripemd160">ripemd160</a>(): u8 {
    <a href="ecdsa_k1.md#0x3_ecdsa_k1_RIPEMD160">RIPEMD160</a>
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_get_public_key_from_authenticator_payload"></a>

## Function `get_public_key_from_authenticator_payload`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> public_key = <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;u8&gt;();
    <b>let</b> i = <a href="ecdsa_k1.md#0x3_ecdsa_k1_scheme_length">scheme_length</a>() + <a href="ecdsa_k1.md#0x3_ecdsa_k1_signature_length">signature_length</a>();
    <b>let</b> public_key_position = <a href="ecdsa_k1.md#0x3_ecdsa_k1_scheme_length">scheme_length</a>() + <a href="ecdsa_k1.md#0x3_ecdsa_k1_signature_length">signature_length</a>() + <a href="ecdsa_k1.md#0x3_ecdsa_k1_public_key_length">public_key_length</a>();
    <b>while</b> (i &lt; public_key_position) {
        <b>let</b> value = <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(authenticator_payload, i);
        <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> public_key, *value);
        i = i + 1;
    };
    public_key
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_get_signature_from_authenticator_payload"></a>

## Function `get_signature_from_authenticator_payload`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_get_signature_from_authenticator_payload">get_signature_from_authenticator_payload</a>(authenticator_payload: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_get_signature_from_authenticator_payload">get_signature_from_authenticator_payload</a>(authenticator_payload: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> sign = <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;u8&gt;();
    <b>let</b> i = <a href="ecdsa_k1.md#0x3_ecdsa_k1_scheme_length">scheme_length</a>();
    <b>let</b> signature_position = <a href="ecdsa_k1.md#0x3_ecdsa_k1_signature_length">signature_length</a>() + 1;
    <b>while</b> (i &lt; signature_position) {
        <b>let</b> value = <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(authenticator_payload, i);
        <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> sign, *value);
        i = i + 1;
    };
    sign
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_verify"></a>

## Function `verify`

@param signature: A 64-bytes signature in form (r, s) that is signed using
Ecdsa. This is an non-recoverable signature without recovery id.
@param public_key: A 33-bytes public key that is used to sign messages.
@param msg: The message that the signature is signed against.
@param hash: The hash function used to hash the message when signing.

If the signature is valid to the pubkey and hashed message, return true. Else false.


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_verify">verify</a>(signature: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, public_key: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, msg: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="../../moveos/moveos-stdlib/move-stdlib/doc/hash.md#0x1_hash">hash</a>: u8): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1_verify">verify</a>(
    signature: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    public_key: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    msg: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    <a href="../../moveos/moveos-stdlib/move-stdlib/doc/hash.md#0x1_hash">hash</a>: u8
): bool;
</code></pre>



</details>
