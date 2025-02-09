
<a name="0x3_indexer"></a>

# Module `0x3::indexer`



-  [Resource `FieldIndexerTablePlaceholder`](#0x3_indexer_FieldIndexerTablePlaceholder)
-  [Struct `FieldIndexerData`](#0x3_indexer_FieldIndexerData)
-  [Struct `AddFieldIndexerEvent`](#0x3_indexer_AddFieldIndexerEvent)
-  [Function `add_field_indexer_entry`](#0x3_indexer_add_field_indexer_entry)
-  [Function `add_field_indexer`](#0x3_indexer_add_field_indexer)


<pre><code><b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="core_addresses.md#0x3_core_addresses">0x3::core_addresses</a>;
<b>use</b> <a href="onchain_config.md#0x3_onchain_config">0x3::onchain_config</a>;
</code></pre>



<a name="0x3_indexer_FieldIndexerTablePlaceholder"></a>

## Resource `FieldIndexerTablePlaceholder`



<pre><code><b>struct</b> <a href="indexer.md#0x3_indexer_FieldIndexerTablePlaceholder">FieldIndexerTablePlaceholder</a> <b>has</b> key
</code></pre>



<a name="0x3_indexer_FieldIndexerData"></a>

## Struct `FieldIndexerData`



<pre><code><b>struct</b> <a href="indexer.md#0x3_indexer_FieldIndexerData">FieldIndexerData</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_indexer_AddFieldIndexerEvent"></a>

## Struct `AddFieldIndexerEvent`



<pre><code><b>struct</b> <a href="indexer.md#0x3_indexer_AddFieldIndexerEvent">AddFieldIndexerEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x3_indexer_add_field_indexer_entry"></a>

## Function `add_field_indexer_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="indexer.md#0x3_indexer_add_field_indexer_entry">add_field_indexer_entry</a>(<a href="">account</a>: &<a href="">signer</a>, id: <a href="_ObjectID">object::ObjectID</a>, path: <a href="_String">string::String</a>, ext: <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_indexer_add_field_indexer"></a>

## Function `add_field_indexer`



<pre><code><b>public</b> <b>fun</b> <a href="indexer.md#0x3_indexer_add_field_indexer">add_field_indexer</a>(<a href="">account</a>: &<a href="">signer</a>, id: <a href="_ObjectID">object::ObjectID</a>, path: <a href="_String">string::String</a>, ext: <a href="_String">string::String</a>)
</code></pre>
