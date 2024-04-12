
<a name="0x4_data_import_config"></a>

# Module `0x4::data_import_config`



-  [Resource `DataImportConfig`](#0x4_data_import_config_DataImportConfig)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_data_import_config_genesis_init)
-  [Function `data_import_mode`](#0x4_data_import_config_data_import_mode)
-  [Function `data_import_mode_none`](#0x4_data_import_config_data_import_mode_none)
-  [Function `data_import_mode_utxo`](#0x4_data_import_config_data_import_mode_utxo)
-  [Function `data_import_mode_ord`](#0x4_data_import_config_data_import_mode_ord)
-  [Function `data_import_mode_full`](#0x4_data_import_config_data_import_mode_full)
-  [Function `is_data_import_mode`](#0x4_data_import_config_is_data_import_mode)
-  [Function `is_ord_mode`](#0x4_data_import_config_is_ord_mode)


<pre><code><b>use</b> <a href="">0x2::object</a>;
</code></pre>



<a name="0x4_data_import_config_DataImportConfig"></a>

## Resource `DataImportConfig`

Bitcoin data import mode onchain configuration.


<pre><code><b>struct</b> <a href="data_import_config.md#0x4_data_import_config_DataImportConfig">DataImportConfig</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_data_import_config_DATA_IMPORT_MODE_FULL"></a>

Bitcoin's full data import mode.
All mode will process full data and indexer


<pre><code><b>const</b> <a href="data_import_config.md#0x4_data_import_config_DATA_IMPORT_MODE_FULL">DATA_IMPORT_MODE_FULL</a>: u8 = 10;
</code></pre>



<a name="0x4_data_import_config_DATA_IMPORT_MODE_NONE"></a>

Currently, Move does not support enum types, so we use constants to represent the data import mode type.
Bitcoin's none data import mode.


<pre><code><b>const</b> <a href="data_import_config.md#0x4_data_import_config_DATA_IMPORT_MODE_NONE">DATA_IMPORT_MODE_NONE</a>: u8 = 0;
</code></pre>



<a name="0x4_data_import_config_DATA_IMPORT_MODE_ORD"></a>

Bitcoin's ord data import mode.


<pre><code><b>const</b> <a href="data_import_config.md#0x4_data_import_config_DATA_IMPORT_MODE_ORD">DATA_IMPORT_MODE_ORD</a>: u8 = 2;
</code></pre>



<a name="0x4_data_import_config_DATA_IMPORT_MODE_UTXO"></a>

Bitcoin's utxo data import mode.


<pre><code><b>const</b> <a href="data_import_config.md#0x4_data_import_config_DATA_IMPORT_MODE_UTXO">DATA_IMPORT_MODE_UTXO</a>: u8 = 1;
</code></pre>



<a name="0x4_data_import_config_ErrorUnknownDataImportMode"></a>



<pre><code><b>const</b> <a href="data_import_config.md#0x4_data_import_config_ErrorUnknownDataImportMode">ErrorUnknownDataImportMode</a>: u64 = 1;
</code></pre>



<a name="0x4_data_import_config_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="data_import_config.md#0x4_data_import_config_genesis_init">genesis_init</a>(data_import_mode: u8)
</code></pre>



<a name="0x4_data_import_config_data_import_mode"></a>

## Function `data_import_mode`

Get the current data import mode from the onchain configuration.


<pre><code><b>public</b> <b>fun</b> <a href="data_import_config.md#0x4_data_import_config_data_import_mode">data_import_mode</a>(): u8
</code></pre>



<a name="0x4_data_import_config_data_import_mode_none"></a>

## Function `data_import_mode_none`



<pre><code><b>public</b> <b>fun</b> <a href="data_import_config.md#0x4_data_import_config_data_import_mode_none">data_import_mode_none</a>(): u8
</code></pre>



<a name="0x4_data_import_config_data_import_mode_utxo"></a>

## Function `data_import_mode_utxo`



<pre><code><b>public</b> <b>fun</b> <a href="data_import_config.md#0x4_data_import_config_data_import_mode_utxo">data_import_mode_utxo</a>(): u8
</code></pre>



<a name="0x4_data_import_config_data_import_mode_ord"></a>

## Function `data_import_mode_ord`



<pre><code><b>public</b> <b>fun</b> <a href="data_import_config.md#0x4_data_import_config_data_import_mode_ord">data_import_mode_ord</a>(): u8
</code></pre>



<a name="0x4_data_import_config_data_import_mode_full"></a>

## Function `data_import_mode_full`



<pre><code><b>public</b> <b>fun</b> <a href="data_import_config.md#0x4_data_import_config_data_import_mode_full">data_import_mode_full</a>(): u8
</code></pre>



<a name="0x4_data_import_config_is_data_import_mode"></a>

## Function `is_data_import_mode`



<pre><code><b>public</b> <b>fun</b> <a href="data_import_config.md#0x4_data_import_config_is_data_import_mode">is_data_import_mode</a>(data_import_mode: u8): bool
</code></pre>



<a name="0x4_data_import_config_is_ord_mode"></a>

## Function `is_ord_mode`



<pre><code><b>public</b> <b>fun</b> <a href="data_import_config.md#0x4_data_import_config_is_ord_mode">is_ord_mode</a>(data_import_mode: u8): bool
</code></pre>
