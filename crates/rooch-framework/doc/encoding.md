
<a name="0x3_encoding"></a>

# Module `0x3::encoding`

Module which defines encoding functions.


-  [Function `base58check`](#0x3_encoding_base58check)
-  [Function `bech32`](#0x3_encoding_bech32)
-  [Function `p2sh`](#0x3_encoding_p2sh)
-  [Function `p2pkh`](#0x3_encoding_p2pkh)


<pre><code></code></pre>



<a name="0x3_encoding_base58check"></a>

## Function `base58check`

@param address_bytes: address bytes on the Bitcoin network
Encode the address bytes with Base58Check algorithm and returns an encoded Bitcoin address


<pre><code><b>public</b> <b>fun</b> <a href="encoding.md#0x3_encoding_base58check">base58check</a>(address_bytes: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="encoding.md#0x3_encoding_base58check">base58check</a>(address_bytes: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;;
</code></pre>



</details>

<a name="0x3_encoding_bech32"></a>

## Function `bech32`

@param public_key: 32 bytes compressed public key
Encode the public key with Bech32 or Bech32m encoding algorithm and returns 42 or 62 length Bitcoin Bech32 address.


<pre><code><b>public</b> <b>fun</b> <a href="encoding.md#0x3_encoding_bech32">bech32</a>(public_key: &<a href="">vector</a>&lt;u8&gt;, version: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="encoding.md#0x3_encoding_bech32">bech32</a>(public_key: &<a href="">vector</a>&lt;u8&gt;, version: u8): <a href="">vector</a>&lt;u8&gt;;
</code></pre>



</details>

<a name="0x3_encoding_p2sh"></a>

## Function `p2sh`

@param public_key: 32 bytes compressed public key
Creates a pay to script hash P2SH address from a script converted from a compressed public key.


<pre><code><b>public</b> <b>fun</b> <a href="encoding.md#0x3_encoding_p2sh">p2sh</a>(public_key: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="encoding.md#0x3_encoding_p2sh">p2sh</a>(public_key: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;;
</code></pre>



</details>

<a name="0x3_encoding_p2pkh"></a>

## Function `p2pkh`

@param public_key: 32 bytes compressed public key
Creates a pay to (compressed) public key hash address from a public key.


<pre><code><b>public</b> <b>fun</b> <a href="encoding.md#0x3_encoding_p2pkh">p2pkh</a>(public_key: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="encoding.md#0x3_encoding_p2pkh">p2pkh</a>(public_key: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;;
</code></pre>



</details>
