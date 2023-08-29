
<a name="0x3_ethereum_address"></a>

# Module `0x3::ethereum_address`



-  [Struct `ETHAddress`](#0x3_ethereum_address_ETHAddress)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x3_ethereum_address_new)
-  [Function `as_bytes`](#0x3_ethereum_address_as_bytes)
-  [Function `into_bytes`](#0x3_ethereum_address_into_bytes)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable">0x3::ecdsa_k1_recoverable</a>;
<b>use</b> <a href="hash.md#0x3_hash">0x3::hash</a>;
</code></pre>



<a name="0x3_ethereum_address_ETHAddress"></a>

## Struct `ETHAddress`



<pre><code><b>struct</b> <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ETHAddress</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>bytes: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_ethereum_address_ErrorDecompressPublicKey"></a>



<pre><code><b>const</b> <a href="ethereum_address.md#0x3_ethereum_address_ErrorDecompressPublicKey">ErrorDecompressPublicKey</a>: u64 = 1;
</code></pre>



<a name="0x3_ethereum_address_ErrorMalformedPublicKey"></a>

error code


<pre><code><b>const</b> <a href="ethereum_address.md#0x3_ethereum_address_ErrorMalformedPublicKey">ErrorMalformedPublicKey</a>: u64 = 0;
</code></pre>



<a name="0x3_ethereum_address_VALID_ETHEREUM_ADDR_LENGTH"></a>



<pre><code><b>const</b> <a href="ethereum_address.md#0x3_ethereum_address_VALID_ETHEREUM_ADDR_LENGTH">VALID_ETHEREUM_ADDR_LENGTH</a>: u64 = 20;
</code></pre>



<a name="0x3_ethereum_address_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="ethereum_address.md#0x3_ethereum_address_new">new</a>(pub_key: <a href="">vector</a>&lt;u8&gt;): <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ethereum_address::ETHAddress</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ethereum_address.md#0x3_ethereum_address_new">new</a>(pub_key: <a href="">vector</a>&lt;u8&gt;): <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ETHAddress</a> {
    // A pubkey is a 33-bytes compressed <b>public</b> key
    <b>assert</b>!(
        <a href="_length">vector::length</a>(&pub_key) == <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_public_key_length">ecdsa_k1_recoverable::public_key_length</a>(),
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="ethereum_address.md#0x3_ethereum_address_ErrorMalformedPublicKey">ErrorMalformedPublicKey</a>)
    );
    // Decompressing the pubkey <b>to</b> a 65-bytes <b>public</b> key.
    <b>let</b> uncompressed = <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_decompress_pubkey">ecdsa_k1_recoverable::decompress_pubkey</a>(&pub_key);
    <b>assert</b>!(
        <a href="_length">vector::length</a>(&uncompressed) == <a href="ecdsa_k1_recoverable.md#0x3_ecdsa_k1_recoverable_uncompressed_public_key_length">ecdsa_k1_recoverable::uncompressed_public_key_length</a>(),
        <a href="_internal">error::internal</a>(<a href="ethereum_address.md#0x3_ethereum_address_ErrorDecompressPublicKey">ErrorDecompressPublicKey</a>)
    );
    // Ignore the first byte and take the last 64-bytes of the uncompressed pubkey.
    <b>let</b> uncompressed_64 = <a href="_empty">vector::empty</a>&lt;u8&gt;();
    <b>let</b> i = 1;
    <b>while</b> (i &lt; 65) {
        <b>let</b> value = <a href="_borrow">vector::borrow</a>(&uncompressed, i);
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> uncompressed_64, *value);
        i = i + 1;
    };
    // Take the last 20 bytes of the <a href="">hash</a> of the 64-bytes uncompressed pubkey.
    <b>let</b> hashed = hash::keccak256(&uncompressed_64);
    <b>let</b> address_bytes = <a href="_empty">vector::empty</a>&lt;u8&gt;();
    <b>let</b> i = 12;
    <b>while</b> (i &lt; 32) {
        <b>let</b> value = <a href="_borrow">vector::borrow</a>(&hashed, i);
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> address_bytes, *value);
        i = i + 1;
    };
    // Return the 20 bytes <b>address</b> <b>as</b> the Ethereum <b>address</b>
    <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ETHAddress</a> {
        bytes: address_bytes,
    }
}
</code></pre>



</details>

<a name="0x3_ethereum_address_as_bytes"></a>

## Function `as_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ethereum_address.md#0x3_ethereum_address_as_bytes">as_bytes</a>(addr: &<a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ethereum_address::ETHAddress</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ethereum_address.md#0x3_ethereum_address_as_bytes">as_bytes</a>(addr: &<a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ETHAddress</a>): &<a href="">vector</a>&lt;u8&gt; {
    &addr.bytes
}
</code></pre>



</details>

<a name="0x3_ethereum_address_into_bytes"></a>

## Function `into_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ethereum_address.md#0x3_ethereum_address_into_bytes">into_bytes</a>(addr: <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ethereum_address::ETHAddress</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ethereum_address.md#0x3_ethereum_address_into_bytes">into_bytes</a>(addr: <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ETHAddress</a>): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ETHAddress</a> { bytes } = addr;
    bytes
}
</code></pre>



</details>
