
<a id="0x2_string_utils"></a>

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
-  [Function `parse_u16_option`](#0x2_string_utils_parse_u16_option)
-  [Function `parse_u16`](#0x2_string_utils_parse_u16)
-  [Function `parse_u32_option`](#0x2_string_utils_parse_u32_option)
-  [Function `parse_u32`](#0x2_string_utils_parse_u32)
-  [Function `parse_decimal_option`](#0x2_string_utils_parse_decimal_option)
-  [Function `parse_decimal`](#0x2_string_utils_parse_decimal)
-  [Function `to_lower_case`](#0x2_string_utils_to_lower_case)
-  [Function `to_upper_case`](#0x2_string_utils_to_upper_case)
-  [Function `to_string_u256`](#0x2_string_utils_to_string_u256)
-  [Function `to_string_u128`](#0x2_string_utils_to_string_u128)
-  [Function `to_string_u64`](#0x2_string_utils_to_string_u64)
-  [Function `to_string_u32`](#0x2_string_utils_to_string_u32)
-  [Function `to_string_u16`](#0x2_string_utils_to_string_u16)
-  [Function `to_string_u8`](#0x2_string_utils_to_string_u8)
-  [Function `starts_with`](#0x2_string_utils_starts_with)
-  [Function `contains`](#0x2_string_utils_contains)
-  [Function `split`](#0x2_string_utils_split)
-  [Function `trim`](#0x2_string_utils_trim)
-  [Function `strip_prefix`](#0x2_string_utils_strip_prefix)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::u256</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x2_string_utils_ErrorInvalidStringNumber"></a>



<pre><code><b>const</b> <a href="string_utils.md#0x2_string_utils_ErrorInvalidStringNumber">ErrorInvalidStringNumber</a>: u64 = 1;
</code></pre>



<a id="0x2_string_utils_SPACE_CHAR"></a>



<pre><code><b>const</b> <a href="string_utils.md#0x2_string_utils_SPACE_CHAR">SPACE_CHAR</a>: u8 = 32;
</code></pre>



<a id="0x2_string_utils_parse_u8_option"></a>

## Function `parse_u8_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u8_option">parse_u8_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u8&gt;
</code></pre>



<a id="0x2_string_utils_parse_u8"></a>

## Function `parse_u8`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u8">parse_u8</a>(s: &<a href="_String">string::String</a>): u8
</code></pre>



<a id="0x2_string_utils_parse_u64_option"></a>

## Function `parse_u64_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u64_option">parse_u64_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<a id="0x2_string_utils_parse_u64"></a>

## Function `parse_u64`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u64">parse_u64</a>(s: &<a href="_String">string::String</a>): u64
</code></pre>



<a id="0x2_string_utils_parse_u128_option"></a>

## Function `parse_u128_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u128_option">parse_u128_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u128&gt;
</code></pre>



<a id="0x2_string_utils_parse_u128"></a>

## Function `parse_u128`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u128">parse_u128</a>(s: &<a href="_String">string::String</a>): u128
</code></pre>



<a id="0x2_string_utils_parse_u256_option"></a>

## Function `parse_u256_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u256_option">parse_u256_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="">u256</a>&gt;
</code></pre>



<a id="0x2_string_utils_parse_u256"></a>

## Function `parse_u256`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u256">parse_u256</a>(s: &<a href="_String">string::String</a>): <a href="">u256</a>
</code></pre>



<a id="0x2_string_utils_parse_u16_option"></a>

## Function `parse_u16_option`

Parse a string into a u16, returning an option


<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u16_option">parse_u16_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u16&gt;
</code></pre>



<a id="0x2_string_utils_parse_u16"></a>

## Function `parse_u16`

Parse a string into a u16, aborting if the string is not a valid number


<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u16">parse_u16</a>(s: &<a href="_String">string::String</a>): u16
</code></pre>



<a id="0x2_string_utils_parse_u32_option"></a>

## Function `parse_u32_option`

Parse a string into a u32, returning an option


<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u32_option">parse_u32_option</a>(s: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u32&gt;
</code></pre>



<a id="0x2_string_utils_parse_u32"></a>

## Function `parse_u32`

Parse a string into a u32, aborting if the string is not a valid number


<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_u32">parse_u32</a>(s: &<a href="_String">string::String</a>): u32
</code></pre>



<a id="0x2_string_utils_parse_decimal_option"></a>

## Function `parse_decimal_option`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_decimal_option">parse_decimal_option</a>(s: &<a href="_String">string::String</a>, decimal: u8): <a href="_Option">option::Option</a>&lt;<a href="">u256</a>&gt;
</code></pre>



<a id="0x2_string_utils_parse_decimal"></a>

## Function `parse_decimal`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_parse_decimal">parse_decimal</a>(s: &<a href="_String">string::String</a>, decimal: u8): <a href="">u256</a>
</code></pre>



<a id="0x2_string_utils_to_lower_case"></a>

## Function `to_lower_case`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_lower_case">to_lower_case</a>(s: &<a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a id="0x2_string_utils_to_upper_case"></a>

## Function `to_upper_case`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_upper_case">to_upper_case</a>(s: &<a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a id="0x2_string_utils_to_string_u256"></a>

## Function `to_string_u256`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_string_u256">to_string_u256</a>(n: <a href="">u256</a>): <a href="_String">string::String</a>
</code></pre>



<a id="0x2_string_utils_to_string_u128"></a>

## Function `to_string_u128`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_string_u128">to_string_u128</a>(n: u128): <a href="_String">string::String</a>
</code></pre>



<a id="0x2_string_utils_to_string_u64"></a>

## Function `to_string_u64`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_string_u64">to_string_u64</a>(n: u64): <a href="_String">string::String</a>
</code></pre>



<a id="0x2_string_utils_to_string_u32"></a>

## Function `to_string_u32`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_string_u32">to_string_u32</a>(n: u32): <a href="_String">string::String</a>
</code></pre>



<a id="0x2_string_utils_to_string_u16"></a>

## Function `to_string_u16`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_string_u16">to_string_u16</a>(n: u16): <a href="_String">string::String</a>
</code></pre>



<a id="0x2_string_utils_to_string_u8"></a>

## Function `to_string_u8`



<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_to_string_u8">to_string_u8</a>(n: u8): <a href="_String">string::String</a>
</code></pre>



<a id="0x2_string_utils_starts_with"></a>

## Function `starts_with`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="string_utils.md#0x2_string_utils_starts_with">starts_with</a>(haystack_str: &<a href="_String">string::String</a>, needle: &<a href="_String">string::String</a>): bool
</code></pre>



<a id="0x2_string_utils_contains"></a>

## Function `contains`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="string_utils.md#0x2_string_utils_contains">contains</a>(s: &<a href="_String">string::String</a>, sub: &<a href="_String">string::String</a>): bool
</code></pre>



<a id="0x2_string_utils_split"></a>

## Function `split`

Split a string by a delimiter


<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_split">split</a>(s: &<a href="_String">string::String</a>, delimiter: &<a href="_String">string::String</a>): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a id="0x2_string_utils_trim"></a>

## Function `trim`

Trim leading and trailing whitespace from a string


<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_trim">trim</a>(s: &<a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a id="0x2_string_utils_strip_prefix"></a>

## Function `strip_prefix`

Strip a prefix from a string


<pre><code><b>public</b> <b>fun</b> <a href="string_utils.md#0x2_string_utils_strip_prefix">strip_prefix</a>(s: <a href="_String">string::String</a>, prefix: &<a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>
