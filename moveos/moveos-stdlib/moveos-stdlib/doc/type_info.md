
<a name="0x2_type_info"></a>

# Module `0x2::type_info`



-  [Struct `TypeInfo`](#0x2_type_info_TypeInfo)
-  [Constants](#@Constants_0)
-  [Function `account_address`](#0x2_type_info_account_address)
-  [Function `module_name`](#0x2_type_info_module_name)
-  [Function `struct_name`](#0x2_type_info_struct_name)
-  [Function `type_of`](#0x2_type_info_type_of)
-  [Function `type_name`](#0x2_type_info_type_name)
-  [Function `size_of_val`](#0x2_type_info_size_of_val)
-  [Module Specification](#@Module_Specification_1)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::type_name</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
</code></pre>



<a name="0x2_type_info_TypeInfo"></a>

## Struct `TypeInfo`



<pre><code><b>struct</b> <a href="type_info.md#0x2_type_info_TypeInfo">TypeInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_type_info_ErrorNativeFunctionNotAvailable"></a>



<pre><code><b>const</b> <a href="type_info.md#0x2_type_info_ErrorNativeFunctionNotAvailable">ErrorNativeFunctionNotAvailable</a>: u64 = 1;
</code></pre>



<a name="0x2_type_info_account_address"></a>

## Function `account_address`



<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_account_address">account_address</a>(<a href="type_info.md#0x2_type_info">type_info</a>: &<a href="type_info.md#0x2_type_info_TypeInfo">type_info::TypeInfo</a>): <b>address</b>
</code></pre>



<a name="0x2_type_info_module_name"></a>

## Function `module_name`



<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_module_name">module_name</a>(<a href="type_info.md#0x2_type_info">type_info</a>: &<a href="type_info.md#0x2_type_info_TypeInfo">type_info::TypeInfo</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_type_info_struct_name"></a>

## Function `struct_name`



<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_struct_name">struct_name</a>(<a href="type_info.md#0x2_type_info">type_info</a>: &<a href="type_info.md#0x2_type_info_TypeInfo">type_info::TypeInfo</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_type_info_type_of"></a>

## Function `type_of`



<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_type_of">type_of</a>&lt;T&gt;(): <a href="type_info.md#0x2_type_info_TypeInfo">type_info::TypeInfo</a>
</code></pre>



<a name="0x2_type_info_type_name"></a>

## Function `type_name`



<pre><code><b>public</b> <b>fun</b> <a href="">type_name</a>&lt;T&gt;(): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_type_info_size_of_val"></a>

## Function `size_of_val`

Return the BCS size, in bytes, of value at <code>val_ref</code>.

See the [BCS spec](https://github.com/diem/bcs)

See <code>test_size_of_val()</code> for an analysis of common types and
nesting patterns, as well as <code>test_size_of_val_vectors()</code> for an
analysis of vector size dynamism.


<pre><code><b>public</b> <b>fun</b> <a href="type_info.md#0x2_type_info_size_of_val">size_of_val</a>&lt;T&gt;(val_ref: &T): u64
</code></pre>



<a name="@Module_Specification_1"></a>

## Module Specification
