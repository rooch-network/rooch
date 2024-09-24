
<a name="0x2_base64"></a>

# Module `0x2::base64`

Module which defines base64 functions.


-  [Constants](#@Constants_0)
-  [Function `encode`](#0x2_base64_encode)
-  [Function `decode`](#0x2_base64_decode)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_base64_E_DECODE_FAILED"></a>



<pre><code><b>const</b> <a href="base64.md#0x2_base64_E_DECODE_FAILED">E_DECODE_FAILED</a>: u64 = 1;
</code></pre>



<a name="0x2_base64_encode"></a>

## Function `encode`

@param input: bytes to be encoded
Encode the input bytes with Base64 algorithm and returns an encoded base64 string


<pre><code><b>public</b> <b>fun</b> <a href="base64.md#0x2_base64_encode">encode</a>(input: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_base64_decode"></a>

## Function `decode`

@param encoded_input: encoded base64 string
Decode the base64 string and returns the original bytes


<pre><code><b>public</b> <b>fun</b> <a href="base64.md#0x2_base64_decode">decode</a>(encoded_input: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>
