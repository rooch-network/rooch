
<a name="0x3_hash"></a>

# Module `0x3::hash`

Module which defines hash functions. Note that Sha-256 and Sha3-256 is available in the std::hash module in the
Move standard library and wrap the functions at here.


-  [Function `sha2_256`](#0x3_hash_sha2_256)
-  [Function `sha3_256`](#0x3_hash_sha3_256)
-  [Function `blake2b256`](#0x3_hash_blake2b256)
-  [Function `keccak256`](#0x3_hash_keccak256)


<pre><code><b>use</b> <a href="">0x1::hash</a>;
</code></pre>



<a name="0x3_hash_sha2_256"></a>

## Function `sha2_256`



<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_sha2_256">sha2_256</a>(data: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_hash_sha3_256"></a>

## Function `sha3_256`



<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_sha3_256">sha3_256</a>(data: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_hash_blake2b256"></a>

## Function `blake2b256`

@param data: Arbitrary binary data to hash
Hash the input bytes using Blake2b-256 and returns 32 bytes.


<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_blake2b256">blake2b256</a>(data: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_hash_keccak256"></a>

## Function `keccak256`

@param data: Arbitrary binary data to hash
Hash the input bytes using keccak256 and returns 32 bytes.


<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x3_hash_keccak256">keccak256</a>(data: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>
