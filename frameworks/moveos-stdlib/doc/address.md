
<a name="0x2_address"></a>

# Module `0x2::address`



-  [Constants](#@Constants_0)
-  [Function `from_bytes`](#0x2_address_from_bytes)
-  [Function `to_bytes`](#0x2_address_to_bytes)
-  [Function `to_ascii_string`](#0x2_address_to_ascii_string)
-  [Function `from_ascii_string`](#0x2_address_from_ascii_string)
-  [Function `length`](#0x2_address_length)
-  [Function `max`](#0x2_address_max)
-  [Function `zero`](#0x2_address_zero)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="hex.md#0x2_hex">0x2::hex</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_address_ErrorAddressParseError"></a>

Error from <code>from_bytes</code> when it is supplied too many or too few bytes.


<pre><code><b>const</b> <a href="address.md#0x2_address_ErrorAddressParseError">ErrorAddressParseError</a>: u64 = 1;
</code></pre>



<a name="0x2_address_ErrorU256TooBigToConvertToAddress"></a>

Error from <code>from_u256</code> when


<pre><code><b>const</b> <a href="address.md#0x2_address_ErrorU256TooBigToConvertToAddress">ErrorU256TooBigToConvertToAddress</a>: u64 = 2;
</code></pre>



<a name="0x2_address_LENGTH"></a>

The length of an address, in bytes


<pre><code><b>const</b> <a href="address.md#0x2_address_LENGTH">LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x2_address_MAX"></a>



<pre><code><b>const</b> <a href="address.md#0x2_address_MAX">MAX</a>: u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935;
</code></pre>



<a name="0x2_address_ZERO"></a>



<pre><code><b>const</b> <a href="address.md#0x2_address_ZERO">ZERO</a>: <b>address</b> = 0x0;
</code></pre>



<a name="0x2_address_from_bytes"></a>

## Function `from_bytes`

Convert <code>a</code> into a u256 by interpreting <code>a</code> as the bytes of a big-endian integer
(e.g., <code>to_u256(0x1) == 1</code>)
Convert <code>n</code> into an address by encoding it as a big-endian integer (e.g., <code>from_u256(1) = @0x1</code>)
Aborts if <code>n</code> > <code>MAX_ADDRESS</code>
Convert <code>bytes</code> into an address.
Aborts with <code><a href="address.md#0x2_address_ErrorAddressParseError">ErrorAddressParseError</a></code> if the length of <code>bytes</code> is invalid length


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_from_bytes">from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<a name="0x2_address_to_bytes"></a>

## Function `to_bytes`

Convert <code>a</code> into BCS-encoded bytes.


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_to_bytes">to_bytes</a>(a: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_address_to_ascii_string"></a>

## Function `to_ascii_string`

Convert <code>a</code> to a hex-encoded ASCII string


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_to_ascii_string">to_ascii_string</a>(a: <b>address</b>): <a href="_String">ascii::String</a>
</code></pre>



<a name="0x2_address_from_ascii_string"></a>

## Function `from_ascii_string`

Convert <code>a</code> from a little endian encoding hex ASCII string


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_from_ascii_string">from_ascii_string</a>(a: <a href="_String">ascii::String</a>): <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;
</code></pre>



<a name="0x2_address_length"></a>

## Function `length`

Convert <code>a</code> to a hex-encoded ASCII string
Length of a Rooch address in bytes


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_length">length</a>(): u64
</code></pre>



<a name="0x2_address_max"></a>

## Function `max`

Largest possible address


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_max">max</a>(): u256
</code></pre>



<a name="0x2_address_zero"></a>

## Function `zero`

all zeros address


<pre><code><b>public</b> <b>fun</b> <a href="address.md#0x2_address_zero">zero</a>(): <b>address</b>
</code></pre>
