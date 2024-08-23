
<a name="0x4_ord"></a>

# Module `0x4::ord`



-  [Struct `InscriptionID`](#0x4_ord_InscriptionID)
-  [Struct `Flotsam`](#0x4_ord_Flotsam)
-  [Struct `SatPoint`](#0x4_ord_SatPoint)
-  [Resource `Inscription`](#0x4_ord_Inscription)
-  [Struct `Envelope`](#0x4_ord_Envelope)
-  [Struct `InscriptionRecord`](#0x4_ord_InscriptionRecord)
-  [Struct `InvalidInscriptionEvent`](#0x4_ord_InvalidInscriptionEvent)
-  [Resource `MetaprotocolRegistry`](#0x4_ord_MetaprotocolRegistry)
-  [Struct `MetaprotocolValidity`](#0x4_ord_MetaprotocolValidity)
-  [Resource `InscriptionStore`](#0x4_ord_InscriptionStore)
-  [Struct `InscriptionEvent`](#0x4_ord_InscriptionEvent)
-  [Struct `InscriptionCreatedEvent`](#0x4_ord_InscriptionCreatedEvent)
-  [Struct `InscriptionTransferredEvent`](#0x4_ord_InscriptionTransferredEvent)
-  [Struct `InscriptionCharm`](#0x4_ord_InscriptionCharm)
-  [Constants](#@Constants_0)
-  [Function `curse_duplicate_field`](#0x4_ord_curse_duplicate_field)
-  [Function `curse_incompleted_field`](#0x4_ord_curse_incompleted_field)
-  [Function `curse_not_at_offset_zero`](#0x4_ord_curse_not_at_offset_zero)
-  [Function `curse_not_in_first_input`](#0x4_ord_curse_not_in_first_input)
-  [Function `curse_pointer`](#0x4_ord_curse_pointer)
-  [Function `curse_pushnum`](#0x4_ord_curse_pushnum)
-  [Function `curse_reinscription`](#0x4_ord_curse_reinscription)
-  [Function `curse_stutter`](#0x4_ord_curse_stutter)
-  [Function `curse_unrecognized_even_field`](#0x4_ord_curse_unrecognized_even_field)
-  [Function `genesis_init`](#0x4_ord_genesis_init)
-  [Function `borrow_mut_inscription_store`](#0x4_ord_borrow_mut_inscription_store)
-  [Function `borrow_inscription_store`](#0x4_ord_borrow_inscription_store)
-  [Function `blessed_inscription_count`](#0x4_ord_blessed_inscription_count)
-  [Function `cursed_inscription_count`](#0x4_ord_cursed_inscription_count)
-  [Function `unbound_inscription_count`](#0x4_ord_unbound_inscription_count)
-  [Function `next_sequence_number`](#0x4_ord_next_sequence_number)
-  [Function `update_cursed_inscription_count`](#0x4_ord_update_cursed_inscription_count)
-  [Function `update_blessed_inscription_count`](#0x4_ord_update_blessed_inscription_count)
-  [Function `update_next_sequence_number`](#0x4_ord_update_next_sequence_number)
-  [Function `update_unbound_inscription_count`](#0x4_ord_update_unbound_inscription_count)
-  [Function `new_inscription_id`](#0x4_ord_new_inscription_id)
-  [Function `derive_inscription_id`](#0x4_ord_derive_inscription_id)
-  [Function `parse_inscription_id`](#0x4_ord_parse_inscription_id)
-  [Function `inscription_id_to_string`](#0x4_ord_inscription_id_to_string)
-  [Function `get_inscription_id_by_sequence_number`](#0x4_ord_get_inscription_id_by_sequence_number)
-  [Function `get_inscription_next_sequence_number`](#0x4_ord_get_inscription_next_sequence_number)
-  [Function `create_object`](#0x4_ord_create_object)
-  [Function `transfer_object`](#0x4_ord_transfer_object)
-  [Function `exists_inscription`](#0x4_ord_exists_inscription)
-  [Function `borrow_inscription`](#0x4_ord_borrow_inscription)
-  [Function `borrow_inscription_by_id`](#0x4_ord_borrow_inscription_by_id)
-  [Function `take_object`](#0x4_ord_take_object)
-  [Function `borrow_object`](#0x4_ord_borrow_object)
-  [Function `txid`](#0x4_ord_txid)
-  [Function `index`](#0x4_ord_index)
-  [Function `location`](#0x4_ord_location)
-  [Function `sequence_number`](#0x4_ord_sequence_number)
-  [Function `inscription_number`](#0x4_ord_inscription_number)
-  [Function `is_cursed`](#0x4_ord_is_cursed)
-  [Function `charms`](#0x4_ord_charms)
-  [Function `offset`](#0x4_ord_offset)
-  [Function `body`](#0x4_ord_body)
-  [Function `content_encoding`](#0x4_ord_content_encoding)
-  [Function `content_type`](#0x4_ord_content_type)
-  [Function `metadata`](#0x4_ord_metadata)
-  [Function `metaprotocol`](#0x4_ord_metaprotocol)
-  [Function `parents`](#0x4_ord_parents)
-  [Function `pointer`](#0x4_ord_pointer)
-  [Function `inscription_id`](#0x4_ord_inscription_id)
-  [Function `inscription_id_txid`](#0x4_ord_inscription_id_txid)
-  [Function `inscription_id_index`](#0x4_ord_inscription_id_index)
-  [Function `new_satpoint`](#0x4_ord_new_satpoint)
-  [Function `unpack_satpoint`](#0x4_ord_unpack_satpoint)
-  [Function `satpoint_offset`](#0x4_ord_satpoint_offset)
-  [Function `satpoint_outpoint`](#0x4_ord_satpoint_outpoint)
-  [Function `satpoint_vout`](#0x4_ord_satpoint_vout)
-  [Function `new_flotsam`](#0x4_ord_new_flotsam)
-  [Function `unpack_flotsam`](#0x4_ord_unpack_flotsam)
-  [Function `parse_inscription_from_tx`](#0x4_ord_parse_inscription_from_tx)
-  [Function `subsidy_by_height`](#0x4_ord_subsidy_by_height)
-  [Function `add_permanent_state`](#0x4_ord_add_permanent_state)
-  [Function `contains_permanent_state`](#0x4_ord_contains_permanent_state)
-  [Function `borrow_permanent_state`](#0x4_ord_borrow_permanent_state)
-  [Function `borrow_mut_permanent_state`](#0x4_ord_borrow_mut_permanent_state)
-  [Function `remove_permanent_state`](#0x4_ord_remove_permanent_state)
-  [Function `destroy_permanent_area`](#0x4_ord_destroy_permanent_area)
-  [Function `add_temp_state`](#0x4_ord_add_temp_state)
-  [Function `contains_temp_state`](#0x4_ord_contains_temp_state)
-  [Function `borrow_temp_state`](#0x4_ord_borrow_temp_state)
-  [Function `borrow_mut_temp_state`](#0x4_ord_borrow_mut_temp_state)
-  [Function `remove_temp_state`](#0x4_ord_remove_temp_state)
-  [Function `drop_temp_area`](#0x4_ord_drop_temp_area)
-  [Function `register_metaprotocol_via_system`](#0x4_ord_register_metaprotocol_via_system)
-  [Function `is_metaprotocol_register`](#0x4_ord_is_metaprotocol_register)
-  [Function `seal_metaprotocol_validity`](#0x4_ord_seal_metaprotocol_validity)
-  [Function `add_metaprotocol_attachment`](#0x4_ord_add_metaprotocol_attachment)
-  [Function `exists_metaprotocol_validity`](#0x4_ord_exists_metaprotocol_validity)
-  [Function `borrow_metaprotocol_validity`](#0x4_ord_borrow_metaprotocol_validity)
-  [Function `metaprotocol_validity_protocol_match`](#0x4_ord_metaprotocol_validity_protocol_match)
-  [Function `metaprotocol_validity_protocol_type`](#0x4_ord_metaprotocol_validity_protocol_type)
-  [Function `metaprotocol_validity_is_valid`](#0x4_ord_metaprotocol_validity_is_valid)
-  [Function `metaprotocol_validity_invalid_reason`](#0x4_ord_metaprotocol_validity_invalid_reason)
-  [Function `view_validity`](#0x4_ord_view_validity)
-  [Function `unpack_inscription_event`](#0x4_ord_unpack_inscription_event)
-  [Function `inscription_event_type_new`](#0x4_ord_inscription_event_type_new)
-  [Function `inscription_event_type_burn`](#0x4_ord_inscription_event_type_burn)
-  [Function `inscription_charm_burned`](#0x4_ord_inscription_charm_burned)
-  [Function `exists_inscription_charm`](#0x4_ord_exists_inscription_charm)
-  [Function `borrow_inscription_charm`](#0x4_ord_borrow_inscription_charm)
-  [Function `view_inscription_charm`](#0x4_ord_view_inscription_charm)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bag</a>;
<b>use</b> <a href="">0x2::core_addresses</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::json</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::string_utils</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="bitcoin_hash.md#0x4_bitcoin_hash">0x4::bitcoin_hash</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
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



<a name="0x4_ord_Envelope"></a>

## Struct `Envelope`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_Envelope">Envelope</a>&lt;T&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_InscriptionRecord"></a>

## Struct `InscriptionRecord`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionRecord">InscriptionRecord</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_InvalidInscriptionEvent"></a>

## Struct `InvalidInscriptionEvent`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InvalidInscriptionEvent">InvalidInscriptionEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_MetaprotocolRegistry"></a>

## Resource `MetaprotocolRegistry`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_MetaprotocolRegistry">MetaprotocolRegistry</a> <b>has</b> key
</code></pre>



<a name="0x4_ord_MetaprotocolValidity"></a>

## Struct `MetaprotocolValidity`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_MetaprotocolValidity">MetaprotocolValidity</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_InscriptionStore"></a>

## Resource `InscriptionStore`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionStore">InscriptionStore</a> <b>has</b> key
</code></pre>



<a name="0x4_ord_InscriptionEvent"></a>

## Struct `InscriptionEvent`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionEvent">InscriptionEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_InscriptionCreatedEvent"></a>

## Struct `InscriptionCreatedEvent`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionCreatedEvent">InscriptionCreatedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_InscriptionTransferredEvent"></a>

## Struct `InscriptionTransferredEvent`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionTransferredEvent">InscriptionTransferredEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_InscriptionCharm"></a>

## Struct `InscriptionCharm`

Represents the charm of an inscription, containing various properties.


<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionCharm">InscriptionCharm</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_ord_TEMPORARY_AREA"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_TEMPORARY_AREA">TEMPORARY_AREA</a>: <a href="">vector</a>&lt;u8&gt; = [116, 101, 109, 112, 111, 114, 97, 114, 121, 95, 97, 114, 101, 97];
</code></pre>



<a name="0x4_ord_COIN_VALUE"></a>

How many satoshis are in "one bitcoin".


<pre><code><b>const</b> <a href="ord.md#0x4_ord_COIN_VALUE">COIN_VALUE</a>: u64 = 100000000;
</code></pre>



<a name="0x4_ord_CURSE_DUPLICATE_FIELD"></a>

Curse Inscription


<pre><code><b>const</b> <a href="ord.md#0x4_ord_CURSE_DUPLICATE_FIELD">CURSE_DUPLICATE_FIELD</a>: <a href="">vector</a>&lt;u8&gt; = [68, 117, 112, 108, 105, 99, 97, 116, 101, 70, 105, 101, 108, 100];
</code></pre>



<a name="0x4_ord_CURSE_INCOMPLETE_FIELD"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CURSE_INCOMPLETE_FIELD">CURSE_INCOMPLETE_FIELD</a>: <a href="">vector</a>&lt;u8&gt; = [73, 110, 99, 111, 109, 112, 108, 101, 116, 101, 70, 105, 101, 108, 100];
</code></pre>



<a name="0x4_ord_CURSE_NOT_AT_OFFSET_ZERO"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CURSE_NOT_AT_OFFSET_ZERO">CURSE_NOT_AT_OFFSET_ZERO</a>: <a href="">vector</a>&lt;u8&gt; = [78, 111, 116, 65, 116, 79, 102, 102, 115, 101, 116, 90, 101, 114, 111];
</code></pre>



<a name="0x4_ord_CURSE_NOT_IN_FIRST_INPUT"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CURSE_NOT_IN_FIRST_INPUT">CURSE_NOT_IN_FIRST_INPUT</a>: <a href="">vector</a>&lt;u8&gt; = [78, 111, 116, 73, 110, 70, 105, 114, 115, 116, 73, 110, 112, 117, 116];
</code></pre>



<a name="0x4_ord_CURSE_POINTER"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CURSE_POINTER">CURSE_POINTER</a>: <a href="">vector</a>&lt;u8&gt; = [80, 111, 105, 110, 116, 101, 114];
</code></pre>



<a name="0x4_ord_CURSE_PUSHNUM"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CURSE_PUSHNUM">CURSE_PUSHNUM</a>: <a href="">vector</a>&lt;u8&gt; = [80, 117, 115, 104, 110, 117, 109];
</code></pre>



<a name="0x4_ord_CURSE_REINSCRIPTION"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CURSE_REINSCRIPTION">CURSE_REINSCRIPTION</a>: <a href="">vector</a>&lt;u8&gt; = [82, 101, 105, 110, 115, 99, 114, 105, 112, 116, 105, 111, 110];
</code></pre>



<a name="0x4_ord_CURSE_STUTTER"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CURSE_STUTTER">CURSE_STUTTER</a>: <a href="">vector</a>&lt;u8&gt; = [83, 116, 117, 116, 116, 101, 114];
</code></pre>



<a name="0x4_ord_CURSE_UNRECOGNIZED_EVEN_FIELD"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CURSE_UNRECOGNIZED_EVEN_FIELD">CURSE_UNRECOGNIZED_EVEN_FIELD</a>: <a href="">vector</a>&lt;u8&gt; = [85, 110, 114, 101, 99, 111, 103, 110, 105, 122, 101, 100, 69, 118, 101, 110, 70, 105, 101, 108, 100];
</code></pre>



<a name="0x4_ord_ErrorMetaprotocolAlreadyRegistered"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_ErrorMetaprotocolAlreadyRegistered">ErrorMetaprotocolAlreadyRegistered</a>: u64 = 1;
</code></pre>



<a name="0x4_ord_ErrorMetaprotocolProtocolMismatch"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_ErrorMetaprotocolProtocolMismatch">ErrorMetaprotocolProtocolMismatch</a>: u64 = 2;
</code></pre>



<a name="0x4_ord_FIRST_POST_SUBSIDY_EPOCH"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_FIRST_POST_SUBSIDY_EPOCH">FIRST_POST_SUBSIDY_EPOCH</a>: u32 = 33;
</code></pre>



<a name="0x4_ord_INSCRIPTION_CHARM"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_INSCRIPTION_CHARM">INSCRIPTION_CHARM</a>: <a href="">vector</a>&lt;u8&gt; = [105, 110, 115, 99, 114, 105, 112, 116, 105, 111, 110, 95, 99, 104, 97, 114, 109];
</code></pre>



<a name="0x4_ord_InscriptionEventTypeBurn"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_InscriptionEventTypeBurn">InscriptionEventTypeBurn</a>: u8 = 1;
</code></pre>



<a name="0x4_ord_InscriptionEventTypeNew"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_InscriptionEventTypeNew">InscriptionEventTypeNew</a>: u8 = 0;
</code></pre>



<a name="0x4_ord_METAPROTOCOL_VALIDITY"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_METAPROTOCOL_VALIDITY">METAPROTOCOL_VALIDITY</a>: <a href="">vector</a>&lt;u8&gt; = [109, 101, 116, 97, 112, 114, 111, 116, 111, 99, 111, 108, 95, 118, 97, 108, 105, 100, 105, 116, 121];
</code></pre>



<a name="0x4_ord_PERMANENT_AREA"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_PERMANENT_AREA">PERMANENT_AREA</a>: <a href="">vector</a>&lt;u8&gt; = [112, 101, 114, 109, 97, 110, 101, 110, 116, 95, 97, 114, 101, 97];
</code></pre>



<a name="0x4_ord_SUBSIDY_HALVING_INTERVAL"></a>

How may blocks between halvings.


<pre><code><b>const</b> <a href="ord.md#0x4_ord_SUBSIDY_HALVING_INTERVAL">SUBSIDY_HALVING_INTERVAL</a>: u32 = 210000;
</code></pre>



<a name="0x4_ord_curse_duplicate_field"></a>

## Function `curse_duplicate_field`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_curse_duplicate_field">curse_duplicate_field</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_curse_incompleted_field"></a>

## Function `curse_incompleted_field`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_curse_incompleted_field">curse_incompleted_field</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_curse_not_at_offset_zero"></a>

## Function `curse_not_at_offset_zero`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_curse_not_at_offset_zero">curse_not_at_offset_zero</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_curse_not_in_first_input"></a>

## Function `curse_not_in_first_input`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_curse_not_in_first_input">curse_not_in_first_input</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_curse_pointer"></a>

## Function `curse_pointer`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_curse_pointer">curse_pointer</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_curse_pushnum"></a>

## Function `curse_pushnum`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_curse_pushnum">curse_pushnum</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_curse_reinscription"></a>

## Function `curse_reinscription`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_curse_reinscription">curse_reinscription</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_curse_stutter"></a>

## Function `curse_stutter`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_curse_stutter">curse_stutter</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_curse_unrecognized_even_field"></a>

## Function `curse_unrecognized_even_field`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_curse_unrecognized_even_field">curse_unrecognized_even_field</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_genesis_init">genesis_init</a>(_genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x4_ord_borrow_mut_inscription_store"></a>

## Function `borrow_mut_inscription_store`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_borrow_mut_inscription_store">borrow_mut_inscription_store</a>(): &<b>mut</b> <a href="ord.md#0x4_ord_InscriptionStore">ord::InscriptionStore</a>
</code></pre>



<a name="0x4_ord_borrow_inscription_store"></a>

## Function `borrow_inscription_store`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_borrow_inscription_store">borrow_inscription_store</a>(): &<a href="ord.md#0x4_ord_InscriptionStore">ord::InscriptionStore</a>
</code></pre>



<a name="0x4_ord_blessed_inscription_count"></a>

## Function `blessed_inscription_count`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_blessed_inscription_count">blessed_inscription_count</a>(inscription_store: &<a href="ord.md#0x4_ord_InscriptionStore">ord::InscriptionStore</a>): u32
</code></pre>



<a name="0x4_ord_cursed_inscription_count"></a>

## Function `cursed_inscription_count`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_cursed_inscription_count">cursed_inscription_count</a>(inscription_store: &<a href="ord.md#0x4_ord_InscriptionStore">ord::InscriptionStore</a>): u32
</code></pre>



<a name="0x4_ord_unbound_inscription_count"></a>

## Function `unbound_inscription_count`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_unbound_inscription_count">unbound_inscription_count</a>(inscription_store: &<a href="ord.md#0x4_ord_InscriptionStore">ord::InscriptionStore</a>): u32
</code></pre>



<a name="0x4_ord_next_sequence_number"></a>

## Function `next_sequence_number`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_next_sequence_number">next_sequence_number</a>(inscription_store: &<a href="ord.md#0x4_ord_InscriptionStore">ord::InscriptionStore</a>): u32
</code></pre>



<a name="0x4_ord_update_cursed_inscription_count"></a>

## Function `update_cursed_inscription_count`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_update_cursed_inscription_count">update_cursed_inscription_count</a>(inscription_store: &<b>mut</b> <a href="ord.md#0x4_ord_InscriptionStore">ord::InscriptionStore</a>, count: u32)
</code></pre>



<a name="0x4_ord_update_blessed_inscription_count"></a>

## Function `update_blessed_inscription_count`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_update_blessed_inscription_count">update_blessed_inscription_count</a>(inscription_store: &<b>mut</b> <a href="ord.md#0x4_ord_InscriptionStore">ord::InscriptionStore</a>, count: u32)
</code></pre>



<a name="0x4_ord_update_next_sequence_number"></a>

## Function `update_next_sequence_number`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_update_next_sequence_number">update_next_sequence_number</a>(inscription_store: &<b>mut</b> <a href="ord.md#0x4_ord_InscriptionStore">ord::InscriptionStore</a>, count: u32)
</code></pre>



<a name="0x4_ord_update_unbound_inscription_count"></a>

## Function `update_unbound_inscription_count`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_update_unbound_inscription_count">update_unbound_inscription_count</a>(inscription_store: &<b>mut</b> <a href="ord.md#0x4_ord_InscriptionStore">ord::InscriptionStore</a>, count: u32)
</code></pre>



<a name="0x4_ord_new_inscription_id"></a>

## Function `new_inscription_id`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_new_inscription_id">new_inscription_id</a>(txid: <b>address</b>, index: u32): <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>
</code></pre>



<a name="0x4_ord_derive_inscription_id"></a>

## Function `derive_inscription_id`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_derive_inscription_id">derive_inscription_id</a>(inscription_id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x4_ord_parse_inscription_id"></a>

## Function `parse_inscription_id`

Prase InscriptionID from String


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_parse_inscription_id">parse_inscription_id</a>(inscription_id: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>&gt;
</code></pre>



<a name="0x4_ord_inscription_id_to_string"></a>

## Function `inscription_id_to_string`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_inscription_id_to_string">inscription_id_to_string</a>(inscription_id: &<a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x4_ord_get_inscription_id_by_sequence_number"></a>

## Function `get_inscription_id_by_sequence_number`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_get_inscription_id_by_sequence_number">get_inscription_id_by_sequence_number</a>(sequence_number: u32): &<a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>
</code></pre>



<a name="0x4_ord_get_inscription_next_sequence_number"></a>

## Function `get_inscription_next_sequence_number`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_get_inscription_next_sequence_number">get_inscription_next_sequence_number</a>(): u32
</code></pre>



<a name="0x4_ord_create_object"></a>

## Function `create_object`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_create_object">create_object</a>(id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>, location: <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>, sequence_number: u32, inscription_number: u32, is_cursed: bool, charms: u16, body: <a href="">vector</a>&lt;u8&gt;, content_encoding: <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, content_type: <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, metadata: <a href="">vector</a>&lt;u8&gt;, metaprotocol: <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, parents: <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>&gt;, pointer: <a href="_Option">option::Option</a>&lt;u64&gt;, rune: <a href="_Option">option::Option</a>&lt;u128&gt;, owner: <b>address</b>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x4_ord_transfer_object"></a>

## Function `transfer_object`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_transfer_object">transfer_object</a>(inscription_obj: <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;, <b>to</b>: <b>address</b>, new_location: <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>, is_op_return: bool)
</code></pre>



<a name="0x4_ord_exists_inscription"></a>

## Function `exists_inscription`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_exists_inscription">exists_inscription</a>(id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): bool
</code></pre>



<a name="0x4_ord_borrow_inscription"></a>

## Function `borrow_inscription`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_borrow_inscription">borrow_inscription</a>(txid: <b>address</b>, index: u32): &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x4_ord_borrow_inscription_by_id"></a>

## Function `borrow_inscription_by_id`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_borrow_inscription_by_id">borrow_inscription_by_id</a>(id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>
</code></pre>



<a name="0x4_ord_take_object"></a>

## Function `take_object`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_take_object">take_object</a>(inscription_obj_id: <a href="_ObjectID">object::ObjectID</a>): <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x4_ord_borrow_object"></a>

## Function `borrow_object`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_borrow_object">borrow_object</a>(inscription_obj_id: <a href="_ObjectID">object::ObjectID</a>): &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x4_ord_txid"></a>

## Function `txid`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_txid">txid</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <b>address</b>
</code></pre>



<a name="0x4_ord_index"></a>

## Function `index`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_index">index</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): u32
</code></pre>



<a name="0x4_ord_location"></a>

## Function `location`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_location">location</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): &<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>
</code></pre>



<a name="0x4_ord_sequence_number"></a>

## Function `sequence_number`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_sequence_number">sequence_number</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): u32
</code></pre>



<a name="0x4_ord_inscription_number"></a>

## Function `inscription_number`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_inscription_number">inscription_number</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): u32
</code></pre>



<a name="0x4_ord_is_cursed"></a>

## Function `is_cursed`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_is_cursed">is_cursed</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): bool
</code></pre>



<a name="0x4_ord_charms"></a>

## Function `charms`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charms">charms</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): u16
</code></pre>



<a name="0x4_ord_offset"></a>

## Function `offset`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_offset">offset</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): u64
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



<a name="0x4_ord_parents"></a>

## Function `parents`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_parents">parents</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>&gt;
</code></pre>



<a name="0x4_ord_pointer"></a>

## Function `pointer`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_pointer">pointer</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<a name="0x4_ord_inscription_id"></a>

## Function `inscription_id`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_inscription_id">inscription_id</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>
</code></pre>



<a name="0x4_ord_inscription_id_txid"></a>

## Function `inscription_id_txid`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_inscription_id_txid">inscription_id_txid</a>(self: &<a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): <b>address</b>
</code></pre>



<a name="0x4_ord_inscription_id_index"></a>

## Function `inscription_id_index`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_inscription_id_index">inscription_id_index</a>(self: &<a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): u32
</code></pre>



<a name="0x4_ord_new_satpoint"></a>

## Function `new_satpoint`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_new_satpoint">new_satpoint</a>(outpoint: <a href="types.md#0x4_types_OutPoint">types::OutPoint</a>, offset: u64): <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>
</code></pre>



<a name="0x4_ord_unpack_satpoint"></a>

## Function `unpack_satpoint`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_unpack_satpoint">unpack_satpoint</a>(satpoint: <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>): (<a href="types.md#0x4_types_OutPoint">types::OutPoint</a>, u64)
</code></pre>



<a name="0x4_ord_satpoint_offset"></a>

## Function `satpoint_offset`

Get the SatPoint's offset


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_satpoint_offset">satpoint_offset</a>(satpoint: &<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>): u64
</code></pre>



<a name="0x4_ord_satpoint_outpoint"></a>

## Function `satpoint_outpoint`

Get the SatPoint's outpoint


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_satpoint_outpoint">satpoint_outpoint</a>(satpoint: &<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>): &<a href="types.md#0x4_types_OutPoint">types::OutPoint</a>
</code></pre>



<a name="0x4_ord_satpoint_vout"></a>

## Function `satpoint_vout`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_satpoint_vout">satpoint_vout</a>(satpoint: &<a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>): u32
</code></pre>



<a name="0x4_ord_new_flotsam"></a>

## Function `new_flotsam`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_new_flotsam">new_flotsam</a>(output_index: u32, offset: u64, object_id: <a href="_ObjectID">object::ObjectID</a>): <a href="ord.md#0x4_ord_Flotsam">ord::Flotsam</a>
</code></pre>



<a name="0x4_ord_unpack_flotsam"></a>

## Function `unpack_flotsam`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_unpack_flotsam">unpack_flotsam</a>(flotsam: <a href="ord.md#0x4_ord_Flotsam">ord::Flotsam</a>): (u32, u64, <a href="_ObjectID">object::ObjectID</a>)
</code></pre>



<a name="0x4_ord_parse_inscription_from_tx"></a>

## Function `parse_inscription_from_tx`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_parse_inscription_from_tx">parse_inscription_from_tx</a>(tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_Envelope">ord::Envelope</a>&lt;<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>&gt;&gt;
</code></pre>



<a name="0x4_ord_subsidy_by_height"></a>

## Function `subsidy_by_height`

Block Rewards


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_subsidy_by_height">subsidy_by_height</a>(height: u64): u64
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



<a name="0x4_ord_destroy_permanent_area"></a>

## Function `destroy_permanent_area`

Destroy permanent area if it's empty. Aborts if it's not empty.


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_destroy_permanent_area">destroy_permanent_area</a>(inscription: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;)
</code></pre>



<a name="0x4_ord_add_temp_state"></a>

## Function `add_temp_state`



<pre><code>#[private_generics(#[S])]
<b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_add_temp_state">add_temp_state</a>&lt;S: drop, store&gt;(inscription: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;, state: S)
</code></pre>



<a name="0x4_ord_contains_temp_state"></a>

## Function `contains_temp_state`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_contains_temp_state">contains_temp_state</a>&lt;S: drop, store&gt;(inscription: &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;): bool
</code></pre>



<a name="0x4_ord_borrow_temp_state"></a>

## Function `borrow_temp_state`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_borrow_temp_state">borrow_temp_state</a>&lt;S: drop, store&gt;(inscription: &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;): &S
</code></pre>



<a name="0x4_ord_borrow_mut_temp_state"></a>

## Function `borrow_mut_temp_state`



<pre><code>#[private_generics(#[S])]
<b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_borrow_mut_temp_state">borrow_mut_temp_state</a>&lt;S: drop, store&gt;(inscription: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;): &<b>mut</b> S
</code></pre>



<a name="0x4_ord_remove_temp_state"></a>

## Function `remove_temp_state`



<pre><code>#[private_generics(#[S])]
<b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_remove_temp_state">remove_temp_state</a>&lt;S: drop, store&gt;(inscription: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;): S
</code></pre>



<a name="0x4_ord_drop_temp_area"></a>

## Function `drop_temp_area`

Drop the bag, whether it's empty or not


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_drop_temp_area">drop_temp_area</a>(inscription: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;)
</code></pre>



<a name="0x4_ord_register_metaprotocol_via_system"></a>

## Function `register_metaprotocol_via_system`

Currently, Only the framework can register metaprotocol.
We need to find a way to allow the user to register metaprotocol.


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_register_metaprotocol_via_system">register_metaprotocol_via_system</a>&lt;T&gt;(system: &<a href="">signer</a>, metaprotocol: <a href="_String">string::String</a>)
</code></pre>



<a name="0x4_ord_is_metaprotocol_register"></a>

## Function `is_metaprotocol_register`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_is_metaprotocol_register">is_metaprotocol_register</a>(metaprotocol: <a href="_String">string::String</a>): bool
</code></pre>



<a name="0x4_ord_seal_metaprotocol_validity"></a>

## Function `seal_metaprotocol_validity`

Seal the metaprotocol validity for the given inscription_id.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_seal_metaprotocol_validity">seal_metaprotocol_validity</a>&lt;T&gt;(inscription_id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>, is_valid: bool, invalid_reason: <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;)
</code></pre>



<a name="0x4_ord_add_metaprotocol_attachment"></a>

## Function `add_metaprotocol_attachment`



<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_add_metaprotocol_attachment">add_metaprotocol_attachment</a>&lt;T&gt;(inscription_id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>, attachment: <a href="_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<a name="0x4_ord_exists_metaprotocol_validity"></a>

## Function `exists_metaprotocol_validity`

Returns true if Inscription <code><a href="">object</a></code> contains metaprotocol validity


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_exists_metaprotocol_validity">exists_metaprotocol_validity</a>(inscription_id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): bool
</code></pre>



<a name="0x4_ord_borrow_metaprotocol_validity"></a>

## Function `borrow_metaprotocol_validity`

Borrow the metaprotocol validity for the given inscription_id.


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_borrow_metaprotocol_validity">borrow_metaprotocol_validity</a>(inscription_id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): &<a href="ord.md#0x4_ord_MetaprotocolValidity">ord::MetaprotocolValidity</a>
</code></pre>



<a name="0x4_ord_metaprotocol_validity_protocol_match"></a>

## Function `metaprotocol_validity_protocol_match`

Check the MetaprotocolValidity's protocol_type whether match


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_metaprotocol_validity_protocol_match">metaprotocol_validity_protocol_match</a>&lt;T&gt;(validity: &<a href="ord.md#0x4_ord_MetaprotocolValidity">ord::MetaprotocolValidity</a>): bool
</code></pre>



<a name="0x4_ord_metaprotocol_validity_protocol_type"></a>

## Function `metaprotocol_validity_protocol_type`

Get the MetaprotocolValidity's protocol_type


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_metaprotocol_validity_protocol_type">metaprotocol_validity_protocol_type</a>(validity: &<a href="ord.md#0x4_ord_MetaprotocolValidity">ord::MetaprotocolValidity</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x4_ord_metaprotocol_validity_is_valid"></a>

## Function `metaprotocol_validity_is_valid`

Get the MetaprotocolValidity's is_valid


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_metaprotocol_validity_is_valid">metaprotocol_validity_is_valid</a>(validity: &<a href="ord.md#0x4_ord_MetaprotocolValidity">ord::MetaprotocolValidity</a>): bool
</code></pre>



<a name="0x4_ord_metaprotocol_validity_invalid_reason"></a>

## Function `metaprotocol_validity_invalid_reason`

Get the MetaprotocolValidity's invalid_reason


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_metaprotocol_validity_invalid_reason">metaprotocol_validity_invalid_reason</a>(validity: &<a href="ord.md#0x4_ord_MetaprotocolValidity">ord::MetaprotocolValidity</a>): <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x4_ord_view_validity"></a>

## Function `view_validity`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_view_validity">view_validity</a>(inscription_id_str: <a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="ord.md#0x4_ord_MetaprotocolValidity">ord::MetaprotocolValidity</a>&gt;
</code></pre>



<a name="0x4_ord_unpack_inscription_event"></a>

## Function `unpack_inscription_event`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_unpack_inscription_event">unpack_inscription_event</a>(<a href="">event</a>: <a href="ord.md#0x4_ord_InscriptionEvent">ord::InscriptionEvent</a>): (<a href="_String">string::String</a>, u32, <a href="_ObjectID">object::ObjectID</a>, u8)
</code></pre>



<a name="0x4_ord_inscription_event_type_new"></a>

## Function `inscription_event_type_new`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_inscription_event_type_new">inscription_event_type_new</a>(): u8
</code></pre>



<a name="0x4_ord_inscription_event_type_burn"></a>

## Function `inscription_event_type_burn`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_inscription_event_type_burn">inscription_event_type_burn</a>(): u8
</code></pre>



<a name="0x4_ord_inscription_charm_burned"></a>

## Function `inscription_charm_burned`

Get the InscriptionCharm's burned


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_inscription_charm_burned">inscription_charm_burned</a>(charm: &<a href="ord.md#0x4_ord_InscriptionCharm">ord::InscriptionCharm</a>): bool
</code></pre>



<a name="0x4_ord_exists_inscription_charm"></a>

## Function `exists_inscription_charm`

Checks if an InscriptionCharm exists for a given InscriptionID.

@param inscription_id - The ID of the inscription
@return Boolean indicating whether the charm exists


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_exists_inscription_charm">exists_inscription_charm</a>(inscription_id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): bool
</code></pre>



<a name="0x4_ord_borrow_inscription_charm"></a>

## Function `borrow_inscription_charm`

Borrows a reference to the InscriptionCharm for a given InscriptionID.

@param inscription_id - The ID of the inscription
@return Reference to the InscriptionCharm


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_borrow_inscription_charm">borrow_inscription_charm</a>(inscription_id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): &<a href="ord.md#0x4_ord_InscriptionCharm">ord::InscriptionCharm</a>
</code></pre>



<a name="0x4_ord_view_inscription_charm"></a>

## Function `view_inscription_charm`

Views the InscriptionCharm for a given inscription ID string.
Returns None if the inscription doesn't exist or doesn't have a charm.

@param inscription_id_str - String representation of the inscription ID
@return Option<InscriptionCharm> - Some(charm) if exists, None otherwise


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_view_inscription_charm">view_inscription_charm</a>(inscription_id_str: <a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="ord.md#0x4_ord_InscriptionCharm">ord::InscriptionCharm</a>&gt;
</code></pre>
