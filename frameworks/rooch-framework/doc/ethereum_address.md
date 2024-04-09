
<a name="0x3_ethereum_address"></a>

# Module `0x3::ethereum_address`



-  [Struct `ETHAddress`](#0x3_ethereum_address_ETHAddress)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x3_ethereum_address_new)
-  [Function `from_bytes`](#0x3_ethereum_address_from_bytes)
-  [Function `as_bytes`](#0x3_ethereum_address_as_bytes)
-  [Function `into_bytes`](#0x3_ethereum_address_into_bytes)


<pre><code><b>use</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1">0x3::ecdsa_k1</a>;
<b>use</b> <a href="hash.md#0x3_hash">0x3::hash</a>;
</code></pre>



<a name="0x3_ethereum_address_ETHAddress"></a>

## Struct `ETHAddress`



<pre><code>#[data_struct]
<b>struct</b> <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ETHAddress</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ethereum_address_ETHEREUM_ADDR_LENGTH"></a>

Ethereum addresses are always 20 bytes


<pre><code><b>const</b> <a href="ethereum_address.md#0x3_ethereum_address_ETHEREUM_ADDR_LENGTH">ETHEREUM_ADDR_LENGTH</a>: u64 = 20;
</code></pre>



<a name="0x3_ethereum_address_ErrorDecompressPublicKey"></a>



<pre><code><b>const</b> <a href="ethereum_address.md#0x3_ethereum_address_ErrorDecompressPublicKey">ErrorDecompressPublicKey</a>: u64 = 2;
</code></pre>



<a name="0x3_ethereum_address_ErrorInvaidAddressBytes"></a>



<pre><code><b>const</b> <a href="ethereum_address.md#0x3_ethereum_address_ErrorInvaidAddressBytes">ErrorInvaidAddressBytes</a>: u64 = 3;
</code></pre>



<a name="0x3_ethereum_address_ErrorMalformedPublicKey"></a>



<pre><code><b>const</b> <a href="ethereum_address.md#0x3_ethereum_address_ErrorMalformedPublicKey">ErrorMalformedPublicKey</a>: u64 = 1;
</code></pre>



<a name="0x3_ethereum_address_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="ethereum_address.md#0x3_ethereum_address_new">new</a>(pub_key: <a href="">vector</a>&lt;u8&gt;): <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ethereum_address::ETHAddress</a>
</code></pre>



<a name="0x3_ethereum_address_from_bytes"></a>

## Function `from_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ethereum_address.md#0x3_ethereum_address_from_bytes">from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ethereum_address::ETHAddress</a>
</code></pre>



<a name="0x3_ethereum_address_as_bytes"></a>

## Function `as_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ethereum_address.md#0x3_ethereum_address_as_bytes">as_bytes</a>(addr: &<a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ethereum_address::ETHAddress</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_ethereum_address_into_bytes"></a>

## Function `into_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ethereum_address.md#0x3_ethereum_address_into_bytes">into_bytes</a>(addr: <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ethereum_address::ETHAddress</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>
