
<a name="0x2_bech32"></a>

# Module `0x2::bech32`

Module which defines bech32 functions.


-  [Constants](#@Constants_0)
-  [Function `encode`](#0x2_bech32_encode)
-  [Function `segwit_encode`](#0x2_bech32_segwit_encode)
-  [Function `decode`](#0x2_bech32_decode)
-  [Function `segwit_decode`](#0x2_bech32_segwit_decode)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_bech32_E_DECODE_FAILED"></a>



<pre><code><b>const</b> <a href="bech32.md#0x2_bech32_E_DECODE_FAILED">E_DECODE_FAILED</a>: u64 = 2;
</code></pre>



<a name="0x2_bech32_E_ENCODE_FAILED"></a>



<pre><code><b>const</b> <a href="bech32.md#0x2_bech32_E_ENCODE_FAILED">E_ENCODE_FAILED</a>: u64 = 1;
</code></pre>



<a name="0x2_bech32_E_INVALID_BIP_CODE"></a>



<pre><code><b>const</b> <a href="bech32.md#0x2_bech32_E_INVALID_BIP_CODE">E_INVALID_BIP_CODE</a>: u64 = 3;
</code></pre>



<a name="0x2_bech32_encode"></a>

## Function `encode`

@param hrp: human-readable part in string
@param data: arbitrary data to be encoded.
Encode arbitrary data using string as the human-readable part and append a bech32 checksum.


<pre><code><b>public</b> <b>fun</b> <a href="bech32.md#0x2_bech32_encode">encode</a>(bip: u16, hrp: <a href="">vector</a>&lt;u8&gt;, data: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_bech32_segwit_encode"></a>

## Function `segwit_encode`

@param network: network to be selected, i.e. bc, tb, or bcrt
@param witness_version: segwit witness version. 0 for bech32, 1 for bech32m and taproot, and 2-16 is included.
@param data: arbitrary data to be encoded.
Encode arbitrary data to a Bitcoin address using string as the network, number as the witness version


<pre><code><b>public</b> <b>fun</b> <a href="bech32.md#0x2_bech32_segwit_encode">segwit_encode</a>(network: <a href="">vector</a>&lt;u8&gt;, witness_version: u8, data: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_bech32_decode"></a>

## Function `decode`

@param hrp: human-readable part bytes to be used as a decoding input
@param encoded: encoded bytes to be decoded as data
Decode a bech32 encoded string that includes a bech32 checksum.


<pre><code><b>public</b> <b>fun</b> <a href="bech32.md#0x2_bech32_decode">decode</a>(hrp: <a href="">vector</a>&lt;u8&gt;, encoded: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_bech32_segwit_decode"></a>

## Function `segwit_decode`

@param hrp: human-readable part bytes to be used as a decoding input
@param witness_ascii_version: segwit witness ASCII version to be used as a decoding input
@param encoded: encoded bytes to be decoded as data
Decode an encoded Bitcoin address


<pre><code><b>public</b> <b>fun</b> <a href="bech32.md#0x2_bech32_segwit_decode">segwit_decode</a>(hrp: <a href="">vector</a>&lt;u8&gt;, witness_ascii_version: u8, encoded: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>
