
<a name="0x2_cbor"></a>

# Module `0x2::cbor`



-  [Constants](#@Constants_0)
-  [Function `from_cbor`](#0x2_cbor_from_cbor)
-  [Function `from_cbor_option`](#0x2_cbor_from_cbor_option)
-  [Function `to_map`](#0x2_cbor_to_map)
-  [Function `to_cbor`](#0x2_cbor_to_cbor)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="simple_map.md#0x2_simple_map">0x2::simple_map</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_cbor_ERROR_INVALID_CBOR_BYTES"></a>

Error if the CBOR bytes are invalid


<pre><code><b>const</b> <a href="cbor.md#0x2_cbor_ERROR_INVALID_CBOR_BYTES">ERROR_INVALID_CBOR_BYTES</a>: u64 = 1;
</code></pre>



<a name="0x2_cbor_from_cbor"></a>

## Function `from_cbor`

Function to deserialize a type T from CBOR bytes.


<pre><code>#[data_struct(#[T])]
<b>public</b> <b>fun</b> <a href="cbor.md#0x2_cbor_from_cbor">from_cbor</a>&lt;T: drop&gt;(bytes: <a href="">vector</a>&lt;u8&gt;): T
</code></pre>



<a name="0x2_cbor_from_cbor_option"></a>

## Function `from_cbor_option`

Function to deserialize a type T from CBOR bytes.
If the CBOR bytes are invalid, it will return None.


<pre><code>#[data_struct(#[T])]
<b>public</b> <b>fun</b> <a href="cbor.md#0x2_cbor_from_cbor_option">from_cbor_option</a>&lt;T: drop&gt;(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;T&gt;
</code></pre>



<a name="0x2_cbor_to_map"></a>

## Function `to_map`

Parse a cbor object bytes to a SimpleMap
If the cbor bytes is invalid, it will return an empty SimpleMap
If the field type is primitive type, it will be parsed to bytes, array or object will be parsed to cbor bytes


<pre><code><b>public</b> <b>fun</b> <a href="cbor.md#0x2_cbor_to_map">to_map</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_cbor_to_cbor"></a>

## Function `to_cbor`

Serialize a value of type T to CBOR bytes.


<pre><code>#[data_struct(#[T])]
<b>public</b> <b>fun</b> <a href="cbor.md#0x2_cbor_to_cbor">to_cbor</a>&lt;T: drop&gt;(value: &T): <a href="">vector</a>&lt;u8&gt;
</code></pre>
