
<a name="0x2_type_info"></a>

# Module `0x2::type_info`

Source from https://github.com/aptos-labs/aptos-core/blob/c76c6b0fc3a1b8e21b6ba2f77151ca20ea31ca32/aptos-move/framework/aptos-stdlib/sources/type_info.move#L1
https://github.com/starcoinorg/starcoin-framework/blob/952c51116e0ef5a97c119205d6f7e038acdd8682/sources/Token.move#L508


-  [Struct `TypeInfo`](#0x2_type_info_TypeInfo)
-  [Constants](#@Constants_0)
-  [Function `account_address`](#0x2_type_info_account_address)
-  [Function `module_name`](#0x2_type_info_module_name)
-  [Function `struct_name`](#0x2_type_info_struct_name)
-  [Function `type_of`](#0x2_type_info_type_of)
-  [Function `type_name`](#0x2_type_info_type_name)
-  [Function `size_of_val`](#0x2_type_info_size_of_val)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::type_name</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
</code></pre>



<a name="0x2_type_info_TypeInfo"></a>

## Struct `TypeInfo`



<pre><code><b>struct</b> <a href="type_info.md#0x2_type_info_TypeInfo">TypeInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>module_name: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>struct_name: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x2_type_info_E_NATIVE_FUN_NOT_AVAILABLE"></a>



<pre><code><b>const</b> <a href="type_info.md#0x2_type_info_E_NATIVE_FUN_NOT_AVAILABLE">E_NATIVE_FUN_NOT_AVAILABLE</a>: u64 = 1;
</code></pre>



<a name="0x2_type_info_account_address"></a>

## Function `account_address`



<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_account_address">account_address</a>(<a href="type_info.md#0x2_type_info">type_info</a>: &<a href="type_info.md#0x2_type_info_TypeInfo">type_info::TypeInfo</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_account_address">account_address</a>(<a href="type_info.md#0x2_type_info">type_info</a>: &<a href="type_info.md#0x2_type_info_TypeInfo">TypeInfo</a>): <b>address</b> {
    <a href="type_info.md#0x2_type_info">type_info</a>.account_address
}
</code></pre>



</details>

<a name="0x2_type_info_module_name"></a>

## Function `module_name`



<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_module_name">module_name</a>(<a href="type_info.md#0x2_type_info">type_info</a>: &<a href="type_info.md#0x2_type_info_TypeInfo">type_info::TypeInfo</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_module_name">module_name</a>(<a href="type_info.md#0x2_type_info">type_info</a>: &<a href="type_info.md#0x2_type_info_TypeInfo">TypeInfo</a>): <a href="">vector</a>&lt;u8&gt; {
    <a href="type_info.md#0x2_type_info">type_info</a>.module_name
}
</code></pre>



</details>

<a name="0x2_type_info_struct_name"></a>

## Function `struct_name`



<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_struct_name">struct_name</a>(<a href="type_info.md#0x2_type_info">type_info</a>: &<a href="type_info.md#0x2_type_info_TypeInfo">type_info::TypeInfo</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_struct_name">struct_name</a>(<a href="type_info.md#0x2_type_info">type_info</a>: &<a href="type_info.md#0x2_type_info_TypeInfo">TypeInfo</a>): <a href="">vector</a>&lt;u8&gt; {
    <a href="type_info.md#0x2_type_info">type_info</a>.struct_name
}
</code></pre>



</details>

<a name="0x2_type_info_type_of"></a>

## Function `type_of`



<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_type_of">type_of</a>&lt;T&gt;(): <a href="type_info.md#0x2_type_info_TypeInfo">type_info::TypeInfo</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_type_of">type_of</a>&lt;T&gt;(): <a href="type_info.md#0x2_type_info_TypeInfo">TypeInfo</a>;
</code></pre>



</details>

<a name="0x2_type_info_type_name"></a>

## Function `type_name`



<pre><code><b>public</b> <b>fun</b> <a href="">type_name</a>&lt;T&gt;(): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="">type_name</a>&lt;T&gt;(): <a href="_String">string::String</a>{
    <b>let</b> <a href="">ascii</a> = std::type_name::into_string(std::type_name::get&lt;T&gt;());
    std::string::utf8(std::ascii::into_bytes(<a href="">ascii</a>))
}
</code></pre>



</details>

<a name="0x2_type_info_size_of_val"></a>

## Function `size_of_val`

Return the BCS size, in bytes, of value at <code>val_ref</code>.

See the [BCS spec](https://github.com/diem/bcs)

See <code>test_size_of_val()</code> for an analysis of common types and
nesting patterns, as well as <code>test_size_of_val_vectors()</code> for an
analysis of vector size dynamism.


<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_size_of_val">size_of_val</a>&lt;T&gt;(val_ref: &T): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_size_of_val">size_of_val</a>&lt;T&gt;(val_ref: &T): u64 {
    // Return <a href="">vector</a> length of vectorized BCS representation.
    <a href="_length">vector::length</a>(&<a href="../doc/bcs.md#0x1_bcs_to_bytes">bcs::to_bytes</a>(val_ref))
}
</code></pre>



</details>
