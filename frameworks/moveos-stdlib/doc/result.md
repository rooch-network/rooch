
<a name="0x2_result"></a>

# Module `0x2::result`



-  [Struct `Result`](#0x2_result_Result)
-  [Constants](#@Constants_0)
-  [Function `ok`](#0x2_result_ok)
-  [Function `is_ok`](#0x2_result_is_ok)
-  [Function `get`](#0x2_result_get)
-  [Function `err`](#0x2_result_err)
-  [Function `err_str`](#0x2_result_err_str)
-  [Function `is_err`](#0x2_result_is_err)
-  [Function `get_err`](#0x2_result_get_err)
-  [Function `as_err`](#0x2_result_as_err)
-  [Function `unpack`](#0x2_result_unpack)
-  [Function `and_then`](#0x2_result_and_then)
-  [Function `unwrap`](#0x2_result_unwrap)
-  [Function `unwrap_err`](#0x2_result_unwrap_err)
-  [Function `assert_ok`](#0x2_result_assert_ok)
-  [Function `assert_err`](#0x2_result_assert_err)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
</code></pre>



<a name="0x2_result_Result"></a>

## Struct `Result`

The same as Rust's Result type.
Most of the time, we do not need the Result type in smart contract, we can directly abort the transaction.
But in some cases, we need to return a result to ensure the caller can handle the error.


<pre><code><b>struct</b> <a href="result.md#0x2_result_Result">Result</a>&lt;T, E&gt; <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_result_ErrorExpectErr"></a>

Expected the result is err but the result is ok.


<pre><code><b>const</b> <a href="result.md#0x2_result_ErrorExpectErr">ErrorExpectErr</a>: u64 = 2;
</code></pre>



<a name="0x2_result_ErrorExpectOk"></a>

Expected the result is ok but the result is err.


<pre><code><b>const</b> <a href="result.md#0x2_result_ErrorExpectOk">ErrorExpectOk</a>: u64 = 1;
</code></pre>



<a name="0x2_result_ok"></a>

## Function `ok`



<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_ok">ok</a>&lt;T, E&gt;(value: T): <a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;
</code></pre>



<a name="0x2_result_is_ok"></a>

## Function `is_ok`



<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_is_ok">is_ok</a>&lt;T, E&gt;(<a href="result.md#0x2_result">result</a>: &<a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;): bool
</code></pre>



<a name="0x2_result_get"></a>

## Function `get`



<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_get">get</a>&lt;T, E&gt;(<a href="result.md#0x2_result">result</a>: &<a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;): &<a href="_Option">option::Option</a>&lt;T&gt;
</code></pre>



<a name="0x2_result_err"></a>

## Function `err`



<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_err">err</a>&lt;T, E&gt;(err: E): <a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;
</code></pre>



<a name="0x2_result_err_str"></a>

## Function `err_str`

A shortcut to create a Result<T, String> with an error String with
err_str(b"msg").


<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_err_str">err_str</a>&lt;T&gt;(err: <a href="">vector</a>&lt;u8&gt;): <a href="result.md#0x2_result_Result">result::Result</a>&lt;T, <a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x2_result_is_err"></a>

## Function `is_err`



<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_is_err">is_err</a>&lt;T, E&gt;(<a href="result.md#0x2_result">result</a>: &<a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;): bool
</code></pre>



<a name="0x2_result_get_err"></a>

## Function `get_err`



<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_get_err">get_err</a>&lt;T, E&gt;(<a href="result.md#0x2_result">result</a>: &<a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;): &<a href="_Option">option::Option</a>&lt;E&gt;
</code></pre>



<a name="0x2_result_as_err"></a>

## Function `as_err`

Convert an error Result<T, String> to error Result<U, String>.


<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_as_err">as_err</a>&lt;U, T&gt;(self: <a href="result.md#0x2_result_Result">result::Result</a>&lt;T, <a href="_String">string::String</a>&gt;): <a href="result.md#0x2_result_Result">result::Result</a>&lt;U, <a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x2_result_unpack"></a>

## Function `unpack`



<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_unpack">unpack</a>&lt;T, E&gt;(<a href="result.md#0x2_result">result</a>: <a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;): (<a href="_Option">option::Option</a>&lt;T&gt;, <a href="_Option">option::Option</a>&lt;E&gt;)
</code></pre>



<a name="0x2_result_and_then"></a>

## Function `and_then`



<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_and_then">and_then</a>&lt;U, T, E&gt;(<a href="result.md#0x2_result">result</a>: <a href="result.md#0x2_result_Result">result::Result</a>&lt;U, E&gt;, f: |U|<a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;): <a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;
</code></pre>



<a name="0x2_result_unwrap"></a>

## Function `unwrap`



<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_unwrap">unwrap</a>&lt;T, E: drop&gt;(<a href="result.md#0x2_result">result</a>: <a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;): T
</code></pre>



<a name="0x2_result_unwrap_err"></a>

## Function `unwrap_err`



<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_unwrap_err">unwrap_err</a>&lt;T, E&gt;(<a href="result.md#0x2_result">result</a>: <a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;): E
</code></pre>



<a name="0x2_result_assert_ok"></a>

## Function `assert_ok`

Assert the result is ok, and return the value.
Otherwise, abort with the abort_code.
This function is inline, so it will be expanded in the caller.
This ensures the abort_code is the caller's location.


<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_assert_ok">assert_ok</a>&lt;T, E&gt;(<a href="result.md#0x2_result">result</a>: <a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;, abort_code: u64): T
</code></pre>



<a name="0x2_result_assert_err"></a>

## Function `assert_err`



<pre><code><b>public</b> <b>fun</b> <a href="result.md#0x2_result_assert_err">assert_err</a>&lt;T, E&gt;(<a href="result.md#0x2_result">result</a>: <a href="result.md#0x2_result_Result">result::Result</a>&lt;T, E&gt;, abort_code: u64): E
</code></pre>
