
<a name="0x3_decoding"></a>

# Module `0x3::decoding`

Module which defines decoding functions.


-  [Constants](#@Constants_0)
-  [Function `base58`](#0x3_decoding_base58)
-  [Function `base58check`](#0x3_decoding_base58check)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_decoding_ErrorDecodeFailed"></a>

Failed to decode an address


<pre><code><b>const</b> <a href="decoding.md#0x3_decoding_ErrorDecodeFailed">ErrorDecodeFailed</a>: u64 = 1;
</code></pre>



<a name="0x3_decoding_base58"></a>

## Function `base58`

@param encoded_address_bytes: encoded Bitcoin address bytes on the Bitcoin network
Decode the Bitcoin address bytes with Base58 algorithm and returns a raw address bytes


<pre><code><b>public</b> <b>fun</b> <a href="decoding.md#0x3_decoding_base58">base58</a>(encoded_address_bytes: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_decoding_base58check"></a>

## Function `base58check`

@param encoded_address_bytes: encoded Bitcoin address bytes on the Bitcoin network
@param version_byte: version byte used on Bitcoin network for verification of different types of addresses
Decode the Bitcoin address bytes with Base58Check algorithm and returns a raw address bytes without checksum


<pre><code><b>public</b> <b>fun</b> <a href="decoding.md#0x3_decoding_base58check">base58check</a>(encoded_address_bytes: &<a href="">vector</a>&lt;u8&gt;, version_byte: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>
