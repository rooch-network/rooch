
<a name="0x3_hash"></a>

# Module `0x3::hash`

Module which defines hash functions. Note that Sha-256 and Sha3-256 is available in the std::hash module in the
Move standard library and wrap the functions at here.


-  [Function `sha2_256`](#0x3_hash_sha2_256)
-  [Function `sha3_256`](#0x3_hash_sha3_256)
-  [Function `blake2b256`](#0x3_hash_blake2b256)
-  [Function `keccak256`](#0x3_hash_keccak256)
-  [Function `ripemd160`](#0x3_hash_ripemd160)


<pre><code><b>use</b> <a href="../../moveos/moveos-stdlib/move-stdlib/doc/hash.md#0x1_hash">0x1::hash</a>;
</code></pre>



<a name="0x3_hash_sha2_256"></a>

## Function `sha2_256`



<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_sha2_256">sha2_256</a>(data: <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_sha2_256">sha2_256</a>(data: <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
   std::hash::sha2_256(data)
}
</code></pre>



</details>

<a name="0x3_hash_sha3_256"></a>

## Function `sha3_256`



<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_sha3_256">sha3_256</a>(data: <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_sha3_256">sha3_256</a>(data: <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
   std::hash::sha3_256(data)
}
</code></pre>



</details>

<a name="0x3_hash_blake2b256"></a>

## Function `blake2b256`

@param data: Arbitrary binary data to hash
Hash the input bytes using Blake2b-256 and returns 32 bytes.


<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_blake2b256">blake2b256</a>(data: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_blake2b256">blake2b256</a>(data: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;;
</code></pre>



</details>

<a name="0x3_hash_keccak256"></a>

## Function `keccak256`

@param data: Arbitrary binary data to hash
Hash the input bytes using keccak256 and returns 32 bytes.


<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_keccak256">keccak256</a>(data: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_keccak256">keccak256</a>(data: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;;
</code></pre>



</details>

<a name="0x3_hash_ripemd160"></a>

## Function `ripemd160`

@param data: Arbitrary binary data to hash
Hash the input bytes using ripemd160 and returns 20 bytes.


<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_ripemd160">ripemd160</a>(data: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_ripemd160">ripemd160</a>(data: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../moveos/moveos-stdlib/move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;;
</code></pre>



</details>
