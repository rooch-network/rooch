
<a name="0x2_address"></a>

# Module `0x2::address`



-  [Constants](#@Constants_0)
-  [Function `from_bytes`](#0x2_address_from_bytes)
-  [Function `to_bytes`](#0x2_address_to_bytes)
-  [Function `to_ascii_string`](#0x2_address_to_ascii_string)
-  [Function `length`](#0x2_address_length)
-  [Function `max`](#0x2_address_max)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x1::bcs</a>;
<b>use</b> <a href="bcd.md#0x2_bcd">0x2::bcd</a>;
<b>use</b> <a href="hex.md#0x2_hex">0x2::hex</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_address_EAddressParseError"></a>

Error from <code>from_bytes</code> when it is supplied too many or too few bytes.


<pre><code><b>const</b> <a href="address.md#0x2_address_EAddressParseError">EAddressParseError</a>: u64 = 0;
</code></pre>



<a name="0x2_address_EU256TooBigToConvertToAddress"></a>

Error from <code>from_u256</code> when


<pre><code><b>const</b> <a href="address.md#0x2_address_EU256TooBigToConvertToAddress">EU256TooBigToConvertToAddress</a>: u64 = 1;
</code></pre>



<a name="0x2_address_LENGTH"></a>

The length of an address, in bytes


<pre><code><b>const</b> <a href="address.md#0x2_address_LENGTH">LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x2_address_MAX"></a>



<pre><code><b>const</b> <a href="address.md#0x2_address_MAX">MAX</a>: u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935;
</code></pre>



<a name="0x2_address_from_bytes"></a>

## Function `from_bytes`

Convert <code>a</code> into a u256 by interpreting <code>a</code> as the bytes of a big-endian integer
(e.g., <code>to_u256(0x1) == 1</code>)
Convert <code>n</code> into an address by encoding it as a big-endian integer (e.g., <code>from_u256(1) = @0x1</code>)
Aborts if <code>n</code> > <code>MAX_ADDRESS</code>
Convert <code>bytes</code> into an address.
Aborts with <code><a href="address.md#0x2_address_EAddressParseError">EAddressParseError</a></code> if the length of <code>bytes</code> is invalid length


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_from_bytes">from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_from_bytes">from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <b>address</b>{
    <a href="bcd.md#0x2_bcd_to_address">bcd::to_address</a>(bytes)
}
</code></pre>



</details>

<a name="0x2_address_to_bytes"></a>

## Function `to_bytes`

Convert <code>a</code> into BCS-encoded bytes.


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_to_bytes">to_bytes</a>(a: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_to_bytes">to_bytes</a>(a: <b>address</b>): <a href="">vector</a>&lt;u8&gt; {
    <a href="_to_bytes">bcs::to_bytes</a>(&a)
}
</code></pre>



</details>

<a name="0x2_address_to_ascii_string"></a>

## Function `to_ascii_string`

Convert <code>a</code> to a hex-encoded ASCII string


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_to_ascii_string">to_ascii_string</a>(a: <b>address</b>): <a href="_String">ascii::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_to_ascii_string">to_ascii_string</a>(a: <b>address</b>): <a href="_String">ascii::String</a> {
    <a href="_string">ascii::string</a>(<a href="hex.md#0x2_hex_encode">hex::encode</a>(<a href="address.md#0x2_address_to_bytes">to_bytes</a>(a)))
}
</code></pre>



</details>

<a name="0x2_address_length"></a>

## Function `length`

Convert <code>a</code> to a hex-encoded ASCII string
Length of a Sui address in bytes


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_length">length</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_length">length</a>(): u64 {
    <a href="address.md#0x2_address_LENGTH">LENGTH</a>
}
</code></pre>



</details>

<a name="0x2_address_max"></a>

## Function `max`

Largest possible address


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_max">max</a>(): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_max">max</a>(): u256 {
    <a href="address.md#0x2_address_MAX">MAX</a>
}
</code></pre>



</details>
