
<a name="0x2_string_utils"></a>

# Module `0x2::string_utils`



-  [Constants](#@Constants_0)
-  [Function `parse_u8_option`](#0x2_string_utils_parse_u8_option)
-  [Function `parse_u8`](#0x2_string_utils_parse_u8)
-  [Function `parse_u64_option`](#0x2_string_utils_parse_u64_option)
-  [Function `parse_u64`](#0x2_string_utils_parse_u64)
-  [Function `parse_u128_option`](#0x2_string_utils_parse_u128_option)
-  [Function `parse_u128`](#0x2_string_utils_parse_u128)
-  [Function `parse_u256_option`](#0x2_string_utils_parse_u256_option)
-  [Function `parse_u256`](#0x2_string_utils_parse_u256)
-  [Function `parse_decimal_option`](#0x2_string_utils_parse_decimal_option)
-  [Function `parse_decimal`](#0x2_string_utils_parse_decimal)
-  [Function `to_lower_case`](#0x2_string_utils_to_lower_case)
-  [Function `to_upper_case`](#0x2_string_utils_to_upper_case)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_string_utils_ErrorInvalidStringNumber"></a>



<pre><code><b>const</b> <a href="string_utils.md#0x2_string_utils_ErrorInvalidStringNumber">ErrorInvalidStringNumber</a>: u64 = 1;
</code></pre>



<a name="0x2_string_utils_parse_u8_option"></a>

## Function `parse_u8_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u8_option">parse_u8_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u8&gt;
</code></pre>



<a name="0x2_string_utils_parse_u8"></a>

## Function `parse_u8`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u8">parse_u8</a>(s: &<a href="_String">string::String</a>): u8
</code></pre>



<a name="0x2_string_utils_parse_u64_option"></a>

## Function `parse_u64_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u64_option">parse_u64_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<a name="0x2_string_utils_parse_u64"></a>

## Function `parse_u64`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u64">parse_u64</a>(s: &<a href="_String">string::String</a>): u64
</code></pre>



<a name="0x2_string_utils_parse_u128_option"></a>

## Function `parse_u128_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u128_option">parse_u128_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u128&gt;
</code></pre>



<a name="0x2_string_utils_parse_u128"></a>

## Function `parse_u128`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u128">parse_u128</a>(s: &<a href="_String">string::String</a>): u128
</code></pre>



<a name="0x2_string_utils_parse_u256_option"></a>

## Function `parse_u256_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u256_option">parse_u256_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u256&gt;
</code></pre>



<a name="0x2_string_utils_parse_u256"></a>

## Function `parse_u256`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u256">parse_u256</a>(s: &<a href="_String">string::String</a>): u256
</code></pre>



<a name="0x2_string_utils_parse_decimal_option"></a>

## Function `parse_decimal_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_decimal_option">parse_decimal_option</a>(s: &<a href="_String">string::String</a>, decimal: u64): <a href="_Option">option::Option</a>&lt;u256&gt;
</code></pre>



<a name="0x2_string_utils_parse_decimal"></a>

## Function `parse_decimal`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_decimal">parse_decimal</a>(s: &<a href="_String">string::String</a>, decimal: u64): u256
</code></pre>



<a name="0x2_string_utils_to_lower_case"></a>

## Function `to_lower_case`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_lower_case">to_lower_case</a>(s: &<a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_string_utils_to_upper_case"></a>

## Function `to_upper_case`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_upper_case">to_upper_case</a>(s: &<a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>
