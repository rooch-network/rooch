
<a name="0x3_oracle"></a>

# Module `0x3::oracle`



-  [Resource `TablePlaceholder`](#0x3_oracle_TablePlaceholder)
-  [Resource `SimpleOracle`](#0x3_oracle_SimpleOracle)
-  [Resource `OracleAdminCap`](#0x3_oracle_OracleAdminCap)
-  [Struct `StoredData`](#0x3_oracle_StoredData)
-  [Struct `NewOracleEvent`](#0x3_oracle_NewOracleEvent)
-  [Constants](#@Constants_0)
-  [Function `get_historical_data`](#0x3_oracle_get_historical_data)
-  [Function `get_latest_data`](#0x3_oracle_get_latest_data)
-  [Function `create_entry`](#0x3_oracle_create_entry)
-  [Function `create`](#0x3_oracle_create)
-  [Function `submit_data`](#0x3_oracle_submit_data)
-  [Function `submit_data_with_timestamp`](#0x3_oracle_submit_data_with_timestamp)
-  [Function `submit_decimal_data`](#0x3_oracle_submit_decimal_data)
-  [Function `archive_data`](#0x3_oracle_archive_data)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::decimal_value</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::timestamp</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="oracle_data.md#0x3_oracle_data">0x3::oracle_data</a>;
</code></pre>



<a name="0x3_oracle_TablePlaceholder"></a>

## Resource `TablePlaceholder`



<pre><code><b>struct</b> <a href="oracle.md#0x3_oracle_TablePlaceholder">TablePlaceholder</a> <b>has</b> key
</code></pre>



<a name="0x3_oracle_SimpleOracle"></a>

## Resource `SimpleOracle`



<pre><code><b>struct</b> <a href="oracle.md#0x3_oracle_SimpleOracle">SimpleOracle</a> <b>has</b> store, key
</code></pre>



<a name="0x3_oracle_OracleAdminCap"></a>

## Resource `OracleAdminCap`



<pre><code><b>struct</b> <a href="oracle.md#0x3_oracle_OracleAdminCap">OracleAdminCap</a> <b>has</b> store, key
</code></pre>



<a name="0x3_oracle_StoredData"></a>

## Struct `StoredData`



<pre><code><b>struct</b> <a href="oracle.md#0x3_oracle_StoredData">StoredData</a>&lt;T: store&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_oracle_NewOracleEvent"></a>

## Struct `NewOracleEvent`



<pre><code><b>struct</b> <a href="oracle.md#0x3_oracle_NewOracleEvent">NewOracleEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_oracle_ErrorSenderNotOracle"></a>



<pre><code><b>const</b> <a href="oracle.md#0x3_oracle_ErrorSenderNotOracle">ErrorSenderNotOracle</a>: u64 = 0;
</code></pre>



<a name="0x3_oracle_ErrorTickerNotExists"></a>



<pre><code><b>const</b> <a href="oracle.md#0x3_oracle_ErrorTickerNotExists">ErrorTickerNotExists</a>: u64 = 1;
</code></pre>



<a name="0x3_oracle_get_historical_data"></a>

## Function `get_historical_data`



<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x3_oracle_get_historical_data">get_historical_data</a>&lt;K: <b>copy</b>, drop, store, V: <b>copy</b>, store&gt;(oracle_obj: &<a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_SimpleOracle">oracle::SimpleOracle</a>&gt;, ticker: <a href="_String">string::String</a>, archival_key: K): <a href="_Option">option::Option</a>&lt;<a href="oracle_data.md#0x3_oracle_data_Data">oracle_data::Data</a>&lt;V&gt;&gt;
</code></pre>



<a name="0x3_oracle_get_latest_data"></a>

## Function `get_latest_data`



<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x3_oracle_get_latest_data">get_latest_data</a>&lt;T: <b>copy</b>, store&gt;(oracle_obj: &<a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_SimpleOracle">oracle::SimpleOracle</a>&gt;, ticker: <a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="oracle_data.md#0x3_oracle_data_Data">oracle_data::Data</a>&lt;T&gt;&gt;
</code></pre>



<a name="0x3_oracle_create_entry"></a>

## Function `create_entry`

Create a new shared SimpleOracle object for publishing data.


<pre><code><b>public</b> entry <b>fun</b> <a href="oracle.md#0x3_oracle_create_entry">create_entry</a>(name: <a href="_String">string::String</a>, url: <a href="_String">string::String</a>, description: <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_oracle_create"></a>

## Function `create`

Create a new SimpleOracle object for publishing data.


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x3_oracle_create">create</a>(name: <a href="_String">string::String</a>, url: <a href="_String">string::String</a>, description: <a href="_String">string::String</a>): (<a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_SimpleOracle">oracle::SimpleOracle</a>&gt;, <a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_OracleAdminCap">oracle::OracleAdminCap</a>&gt;)
</code></pre>



<a name="0x3_oracle_submit_data"></a>

## Function `submit_data`



<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x3_oracle_submit_data">submit_data</a>&lt;T: <b>copy</b>, drop, store&gt;(oracle_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_SimpleOracle">oracle::SimpleOracle</a>&gt;, ticker: <a href="_String">string::String</a>, value: T, identifier: <a href="_String">string::String</a>, admin_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_OracleAdminCap">oracle::OracleAdminCap</a>&gt;)
</code></pre>



<a name="0x3_oracle_submit_data_with_timestamp"></a>

## Function `submit_data_with_timestamp`

Submit data with timestamp.
This function is used to submit data with a specific timestamp.
The timestamp is the time from the oracle's data source.
The timestamp is measured in milliseconds.


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x3_oracle_submit_data_with_timestamp">submit_data_with_timestamp</a>&lt;T: <b>copy</b>, drop, store&gt;(oracle_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_SimpleOracle">oracle::SimpleOracle</a>&gt;, ticker: <a href="_String">string::String</a>, value: T, identifier: <a href="_String">string::String</a>, <a href="">timestamp</a>: u64, admin_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_OracleAdminCap">oracle::OracleAdminCap</a>&gt;)
</code></pre>



<a name="0x3_oracle_submit_decimal_data"></a>

## Function `submit_decimal_data`



<pre><code><b>public</b> entry <b>fun</b> <a href="oracle.md#0x3_oracle_submit_decimal_data">submit_decimal_data</a>(oracle_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_SimpleOracle">oracle::SimpleOracle</a>&gt;, ticker: <a href="_String">string::String</a>, value: <a href="">u256</a>, decimal: u8, identifier: <a href="_String">string::String</a>, <a href="">timestamp</a>: u64, admin_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_OracleAdminCap">oracle::OracleAdminCap</a>&gt;)
</code></pre>



<a name="0x3_oracle_archive_data"></a>

## Function `archive_data`



<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x3_oracle_archive_data">archive_data</a>&lt;K: <b>copy</b>, drop, store, V: <b>copy</b>, drop, store&gt;(oracle_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_SimpleOracle">oracle::SimpleOracle</a>&gt;, ticker: <a href="_String">string::String</a>, archival_key: K, admin_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="oracle.md#0x3_oracle_OracleAdminCap">oracle::OracleAdminCap</a>&gt;)
</code></pre>
