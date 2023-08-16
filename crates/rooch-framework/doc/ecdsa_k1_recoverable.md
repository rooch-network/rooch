
<a name="0x3_ecdsa_k1_recoverable"></a>

# Module `0x3::ecdsa_k1_recoverable`



-  [Constants](#@Constants_0)
-  [Function `scheme_length`](#0x3_ecdsa_k1_recoverable_scheme_length)
-  [Function `public_key_length`](#0x3_ecdsa_k1_recoverable_public_key_length)
-  [Function `signature_length`](#0x3_ecdsa_k1_recoverable_signature_length)
-  [Function `keccak256`](#0x3_ecdsa_k1_recoverable_keccak256)
-  [Function `sha256`](#0x3_ecdsa_k1_recoverable_sha256)
-  [Function `get_public_key_from_authenticator_payload`](#0x3_ecdsa_k1_recoverable_get_public_key_from_authenticator_payload)
-  [Function `get_signature_from_authenticator_payload`](#0x3_ecdsa_k1_recoverable_get_signature_from_authenticator_payload)
-  [Function `ecrecover`](#0x3_ecdsa_k1_recoverable_ecrecover)
-  [Function `decompress_pubkey`](#0x3_ecdsa_k1_recoverable_decompress_pubkey)
-  [Function `verify`](#0x3_ecdsa_k1_recoverable_verify)


<pre><code><b>use</b> <a href="hash.md#0x3_hash">0x3::hash</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ecdsa_k1_recoverable_EInvalidPubKey"></a>

Error if the public key is invalid.


<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_EInvalidPubKey">EInvalidPubKey</a>: u64 = 2;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_EInvalidSignature"></a>

Error if the signature is invalid.


<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_EInvalidSignature">EInvalidSignature</a>: u64 = 1;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_KECCAK256"></a>

Hash function name that are valid for ecrecover and verify.


<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_KECCAK256">KECCAK256</a>: u8 = 0;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_SHA256"></a>



<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_SHA256">SHA256</a>: u8 = 1;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_EFailToRecoverPubKey"></a>

Error if the public key cannot be recovered from the signature.


<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_EFailToRecoverPubKey">EFailToRecoverPubKey</a>: u64 = 0;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_V_ECDSA_K1_RECOVERABLE_PUBKEY_LENGTH"></a>



<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_V_ECDSA_K1_RECOVERABLE_PUBKEY_LENGTH">V_ECDSA_K1_RECOVERABLE_PUBKEY_LENGTH</a>: u64 = 33;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_V_ECDSA_K1_RECOVERABLE_SIG_LENGTH"></a>



<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_V_ECDSA_K1_RECOVERABLE_SIG_LENGTH">V_ECDSA_K1_RECOVERABLE_SIG_LENGTH</a>: u64 = 65;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_V_ECDSA_K1_RECOVERABLE_TO_ETHEREUM_SCHEME_LENGTH"></a>

constant codes


<pre><code><b>const</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_V_ECDSA_K1_RECOVERABLE_TO_ETHEREUM_SCHEME_LENGTH">V_ECDSA_K1_RECOVERABLE_TO_ETHEREUM_SCHEME_LENGTH</a>: u64 = 1;
</code></pre>



<a name="0x3_ecdsa_k1_recoverable_scheme_length"></a>

## Function `scheme_length`

built-in functions


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_scheme_length">scheme_length</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_scheme_length">scheme_length</a>(): u64 {
    <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_V_ECDSA_K1_RECOVERABLE_TO_ETHEREUM_SCHEME_LENGTH">V_ECDSA_K1_RECOVERABLE_TO_ETHEREUM_SCHEME_LENGTH</a>
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_recoverable_public_key_length"></a>

## Function `public_key_length`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_public_key_length">public_key_length</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_public_key_length">public_key_length</a>(): u64 {
    <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_V_ECDSA_K1_RECOVERABLE_PUBKEY_LENGTH">V_ECDSA_K1_RECOVERABLE_PUBKEY_LENGTH</a>
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_recoverable_signature_length"></a>

## Function `signature_length`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_signature_length">signature_length</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_signature_length">signature_length</a>(): u64 {
    <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_V_ECDSA_K1_RECOVERABLE_SIG_LENGTH">V_ECDSA_K1_RECOVERABLE_SIG_LENGTH</a>
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_recoverable_keccak256"></a>

## Function `keccak256`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_keccak256">keccak256</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_keccak256">keccak256</a>(): u8 {
    <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_KECCAK256">KECCAK256</a>
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_recoverable_sha256"></a>

## Function `sha256`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_sha256">sha256</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_sha256">sha256</a>(): u8 {
    <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_SHA256">SHA256</a>
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_recoverable_get_public_key_from_authenticator_payload"></a>

## Function `get_public_key_from_authenticator_payload`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> public_key = <a href="_empty">vector::empty</a>&lt;u8&gt;();
    <b>let</b> i = <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_scheme_length">scheme_length</a>() + <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_signature_length">signature_length</a>();
    <b>let</b> public_key_position = <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_scheme_length">scheme_length</a>() + <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_signature_length">signature_length</a>() + <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_public_key_length">public_key_length</a>();
    <b>while</b> (i &lt; public_key_position) {
            <b>let</b> value = <a href="_borrow">vector::borrow</a>(authenticator_payload, i);
            <a href="_push_back">vector::push_back</a>(&<b>mut</b> public_key, *value);
            i = i + 1;
    };
    public_key
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_recoverable_get_signature_from_authenticator_payload"></a>

## Function `get_signature_from_authenticator_payload`



<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_get_signature_from_authenticator_payload">get_signature_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_get_signature_from_authenticator_payload">get_signature_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> sign = <a href="_empty">vector::empty</a>&lt;u8&gt;();
    <b>let</b> i = <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_scheme_length">scheme_length</a>();
    <b>let</b> signature_position = <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_signature_length">signature_length</a>() + 1;
    <b>while</b> (i &lt; signature_position) {
            <b>let</b> value = <a href="_borrow">vector::borrow</a>(authenticator_payload, i);
            <a href="_push_back">vector::push_back</a>(&<b>mut</b> sign, *value);
            i = i + 1;
    };
    sign
}
</code></pre>



</details>

<a name="0x3_ecdsa_k1_recoverable_ecrecover"></a>

## Function `ecrecover`

@param signature: A 65-bytes signature in form (r, s, v) that is signed using
The accepted v values are {0, 1, 2, 3}.
@param msg: The message that the signature is signed against, this is raw message without hashing.
@param hash: The hash function used to hash the message when signing.

If the signature is valid, return the corresponding recovered Secpk256k1 public
key, otherwise throw error. This is similar to ecrecover in Ethereum, can only be
applied to Ecdsa signatures.


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_ecrecover">ecrecover</a>(signature: &<a href="">vector</a>&lt;u8&gt;, msg: &<a href="">vector</a>&lt;u8&gt;, <a href="../doc/hash.md#0x1_hash">hash</a>: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_ecrecover">ecrecover</a>(signature: &<a href="">vector</a>&lt;u8&gt;, msg: &<a href="">vector</a>&lt;u8&gt;, <a href="../doc/hash.md#0x1_hash">hash</a>: u8): <a href="">vector</a>&lt;u8&gt;;
</code></pre>



</details>

<a name="0x3_ecdsa_k1_recoverable_decompress_pubkey"></a>

## Function `decompress_pubkey`

@param pubkey: A 33-bytes compressed public key, a prefix either 0x02 or 0x03 and a 256-bit integer.

If the compressed public key is valid, return the 65-bytes uncompressed public key,
otherwise throw error.


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_decompress_pubkey">decompress_pubkey</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_decompress_pubkey">decompress_pubkey</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;;
</code></pre>



</details>

<a name="0x3_ecdsa_k1_recoverable_verify"></a>

## Function `verify`

@param signature: A 65-bytes signature in form (r, s, v) that is signed using
Ecdsa. This is a recoverable signature with a recovery id.
@param msg: The message that the signature is signed against.
@param hash: The hash function used to hash the message when signing.

If the signature is valid to the pubkey and hashed message, return true. Else false.


<pre><code><b>public</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_verify">verify</a>(signature: &<a href="">vector</a>&lt;u8&gt;, msg: &<a href="">vector</a>&lt;u8&gt;, <a href="../doc/hash.md#0x1_hash">hash</a>: u8): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_verify">verify</a>(
    signature: &<a href="">vector</a>&lt;u8&gt;,
    msg: &<a href="">vector</a>&lt;u8&gt;,
    <a href="../doc/hash.md#0x1_hash">hash</a>: u8
): bool;
</code></pre>



</details>
