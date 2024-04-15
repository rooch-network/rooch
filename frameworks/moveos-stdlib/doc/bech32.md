
<a name="0x2_bech32"></a>

# Module `0x2::bech32`

Module which defines bech32 functions.


-  [Constants](#@Constants_0)
-  [Function `encoding`](#0x2_bech32_encoding)
-  [Function `decoding`](#0x2_bech32_decoding)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_bech32_E_DECODE_FAILED"></a>



<pre><code><b>const</b> <a href="bech32.md#0x2_bech32_E_DECODE_FAILED">E_DECODE_FAILED</a>: u64 = 1;
</code></pre>



<a name="0x2_bech32_encoding"></a>

## Function `encoding`

@param public_key: 20 or 32 bytes public keys
@param witness_version: 0 for bech32 encoding and 1-16 for bech32m encoding.
Encode the public keys with Bech32 or Bech32m encoding algorithm and returns 42 or 62 length Bech32 or Bech32m addresses.


<pre><code><b>public</b> <b>fun</b> <a href="bech32.md#0x2_bech32_encoding">encoding</a>(public_key: &<a href="">vector</a>&lt;u8&gt;, witness_version: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_bech32_decoding"></a>

## Function `decoding`

@param data: 42 or 62 length Bech32 or Bech32m address bytes
Decode the encoded 42 or 62 length Bech32 or Bech32m address bytes with Bech32 or Bech32m decoding algorithm and returns 20 or 32 bytes of public keys.


<pre><code><b>public</b> <b>fun</b> <a href="bech32.md#0x2_bech32_decoding">decoding</a>(data: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>
