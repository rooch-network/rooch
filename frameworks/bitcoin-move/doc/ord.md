
<a name="0x4_ord"></a>

# Module `0x4::ord`



-  [Struct `InscriptionId`](#0x4_ord_InscriptionId)
-  [Resource `Inscription`](#0x4_ord_Inscription)
-  [Struct `InscriptionRecord`](#0x4_ord_InscriptionRecord)
-  [Struct `InvalidInscriptionEvent`](#0x4_ord_InvalidInscriptionEvent)
-  [Function `spend_utxo`](#0x4_ord_spend_utxo)
-  [Function `progress_transaction`](#0x4_ord_progress_transaction)
-  [Function `txid`](#0x4_ord_txid)
-  [Function `index`](#0x4_ord_index)
-  [Function `body`](#0x4_ord_body)
-  [Function `content_encoding`](#0x4_ord_content_encoding)
-  [Function `content_type`](#0x4_ord_content_type)
-  [Function `metadata`](#0x4_ord_metadata)
-  [Function `metaprotocol`](#0x4_ord_metaprotocol)
-  [Function `parent`](#0x4_ord_parent)
-  [Function `pointer`](#0x4_ord_pointer)
-  [Function `unpack_record`](#0x4_ord_unpack_record)
-  [Function `from_transaction`](#0x4_ord_from_transaction)
-  [Function `from_transaction_bytes`](#0x4_ord_from_transaction_bytes)
-  [Function `bind_multichain_address`](#0x4_ord_bind_multichain_address)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x3::address_mapping</a>;
<b>use</b> <a href="">0x3::bitcoin_address</a>;
<b>use</b> <a href="">0x3::multichain_address</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
<b>use</b> <a href="utxo.md#0x4_utxo">0x4::utxo</a>;
</code></pre>



<a name="0x4_ord_InscriptionId"></a>

## Struct `InscriptionId`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionId">InscriptionId</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_Inscription"></a>

## Resource `Inscription`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_Inscription">Inscription</a> <b>has</b> key
</code></pre>



<a name="0x4_ord_InscriptionRecord"></a>

## Struct `InscriptionRecord`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionRecord">InscriptionRecord</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_InvalidInscriptionEvent"></a>

## Struct `InvalidInscriptionEvent`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InvalidInscriptionEvent">InvalidInscriptionEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_spend_utxo"></a>

## Function `spend_utxo`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_spend_utxo">spend_utxo</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, utxo_obj: &<a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;, tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>): <a href="">vector</a>&lt;<a href="utxo.md#0x4_utxo_SealOut">utxo::SealOut</a>&gt;
</code></pre>



<a name="0x4_ord_progress_transaction"></a>

## Function `progress_transaction`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_progress_transaction">progress_transaction</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>): <a href="">vector</a>&lt;<a href="utxo.md#0x4_utxo_SealOut">utxo::SealOut</a>&gt;
</code></pre>



<a name="0x4_ord_txid"></a>

## Function `txid`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_txid">txid</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <b>address</b>
</code></pre>



<a name="0x4_ord_index"></a>

## Function `index`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_index">index</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): u32
</code></pre>



<a name="0x4_ord_body"></a>

## Function `body`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_body">body</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x4_ord_content_encoding"></a>

## Function `content_encoding`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_content_encoding">content_encoding</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x4_ord_content_type"></a>

## Function `content_type`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_content_type">content_type</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x4_ord_metadata"></a>

## Function `metadata`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_metadata">metadata</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x4_ord_metaprotocol"></a>

## Function `metaprotocol`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_metaprotocol">metaprotocol</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x4_ord_parent"></a>

## Function `parent`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_parent">parent</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<a name="0x4_ord_pointer"></a>

## Function `pointer`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_pointer">pointer</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x4_ord_unpack_record"></a>

## Function `unpack_record`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_unpack_record">unpack_record</a>(record: <a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): (<a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;)
</code></pre>



<a name="0x4_ord_from_transaction"></a>

## Function `from_transaction`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_from_transaction">from_transaction</a>(tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>&gt;
</code></pre>



<a name="0x4_ord_from_transaction_bytes"></a>

## Function `from_transaction_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_from_transaction_bytes">from_transaction_bytes</a>(transaction_bytes: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>&gt;
</code></pre>



<a name="0x4_ord_bind_multichain_address"></a>

## Function `bind_multichain_address`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_bind_multichain_address">bind_multichain_address</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, rooch_address: <b>address</b>, bitcoin_address_opt: <a href="_Option">option::Option</a>&lt;<a href="_BitcoinAddress">bitcoin_address::BitcoinAddress</a>&gt;)
</code></pre>
