
<a name="0x2_bcs"></a>

# Module `0x2::bcs`

Part source from https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-stdlib/sources/from_bcs.move
This module provides a number of functions to convert _primitive_ types from their representation in <code>std::bcs</code>
to values. This is the opposite of <code><a href="_to_bytes">bcs::to_bytes</a></code>.
Note we provie a generic public <code>from_bytes</code> function and protected it with <code>#[data_struct(T)]</code>.


-  [Struct `BCS`](#0x2_bcs_BCS)
-  [Constants](#@Constants_0)
-  [Function `to_bytes`](#0x2_bcs_to_bytes)
-  [Function `to_bool`](#0x2_bcs_to_bool)
-  [Function `to_u8`](#0x2_bcs_to_u8)
-  [Function `to_u64`](#0x2_bcs_to_u64)
-  [Function `to_u128`](#0x2_bcs_to_u128)
-  [Function `to_address`](#0x2_bcs_to_address)
-  [Function `new`](#0x2_bcs_new)
-  [Function `into_remainder_bytes`](#0x2_bcs_into_remainder_bytes)
-  [Function `peel_address`](#0x2_bcs_peel_address)
-  [Function `peel_bool`](#0x2_bcs_peel_bool)
-  [Function `peel_u8`](#0x2_bcs_peel_u8)
-  [Function `peel_u16`](#0x2_bcs_peel_u16)
-  [Function `peel_u32`](#0x2_bcs_peel_u32)
-  [Function `peel_u64`](#0x2_bcs_peel_u64)
-  [Function `peel_u128`](#0x2_bcs_peel_u128)
-  [Function `peel_u256`](#0x2_bcs_peel_u256)
-  [Function `peel_vec_length`](#0x2_bcs_peel_vec_length)
-  [Function `peel_vec_address`](#0x2_bcs_peel_vec_address)
-  [Function `peel_vec_bool`](#0x2_bcs_peel_vec_bool)
-  [Function `peel_vec_u8`](#0x2_bcs_peel_vec_u8)
-  [Function `peel_vec_vec_u8`](#0x2_bcs_peel_vec_vec_u8)
-  [Function `peel_vec_u16`](#0x2_bcs_peel_vec_u16)
-  [Function `peel_vec_u32`](#0x2_bcs_peel_vec_u32)
-  [Function `peel_vec_u64`](#0x2_bcs_peel_vec_u64)
-  [Function `peel_vec_u128`](#0x2_bcs_peel_vec_u128)
-  [Function `peel_vec_u256`](#0x2_bcs_peel_vec_u256)
-  [Function `peel_option_address`](#0x2_bcs_peel_option_address)
-  [Function `peel_option_bool`](#0x2_bcs_peel_option_bool)
-  [Function `peel_option_u8`](#0x2_bcs_peel_option_u8)
-  [Function `peel_option_u16`](#0x2_bcs_peel_option_u16)
-  [Function `peel_option_u32`](#0x2_bcs_peel_option_u32)
-  [Function `peel_option_u64`](#0x2_bcs_peel_option_u64)
-  [Function `peel_option_u128`](#0x2_bcs_peel_option_u128)
-  [Function `peel_option_u256`](#0x2_bcs_peel_option_u256)
-  [Function `from_bytes`](#0x2_bcs_from_bytes)
-  [Function `from_bytes_option`](#0x2_bcs_from_bytes_option)
-  [Function `native_from_bytes`](#0x2_bcs_native_from_bytes)


<pre><code><b>use</b> <a href="">0x1::bcs</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x2_bcs_BCS"></a>

## Struct `BCS`

A helper struct that saves resources on operations. For better
vector performance, it stores reversed bytes of the BCS and
enables use of <code><a href="_pop_back">vector::pop_back</a></code>.


<pre><code><b>struct</b> <a href="bcs.md#0x2_bcs_BCS">BCS</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_bcs_ErrorInvalidBool"></a>



<pre><code><b>const</b> <a href="bcs.md#0x2_bcs_ErrorInvalidBool">ErrorInvalidBool</a>: u64 = 4;
</code></pre>



<a name="0x2_bcs_ErrorInvalidBytes"></a>



<pre><code><b>const</b> <a href="bcs.md#0x2_bcs_ErrorInvalidBytes">ErrorInvalidBytes</a>: u64 = 2;
</code></pre>



<a name="0x2_bcs_ErrorInvalidLength"></a>



<pre><code><b>const</b> <a href="bcs.md#0x2_bcs_ErrorInvalidLength">ErrorInvalidLength</a>: u64 = 3;
</code></pre>



<a name="0x2_bcs_ErrorLengthOutOfRange"></a>



<pre><code><b>const</b> <a href="bcs.md#0x2_bcs_ErrorLengthOutOfRange">ErrorLengthOutOfRange</a>: u64 = 6;
</code></pre>



<a name="0x2_bcs_ErrorOutOfRange"></a>



<pre><code><b>const</b> <a href="bcs.md#0x2_bcs_ErrorOutOfRange">ErrorOutOfRange</a>: u64 = 5;
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



<a name="0x2_bcs_new"></a>

## Function `new`

Creates a new instance of BCS wrapper that holds inversed
bytes for better performance.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_new">new</a>(bytes: <a href="">vector</a>&lt;u8&gt;): bcs::BCS
</code></pre>



<a name="0x2_bcs_into_remainder_bytes"></a>

## Function `into_remainder_bytes`

Unpack the <code><a href="bcs.md#0x2_bcs_BCS">BCS</a></code> struct returning the leftover bytes.
Useful for passing the data further after partial deserialization.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_into_remainder_bytes">into_remainder_bytes</a>(<a href="">bcs</a>: bcs::BCS): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_bcs_peel_address"></a>

## Function `peel_address`

Read <code><b>address</b></code> value from the bcs-serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_address">peel_address</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <b>address</b>
</code></pre>



<a name="0x2_bcs_peel_bool"></a>

## Function `peel_bool`

Read a <code>bool</code> value from bcs-serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_bool">peel_bool</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): bool
</code></pre>



<a name="0x2_bcs_peel_u8"></a>

## Function `peel_u8`

Read <code>u8</code> value from bcs-serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_u8">peel_u8</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): u8
</code></pre>



<a name="0x2_bcs_peel_u16"></a>

## Function `peel_u16`

Read <code>u16</code> value from bcs-serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_u16">peel_u16</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): u16
</code></pre>



<a name="0x2_bcs_peel_u32"></a>

## Function `peel_u32`

Read <code>u32</code> value from bcs-serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_u32">peel_u32</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): u32
</code></pre>



<a name="0x2_bcs_peel_u64"></a>

## Function `peel_u64`

Read <code>u64</code> value from bcs-serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_u64">peel_u64</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): u64
</code></pre>



<a name="0x2_bcs_peel_u128"></a>

## Function `peel_u128`

Read <code>u128</code> value from bcs-serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_u128">peel_u128</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): u128
</code></pre>



<a name="0x2_bcs_peel_u256"></a>

## Function `peel_u256`

Read <code><a href="">u256</a></code> value from bcs-serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_u256">peel_u256</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="">u256</a>
</code></pre>



<a name="0x2_bcs_peel_vec_length"></a>

## Function `peel_vec_length`

Read ULEB bytes expecting a vector length. Result should
then be used to perform <code>peel_*</code> operation LEN times.

In BCS <code><a href="">vector</a></code> length is implemented with ULEB128;
See more here: https://en.wikipedia.org/wiki/LEB128


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_vec_length">peel_vec_length</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): u64
</code></pre>



<a name="0x2_bcs_peel_vec_address"></a>

## Function `peel_vec_address`

Peel a vector of <code><b>address</b></code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_vec_address">peel_vec_address</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<a name="0x2_bcs_peel_vec_bool"></a>

## Function `peel_vec_bool`

Peel a vector of <code>bool</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_vec_bool">peel_vec_bool</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="">vector</a>&lt;bool&gt;
</code></pre>



<a name="0x2_bcs_peel_vec_u8"></a>

## Function `peel_vec_u8`

Peel a vector of <code>u8</code> (eg string) from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_vec_u8">peel_vec_u8</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_bcs_peel_vec_vec_u8"></a>

## Function `peel_vec_vec_u8`

Peel a <code><a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;</code> (eg vec of string) from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_vec_vec_u8">peel_vec_vec_u8</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_bcs_peel_vec_u16"></a>

## Function `peel_vec_u16`

Peel a vector of <code>u16</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_vec_u16">peel_vec_u16</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="">vector</a>&lt;u16&gt;
</code></pre>



<a name="0x2_bcs_peel_vec_u32"></a>

## Function `peel_vec_u32`

Peel a vector of <code>u32</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_vec_u32">peel_vec_u32</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="">vector</a>&lt;u32&gt;
</code></pre>



<a name="0x2_bcs_peel_vec_u64"></a>

## Function `peel_vec_u64`

Peel a vector of <code>u64</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_vec_u64">peel_vec_u64</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="">vector</a>&lt;u64&gt;
</code></pre>



<a name="0x2_bcs_peel_vec_u128"></a>

## Function `peel_vec_u128`

Peel a vector of <code>u128</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_vec_u128">peel_vec_u128</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="">vector</a>&lt;u128&gt;
</code></pre>



<a name="0x2_bcs_peel_vec_u256"></a>

## Function `peel_vec_u256`

Peel a vector of <code><a href="">u256</a></code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_vec_u256">peel_vec_u256</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="">vector</a>&lt;<a href="">u256</a>&gt;
</code></pre>



<a name="0x2_bcs_peel_option_address"></a>

## Function `peel_option_address`

Peel <code>Option&lt;<b>address</b>&gt;</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_option_address">peel_option_address</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;
</code></pre>



<a name="0x2_bcs_peel_option_bool"></a>

## Function `peel_option_bool`

Peel <code>Option&lt;bool&gt;</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_option_bool">peel_option_bool</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="_Option">option::Option</a>&lt;bool&gt;
</code></pre>



<a name="0x2_bcs_peel_option_u8"></a>

## Function `peel_option_u8`

Peel <code>Option&lt;u8&gt;</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_option_u8">peel_option_u8</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="_Option">option::Option</a>&lt;u8&gt;
</code></pre>



<a name="0x2_bcs_peel_option_u16"></a>

## Function `peel_option_u16`

Peel <code>Option&lt;u16&gt;</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_option_u16">peel_option_u16</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="_Option">option::Option</a>&lt;u16&gt;
</code></pre>



<a name="0x2_bcs_peel_option_u32"></a>

## Function `peel_option_u32`

Peel <code>Option&lt;u32&gt;</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_option_u32">peel_option_u32</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="_Option">option::Option</a>&lt;u32&gt;
</code></pre>



<a name="0x2_bcs_peel_option_u64"></a>

## Function `peel_option_u64`

Peel <code>Option&lt;u64&gt;</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_option_u64">peel_option_u64</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<a name="0x2_bcs_peel_option_u128"></a>

## Function `peel_option_u128`

Peel <code>Option&lt;u128&gt;</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_option_u128">peel_option_u128</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="_Option">option::Option</a>&lt;u128&gt;
</code></pre>



<a name="0x2_bcs_peel_option_u256"></a>

## Function `peel_option_u256`

Peel <code>Option&lt;<a href="">u256</a>&gt;</code> from serialized bytes.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_peel_option_u256">peel_option_u256</a>(<a href="">bcs</a>: &<b>mut</b> bcs::BCS): <a href="_Option">option::Option</a>&lt;<a href="">u256</a>&gt;
</code></pre>



<a name="0x2_bcs_from_bytes"></a>

## Function `from_bytes`

Function to deserialize a type T.
Note the <code>data_struct</code> ensure the <code>T</code> must be a <code>#[data_struct]</code> type


<pre><code>#[data_struct(#[T])]
<b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_from_bytes">from_bytes</a>&lt;T&gt;(bytes: <a href="">vector</a>&lt;u8&gt;): T
</code></pre>



<a name="0x2_bcs_from_bytes_option"></a>

## Function `from_bytes_option`

Function to deserialize a type T.
Note the <code>data_struct</code> ensure the <code>T</code> must be a <code>#[data_struct]</code> type
If the bytes are invalid, it will return None.


<pre><code>#[data_struct(#[T])]
<b>public</b> <b>fun</b> <a href="bcs.md#0x2_bcs_from_bytes_option">from_bytes_option</a>&lt;T&gt;(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;T&gt;
</code></pre>



<a name="0x2_bcs_native_from_bytes"></a>

## Function `native_from_bytes`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bcs.md#0x2_bcs_native_from_bytes">native_from_bytes</a>&lt;T&gt;(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;T&gt;
</code></pre>
