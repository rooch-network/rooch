
<a name="0x2_copyable_any"></a>

# Module `0x2::copyable_any`

Source from https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-stdlib/sources/copyable_any.move


-  [Struct `Any`](#0x2_copyable_any_Any)
-  [Constants](#@Constants_0)
-  [Function `pack`](#0x2_copyable_any_pack)
-  [Function `unpack`](#0x2_copyable_any_unpack)
-  [Function `type_name`](#0x2_copyable_any_type_name)


<pre><code><b>use</b> <a href="">0x1::bcs</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="bcd.md#0x2_bcd">0x2::bcd</a>;
<b>use</b> <a href="type_info.md#0x2_type_info">0x2::type_info</a>;
</code></pre>



<a name="0x2_copyable_any_Any"></a>

## Struct `Any`

The same as <code><a href="any.md#0x2_any_Any">any::Any</a></code> but with the copy ability.


<pre><code><b>struct</b> <a href="copyable_any.md#0x2_copyable_any_Any">Any</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="">type_name</a>: <a href="_String">string::String</a></code>
</dt>
<dd>

</dd>
<dt>
<code>data: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x2_copyable_any_ETYPE_MISMATCH"></a>

The type provided for <code>unpack</code> is not the same as was given for <code>pack</code>.


<pre><code><b>const</b> <a href="copyable_any.md#0x2_copyable_any_ETYPE_MISMATCH">ETYPE_MISMATCH</a>: u64 = 0;
</code></pre>



<a name="0x2_copyable_any_pack"></a>

## Function `pack`

Pack a value into the <code><a href="copyable_any.md#0x2_copyable_any_Any">Any</a></code> representation. Because Any can be stored, dropped, and copied this is
also required from <code>T</code>.


<pre><code><b>public</b> <b>fun</b> <a href="copyable_any.md#0x2_copyable_any_pack">pack</a>&lt;T: <b>copy</b>, drop, store&gt;(x: T): <a href="copyable_any.md#0x2_copyable_any_Any">copyable_any::Any</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="copyable_any.md#0x2_copyable_any_pack">pack</a>&lt;T: drop + store + <b>copy</b>&gt;(x: T): <a href="copyable_any.md#0x2_copyable_any_Any">Any</a> {
    <a href="copyable_any.md#0x2_copyable_any_Any">Any</a> {
        <a href="">type_name</a>: <a href="type_info.md#0x2_type_info_type_name">type_info::type_name</a>&lt;T&gt;(),
        data: <a href="_to_bytes">bcs::to_bytes</a>(&x)
    }
}
</code></pre>



</details>

<a name="0x2_copyable_any_unpack"></a>

## Function `unpack`

Unpack a value from the <code><a href="copyable_any.md#0x2_copyable_any_Any">Any</a></code> representation. This aborts if the value has not the expected type <code>T</code>.


<pre><code><b>public</b> <b>fun</b> <a href="copyable_any.md#0x2_copyable_any_unpack">unpack</a>&lt;T&gt;(x: <a href="copyable_any.md#0x2_copyable_any_Any">copyable_any::Any</a>): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="copyable_any.md#0x2_copyable_any_unpack">unpack</a>&lt;T&gt;(x: <a href="copyable_any.md#0x2_copyable_any_Any">Any</a>): T {
    <b>assert</b>!(<a href="type_info.md#0x2_type_info_type_name">type_info::type_name</a>&lt;T&gt;() == x.<a href="">type_name</a>, <a href="_invalid_argument">error::invalid_argument</a>(<a href="copyable_any.md#0x2_copyable_any_ETYPE_MISMATCH">ETYPE_MISMATCH</a>));
    <a href="bcd.md#0x2_bcd_from_bytes">bcd::from_bytes</a>&lt;T&gt;(x.data)
}
</code></pre>



</details>

<a name="0x2_copyable_any_type_name"></a>

## Function `type_name`

Returns the type name of this Any


<pre><code><b>public</b> <b>fun</b> <a href="">type_name</a>(x: &<a href="copyable_any.md#0x2_copyable_any_Any">copyable_any::Any</a>): &<a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="">type_name</a>(x: &<a href="copyable_any.md#0x2_copyable_any_Any">Any</a>): &String {
    &x.<a href="">type_name</a>
}
</code></pre>



</details>
