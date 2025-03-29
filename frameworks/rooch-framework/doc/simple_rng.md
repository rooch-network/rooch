
<a id="0x3_simple_rng"></a>

# Module `0x3::simple_rng`

A simple random number generator in Move language.


-  [Constants](#@Constants_0)
-  [Function `bytes_to_u64`](#0x3_simple_rng_bytes_to_u64)
-  [Function `bytes_to_u128`](#0x3_simple_rng_bytes_to_u128)
-  [Function `rand_u64`](#0x3_simple_rng_rand_u64)
-  [Function `rand_u64_with_count`](#0x3_simple_rng_rand_u64_with_count)
-  [Function `rand_u128`](#0x3_simple_rng_rand_u128)
-  [Function `rand_u128_with_count`](#0x3_simple_rng_rand_u128_with_count)
-  [Function `rand_u64_range`](#0x3_simple_rng_rand_u64_range)
-  [Function `rand_u64_range_with_count`](#0x3_simple_rng_rand_u64_range_with_count)
-  [Function `rand_u128_range`](#0x3_simple_rng_rand_u128_range)
-  [Function `rand_u128_range_with_count`](#0x3_simple_rng_rand_u128_range_with_count)


<pre><code><b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::timestamp</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="transaction.md#0x3_transaction">0x3::transaction</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x3_simple_rng_ErrorInvalidArg"></a>



<pre><code><b>const</b> <a href="simple_rng.md#0x3_simple_rng_ErrorInvalidArg">ErrorInvalidArg</a>: u64 = 0;
</code></pre>



<a id="0x3_simple_rng_ErrorInvalidSeed"></a>



<pre><code><b>const</b> <a href="simple_rng.md#0x3_simple_rng_ErrorInvalidSeed">ErrorInvalidSeed</a>: u64 = 3;
</code></pre>



<a id="0x3_simple_rng_ErrorInvalidU128"></a>



<pre><code><b>const</b> <a href="simple_rng.md#0x3_simple_rng_ErrorInvalidU128">ErrorInvalidU128</a>: u64 = 2;
</code></pre>



<a id="0x3_simple_rng_ErrorInvalidU64"></a>



<pre><code><b>const</b> <a href="simple_rng.md#0x3_simple_rng_ErrorInvalidU64">ErrorInvalidU64</a>: u64 = 1;
</code></pre>



<a id="0x3_simple_rng_bytes_to_u64"></a>

## Function `bytes_to_u64`



<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x3_simple_rng_bytes_to_u64">bytes_to_u64</a>(bytes: <a href="">vector</a>&lt;u8&gt;): u64
</code></pre>



<a id="0x3_simple_rng_bytes_to_u128"></a>

## Function `bytes_to_u128`



<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x3_simple_rng_bytes_to_u128">bytes_to_u128</a>(bytes: <a href="">vector</a>&lt;u8&gt;): u128
</code></pre>



<a id="0x3_simple_rng_rand_u64"></a>

## Function `rand_u64`

Generate a random u64 from seed


<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x3_simple_rng_rand_u64">rand_u64</a>(): u64
</code></pre>



<a id="0x3_simple_rng_rand_u64_with_count"></a>

## Function `rand_u64_with_count`

Generate a random u64 value with a count parameter to ensure unique randomness within a transaction.


<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x3_simple_rng_rand_u64_with_count">rand_u64_with_count</a>(count: u64): u64
</code></pre>



<a id="0x3_simple_rng_rand_u128"></a>

## Function `rand_u128`

Generate a random u128 from seed


<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x3_simple_rng_rand_u128">rand_u128</a>(): u128
</code></pre>



<a id="0x3_simple_rng_rand_u128_with_count"></a>

## Function `rand_u128_with_count`

Generate a random u128 value with a count parameter to ensure unique randomness within a transaction.


<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x3_simple_rng_rand_u128_with_count">rand_u128_with_count</a>(count: u64): u128
</code></pre>



<a id="0x3_simple_rng_rand_u64_range"></a>

## Function `rand_u64_range`

Generate a random integer range in [low, high) for u64.


<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x3_simple_rng_rand_u64_range">rand_u64_range</a>(low: u64, high: u64): u64
</code></pre>



<a id="0x3_simple_rng_rand_u64_range_with_count"></a>

## Function `rand_u64_range_with_count`

Generate a random integer range in [low, high) for u64 with count.


<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x3_simple_rng_rand_u64_range_with_count">rand_u64_range_with_count</a>(low: u64, high: u64, count: u64): u64
</code></pre>



<a id="0x3_simple_rng_rand_u128_range"></a>

## Function `rand_u128_range`

Generate a random integer range in [low, high) for u128.


<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x3_simple_rng_rand_u128_range">rand_u128_range</a>(low: u128, high: u128): u128
</code></pre>



<a id="0x3_simple_rng_rand_u128_range_with_count"></a>

## Function `rand_u128_range_with_count`

Generate a random integer range in [low, high) for u128 with count.


<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x3_simple_rng_rand_u128_range_with_count">rand_u128_range_with_count</a>(low: u128, high: u128, count: u64): u128
</code></pre>
