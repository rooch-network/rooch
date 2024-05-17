
<a name="0x2_bcs"></a>

# Module `0x2::bcs`

Source from https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-stdlib/sources/from_bcs.move
This module provides a number of functions to convert _primitive_ types from their representation in <code>std::bcs</code>
to values. This is the opposite of <code><a href="_to_bytes">bcs::to_bytes</a></code>. Note that it is not safe to define a generic public <code>from_bytes</code>
function because this can violate implicit struct invariants, therefore only primitive types are offerred. If
a general conversion back-and-force is needed, consider the <code>moveos_std::Any</code> type which preserves invariants.


-  [Constants](#@Constants_0)
-  [Function `to_bytes`](#0x2_bcs_to_bytes)
-  [Function `to_bool`](#0x2_bcs_to_bool)
-  [Function `to_u8`](#0x2_bcs_to_u8)
-  [Function `to_u64`](#0x2_bcs_to_u64)
-  [Function `to_u128`](#0x2_bcs_to_u128)
-  [Function `to_address`](#0x2_bcs_to_address)
-  [Function `from_bytes`](#0x2_bcs_from_bytes)
-  [Function `from_bytes_option`](#0x2_bcs_from_bytes_option)
-  [Function `native_from_bytes`](#0x2_bcs_native_from_bytes)


<pre><code><b>use</b> <a href="">0x1::bcs</a>;
<b>use</b> <a href="">0x1::option</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_bcs_ErrorInvalidBytes"></a>



<pre><code><b>const</b> <a href="bcs.md#0x2_bcs_ErrorInvalidBytes">ErrorInvalidBytes</a>: u64 = 2;
</code></pre>



<a name="0x2_bcs_ErrorTypeNotMatch"></a>

The request Move type is not match with input Move type.


<pre><code><b>const</b> <a href="bcs.md#0x2_bcs_ErrorTypeNotMatch">ErrorTypeNotMatch</a>: u64 = 1;
</code></pre>



<a name="0x2_bcs_to_bytes"></a>

## Function `to_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_to_bytes">to_bytes</a>&lt;MoveValue&gt;(v: &MoveValue): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_bcs_to_bool"></a>

## Function `to_bool`



<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_to_bool">to_bool</a>(v: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<a name="0x2_bcs_to_u8"></a>

## Function `to_u8`



<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_to_u8">to_u8</a>(v: <a href="">vector</a>&lt;u8&gt;): u8
</code></pre>



<a name="0x2_bcs_to_u64"></a>

## Function `to_u64`



<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_to_u64">to_u64</a>(v: <a href="">vector</a>&lt;u8&gt;): u64
</code></pre>



<a name="0x2_bcs_to_u128"></a>

## Function `to_u128`



<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_to_u128">to_u128</a>(v: <a href="">vector</a>&lt;u8&gt;): u128
</code></pre>



<a name="0x2_bcs_to_address"></a>

## Function `to_address`



<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_to_address">to_address</a>(v: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<a name="0x2_bcs_from_bytes"></a>

## Function `from_bytes`

Function to deserialize a type T.
Note the <code>private_generics</code> ensure only the <code>MoveValue</code>'s owner module can call this function


<pre><code>#[data_struct(#[MoveValue])]
<b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_from_bytes">from_bytes</a>&lt;MoveValue&gt;(bytes: <a href="">vector</a>&lt;u8&gt;): MoveValue
</code></pre>



<a name="0x2_bcs_from_bytes_option"></a>

## Function `from_bytes_option`

Function to deserialize a type T.
Note the <code>private_generics</code> ensure only the <code>MoveValue</code>'s owner module can call this function
If the bytes are invalid, it will return None.


<pre><code>#[data_struct(#[MoveValue])]
<b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_from_bytes_option">from_bytes_option</a>&lt;MoveValue&gt;(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;MoveValue&gt;
</code></pre>



<a name="0x2_bcs_native_from_bytes"></a>

## Function `native_from_bytes`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bcs.md#0x2_bcs_native_from_bytes">native_from_bytes</a>&lt;MoveValue&gt;(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;MoveValue&gt;
</code></pre>
