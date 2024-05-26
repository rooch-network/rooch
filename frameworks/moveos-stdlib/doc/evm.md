
<a name="0x2_evm"></a>

# Module `0x2::evm`



-  [Constants](#@Constants_0)
-  [Function `sha2_256`](#0x2_evm_sha2_256)
-  [Function `ec_add`](#0x2_evm_ec_add)
-  [Function `ec_pairing`](#0x2_evm_ec_pairing)


<pre><code><b>use</b> <a href="">0x1::hash</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_evm_E_EC_ADD_FAILED"></a>



<pre><code><b>const</b> <a href="evm.md#0x2_evm_E_EC_ADD_FAILED">E_EC_ADD_FAILED</a>: u64 = 6;
</code></pre>



<a name="0x2_evm_E_EC_PAIRING_FAILED"></a>



<pre><code><b>const</b> <a href="evm.md#0x2_evm_E_EC_PAIRING_FAILED">E_EC_PAIRING_FAILED</a>: u64 = 8;
</code></pre>



<a name="0x2_evm_E_EXPECT_32_BYTES"></a>



<pre><code><b>const</b> <a href="evm.md#0x2_evm_E_EXPECT_32_BYTES">E_EXPECT_32_BYTES</a>: u64 = 11;
</code></pre>



<a name="0x2_evm_sha2_256"></a>

## Function `sha2_256`

@param data: Arbitrary binary data to hash

Hash function.


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_sha2_256">sha2_256</a>(data: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_evm_ec_add"></a>

## Function `ec_add`

@param x1: X coordinate of the first point on the elliptic curve 'alt_bn128'.
@param y1: Y coordinate of the first point on the elliptic curve 'alt_bn128'.
@param x2: X coordinate of the second point on the elliptic curve 'alt_bn128'.
@param y2: Y coordinate of the second point on the elliptic curve 'alt_bn128'.

Notes: The point at infinity is encoded with both field x and y at 0.

Point addition (ADD) on the elliptic curve 'alt_bn128'


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_ec_add">ec_add</a>(x1: <a href="">vector</a>&lt;u8&gt;, y1: <a href="">vector</a>&lt;u8&gt;, x2: <a href="">vector</a>&lt;u8&gt;, y2: <a href="">vector</a>&lt;u8&gt;): (<a href="">vector</a>&lt;u8&gt;, <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x2_evm_ec_pairing"></a>

## Function `ec_pairing`

@param data: Coordinates of the points.
The input must always be a multiple of 6 32-byte values. 0 inputs is valid and returns 1.

Notes: The point at infinity is encoded with both field x and y at 0.

Bilinear function on groups on the elliptic curve 'alt_bn128'.


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_ec_pairing">ec_pairing</a>(data: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>
