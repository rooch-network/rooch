
<a name="0x2_string_utils"></a>

# Module `0x2::string_utils`



-  [Constants](#@Constants_0)
-  [Function `to_u8_option`](#0x2_string_utils_to_u8_option)
-  [Function `to_u8`](#0x2_string_utils_to_u8)
-  [Function `to_u64_option`](#0x2_string_utils_to_u64_option)
-  [Function `to_u64`](#0x2_string_utils_to_u64)
-  [Function `to_u128_option`](#0x2_string_utils_to_u128_option)
-  [Function `to_u128`](#0x2_string_utils_to_u128)
-  [Function `to_u256_option`](#0x2_string_utils_to_u256_option)
-  [Function `to_u256`](#0x2_string_utils_to_u256)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_string_utils_ErrorInvalidStringNumber"></a>



<pre><code><b>const</b> <a href="string_utils.md#0x2_string_utils_ErrorInvalidStringNumber">ErrorInvalidStringNumber</a>: u64 = 1;
</code></pre>



<a name="0x2_string_utils_to_u8_option"></a>

## Function `to_u8_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_u8_option">to_u8_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u8&gt;
</code></pre>



<a name="0x2_string_utils_to_u8"></a>

## Function `to_u8`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_u8">to_u8</a>(s: &<a href="_String">string::String</a>): u8
</code></pre>



<a name="0x2_string_utils_to_u64_option"></a>

## Function `to_u64_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_u64_option">to_u64_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<a name="0x2_string_utils_to_u64"></a>

## Function `to_u64`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_u64">to_u64</a>(s: &<a href="_String">string::String</a>): u64
</code></pre>



<a name="0x2_string_utils_to_u128_option"></a>

## Function `to_u128_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_u128_option">to_u128_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u128&gt;
</code></pre>



<a name="0x2_string_utils_to_u128"></a>

## Function `to_u128`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_u128">to_u128</a>(s: &<a href="_String">string::String</a>): u128
</code></pre>



<a name="0x2_string_utils_to_u256_option"></a>

## Function `to_u256_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_u256_option">to_u256_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u256&gt;
</code></pre>



<a name="0x2_string_utils_to_u256"></a>

## Function `to_u256`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_u256">to_u256</a>(s: &<a href="_String">string::String</a>): u256
</code></pre>
