
<a name="0x3_oracle_data"></a>

# Module `0x3::oracle_data`



-  [Struct `Data`](#0x3_oracle_data_Data)
-  [Struct `Metadata`](#0x3_oracle_data_Metadata)
-  [Function `new`](#0x3_oracle_data_new)
-  [Function `value`](#0x3_oracle_data_value)
-  [Function `oracle_address`](#0x3_oracle_data_oracle_address)
-  [Function `timestamp`](#0x3_oracle_data_timestamp)


<pre><code><b>use</b> <a href="">0x1::string</a>;
</code></pre>



<a name="0x3_oracle_data_Data"></a>

## Struct `Data`



<pre><code><b>struct</b> <a href="oracle_data.md#0x3_oracle_data_Data">Data</a>&lt;T&gt; <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_oracle_data_Metadata"></a>

## Struct `Metadata`



<pre><code><b>struct</b> <a href="oracle_data.md#0x3_oracle_data_Metadata">Metadata</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_oracle_data_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_data.md#0x3_oracle_data_new">new</a>&lt;T&gt;(value: T, ticker: <a href="_String">string::String</a>, sequence_number: u64, <a href="">timestamp</a>: u64, <a href="oracle.md#0x3_oracle">oracle</a>: <b>address</b>, identifier: <a href="_String">string::String</a>): <a href="oracle_data.md#0x3_oracle_data_Data">oracle_data::Data</a>&lt;T&gt;
</code></pre>



<a name="0x3_oracle_data_value"></a>

## Function `value`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_data.md#0x3_oracle_data_value">value</a>&lt;T&gt;(data: &<a href="oracle_data.md#0x3_oracle_data_Data">oracle_data::Data</a>&lt;T&gt;): &T
</code></pre>



<a name="0x3_oracle_data_oracle_address"></a>

## Function `oracle_address`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_data.md#0x3_oracle_data_oracle_address">oracle_address</a>&lt;T&gt;(data: &<a href="oracle_data.md#0x3_oracle_data_Data">oracle_data::Data</a>&lt;T&gt;): &<b>address</b>
</code></pre>



<a name="0x3_oracle_data_timestamp"></a>

## Function `timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="">timestamp</a>&lt;T&gt;(data: &<a href="oracle_data.md#0x3_oracle_data_Data">oracle_data::Data</a>&lt;T&gt;): u64
</code></pre>
