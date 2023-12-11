
<a name="0x2_json"></a>

# Module `0x2::json`



-  [Function `from_json`](#0x2_json_from_json)


<pre><code></code></pre>



<a name="0x2_json_from_json"></a>

## Function `from_json`

Function to deserialize a type T.
Note the <code>private_generics</code> ensure only the <code>T</code>'s owner module can call this function
The u128 and u256 types must be json String type instead of Number type


<pre><code>#[data_struct(#[T])]
#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="json.md#0x2_json_from_json">from_json</a>&lt;T&gt;(json_str: <a href="">vector</a>&lt;u8&gt;): T
</code></pre>
