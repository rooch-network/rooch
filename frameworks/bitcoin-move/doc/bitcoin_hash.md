
<a name="0x4_bitcoin_hash"></a>

# Module `0x4::bitcoin_hash`



-  [Constants](#@Constants_0)
-  [Function `from_ascii_bytes`](#0x4_bitcoin_hash_from_ascii_bytes)
-  [Function `from_ascii_bytes_option`](#0x4_bitcoin_hash_from_ascii_bytes_option)
-  [Function `to_string`](#0x4_bitcoin_hash_to_string)
-  [Function `sha256d`](#0x4_bitcoin_hash_sha256d)


<pre><code><b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::address</a>;
<b>use</b> <a href="">0x2::hex</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_bitcoin_hash_ErrorInvalidHex"></a>



<pre><code><b>const</b> <a href="bitcoin_hash.md#0x4_bitcoin_hash_ErrorInvalidHex">ErrorInvalidHex</a>: u64 = 1;
</code></pre>



<a name="0x4_bitcoin_hash_from_ascii_bytes"></a>

## Function `from_ascii_bytes`

Convert an ascii hex string bytes to Bitcoin Hash
Because Bitcoin Hash hex is reversed, we need to reverse the bytes
Abort if the input is not a valid hex


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_hash.md#0x4_bitcoin_hash_from_ascii_bytes">from_ascii_bytes</a>(bytes: &<a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<a name="0x4_bitcoin_hash_from_ascii_bytes_option"></a>

## Function `from_ascii_bytes_option`

Convert an ascii hex string bytes to Bitcoin Hash
Because Bitcoin Hash hex is reversed, we need to reverse the bytes
Return None if the input is not a valid hex


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_hash.md#0x4_bitcoin_hash_from_ascii_bytes_option">from_ascii_bytes_option</a>(bytes: &<a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;
</code></pre>



<a name="0x4_bitcoin_hash_to_string"></a>

## Function `to_string`

Convert Bitcoin Hash to hex string
Because Bitcoin Hash hex is reversed, we need to reverse the bytes


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_hash.md#0x4_bitcoin_hash_to_string">to_string</a>(<a href="">hash</a>: <b>address</b>): <a href="_String">string::String</a>
</code></pre>



<a name="0x4_bitcoin_hash_sha256d"></a>

## Function `sha256d`

Bitcoin hash is double sha256 of the input


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_hash.md#0x4_bitcoin_hash_sha256d">sha256d</a>(input: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>
