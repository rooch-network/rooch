
<a name="0x3_oracle_meta"></a>

# Module `0x3::oracle_meta`



-  [Struct `MetaOracle`](#0x3_oracle_meta_MetaOracle)
-  [Struct `TrustedData`](#0x3_oracle_meta_TrustedData)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x3_oracle_meta_new)
-  [Function `add_simple_oracle`](#0x3_oracle_meta_add_simple_oracle)
-  [Function `median`](#0x3_oracle_meta_median)
-  [Function `data`](#0x3_oracle_meta_data)
-  [Function `threshold`](#0x3_oracle_meta_threshold)
-  [Function `time_window_ms`](#0x3_oracle_meta_time_window_ms)
-  [Function `ticker`](#0x3_oracle_meta_ticker)
-  [Function `max_timestamp`](#0x3_oracle_meta_max_timestamp)
-  [Function `value`](#0x3_oracle_meta_value)
-  [Function `oracles`](#0x3_oracle_meta_oracles)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::sort</a>;
<b>use</b> <a href="oracle.md#0x3_oracle">0x3::oracle</a>;
<b>use</b> <a href="oracle_data.md#0x3_oracle_data">0x3::oracle_data</a>;
</code></pre>



<a name="0x3_oracle_meta_MetaOracle"></a>

## Struct `MetaOracle`



<pre><code><b>struct</b> <a href="oracle_meta.md#0x3_oracle_meta_MetaOracle">MetaOracle</a>&lt;T&gt;
</code></pre>



<a name="0x3_oracle_meta_TrustedData"></a>

## Struct `TrustedData`



<pre><code><b>struct</b> <a href="oracle_meta.md#0x3_oracle_meta_TrustedData">TrustedData</a>&lt;T&gt; <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_oracle_meta_ErrorUnsupportedDataType"></a>



<pre><code><b>const</b> <a href="oracle_meta.md#0x3_oracle_meta_ErrorUnsupportedDataType">ErrorUnsupportedDataType</a>: u64 = 1;
</code></pre>



<a name="0x3_oracle_meta_ErrorValidDataSizeLessThanThreshold"></a>



<pre><code><b>const</b> <a href="oracle_meta.md#0x3_oracle_meta_ErrorValidDataSizeLessThanThreshold">ErrorValidDataSizeLessThanThreshold</a>: u64 = 0;
</code></pre>



<a name="0x3_oracle_meta_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_meta.md#0x3_oracle_meta_new">new</a>&lt;T: <b>copy</b>, drop&gt;(threshold: u64, time_window_ms: u64, ticker: <a href="_String">string::String</a>): <a href="oracle_meta.md#0x3_oracle_meta_MetaOracle">oracle_meta::MetaOracle</a>&lt;T&gt;
</code></pre>



<a name="0x3_oracle_meta_add_simple_oracle"></a>

## Function `add_simple_oracle`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_meta.md#0x3_oracle_meta_add_simple_oracle">add_simple_oracle</a>&lt;T: <b>copy</b>, drop, store&gt;(meta_oracle: &<b>mut</b> <a href="oracle_meta.md#0x3_oracle_meta_MetaOracle">oracle_meta::MetaOracle</a>&lt;T&gt;, <a href="oracle.md#0x3_oracle">oracle</a>: &<a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_SimpleOracle">oracle::SimpleOracle</a>&gt;)
</code></pre>



<a name="0x3_oracle_meta_median"></a>

## Function `median`

take the median value


<pre><code><b>public</b> <b>fun</b> <a href="oracle_meta.md#0x3_oracle_meta_median">median</a>&lt;T: <b>copy</b>, drop&gt;(meta_oracle: <a href="oracle_meta.md#0x3_oracle_meta_MetaOracle">oracle_meta::MetaOracle</a>&lt;T&gt;): <a href="oracle_meta.md#0x3_oracle_meta_TrustedData">oracle_meta::TrustedData</a>&lt;T&gt;
</code></pre>



<a name="0x3_oracle_meta_data"></a>

## Function `data`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_meta.md#0x3_oracle_meta_data">data</a>&lt;T&gt;(meta: &<a href="oracle_meta.md#0x3_oracle_meta_MetaOracle">oracle_meta::MetaOracle</a>&lt;T&gt;): &<a href="">vector</a>&lt;<a href="_Option">option::Option</a>&lt;<a href="oracle_data.md#0x3_oracle_data_Data">oracle_data::Data</a>&lt;T&gt;&gt;&gt;
</code></pre>



<a name="0x3_oracle_meta_threshold"></a>

## Function `threshold`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_meta.md#0x3_oracle_meta_threshold">threshold</a>&lt;T&gt;(meta: &<a href="oracle_meta.md#0x3_oracle_meta_MetaOracle">oracle_meta::MetaOracle</a>&lt;T&gt;): u64
</code></pre>



<a name="0x3_oracle_meta_time_window_ms"></a>

## Function `time_window_ms`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_meta.md#0x3_oracle_meta_time_window_ms">time_window_ms</a>&lt;T&gt;(meta: &<a href="oracle_meta.md#0x3_oracle_meta_MetaOracle">oracle_meta::MetaOracle</a>&lt;T&gt;): u64
</code></pre>



<a name="0x3_oracle_meta_ticker"></a>

## Function `ticker`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_meta.md#0x3_oracle_meta_ticker">ticker</a>&lt;T&gt;(meta: &<a href="oracle_meta.md#0x3_oracle_meta_MetaOracle">oracle_meta::MetaOracle</a>&lt;T&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_oracle_meta_max_timestamp"></a>

## Function `max_timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_meta.md#0x3_oracle_meta_max_timestamp">max_timestamp</a>&lt;T&gt;(meta: &<a href="oracle_meta.md#0x3_oracle_meta_MetaOracle">oracle_meta::MetaOracle</a>&lt;T&gt;): u64
</code></pre>



<a name="0x3_oracle_meta_value"></a>

## Function `value`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_meta.md#0x3_oracle_meta_value">value</a>&lt;T&gt;(data: &<a href="oracle_meta.md#0x3_oracle_meta_TrustedData">oracle_meta::TrustedData</a>&lt;T&gt;): &T
</code></pre>



<a name="0x3_oracle_meta_oracles"></a>

## Function `oracles`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_meta.md#0x3_oracle_meta_oracles">oracles</a>&lt;T&gt;(data: &<a href="oracle_meta.md#0x3_oracle_meta_TrustedData">oracle_meta::TrustedData</a>&lt;T&gt;): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>
