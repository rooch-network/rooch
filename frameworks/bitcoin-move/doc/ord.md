
<a name="0x4_ord"></a>

# Module `0x4::ord`



-  [Struct `InscriptionID`](#0x4_ord_InscriptionID)
-  [Resource `Inscription`](#0x4_ord_Inscription)
-  [Struct `InscriptionRecord`](#0x4_ord_InscriptionRecord)
-  [Struct `InvalidInscriptionEvent`](#0x4_ord_InvalidInscriptionEvent)
-  [Resource `InscriptionStore`](#0x4_ord_InscriptionStore)
-  [Struct `SatPoint`](#0x4_ord_SatPoint)
-  [Struct `SatPointRange`](#0x4_ord_SatPointRange)
-  [Struct `SatPointMapping`](#0x4_ord_SatPointMapping)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_ord_genesis_init)
-  [Function `new_inscription_id`](#0x4_ord_new_inscription_id)
-  [Function `new_satpoint`](#0x4_ord_new_satpoint)
-  [Function `satpoint_offset`](#0x4_ord_satpoint_offset)
-  [Function `satpoint_txid`](#0x4_ord_satpoint_txid)
-  [Function `new_satpoint_mapping`](#0x4_ord_new_satpoint_mapping)
-  [Function `unpack_satpoint_mapping`](#0x4_ord_unpack_satpoint_mapping)
-  [Function `exists_inscription`](#0x4_ord_exists_inscription)
-  [Function `borrow_inscription`](#0x4_ord_borrow_inscription)
-  [Function `update_inscription_index`](#0x4_ord_update_inscription_index)
-  [Function `remove_inscription_index`](#0x4_ord_remove_inscription_index)
-  [Function `inscriptions_on_output`](#0x4_ord_inscriptions_on_output)
-  [Function `process_transaction`](#0x4_ord_process_transaction)
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
-  [Function `pack_inscribe_generate_args`](#0x4_ord_pack_inscribe_generate_args)
-  [Function `bind_multichain_address`](#0x4_ord_bind_multichain_address)


<pre><code><b>use</b> <a href="">0x1::debug</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::big_vector</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::json</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::table_vec</a>;
<b>use</b> <a href="">0x3::address_mapping</a>;
<b>use</b> <a href="">0x3::bitcoin_address</a>;
<b>use</b> <a href="">0x3::multichain_address</a>;
<b>use</b> <a href="brc20.md#0x4_brc20">0x4::brc20</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
<b>use</b> <a href="utxo.md#0x4_utxo">0x4::utxo</a>;
</code></pre>



<a name="0x4_ord_InscriptionID"></a>

## Struct `InscriptionID`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionID">InscriptionID</a> <b>has</b> <b>copy</b>, drop, store
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



<a name="0x4_ord_InscriptionStore"></a>

## Resource `InscriptionStore`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionStore">InscriptionStore</a> <b>has</b> key
</code></pre>



<a name="0x4_ord_SatPoint"></a>

## Struct `SatPoint`



<pre><code>#[data_struct]
<b>struct</b> <a href="ord.md#0x4_ord_SatPoint">SatPoint</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_SatPointRange"></a>

## Struct `SatPointRange`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_SatPointRange">SatPointRange</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_SatPointMapping"></a>

## Struct `SatPointMapping`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_SatPointMapping">SatPointMapping</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_ord_OUTPOINT_TO_SATPOINT_BUCKET_SIZE"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_OUTPOINT_TO_SATPOINT_BUCKET_SIZE">OUTPOINT_TO_SATPOINT_BUCKET_SIZE</a>: u64 = 1000;
</code></pre>



<a name="0x4_ord_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_genesis_init">genesis_init</a>(_genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x4_ord_new_inscription_id"></a>

## Function `new_inscription_id`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_new_inscription_id">new_inscription_id</a>(txid: <b>address</b>, index: u32): <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>
</code></pre>



<a name="0x4_ord_new_satpoint"></a>

## Function `new_satpoint`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_new_satpoint">new_satpoint</a>(txid: <b>address</b>, vout: u32, offset: u64): <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>
</code></pre>



<a name="0x4_ord_satpoint_offset"></a>

## Function `satpoint_offset`

Get the SatPoint's offset


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_satpoint_offset">satpoint_offset</a>(satpoint: &<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>): u64
</code></pre>



<a name="0x4_ord_satpoint_txid"></a>

## Function `satpoint_txid`

Get the SatPoint's offset


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_satpoint_txid">satpoint_txid</a>(satpoint: &<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>): <b>address</b>
</code></pre>



<a name="0x4_ord_new_satpoint_mapping"></a>

## Function `new_satpoint_mapping`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_new_satpoint_mapping">new_satpoint_mapping</a>(old_satpoint: <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>, new_satpoint: <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>): <a href="ord.md#0x4_ord_SatPointMapping">ord::SatPointMapping</a>
</code></pre>



<a name="0x4_ord_unpack_satpoint_mapping"></a>

## Function `unpack_satpoint_mapping`

Get the SatPoint's mapping


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_unpack_satpoint_mapping">unpack_satpoint_mapping</a>(satpoint_mapping: &<a href="ord.md#0x4_ord_SatPointMapping">ord::SatPointMapping</a>): (<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>, <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>)
</code></pre>



<a name="0x4_ord_exists_inscription"></a>

## Function `exists_inscription`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_exists_inscription">exists_inscription</a>(txid: <b>address</b>, index: u32): bool
</code></pre>



<a name="0x4_ord_borrow_inscription"></a>

## Function `borrow_inscription`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_borrow_inscription">borrow_inscription</a>(txid: <b>address</b>, index: u32): &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x4_ord_update_inscription_index"></a>

## Function `update_inscription_index`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_update_inscription_index">update_inscription_index</a>(txout: &<a href="types.md#0x4_types_TxOut">types::TxOut</a>, outpoint: <a href="types.md#0x4_types_OutPoint">types::OutPoint</a>, old_satpoint: <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>, new_satpoint: <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>, _tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>): <a href="utxo.md#0x4_utxo_SealOut">utxo::SealOut</a>
</code></pre>



<a name="0x4_ord_remove_inscription_index"></a>

## Function `remove_inscription_index`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_remove_inscription_index">remove_inscription_index</a>(outpoint: <a href="types.md#0x4_types_OutPoint">types::OutPoint</a>)
</code></pre>



<a name="0x4_ord_inscriptions_on_output"></a>

## Function `inscriptions_on_output`

Find existing inscriptions on input (transfers of inscriptions)


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_inscriptions_on_output">inscriptions_on_output</a>(outpoint: &<a href="types.md#0x4_types_OutPoint">types::OutPoint</a>): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>&gt;
</code></pre>



<a name="0x4_ord_process_transaction"></a>

## Function `process_transaction`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_process_transaction">process_transaction</a>(tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>): <a href="">vector</a>&lt;<a href="utxo.md#0x4_utxo_SealOut">utxo::SealOut</a>&gt;
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



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_body">body</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_content_encoding"></a>

## Function `content_encoding`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_content_encoding">content_encoding</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x4_ord_content_type"></a>

## Function `content_type`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_content_type">content_type</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x4_ord_metadata"></a>

## Function `metadata`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_metadata">metadata</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_metaprotocol"></a>

## Function `metaprotocol`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_metaprotocol">metaprotocol</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x4_ord_parent"></a>

## Function `parent`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_parent">parent</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<a name="0x4_ord_pointer"></a>

## Function `pointer`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_pointer">pointer</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<a name="0x4_ord_unpack_record"></a>

## Function `unpack_record`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_unpack_record">unpack_record</a>(record: <a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): (<a href="">vector</a>&lt;u8&gt;, <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, <a href="">vector</a>&lt;u8&gt;, <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, <a href="_Option">option::Option</a>&lt;<a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>&gt;, <a href="_Option">option::Option</a>&lt;u64&gt;)
</code></pre>



<a name="0x4_ord_from_transaction"></a>

## Function `from_transaction`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_from_transaction">from_transaction</a>(tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x4_ord_from_transaction_bytes"></a>

## Function `from_transaction_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_from_transaction_bytes">from_transaction_bytes</a>(transaction_bytes: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x4_ord_pack_inscribe_generate_args"></a>

## Function `pack_inscribe_generate_args`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_pack_inscribe_generate_args">pack_inscribe_generate_args</a>(deploy_args: <a href="">vector</a>&lt;u8&gt;, seed: <a href="">vector</a>&lt;u8&gt;, user_input: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_bind_multichain_address"></a>

## Function `bind_multichain_address`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_bind_multichain_address">bind_multichain_address</a>(rooch_address: <b>address</b>, bitcoin_address_opt: <a href="_Option">option::Option</a>&lt;<a href="_BitcoinAddress">bitcoin_address::BitcoinAddress</a>&gt;)
</code></pre>
