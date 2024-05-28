
<a name="0x2_evm"></a>

# Module `0x2::evm`



-  [Constants](#@Constants_0)
-  [Function `ec_recover`](#0x2_evm_ec_recover)
-  [Function `sha2_256`](#0x2_evm_sha2_256)
-  [Function `ripemd_160`](#0x2_evm_ripemd_160)
-  [Function `identity`](#0x2_evm_identity)
-  [Function `modexp`](#0x2_evm_modexp)
-  [Function `ec_add`](#0x2_evm_ec_add)
-  [Function `ec_mul`](#0x2_evm_ec_mul)
-  [Function `ec_pairing`](#0x2_evm_ec_pairing)
-  [Function `blake2f`](#0x2_evm_blake2f)
-  [Function `point_evaluation`](#0x2_evm_point_evaluation)


<pre><code><b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="hash.md#0x2_hash">0x2::hash</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_evm_ErrorBlake2fFailed"></a>



<pre><code><b>const</b> <a href="evm.md#0x2_evm_ErrorBlake2fFailed">ErrorBlake2fFailed</a>: u64 = 9;
</code></pre>



<a name="0x2_evm_ErrorEcAddFailed"></a>



<pre><code><b>const</b> <a href="evm.md#0x2_evm_ErrorEcAddFailed">ErrorEcAddFailed</a>: u64 = 6;
</code></pre>



<a name="0x2_evm_ErrorEcMulFailed"></a>



<pre><code><b>const</b> <a href="evm.md#0x2_evm_ErrorEcMulFailed">ErrorEcMulFailed</a>: u64 = 7;
</code></pre>



<a name="0x2_evm_ErrorEcPairingFailed"></a>



<pre><code><b>const</b> <a href="evm.md#0x2_evm_ErrorEcPairingFailed">ErrorEcPairingFailed</a>: u64 = 8;
</code></pre>



<a name="0x2_evm_ErrorEcRecoverFailed"></a>



<pre><code><b>const</b> <a href="evm.md#0x2_evm_ErrorEcRecoverFailed">ErrorEcRecoverFailed</a>: u64 = 1;
</code></pre>



<a name="0x2_evm_ErrorInvalidInputSize"></a>



<pre><code><b>const</b> <a href="evm.md#0x2_evm_ErrorInvalidInputSize">ErrorInvalidInputSize</a>: u64 = 11;
</code></pre>



<a name="0x2_evm_ErrorModexpFailed"></a>



<pre><code><b>const</b> <a href="evm.md#0x2_evm_ErrorModexpFailed">ErrorModexpFailed</a>: u64 = 5;
</code></pre>



<a name="0x2_evm_ErrorPointEvaluationFailed"></a>



<pre><code><b>const</b> <a href="evm.md#0x2_evm_ErrorPointEvaluationFailed">ErrorPointEvaluationFailed</a>: u64 = 10;
</code></pre>



<a name="0x2_evm_ec_recover"></a>

## Function `ec_recover`

@param hash: Keccack-256 hash of the transaction.
@param v: Recovery identifier, expected to be either 27 or 28.
@param r: x-value, expected to be in the range ]0; secp256k1n[.
@param s: Expected to be in the range ]0; secp256k1n[.

@return public_address: The recovered 20-byte address right aligned to 32 bytes.

Elliptic curve digital signature algorithm (ECDSA) public key recovery function.


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_ec_recover">ec_recover</a>(<a href="">hash</a>: <a href="">vector</a>&lt;u8&gt;, v: <a href="">vector</a>&lt;u8&gt;, r: <a href="">vector</a>&lt;u8&gt;, s: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_evm_sha2_256"></a>

## Function `sha2_256`

@param data: Data to hash with SHA2-256.

@return hash: The result hash.

Hash function.


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_sha2_256">sha2_256</a>(data: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_evm_ripemd_160"></a>

## Function `ripemd_160`

@param data: Data to hash with RIPEMD-160.

@return hash: The result 20-byte hash right aligned to 32 bytes.

Hash function.


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_ripemd_160">ripemd_160</a>(data: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_evm_identity"></a>

## Function `identity`

@param data: Data to return.

@return data: Data from input.

Returns the input.


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_identity">identity</a>(data: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_evm_modexp"></a>

## Function `modexp`

@param b_size: Byte size of B.
@param e_size: Byte size of E.
@param m_size: Byte size of M.
@param b: Base as unsigned integer.
@param e: Exponent as unsigned integer, if zero, then B ** E will be one.
@param m: Modulo as unsigned integer, if zero, then returns zero.

@return value: Result of the computation, with the same number of bytes as M.

Arbitrary-precision exponentiation under modulo.


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_modexp">modexp</a>(b_size: <a href="">vector</a>&lt;u8&gt;, e_size: <a href="">vector</a>&lt;u8&gt;, m_size: <a href="">vector</a>&lt;u8&gt;, b: <a href="">vector</a>&lt;u8&gt;, e: <a href="">vector</a>&lt;u8&gt;, m: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_evm_ec_add"></a>

## Function `ec_add`

@param x1: X coordinate of the first point on the elliptic curve 'alt_bn128'.
@param y1: Y coordinate of the first point on the elliptic curve 'alt_bn128'.
@param x2: X coordinate of the second point on the elliptic curve 'alt_bn128'.
@param y2: Y coordinate of the second point on the elliptic curve 'alt_bn128'.

@return x: X coordinate of the result point on the elliptic curve 'alt_bn128'.
@return y: Y coordinate of the result point on the elliptic curve 'alt_bn128'.

Notes: The point at infinity is encoded with both field x and y at 0.

Point addition (ADD) on the elliptic curve 'alt_bn128'.


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_ec_add">ec_add</a>(x1: <a href="">vector</a>&lt;u8&gt;, y1: <a href="">vector</a>&lt;u8&gt;, x2: <a href="">vector</a>&lt;u8&gt;, y2: <a href="">vector</a>&lt;u8&gt;): (<a href="">vector</a>&lt;u8&gt;, <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x2_evm_ec_mul"></a>

## Function `ec_mul`

@param x1: X coordinate of the first point on the elliptic curve 'alt_bn128'.
@param y1: Y coordinate of the first point on the elliptic curve 'alt_bn128'.
@param s: Scalar to use for the multiplication.

@return x: X coordinate of the result point on the elliptic curve 'alt_bn128'.
@return y: Y coordinate of the result point on the elliptic curve 'alt_bn128'.

Notes: The point at infinity is encoded with both field x and y at 0.

Scalar multiplication (MUL) on the elliptic curve 'alt_bn128'.


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_ec_mul">ec_mul</a>(x1: <a href="">vector</a>&lt;u8&gt;, y1: <a href="">vector</a>&lt;u8&gt;, s: <a href="">vector</a>&lt;u8&gt;): (<a href="">vector</a>&lt;u8&gt;, <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x2_evm_ec_pairing"></a>

## Function `ec_pairing`

@param data: Coordinates of the points.
The input must always be a multiple of 6 32-byte values. 0 inputs is valid and returns 1.

@return success: 1 if the pairing was a success, 0 otherwise.

Notes: The point at infinity is encoded with both field x and y at 0.

Bilinear function on groups on the elliptic curve 'alt_bn128'.


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_ec_pairing">ec_pairing</a>(data: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_evm_blake2f"></a>

## Function `blake2f`

@param rounds: Number of rounds (big-endian unsigned integer).
@param h: State vector (8 8-byte little-endian unsigned integer).
@param m: Message block vector (16 8-byte little-endian unsigned integer).
@param t: Offset counters (2 8-byte little-endian integer).
@param f: Final block indicator flag (0 or 1).

@return h: State vector (8 8-byte little-endian unsigned integer).

Compression function F used in the BLAKE2 cryptographic hashing algorithm.


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_blake2f">blake2f</a>(rounds: <a href="">vector</a>&lt;u8&gt;, h: <a href="">vector</a>&lt;u8&gt;, m: <a href="">vector</a>&lt;u8&gt;, t: <a href="">vector</a>&lt;u8&gt;, f: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_evm_point_evaluation"></a>

## Function `point_evaluation`

@param versioned_hash: Reference to a blob in the execution layer.
@param x: x-coordinate at which the blob is being evaluated.
@param y: y-coordinate at which the blob is being evaluated.
@param commitment: Commitment to the blob being evaluated.
@param proof: Proof associated with the commitment.

@return FIELD_ELEMENTS_PER_BLOB: The number of field elements in the blob.
@return : BLS_MODULUS: The modulus used in the BLS signature scheme.

Verify p(z) = y given commitment that corresponds to the polynomial p(x) and a KZG proof. Also verify that the provided commitment matches the provided versioned_hash.


<pre><code><b>public</b> <b>fun</b> <a href="evm.md#0x2_evm_point_evaluation">point_evaluation</a>(versioned_hash: <a href="">vector</a>&lt;u8&gt;, x: <a href="">vector</a>&lt;u8&gt;, y: <a href="">vector</a>&lt;u8&gt;, commitment: <a href="">vector</a>&lt;u8&gt;, proof: <a href="">vector</a>&lt;u8&gt;): (<a href="">vector</a>&lt;u8&gt;, <a href="">vector</a>&lt;u8&gt;)
</code></pre>
