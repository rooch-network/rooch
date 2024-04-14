
<a name="0x2_base58"></a>

# Module `0x2::base58`

Module which defines base58 functions.


-  [Constants](#@Constants_0)
-  [Function `encoding`](#0x2_base58_encoding)
-  [Function `checksum_encoding`](#0x2_base58_checksum_encoding)
-  [Function `decoding`](#0x2_base58_decoding)
-  [Function `checksum_decoding`](#0x2_base58_checksum_decoding)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_base58_E_DECODE_FAILED"></a>



<pre><code><b>const</b> <a href="base58.md#0x2_base58_E_DECODE_FAILED">E_DECODE_FAILED</a>: u64 = 1;
</code></pre>



<a name="0x2_base58_encoding"></a>

## Function `encoding`

@param address_bytes: address bytes for base58 format
Encode the address bytes with Base58 algorithm and returns an encoded base58 bytes


<pre><code><b>public</b> <b>fun</b> <a href="base58.md#0x2_base58_encoding">encoding</a>(address_bytes: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_base58_checksum_encoding"></a>

## Function `checksum_encoding`

@param address_bytes: address bytes on the base58 checksum format
@param version_byte: version byte used for verification of different types of checksum addresses
Encode the address bytes with Base58Check algorithm and returns an encoded base58 bytes with checksum


<pre><code><b>public</b> <b>fun</b> <a href="base58.md#0x2_base58_checksum_encoding">checksum_encoding</a>(address_bytes: &<a href="">vector</a>&lt;u8&gt;, version_byte: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_base58_decoding"></a>

## Function `decoding`

@param encoded_address_bytes: encoded base58 address bytes
Decode the base58 address bytes with Base58 algorithm and returns a raw base58 address bytes


<pre><code><b>public</b> <b>fun</b> <a href="base58.md#0x2_base58_decoding">decoding</a>(encoded_address_bytes: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_base58_checksum_decoding"></a>

## Function `checksum_decoding`

@param encoded_address_bytes: encoded base58 address bytes
@param version_byte: version byte used for verification of different types of base58 addresses
Decode the base58 address bytes with Base58Check algorithm and returns a raw base58 address bytes without checksum


<pre><code><b>public</b> <b>fun</b> <a href="base58.md#0x2_base58_checksum_decoding">checksum_decoding</a>(encoded_address_bytes: &<a href="">vector</a>&lt;u8&gt;, version_byte: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>
