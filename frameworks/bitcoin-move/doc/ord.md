
<a name="0x4_ord"></a>

# Module `0x4::ord`



-  [Struct `InscriptionID`](#0x4_ord_InscriptionID)
-  [Struct `SatPoint`](#0x4_ord_SatPoint)
-  [Resource `Inscription`](#0x4_ord_Inscription)
-  [Struct `Envelope`](#0x4_ord_Envelope)
-  [Struct `InscriptionRecord`](#0x4_ord_InscriptionRecord)
-  [Struct `InvalidInscriptionEvent`](#0x4_ord_InvalidInscriptionEvent)
-  [Resource `MetaprotocolRegistry`](#0x4_ord_MetaprotocolRegistry)
-  [Struct `MetaprotocolValidity`](#0x4_ord_MetaprotocolValidity)
-  [Resource `InscriptionStore`](#0x4_ord_InscriptionStore)
-  [Struct `InscriptionEvent`](#0x4_ord_InscriptionEvent)
-  [Struct `TempStateDropEvent`](#0x4_ord_TempStateDropEvent)
-  [Struct `InscriptionCharm`](#0x4_ord_InscriptionCharm)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_ord_genesis_init)
-  [Function `borrow_mut_inscription_store`](#0x4_ord_borrow_mut_inscription_store)
-  [Function `borrow_inscription_store`](#0x4_ord_borrow_inscription_store)
-  [Function `blessed_inscription_count`](#0x4_ord_blessed_inscription_count)
-  [Function `cursed_inscription_count`](#0x4_ord_cursed_inscription_count)
-  [Function `unbound_inscription_count`](#0x4_ord_unbound_inscription_count)
-  [Function `lost_sats`](#0x4_ord_lost_sats)
-  [Function `next_sequence_number`](#0x4_ord_next_sequence_number)
-  [Function `update_cursed_inscription_count`](#0x4_ord_update_cursed_inscription_count)
-  [Function `update_blessed_inscription_count`](#0x4_ord_update_blessed_inscription_count)
-  [Function `update_next_sequence_number`](#0x4_ord_update_next_sequence_number)
-  [Function `update_unbound_inscription_count`](#0x4_ord_update_unbound_inscription_count)
-  [Function `update_lost_sats`](#0x4_ord_update_lost_sats)
-  [Function `new_inscription_id`](#0x4_ord_new_inscription_id)
-  [Function `derive_inscription_id`](#0x4_ord_derive_inscription_id)
-  [Function `parse_inscription_id`](#0x4_ord_parse_inscription_id)
-  [Function `inscription_id_to_string`](#0x4_ord_inscription_id_to_string)
-  [Function `get_inscription_id_by_sequence_number`](#0x4_ord_get_inscription_id_by_sequence_number)
-  [Function `get_inscription_next_sequence_number`](#0x4_ord_get_inscription_next_sequence_number)
-  [Function `create_object`](#0x4_ord_create_object)
-  [Function `transfer_object`](#0x4_ord_transfer_object)
-  [Function `take_object`](#0x4_ord_take_object)
-  [Function `borrow_object`](#0x4_ord_borrow_object)
-  [Function `exists_inscription`](#0x4_ord_exists_inscription)
-  [Function `borrow_inscription`](#0x4_ord_borrow_inscription)
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
-  [Function `id`](#0x4_ord_id)
-  [Function `inscription_id_txid`](#0x4_ord_inscription_id_txid)
-  [Function `inscription_id_index`](#0x4_ord_inscription_id_index)
-  [Function `new_satpoint`](#0x4_ord_new_satpoint)
-  [Function `unpack_satpoint`](#0x4_ord_unpack_satpoint)
-  [Function `satpoint_offset`](#0x4_ord_satpoint_offset)
-  [Function `satpoint_outpoint`](#0x4_ord_satpoint_outpoint)
-  [Function `satpoint_vout`](#0x4_ord_satpoint_vout)
-  [Function `parse_inscription_from_tx`](#0x4_ord_parse_inscription_from_tx)
-  [Function `envelope_input`](#0x4_ord_envelope_input)
-  [Function `envelope_offset`](#0x4_ord_envelope_offset)
-  [Function `envelope_payload`](#0x4_ord_envelope_payload)
-  [Function `envelope_pushnum`](#0x4_ord_envelope_pushnum)
-  [Function `envelope_stutter`](#0x4_ord_envelope_stutter)
-  [Function `inscription_record_pointer`](#0x4_ord_inscription_record_pointer)
-  [Function `inscription_record_parents`](#0x4_ord_inscription_record_parents)
-  [Function `inscription_record_unrecognized_even_field`](#0x4_ord_inscription_record_unrecognized_even_field)
-  [Function `inscription_record_duplicate_field`](#0x4_ord_inscription_record_duplicate_field)
-  [Function `inscription_record_incomplete_field`](#0x4_ord_inscription_record_incomplete_field)
-  [Function `inscription_record_metaprotocol`](#0x4_ord_inscription_record_metaprotocol)
-  [Function `inscription_record_rune`](#0x4_ord_inscription_record_rune)
-  [Function `inscription_record_metadata`](#0x4_ord_inscription_record_metadata)
-  [Function `inscription_record_content_type`](#0x4_ord_inscription_record_content_type)
-  [Function `inscription_record_content_encoding`](#0x4_ord_inscription_record_content_encoding)
-  [Function `inscription_record_body`](#0x4_ord_inscription_record_body)
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
-  [Function `remove_metaprotocol_attachment`](#0x4_ord_remove_metaprotocol_attachment)
-  [Function `exists_metaprotocol_attachment`](#0x4_ord_exists_metaprotocol_attachment)
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
-  [Function `unpack_temp_state_drop_event`](#0x4_ord_unpack_temp_state_drop_event)
-  [Function `charm_coin_flag`](#0x4_ord_charm_coin_flag)
-  [Function `charm_cursed_flag`](#0x4_ord_charm_cursed_flag)
-  [Function `charm_epic_flag`](#0x4_ord_charm_epic_flag)
-  [Function `charm_legendary_flag`](#0x4_ord_charm_legendary_flag)
-  [Function `charm_lost_flag`](#0x4_ord_charm_lost_flag)
-  [Function `charm_nineball_flag`](#0x4_ord_charm_nineball_flag)
-  [Function `charm_rare_flag`](#0x4_ord_charm_rare_flag)
-  [Function `charm_reinscription_flag`](#0x4_ord_charm_reinscription_flag)
-  [Function `charm_unbound_flag`](#0x4_ord_charm_unbound_flag)
-  [Function `charm_uncommon_flag`](#0x4_ord_charm_uncommon_flag)
-  [Function `charm_vindicated_flag`](#0x4_ord_charm_vindicated_flag)
-  [Function `charm_mythic_flag`](#0x4_ord_charm_mythic_flag)
-  [Function `charm_burned_flag`](#0x4_ord_charm_burned_flag)
-  [Function `set_charm`](#0x4_ord_set_charm)
-  [Function `is_set_charm`](#0x4_ord_is_set_charm)
-  [Function `view_inscription_charm`](#0x4_ord_view_inscription_charm)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bag</a>;
<b>use</b> <a href="">0x2::core_addresses</a>;
<b>use</b> <a href="">0x2::event_queue</a>;
<b>use</b> <a href="">0x2::json</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::string_utils</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="bitcoin_hash.md#0x4_bitcoin_hash">0x4::bitcoin_hash</a>;
<b>use</b> <a href="temp_state.md#0x4_temp_state">0x4::temp_state</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
</code></pre>



<a name="0x4_ord_InscriptionID"></a>

## Struct `InscriptionID`



<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionID">InscriptionID</a> <b>has</b> <b>copy</b>, drop, store
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

Inscription event for metaprotocol

This event is used to record inscription operations related to metaprotocols.
Compared to the events in inscription_updater, the main differences are:
1. This event focuses on metaprotocol-related operations, it is an on-chain event.
2. This event is only emitted when the inscription is created or burned, not when it is transferred.
3. This event is only emitted if the inscription has a metaprotocol.

@param metaprotocol: The name of the metaprotocol
@param sequence_number: The sequence number of the inscription
@param inscription_obj_id: The ID of the inscription object
@param event_type: Event type, 0 for creation, 1 for burn


<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionEvent">InscriptionEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_TempStateDropEvent"></a>

## Struct `TempStateDropEvent`

Event emitted when the temporary state of an Inscription is dropped
The temporary state is dropped when the inscription is transferred
The event is onchain event, and the event_queue name is type_name of the temporary state


<pre><code><b>struct</b> <a href="ord.md#0x4_ord_TempStateDropEvent">TempStateDropEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_ord_InscriptionCharm"></a>

## Struct `InscriptionCharm`

A struct to represent the Inscription Charm


<pre><code><b>struct</b> <a href="ord.md#0x4_ord_InscriptionCharm">InscriptionCharm</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_ord_TEMPORARY_AREA"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_TEMPORARY_AREA">TEMPORARY_AREA</a>: <a href="">vector</a>&lt;u8&gt; = [116, 101, 109, 112, 111, 114, 97, 114, 121, 95, 97, 114, 101, 97];
</code></pre>



<a name="0x4_ord_CHARM_BURNED_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_BURNED_FLAG">CHARM_BURNED_FLAG</a>: u16 = 4096;
</code></pre>



<a name="0x4_ord_CHARM_COIN_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_COIN_FLAG">CHARM_COIN_FLAG</a>: u16 = 1;
</code></pre>



<a name="0x4_ord_CHARM_CURSED_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_CURSED_FLAG">CHARM_CURSED_FLAG</a>: u16 = 2;
</code></pre>



<a name="0x4_ord_CHARM_EPIC_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_EPIC_FLAG">CHARM_EPIC_FLAG</a>: u16 = 4;
</code></pre>



<a name="0x4_ord_CHARM_LEGENDARY_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_LEGENDARY_FLAG">CHARM_LEGENDARY_FLAG</a>: u16 = 8;
</code></pre>



<a name="0x4_ord_CHARM_LOST_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_LOST_FLAG">CHARM_LOST_FLAG</a>: u16 = 16;
</code></pre>



<a name="0x4_ord_CHARM_MYTHIC_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_MYTHIC_FLAG">CHARM_MYTHIC_FLAG</a>: u16 = 2048;
</code></pre>



<a name="0x4_ord_CHARM_NINEBALL_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_NINEBALL_FLAG">CHARM_NINEBALL_FLAG</a>: u16 = 32;
</code></pre>



<a name="0x4_ord_CHARM_RARE_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_RARE_FLAG">CHARM_RARE_FLAG</a>: u16 = 64;
</code></pre>



<a name="0x4_ord_CHARM_REINSCRIPTION_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_REINSCRIPTION_FLAG">CHARM_REINSCRIPTION_FLAG</a>: u16 = 128;
</code></pre>



<a name="0x4_ord_CHARM_UNBOUND_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_UNBOUND_FLAG">CHARM_UNBOUND_FLAG</a>: u16 = 256;
</code></pre>



<a name="0x4_ord_CHARM_UNCOMMON_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_UNCOMMON_FLAG">CHARM_UNCOMMON_FLAG</a>: u16 = 512;
</code></pre>



<a name="0x4_ord_CHARM_VINDICATED_FLAG"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_CHARM_VINDICATED_FLAG">CHARM_VINDICATED_FLAG</a>: u16 = 1024;
</code></pre>



<a name="0x4_ord_ErrorInscriptionNotExists"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_ErrorInscriptionNotExists">ErrorInscriptionNotExists</a>: u64 = 3;
</code></pre>



<a name="0x4_ord_ErrorMetaprotocolAlreadyRegistered"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_ErrorMetaprotocolAlreadyRegistered">ErrorMetaprotocolAlreadyRegistered</a>: u64 = 1;
</code></pre>



<a name="0x4_ord_ErrorMetaprotocolProtocolMismatch"></a>



<pre><code><b>const</b> <a href="ord.md#0x4_ord_ErrorMetaprotocolProtocolMismatch">ErrorMetaprotocolProtocolMismatch</a>: u64 = 2;
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



<a name="0x4_ord_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_genesis_init">genesis_init</a>()
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



<a name="0x4_ord_lost_sats"></a>

## Function `lost_sats`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_lost_sats">lost_sats</a>(inscription_store: &<a href="ord.md#0x4_ord_InscriptionStore">ord::InscriptionStore</a>): u64
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



<a name="0x4_ord_update_lost_sats"></a>

## Function `update_lost_sats`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_update_lost_sats">update_lost_sats</a>(inscription_store: &<b>mut</b> <a href="ord.md#0x4_ord_InscriptionStore">ord::InscriptionStore</a>, count: u64)
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



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_create_object">create_object</a>(id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>, location: <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>, sequence_number: u32, inscription_number: u32, is_cursed: bool, charms: u16, envelope: <a href="ord.md#0x4_ord_Envelope">ord::Envelope</a>&lt;<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>&gt;, owner: <b>address</b>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x4_ord_transfer_object"></a>

## Function `transfer_object`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_transfer_object">transfer_object</a>(inscription_obj: <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;, <b>to</b>: <b>address</b>, new_location: <a href="ord.md#0x4_ord_SatPoint">ord::SatPoint</a>, is_op_return: bool)
</code></pre>



<a name="0x4_ord_take_object"></a>

## Function `take_object`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_take_object">take_object</a>(inscription_obj_id: <a href="_ObjectID">object::ObjectID</a>): <a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x4_ord_borrow_object"></a>

## Function `borrow_object`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_borrow_object">borrow_object</a>(inscription_obj_id: <a href="_ObjectID">object::ObjectID</a>): &<a href="_Object">object::Object</a>&lt;<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>&gt;
</code></pre>



<a name="0x4_ord_exists_inscription"></a>

## Function `exists_inscription`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_exists_inscription">exists_inscription</a>(id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): bool
</code></pre>



<a name="0x4_ord_borrow_inscription"></a>

## Function `borrow_inscription`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_borrow_inscription">borrow_inscription</a>(id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>
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



<a name="0x4_ord_id"></a>

## Function `id`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_id">id</a>(self: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): &<a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>
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



<a name="0x4_ord_parse_inscription_from_tx"></a>

## Function `parse_inscription_from_tx`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_parse_inscription_from_tx">parse_inscription_from_tx</a>(tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_Envelope">ord::Envelope</a>&lt;<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>&gt;&gt;
</code></pre>



<a name="0x4_ord_envelope_input"></a>

## Function `envelope_input`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_envelope_input">envelope_input</a>&lt;T&gt;(envelope: &<a href="ord.md#0x4_ord_Envelope">ord::Envelope</a>&lt;T&gt;): u32
</code></pre>



<a name="0x4_ord_envelope_offset"></a>

## Function `envelope_offset`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_envelope_offset">envelope_offset</a>&lt;T&gt;(envelope: &<a href="ord.md#0x4_ord_Envelope">ord::Envelope</a>&lt;T&gt;): u32
</code></pre>



<a name="0x4_ord_envelope_payload"></a>

## Function `envelope_payload`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_envelope_payload">envelope_payload</a>&lt;T&gt;(envelope: &<a href="ord.md#0x4_ord_Envelope">ord::Envelope</a>&lt;T&gt;): &T
</code></pre>



<a name="0x4_ord_envelope_pushnum"></a>

## Function `envelope_pushnum`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_envelope_pushnum">envelope_pushnum</a>&lt;T&gt;(envelope: &<a href="ord.md#0x4_ord_Envelope">ord::Envelope</a>&lt;T&gt;): bool
</code></pre>



<a name="0x4_ord_envelope_stutter"></a>

## Function `envelope_stutter`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_envelope_stutter">envelope_stutter</a>&lt;T&gt;(envelope: &<a href="ord.md#0x4_ord_Envelope">ord::Envelope</a>&lt;T&gt;): bool
</code></pre>



<a name="0x4_ord_inscription_record_pointer"></a>

## Function `inscription_record_pointer`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_inscription_record_pointer">inscription_record_pointer</a>(record: &<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): &<a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<a name="0x4_ord_inscription_record_parents"></a>

## Function `inscription_record_parents`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_inscription_record_parents">inscription_record_parents</a>(record: &<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): &<a href="">vector</a>&lt;<a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>&gt;
</code></pre>



<a name="0x4_ord_inscription_record_unrecognized_even_field"></a>

## Function `inscription_record_unrecognized_even_field`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_inscription_record_unrecognized_even_field">inscription_record_unrecognized_even_field</a>(record: &<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): bool
</code></pre>



<a name="0x4_ord_inscription_record_duplicate_field"></a>

## Function `inscription_record_duplicate_field`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_inscription_record_duplicate_field">inscription_record_duplicate_field</a>(record: &<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): bool
</code></pre>



<a name="0x4_ord_inscription_record_incomplete_field"></a>

## Function `inscription_record_incomplete_field`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_inscription_record_incomplete_field">inscription_record_incomplete_field</a>(record: &<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): bool
</code></pre>



<a name="0x4_ord_inscription_record_metaprotocol"></a>

## Function `inscription_record_metaprotocol`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_inscription_record_metaprotocol">inscription_record_metaprotocol</a>(record: &<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): &<a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x4_ord_inscription_record_rune"></a>

## Function `inscription_record_rune`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_inscription_record_rune">inscription_record_rune</a>(record: &<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): &<a href="_Option">option::Option</a>&lt;u128&gt;
</code></pre>



<a name="0x4_ord_inscription_record_metadata"></a>

## Function `inscription_record_metadata`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_inscription_record_metadata">inscription_record_metadata</a>(record: &<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_ord_inscription_record_content_type"></a>

## Function `inscription_record_content_type`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_inscription_record_content_type">inscription_record_content_type</a>(record: &<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): &<a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x4_ord_inscription_record_content_encoding"></a>

## Function `inscription_record_content_encoding`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_inscription_record_content_encoding">inscription_record_content_encoding</a>(record: &<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): &<a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x4_ord_inscription_record_body"></a>

## Function `inscription_record_body`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ord.md#0x4_ord_inscription_record_body">inscription_record_body</a>(record: &<a href="ord.md#0x4_ord_InscriptionRecord">ord::InscriptionRecord</a>): &<a href="">vector</a>&lt;u8&gt;
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



<a name="0x4_ord_remove_metaprotocol_attachment"></a>

## Function `remove_metaprotocol_attachment`



<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_remove_metaprotocol_attachment">remove_metaprotocol_attachment</a>&lt;T&gt;(inscription_id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): <a href="_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x4_ord_exists_metaprotocol_attachment"></a>

## Function `exists_metaprotocol_attachment`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_exists_metaprotocol_attachment">exists_metaprotocol_attachment</a>&lt;T&gt;(inscription_id: <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>): bool
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



<a name="0x4_ord_unpack_temp_state_drop_event"></a>

## Function `unpack_temp_state_drop_event`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_unpack_temp_state_drop_event">unpack_temp_state_drop_event</a>(<a href="">event</a>: <a href="ord.md#0x4_ord_TempStateDropEvent">ord::TempStateDropEvent</a>): (<a href="_ObjectID">object::ObjectID</a>, <a href="ord.md#0x4_ord_InscriptionID">ord::InscriptionID</a>)
</code></pre>



<a name="0x4_ord_charm_coin_flag"></a>

## Function `charm_coin_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_coin_flag">charm_coin_flag</a>(): u16
</code></pre>



<a name="0x4_ord_charm_cursed_flag"></a>

## Function `charm_cursed_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_cursed_flag">charm_cursed_flag</a>(): u16
</code></pre>



<a name="0x4_ord_charm_epic_flag"></a>

## Function `charm_epic_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_epic_flag">charm_epic_flag</a>(): u16
</code></pre>



<a name="0x4_ord_charm_legendary_flag"></a>

## Function `charm_legendary_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_legendary_flag">charm_legendary_flag</a>(): u16
</code></pre>



<a name="0x4_ord_charm_lost_flag"></a>

## Function `charm_lost_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_lost_flag">charm_lost_flag</a>(): u16
</code></pre>



<a name="0x4_ord_charm_nineball_flag"></a>

## Function `charm_nineball_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_nineball_flag">charm_nineball_flag</a>(): u16
</code></pre>



<a name="0x4_ord_charm_rare_flag"></a>

## Function `charm_rare_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_rare_flag">charm_rare_flag</a>(): u16
</code></pre>



<a name="0x4_ord_charm_reinscription_flag"></a>

## Function `charm_reinscription_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_reinscription_flag">charm_reinscription_flag</a>(): u16
</code></pre>



<a name="0x4_ord_charm_unbound_flag"></a>

## Function `charm_unbound_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_unbound_flag">charm_unbound_flag</a>(): u16
</code></pre>



<a name="0x4_ord_charm_uncommon_flag"></a>

## Function `charm_uncommon_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_uncommon_flag">charm_uncommon_flag</a>(): u16
</code></pre>



<a name="0x4_ord_charm_vindicated_flag"></a>

## Function `charm_vindicated_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_vindicated_flag">charm_vindicated_flag</a>(): u16
</code></pre>



<a name="0x4_ord_charm_mythic_flag"></a>

## Function `charm_mythic_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_mythic_flag">charm_mythic_flag</a>(): u16
</code></pre>



<a name="0x4_ord_charm_burned_flag"></a>

## Function `charm_burned_flag`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_charm_burned_flag">charm_burned_flag</a>(): u16
</code></pre>



<a name="0x4_ord_set_charm"></a>

## Function `set_charm`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_set_charm">set_charm</a>(charms: u16, flag: u16): u16
</code></pre>



<a name="0x4_ord_is_set_charm"></a>

## Function `is_set_charm`



<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_is_set_charm">is_set_charm</a>(charms: u16, flag: u16): bool
</code></pre>



<a name="0x4_ord_view_inscription_charm"></a>

## Function `view_inscription_charm`

Views the Inscription charms for a given inscription ID string.
Returns None if the inscription doesn't exist

@param inscription_id_str - String representation of the inscription ID
@return Option<InscriptionCharm> - Some(charm) if exists, None otherwise


<pre><code><b>public</b> <b>fun</b> <a href="ord.md#0x4_ord_view_inscription_charm">view_inscription_charm</a>(inscription_id_str: <a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="ord.md#0x4_ord_InscriptionCharm">ord::InscriptionCharm</a>&gt;
</code></pre>
