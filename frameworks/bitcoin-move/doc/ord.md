
<a name="0x4_ord"></a>

# Module `0x4::ord`



-  [Struct `InscriptionID`](#0x4_ord_InscriptionID)
-  [Struct `Flotsam`](#0x4_ord_Flotsam)
-  [Struct `SatPoint`](#0x4_ord_SatPoint)
-  [Resource `Inscription`](#0x4_ord_Inscription)
-  [Struct `InscriptionRecord`](#0x4_ord_InscriptionRecord)
-  [Struct `InvalidInscriptionEvent`](#0x4_ord_InvalidInscriptionEvent)
-  [Resource `InscriptionStore`](#0x4_ord_InscriptionStore)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_ord_genesis_init)
-  [Function `new_inscription_id`](#0x4_ord_new_inscription_id)
-  [Function `derive_inscription_id`](#0x4_ord_derive_inscription_id)
-  [Function `get_inscription_id_by_index`](#0x4_ord_get_inscription_id_by_index)
-  [Function `exists_inscription`](#0x4_ord_exists_inscription)
-  [Function `borrow_inscription`](#0x4_ord_borrow_inscription)
-  [Function `spend_utxo`](#0x4_ord_spend_utxo)
-  [Function `handle_coinbase_tx`](#0x4_ord_handle_coinbase_tx)
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
-  [Function `new_sat_point`](#0x4_ord_new_sat_point)
-  [Function `unpack_sat_point`](#0x4_ord_unpack_sat_point)
-  [Function `sat_point_object_id`](#0x4_ord_sat_point_object_id)
-  [Function `sat_point_offset`](#0x4_ord_sat_point_offset)
-  [Function `sat_point_output_index`](#0x4_ord_sat_point_output_index)
-  [Function `new_flotsam`](#0x4_ord_new_flotsam)
-  [Function `unpack_flotsam`](#0x4_ord_unpack_flotsam)
-  [Function `unpack_record`](#0x4_ord_unpack_record)
-  [Function `from_transaction`](#0x4_ord_from_transaction)
-  [Function `from_transaction_bytes`](#0x4_ord_from_transaction_bytes)
-  [Function `subsidy_by_height`](#0x4_ord_subsidy_by_height)
-  [Function `bind_multichain_address`](#0x4_ord_bind_multichain_address)
-  [Function `add_permanent_state`](#0x4_ord_add_permanent_state)
-  [Function `contains_permanent_state`](#0x4_ord_contains_permanent_state)
-  [Function `borrow_permanent_state`](#0x4_ord_borrow_permanent_state)
-  [Function `borrow_mut_permanent_state`](#0x4_ord_borrow_mut_permanent_state)
-  [Function `remove_permanent_state`](#0x4_ord_remove_permanent_state)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bag</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::json</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::table_vec</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="">0x3::address_mapping</a>;
<b>use</b> <a href="">0x3::bitcoin_address</a>;
<b>use</b> <a href="">0x3::multichain_address</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
<b>use</b> <a href="utxo.md#0x4_utxo">0x4::utxo</a>;
</code></pre>



<a name="0x4_ord_InscriptionID"></a>

## Struct `InscriptionID`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionID">InscriptionID</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_Flotsam"></a>

## Struct `Flotsam`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_Flotsam">Flotsam</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_SatPoint"></a>

## Struct `SatPoint`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_SatPoint">SatPoint</a> <b>has</b> <b>copy</b>, drop, store
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



<a name="@Constants_0"></a>

## Constants


<a name="0x4_ord_COIN_VALUE"></a>

How many satoshis are in "one bitcoin".


<pre><code><b>const</b> <a href="ord.md#0x4_ord_COIN_VALUE">COIN_VALUE</a>: u64 = 100000000;
</code></pre>



<a name="0x4_ord_FIRST_POST_SUBSIDY_EPOCH"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_FIRST_POST_SUBSIDY_EPOCH">FIRST_POST_SUBSIDY_EPOCH</a>: u32 = 33;
</code></pre>



<a name="0x4_ord_PERMANENT_AREA"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_PERMANENT_AREA">PERMANENT_AREA</a>: <a href="">vector</a>&lt;u8&gt; = [112, 101, 114, 109, 97, 110, 101, 110, 116, 95, 97, 114, 101, 97];
</code></pre>



<a name="0x4_ord_SUBSIDY_HALVING_INTERVAL"></a>

How may blocks between halvings.


<pre><code><b>const</b> <a href="ord.md#0x4_ord_SUBSIDY_HALVING_INTERVAL">SUBSIDY_HALVING_INTERVAL</a>: u32 = 210000;
</code></pre>



<a name="0x4_ord_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_genesis_init">genesis_init</a>(_genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x4_ord_new_inscription_id"></a>

## Function `new_inscription_id`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_new_inscription_id">new_inscription_id</a>(txid: <b>address</b>, index: u32): <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>
</code></pre>



<a name="0x4_ord_derive_inscription_id"></a>

## Function `derive_inscription_id`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_derive_inscription_id">derive_inscription_id</a>(inscription_id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x4_ord_get_inscription_id_by_index"></a>

## Function `get_inscription_id_by_index`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_get_inscription_id_by_index">get_inscription_id_by_index</a>(index: u64): &<a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>
</code></pre>



<a name="0x4_ord_exists_inscription"></a>

## Function `exists_inscription`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_exists_inscription">exists_inscription</a>(txid: <b>address</b>, index: u32): bool
</code></pre>



<a name="0x4_ord_borrow_inscription"></a>

## Function `borrow_inscription`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_borrow_inscription">borrow_inscription</a>(txid: <b>address</b>, index: u32): &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x4_ord_spend_utxo"></a>

## Function `spend_utxo`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_spend_utxo">spend_utxo</a>(utxo_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;, tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>, input_utxo_values: <a href="">vector</a>&lt;u64&gt;, input_index: u64): (<a href="">vector</a>&lt;<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>&gt;, <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_Flotsam">ord::Flotsam</a>&gt;)
</code></pre>



<a name="0x4_ord_handle_coinbase_tx"></a>

## Function `handle_coinbase_tx`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_handle_coinbase_tx">handle_coinbase_tx</a>(tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>, flotsams: <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_Flotsam">ord::Flotsam</a>&gt;, block_height: u64): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>&gt;
</code></pre>



<a name="0x4_ord_process_transaction"></a>

## Function `process_transaction`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_process_transaction">process_transaction</a>(tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>, input_utxo_values: <a href="">vector</a>&lt;u64&gt;): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>&gt;
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



<a name="0x4_ord_new_sat_point"></a>

## Function `new_sat_point`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_new_sat_point">new_sat_point</a>(output_index: u32, offset: u64, object_id: <a href="_ObjectID">object::ObjectID</a>): <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>
</code></pre>



<a name="0x4_ord_unpack_sat_point"></a>

## Function `unpack_sat_point`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_unpack_sat_point">unpack_sat_point</a>(sat_point: <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>): (u32, u64, <a href="_ObjectID">object::ObjectID</a>)
</code></pre>



<a name="0x4_ord_sat_point_object_id"></a>

## Function `sat_point_object_id`

Get the SatPoint's object_id


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_sat_point_object_id">sat_point_object_id</a>(sat_point: &<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x4_ord_sat_point_offset"></a>

## Function `sat_point_offset`

Get the SatPoint's offset


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_sat_point_offset">sat_point_offset</a>(sat_point: &<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>): u64
</code></pre>



<a name="0x4_ord_sat_point_output_index"></a>

## Function `sat_point_output_index`

Get the SatPoint's output_index


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_sat_point_output_index">sat_point_output_index</a>(sat_point: &<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>): u32
</code></pre>



<a name="0x4_ord_new_flotsam"></a>

## Function `new_flotsam`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_new_flotsam">new_flotsam</a>(output_index: u32, offset: u64, object_id: <a href="_ObjectID">object::ObjectID</a>): <a href="ord.md#0x4_ord_Flotsam">ord::Flotsam</a>
</code></pre>



<a name="0x4_ord_unpack_flotsam"></a>

## Function `unpack_flotsam`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_unpack_flotsam">unpack_flotsam</a>(flotsam: <a href="ord.md#0x4_ord_Flotsam">ord::Flotsam</a>): (u32, u64, <a href="_ObjectID">object::ObjectID</a>)
</code></pre>



<a name="0x4_ord_unpack_record"></a>

## Function `unpack_record`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_unpack_record">unpack_record</a>(record: <a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): (<a href="">vector</a>&lt;u8&gt;, <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, <a href="">vector</a>&lt;u8&gt;, <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, <a href="_Option">option::Option</a>&lt;<a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>&gt;, <a href="_Option">option::Option</a>&lt;u64&gt;)
</code></pre>



<a name="0x4_ord_from_transaction"></a>

## Function `from_transaction`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_from_transaction">from_transaction</a>(tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>, input_utxo_values: <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u64&gt;&gt;): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x4_ord_from_transaction_bytes"></a>

## Function `from_transaction_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_from_transaction_bytes">from_transaction_bytes</a>(transaction_bytes: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x4_ord_subsidy_by_height"></a>

## Function `subsidy_by_height`

Block Rewards


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_subsidy_by_height">subsidy_by_height</a>(height: u64): u64
</code></pre>



<a name="0x4_ord_bind_multichain_address"></a>

## Function `bind_multichain_address`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_bind_multichain_address">bind_multichain_address</a>(rooch_address: <b>address</b>, bitcoin_address_opt: <a href="_Option">option::Option</a>&lt;<a href="_BitcoinAddress">bitcoin_address::BitcoinAddress</a>&gt;)
</code></pre>



<a name="0x4_ord_add_permanent_state"></a>

## Function `add_permanent_state`



<pre><code>#[private_generics(#[S])]
<b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_add_permanent_state">add_permanent_state</a>&lt;S: store&gt;(inscription: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;, state: S)
</code></pre>



<a name="0x4_ord_contains_permanent_state"></a>

## Function `contains_permanent_state`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_contains_permanent_state">contains_permanent_state</a>&lt;S: store&gt;(inscription: &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;): bool
</code></pre>



<a name="0x4_ord_borrow_permanent_state"></a>

## Function `borrow_permanent_state`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_borrow_permanent_state">borrow_permanent_state</a>&lt;S: store&gt;(inscription: &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;): &S
</code></pre>



<a name="0x4_ord_borrow_mut_permanent_state"></a>

## Function `borrow_mut_permanent_state`



<pre><code>#[private_generics(#[S])]
<b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_borrow_mut_permanent_state">borrow_mut_permanent_state</a>&lt;S: store&gt;(inscription: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;): &<b>mut</b> S
</code></pre>



<a name="0x4_ord_remove_permanent_state"></a>

## Function `remove_permanent_state`



<pre><code>#[private_generics(#[S])]
<b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_remove_permanent_state">remove_permanent_state</a>&lt;S: store&gt;(inscription: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;): S
</code></pre>
