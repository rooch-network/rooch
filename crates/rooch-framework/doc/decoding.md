
<a name="0x3_decoding"></a>

# Module `0x3::decoding`

Module which defines decoding functions.


-  [Constants](#@Constants_0)
-  [Function `base58`](#0x3_decoding_base58)
-  [Function `base58check`](#0x3_decoding_base58check)
-  [Function `bech32`](#0x3_decoding_bech32)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_decoding_ErrorDecodeFailed"></a>

Failed to decode an address


<pre><code><b>const</b> <a href="decoding.md#0x3_decoding_ErrorDecodeFailed">ErrorDecodeFailed</a>: u64 = 1;
</code></pre>



<a name="0x3_decoding_base58"></a>

## Function `base58`

@param encoded_address_bytes: encoded base58 address bytes
Decode the base58 address bytes with Base58 algorithm and returns a raw base58 address bytes


<pre><code><b>public</b> <b>fun</b> <a href="decoding.md#0x3_decoding_base58">base58</a>(encoded_address_bytes: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_decoding_base58check"></a>

## Function `base58check`

@param encoded_address_bytes: encoded base58 address bytes
@param version_byte: version byte used for verification of different types of base58 addresses
Decode the base58 address bytes with Base58Check algorithm and returns a raw address bytes without checksum


<pre><code><b>public</b> <b>fun</b> <a href="decoding.md#0x3_decoding_base58check">base58check</a>(encoded_address_bytes: &<a href="">vector</a>&lt;u8&gt;, version_byte: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_decoding_bech32"></a>

## Function `bech32`

@param encoded_bech32_address_bytes: 42 or 62 length Bech32 or Bech32m addresses
Decode the encoded 42 or 62 length Bech32 or Bech32m addresses with Bech32 or Bech32m decoding algorithm and returns 20 or 32 bytes of public keys.


<pre><code><b>public</b> <b>fun</b> <a href="decoding.md#0x3_decoding_bech32">bech32</a>(encoded_bech32_address_bytes: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>
