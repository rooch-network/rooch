
<a name="0x3_ord"></a>

# Module `0x3::ord`



-  [Struct `InscriptionId`](#0x3_ord_InscriptionId)
-  [Struct `Inscription`](#0x3_ord_Inscription)
-  [Resource `InscriptionStore`](#0x3_ord_InscriptionStore)
-  [Function `genesis_init`](#0x3_ord_genesis_init)
-  [Function `from_transaction`](#0x3_ord_from_transaction)
-  [Function `from_transaction_bytes`](#0x3_ord_from_transaction_bytes)
-  [Function `body`](#0x3_ord_body)
-  [Function `content_encoding`](#0x3_ord_content_encoding)
-  [Function `content_type`](#0x3_ord_content_type)
-  [Function `duplicate_field`](#0x3_ord_duplicate_field)
-  [Function `incomplete_field`](#0x3_ord_incomplete_field)
-  [Function `metadata`](#0x3_ord_metadata)
-  [Function `metaprotocol`](#0x3_ord_metaprotocol)
-  [Function `parent`](#0x3_ord_parent)
-  [Function `pointer`](#0x3_ord_pointer)
-  [Function `unrecognized_even_field`](#0x3_ord_unrecognized_even_field)
-  [Function `total_inscriptions`](#0x3_ord_total_inscriptions)
-  [Function `inscription_ids`](#0x3_ord_inscription_ids)
-  [Function `inscriptions`](#0x3_ord_inscriptions)
-  [Function `remaining_tx_count`](#0x3_ord_remaining_tx_count)
-  [Function `progress_inscriptions`](#0x3_ord_progress_inscriptions)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::table_vec</a>;
<b>use</b> <a href="bitcoin_light_client.md#0x3_bitcoin_light_client">0x3::bitcoin_light_client</a>;
<b>use</b> <a href="bitcoin_types.md#0x3_bitcoin_types">0x3::bitcoin_types</a>;
</code></pre>



<a name="0x3_ord_InscriptionId"></a>

## Struct `InscriptionId`



<pre><code><b>struct</b> <a href="ord.md#0x3_ord_InscriptionId">InscriptionId</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_ord_Inscription"></a>

## Struct `Inscription`



<pre><code><b>struct</b> <a href="ord.md#0x3_ord_Inscription">Inscription</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_ord_InscriptionStore"></a>

## Resource `InscriptionStore`



<pre><code><b>struct</b> <a href="ord.md#0x3_ord_InscriptionStore">InscriptionStore</a> <b>has</b> key
</code></pre>



<a name="0x3_ord_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x3_ord_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, _genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x3_ord_from_transaction"></a>

## Function `from_transaction`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_from_transaction">from_transaction</a>(transaction: &<a href="bitcoin_types.md#0x3_bitcoin_types_Transaction">bitcoin_types::Transaction</a>): <a href="">vector</a>&lt;<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x3_ord_from_transaction_bytes"></a>

## Function `from_transaction_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_from_transaction_bytes">from_transaction_bytes</a>(transaction_bytes: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x3_ord_body"></a>

## Function `body`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_body">body</a>(self: &<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_ord_content_encoding"></a>

## Function `content_encoding`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_content_encoding">content_encoding</a>(self: &<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_ord_content_type"></a>

## Function `content_type`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_content_type">content_type</a>(self: &<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x3_ord_duplicate_field"></a>

## Function `duplicate_field`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_duplicate_field">duplicate_field</a>(self: &<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>): bool
</code></pre>



<a name="0x3_ord_incomplete_field"></a>

## Function `incomplete_field`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_incomplete_field">incomplete_field</a>(self: &<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>): bool
</code></pre>



<a name="0x3_ord_metadata"></a>

## Function `metadata`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_metadata">metadata</a>(self: &<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_ord_metaprotocol"></a>

## Function `metaprotocol`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_metaprotocol">metaprotocol</a>(self: &<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_ord_parent"></a>

## Function `parent`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_parent">parent</a>(self: &<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_ord_pointer"></a>

## Function `pointer`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_pointer">pointer</a>(self: &<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_ord_unrecognized_even_field"></a>

## Function `unrecognized_even_field`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_unrecognized_even_field">unrecognized_even_field</a>(self: &<a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>): bool
</code></pre>



<a name="0x3_ord_total_inscriptions"></a>

## Function `total_inscriptions`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_total_inscriptions">total_inscriptions</a>(inscription_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x3_ord_InscriptionStore">ord::InscriptionStore</a>&gt;): u64
</code></pre>



<a name="0x3_ord_inscription_ids"></a>

## Function `inscription_ids`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_inscription_ids">inscription_ids</a>(inscription_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x3_ord_InscriptionStore">ord::InscriptionStore</a>&gt;): &<a href="_TableVec">table_vec::TableVec</a>&lt;<a href="ord.md#0x3_ord_InscriptionId">ord::InscriptionId</a>&gt;
</code></pre>



<a name="0x3_ord_inscriptions"></a>

## Function `inscriptions`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_inscriptions">inscriptions</a>(inscription_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x3_ord_InscriptionStore">ord::InscriptionStore</a>&gt;): &<a href="_Table">table::Table</a>&lt;<a href="ord.md#0x3_ord_InscriptionId">ord::InscriptionId</a>, <a href="ord.md#0x3_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x3_ord_remaining_tx_count"></a>

## Function `remaining_tx_count`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x3_ord_remaining_tx_count">remaining_tx_count</a>(btc_block_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitcoin_light_client.md#0x3_bitcoin_light_client_BitcoinBlockStore">bitcoin_light_client::BitcoinBlockStore</a>&gt;, inscription_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x3_ord_InscriptionStore">ord::InscriptionStore</a>&gt;): u64
</code></pre>



<a name="0x3_ord_progress_inscriptions"></a>

## Function `progress_inscriptions`



<pre><code>entry <b>fun</b> <a href="ord.md#0x3_ord_progress_inscriptions">progress_inscriptions</a>(btc_block_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitcoin_light_client.md#0x3_bitcoin_light_client_BitcoinBlockStore">bitcoin_light_client::BitcoinBlockStore</a>&gt;, inscription_store_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x3_ord_InscriptionStore">ord::InscriptionStore</a>&gt;, batch_size: u64)
</code></pre>
