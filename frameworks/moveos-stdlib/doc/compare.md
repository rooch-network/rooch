
<a name="0x2_compare"></a>

# Module `0x2::compare`

Utilities for comparing Move values


-  [Constants](#@Constants_0)
-  [Function `result_equal`](#0x2_compare_result_equal)
-  [Function `result_less_than`](#0x2_compare_result_less_than)
-  [Function `result_greater_than`](#0x2_compare_result_greater_than)
-  [Function `compare`](#0x2_compare_compare)
-  [Function `compare_vector_u8`](#0x2_compare_compare_vector_u8)
-  [Function `cmp_bcs_bytes`](#0x2_compare_cmp_bcs_bytes)


<pre><code><b>use</b> <a href="">0x1::compare</a>;
<b>use</b> <a href="">0x1::type_name</a>;
<b>use</b> <a href="">0x1::u256</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="decimal_value.md#0x2_decimal_value">0x2::decimal_value</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_compare_EQUAL"></a>



<pre><code><b>const</b> <a href="compare.md#0x2_compare_EQUAL">EQUAL</a>: u8 = 0;
</code></pre>



<a name="0x2_compare_GREATER_THAN"></a>



<pre><code><b>const</b> <a href="compare.md#0x2_compare_GREATER_THAN">GREATER_THAN</a>: u8 = 2;
</code></pre>



<a name="0x2_compare_LESS_THAN"></a>



<pre><code><b>const</b> <a href="compare.md#0x2_compare_LESS_THAN">LESS_THAN</a>: u8 = 1;
</code></pre>



<a name="0x2_compare_result_equal"></a>

## Function `result_equal`



<pre><code><b>public</b> <b>fun</b> <a href="compare.md#0x2_compare_result_equal">result_equal</a>(): u8
</code></pre>



<a name="0x2_compare_result_less_than"></a>

## Function `result_less_than`



<pre><code><b>public</b> <b>fun</b> <a href="compare.md#0x2_compare_result_less_than">result_less_than</a>(): u8
</code></pre>



<a name="0x2_compare_result_greater_than"></a>

## Function `result_greater_than`



<pre><code><b>public</b> <b>fun</b> <a href="compare.md#0x2_compare_result_greater_than">result_greater_than</a>(): u8
</code></pre>



<a name="0x2_compare_compare"></a>

## Function `compare`

Compare two values of the same type
This function will detect the type of the value and compare them accordingly
If the type is numeric, it will compare the numeric value, otherwise it will compare the bytes


<pre><code><b>public</b> <b>fun</b> <a href="">compare</a>&lt;T&gt;(a: &T, b: &T): u8
</code></pre>



<a name="0x2_compare_compare_vector_u8"></a>

## Function `compare_vector_u8`

Compare two vector<u8> values
This function is different with std::compare::cmp_bcs_bytes, which compares the vector contents from right to left,
But this function compares the vector contents from left to right.


<pre><code><b>public</b> <b>fun</b> <a href="compare.md#0x2_compare_compare_vector_u8">compare_vector_u8</a>(v1: &<a href="">vector</a>&lt;u8&gt;, v2: &<a href="">vector</a>&lt;u8&gt;): u8
</code></pre>



<a name="0x2_compare_cmp_bcs_bytes"></a>

## Function `cmp_bcs_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="compare.md#0x2_compare_cmp_bcs_bytes">cmp_bcs_bytes</a>(v1: &<a href="">vector</a>&lt;u8&gt;, v2: &<a href="">vector</a>&lt;u8&gt;): u8
</code></pre>
