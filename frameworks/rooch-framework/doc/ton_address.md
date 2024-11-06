
<a name="0x3_ton_address"></a>

# Module `0x3::ton_address`



-  [Struct `TonAddress`](#0x3_ton_address_TonAddress)
-  [Constants](#@Constants_0)
-  [Function `from_hex_str`](#0x3_ton_address_from_hex_str)
-  [Function `from_string`](#0x3_ton_address_from_string)
-  [Function `from_bytes`](#0x3_ton_address_from_bytes)
-  [Function `into_bytes`](#0x3_ton_address_into_bytes)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::hex</a>;
<b>use</b> <a href="">0x2::string_utils</a>;
</code></pre>



<a name="0x3_ton_address_TonAddress"></a>

## Struct `TonAddress`



<pre><code>#[data_struct]
<b>struct</b> <a href="ton_address.md#0x3_ton_address_TonAddress">TonAddress</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_ton_address_ErrorInvalidAddress"></a>



<pre><code><b>const</b> <a href="ton_address.md#0x3_ton_address_ErrorInvalidAddress">ErrorInvalidAddress</a>: u64 = 1;
</code></pre>



<a name="0x3_ton_address_ErrorInvalidWorkchain"></a>



<pre><code><b>const</b> <a href="ton_address.md#0x3_ton_address_ErrorInvalidWorkchain">ErrorInvalidWorkchain</a>: u64 = 2;
</code></pre>



<a name="0x3_ton_address_MINUS_CHAR"></a>

The minus char in hex address string: <code>-</code>


<pre><code><b>const</b> <a href="ton_address.md#0x3_ton_address_MINUS_CHAR">MINUS_CHAR</a>: u8 = 45;
</code></pre>



<a name="0x3_ton_address_SPLIT_CHAR"></a>

The split char in hex address string: <code>:</code>


<pre><code><b>const</b> <a href="ton_address.md#0x3_ton_address_SPLIT_CHAR">SPLIT_CHAR</a>: u8 = 58;
</code></pre>



<a name="0x3_ton_address_from_hex_str"></a>

## Function `from_hex_str`



<pre><code><b>public</b> <b>fun</b> <a href="ton_address.md#0x3_ton_address_from_hex_str">from_hex_str</a>(s: &<a href="_String">string::String</a>): <a href="ton_address.md#0x3_ton_address_TonAddress">ton_address::TonAddress</a>
</code></pre>



<a name="0x3_ton_address_from_string"></a>

## Function `from_string`



<pre><code><b>public</b> <b>fun</b> <a href="ton_address.md#0x3_ton_address_from_string">from_string</a>(addr_str: &<a href="_String">string::String</a>): <a href="ton_address.md#0x3_ton_address_TonAddress">ton_address::TonAddress</a>
</code></pre>



<a name="0x3_ton_address_from_bytes"></a>

## Function `from_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ton_address.md#0x3_ton_address_from_bytes">from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="ton_address.md#0x3_ton_address_TonAddress">ton_address::TonAddress</a>
</code></pre>



<a name="0x3_ton_address_into_bytes"></a>

## Function `into_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ton_address.md#0x3_ton_address_into_bytes">into_bytes</a>(addr: <a href="ton_address.md#0x3_ton_address_TonAddress">ton_address::TonAddress</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>
