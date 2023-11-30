
<a name="0x2_json"></a>

# Module `0x2::json`



-  [Constants](#@Constants_0)
-  [Function `from_json`](#0x2_json_from_json)
-  [Function `to_map`](#0x2_json_to_map)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="simple_map.md#0x2_simple_map">0x2::simple_map</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_json_ErrorTypeNotMatch"></a>



<pre><code><b>const</b> <a href="json.md#0x2_json_ErrorTypeNotMatch">ErrorTypeNotMatch</a>: u64 = 1;
</code></pre>



<a name="0x2_json_ErrorInvalidJSONString"></a>



<pre><code><b>const</b> <a href="json.md#0x2_json_ErrorInvalidJSONString">ErrorInvalidJSONString</a>: u64 = 2;
</code></pre>



<a name="0x2_json_from_json"></a>

## Function `from_json`

Function to deserialize a type T.
The u128 and u256 types must be json String type instead of Number type


<pre><code><b>public</b> <b>fun</b> <a href="json.md#0x2_json_from_json">from_json</a>&lt;T&gt;(json_str: <a href="">vector</a>&lt;u8&gt;): T
</code></pre>



<a name="0x2_json_to_map"></a>

## Function `to_map`

Parse a json object string to a SimpleMap
If the field type is primitive type, it will be parsed to String, otherwise it will abort.


<pre><code><b>public</b> <b>fun</b> <a href="json.md#0x2_json_to_map">to_map</a>(json_str: <a href="">vector</a>&lt;u8&gt;): <a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;
</code></pre>
