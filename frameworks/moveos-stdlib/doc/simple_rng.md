
<a name="0x2_simple_rng"></a>

# Module `0x2::simple_rng`

A simple random number generator in Move language.


-  [Struct `TransactionSequenceInfo`](#0x2_simple_rng_TransactionSequenceInfo)
-  [Constants](#@Constants_0)
-  [Function `rand_u64`](#0x2_simple_rng_rand_u64)
-  [Function `rand_u128`](#0x2_simple_rng_rand_u128)
-  [Function `rand_u64_range`](#0x2_simple_rng_rand_u64_range)
-  [Function `rand_u128_range`](#0x2_simple_rng_rand_u128_range)


<pre><code><b>use</b> <a href="">0x1::debug</a>;
<b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="timestamp.md#0x2_timestamp">0x2::timestamp</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_simple_rng_TransactionSequenceInfo"></a>

## Struct `TransactionSequenceInfo`



<pre><code><b>struct</b> <a href="simple_rng.md#0x2_simple_rng_TransactionSequenceInfo">TransactionSequenceInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_simple_rng_ErrorInvalidArg"></a>



<pre><code><b>const</b> <a href="simple_rng.md#0x2_simple_rng_ErrorInvalidArg">ErrorInvalidArg</a>: u64 = 0;
</code></pre>



<a name="0x2_simple_rng_ErrorInvalidSeed"></a>



<pre><code><b>const</b> <a href="simple_rng.md#0x2_simple_rng_ErrorInvalidSeed">ErrorInvalidSeed</a>: u64 = 3;
</code></pre>



<a name="0x2_simple_rng_ErrorInvalidU128"></a>



<pre><code><b>const</b> <a href="simple_rng.md#0x2_simple_rng_ErrorInvalidU128">ErrorInvalidU128</a>: u64 = 2;
</code></pre>



<a name="0x2_simple_rng_ErrorInvalidU64"></a>



<pre><code><b>const</b> <a href="simple_rng.md#0x2_simple_rng_ErrorInvalidU64">ErrorInvalidU64</a>: u64 = 1;
</code></pre>



<a name="0x2_simple_rng_rand_u64"></a>

## Function `rand_u64`

Generate a random u64 from seed


<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x2_simple_rng_rand_u64">rand_u64</a>(): u64
</code></pre>



<a name="0x2_simple_rng_rand_u128"></a>

## Function `rand_u128`

Generate a random u128 from seed


<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x2_simple_rng_rand_u128">rand_u128</a>(): u128
</code></pre>



<a name="0x2_simple_rng_rand_u64_range"></a>

## Function `rand_u64_range`

Generate a random integer range in [low, high] for u64.


<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x2_simple_rng_rand_u64_range">rand_u64_range</a>(low: u64, high: u64): u64
</code></pre>



<a name="0x2_simple_rng_rand_u128_range"></a>

## Function `rand_u128_range`

Generate a random integer range in [low, high] for u128.


<pre><code><b>public</b> <b>fun</b> <a href="simple_rng.md#0x2_simple_rng_rand_u128_range">rand_u128_range</a>(low: u128, high: u128): u128
</code></pre>
