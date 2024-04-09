
<a name="0x3_encoding"></a>

# Module `0x3::encoding`

Module which defines encoding functions.


-  [Constants](#@Constants_0)
-  [Function `base58`](#0x3_encoding_base58)
-  [Function `base58check`](#0x3_encoding_base58check)
-  [Function `bech32`](#0x3_encoding_bech32)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_encoding_ErrorExcessiveScriptSize"></a>

Excessive script size


<pre><code><b>const</b> <a href="encoding.md#0x3_encoding_ErrorExcessiveScriptSize">ErrorExcessiveScriptSize</a>: u64 = 2;
</code></pre>



<a name="0x3_encoding_ErrorInvalidData"></a>

Invalid data


<pre><code><b>const</b> <a href="encoding.md#0x3_encoding_ErrorInvalidData">ErrorInvalidData</a>: u64 = 3;
</code></pre>



<a name="0x3_encoding_ErrorInvalidPubkey"></a>

Invalid publich key


<pre><code><b>const</b> <a href="encoding.md#0x3_encoding_ErrorInvalidPubkey">ErrorInvalidPubkey</a>: u64 = 1;
</code></pre>



<a name="0x3_encoding_ErrorInvalidScriptVersion"></a>

Invalid script version


<pre><code><b>const</b> <a href="encoding.md#0x3_encoding_ErrorInvalidScriptVersion">ErrorInvalidScriptVersion</a>: u64 = 4;
</code></pre>



<a name="0x3_encoding_base58"></a>

## Function `base58`

@param address_bytes: address bytes for base58 format
Encode the address bytes with Base58 algorithm and returns an encoded base58 bytes


<pre><code><b>public</b> <b>fun</b> <a href="encoding.md#0x3_encoding_base58">base58</a>(address_bytes: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_encoding_base58check"></a>

## Function `base58check`

@param address_bytes: address bytes on the base58 checksum format
@param version_byte: version byte used for verification of different types of checksum addresses
Encode the address bytes with Base58Check algorithm and returns an encoded base58 bytes with checksum


<pre><code><b>public</b> <b>fun</b> <a href="encoding.md#0x3_encoding_base58check">base58check</a>(address_bytes: &<a href="">vector</a>&lt;u8&gt;, version_byte: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_encoding_bech32"></a>

## Function `bech32`

@param public_key: 20 or 32 bytes public keys
@param version: 0 for bech32 encoding and 1 for bech32m encoding. 2-16 are held.
Encode the public keys with Bech32 or Bech32m encoding algorithm and returns 42 or 62 length Bech32 or Bech32m addresses.


<pre><code><b>public</b> <b>fun</b> <a href="encoding.md#0x3_encoding_bech32">bech32</a>(public_key: &<a href="">vector</a>&lt;u8&gt;, version: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>
