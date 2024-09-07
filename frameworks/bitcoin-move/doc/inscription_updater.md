
<a name="0x4_inscription_updater"></a>

# Module `0x4::inscription_updater`

The move version inscription_updater
https://github.com/ordinals/ord/blob/e59bd3e73d30ed9bc0b252ba2084bba670d6b0db/src/index/updater/inscription_updater.rs


-  [Struct `FlotsamNew`](#0x4_inscription_updater_FlotsamNew)
-  [Struct `Flotsam`](#0x4_inscription_updater_Flotsam)
-  [Struct `InscriptionCreatedEvent`](#0x4_inscription_updater_InscriptionCreatedEvent)
-  [Struct `InscriptionTransferredEvent`](#0x4_inscription_updater_InscriptionTransferredEvent)
-  [Struct `InscriptionUpdater`](#0x4_inscription_updater_InscriptionUpdater)
-  [Struct `Location`](#0x4_inscription_updater_Location)
-  [Struct `Range`](#0x4_inscription_updater_Range)
-  [Struct `ReinscribeCounter`](#0x4_inscription_updater_ReinscribeCounter)
-  [Constants](#@Constants_0)
-  [Function `process_tx`](#0x4_inscription_updater_process_tx)
-  [Function `need_process_oridinals`](#0x4_inscription_updater_need_process_oridinals)
-  [Function `curse_duplicate_field`](#0x4_inscription_updater_curse_duplicate_field)
-  [Function `curse_incompleted_field`](#0x4_inscription_updater_curse_incompleted_field)
-  [Function `curse_not_at_offset_zero`](#0x4_inscription_updater_curse_not_at_offset_zero)
-  [Function `curse_not_in_first_input`](#0x4_inscription_updater_curse_not_in_first_input)
-  [Function `curse_pointer`](#0x4_inscription_updater_curse_pointer)
-  [Function `curse_pushnum`](#0x4_inscription_updater_curse_pushnum)
-  [Function `curse_reinscription`](#0x4_inscription_updater_curse_reinscription)
-  [Function `curse_stutter`](#0x4_inscription_updater_curse_stutter)
-  [Function `curse_unrecognized_even_field`](#0x4_inscription_updater_curse_unrecognized_even_field)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::compare</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="network.md#0x4_network">0x4::network</a>;
<b>use</b> <a href="ord.md#0x4_ord">0x4::ord</a>;
<b>use</b> <a href="pending_block.md#0x4_pending_block">0x4::pending_block</a>;
<b>use</b> <a href="script_buf.md#0x4_script_buf">0x4::script_buf</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
<b>use</b> <a href="utxo.md#0x4_utxo">0x4::utxo</a>;
</code></pre>



<a name="0x4_inscription_updater_FlotsamNew"></a>

## Struct `FlotsamNew`



<pre><code><b>struct</b> <a href="inscription_updater.md#0x4_inscription_updater_FlotsamNew">FlotsamNew</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_inscription_updater_Flotsam"></a>

## Struct `Flotsam`



<pre><code><b>struct</b> <a href="inscription_updater.md#0x4_inscription_updater_Flotsam">Flotsam</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_inscription_updater_InscriptionCreatedEvent"></a>

## Struct `InscriptionCreatedEvent`

Triggered when a new inscription is created
@param block_height: The block height at which the inscription is created
@param charms: The charm value of the inscription, representing its special attributes
@param inscription_id: The unique identifier of the newly created inscription
@param location: The location of the inscription, which may be None
@param parent_inscription_ids: A list of parent inscription IDs, used to represent relationships between inscriptions
@param sequence_number: The sequence number of the inscription


<pre><code><b>struct</b> <a href="inscription_updater.md#0x4_inscription_updater_InscriptionCreatedEvent">InscriptionCreatedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_inscription_updater_InscriptionTransferredEvent"></a>

## Struct `InscriptionTransferredEvent`

Triggered when an inscription is transferred
@param block_height: The block height at which the inscription is transferred
@param inscription_id: The unique identifier of the inscription being transferred
@param new_location: The new location of the inscription
@param old_location: The old location of the inscription
@param sequence_number: The sequence number of the inscription
@param is_burned: A boolean indicating whether the inscription is burned


<pre><code><b>struct</b> <a href="inscription_updater.md#0x4_inscription_updater_InscriptionTransferredEvent">InscriptionTransferredEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_inscription_updater_InscriptionUpdater"></a>

## Struct `InscriptionUpdater`



<pre><code><b>struct</b> <a href="inscription_updater.md#0x4_inscription_updater_InscriptionUpdater">InscriptionUpdater</a> <b>has</b> store
</code></pre>



<a name="0x4_inscription_updater_Location"></a>

## Struct `Location`



<pre><code><b>struct</b> <a href="inscription_updater.md#0x4_inscription_updater_Location">Location</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_inscription_updater_Range"></a>

## Struct `Range`



<pre><code><b>struct</b> <a href="inscription_updater.md#0x4_inscription_updater_Range">Range</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_inscription_updater_ReinscribeCounter"></a>

## Struct `ReinscribeCounter`



<pre><code><b>struct</b> <a href="inscription_updater.md#0x4_inscription_updater_ReinscribeCounter">ReinscribeCounter</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_inscription_updater_CURSE_DUPLICATE_FIELD"></a>

Curse Inscription


<pre><code><b>const</b> <a href="inscription_updater.md#0x4_inscription_updater_CURSE_DUPLICATE_FIELD">CURSE_DUPLICATE_FIELD</a>: <a href="">vector</a>&lt;u8&gt; = [68, 117, 112, 108, 105, 99, 97, 116, 101, 70, 105, 101, 108, 100];
</code></pre>



<a name="0x4_inscription_updater_CURSE_INCOMPLETE_FIELD"></a>



<pre><code><b>const</b> <a href="inscription_updater.md#0x4_inscription_updater_CURSE_INCOMPLETE_FIELD">CURSE_INCOMPLETE_FIELD</a>: <a href="">vector</a>&lt;u8&gt; = [73, 110, 99, 111, 109, 112, 108, 101, 116, 101, 70, 105, 101, 108, 100];
</code></pre>



<a name="0x4_inscription_updater_CURSE_NOT_AT_OFFSET_ZERO"></a>



<pre><code><b>const</b> <a href="inscription_updater.md#0x4_inscription_updater_CURSE_NOT_AT_OFFSET_ZERO">CURSE_NOT_AT_OFFSET_ZERO</a>: <a href="">vector</a>&lt;u8&gt; = [78, 111, 116, 65, 116, 79, 102, 102, 115, 101, 116, 90, 101, 114, 111];
</code></pre>



<a name="0x4_inscription_updater_CURSE_NOT_IN_FIRST_INPUT"></a>



<pre><code><b>const</b> <a href="inscription_updater.md#0x4_inscription_updater_CURSE_NOT_IN_FIRST_INPUT">CURSE_NOT_IN_FIRST_INPUT</a>: <a href="">vector</a>&lt;u8&gt; = [78, 111, 116, 73, 110, 70, 105, 114, 115, 116, 73, 110, 112, 117, 116];
</code></pre>



<a name="0x4_inscription_updater_CURSE_POINTER"></a>



<pre><code><b>const</b> <a href="inscription_updater.md#0x4_inscription_updater_CURSE_POINTER">CURSE_POINTER</a>: <a href="">vector</a>&lt;u8&gt; = [80, 111, 105, 110, 116, 101, 114];
</code></pre>



<a name="0x4_inscription_updater_CURSE_PUSHNUM"></a>



<pre><code><b>const</b> <a href="inscription_updater.md#0x4_inscription_updater_CURSE_PUSHNUM">CURSE_PUSHNUM</a>: <a href="">vector</a>&lt;u8&gt; = [80, 117, 115, 104, 110, 117, 109];
</code></pre>



<a name="0x4_inscription_updater_CURSE_REINSCRIPTION"></a>



<pre><code><b>const</b> <a href="inscription_updater.md#0x4_inscription_updater_CURSE_REINSCRIPTION">CURSE_REINSCRIPTION</a>: <a href="">vector</a>&lt;u8&gt; = [82, 101, 105, 110, 115, 99, 114, 105, 112, 116, 105, 111, 110];
</code></pre>



<a name="0x4_inscription_updater_CURSE_STUTTER"></a>



<pre><code><b>const</b> <a href="inscription_updater.md#0x4_inscription_updater_CURSE_STUTTER">CURSE_STUTTER</a>: <a href="">vector</a>&lt;u8&gt; = [83, 116, 117, 116, 116, 101, 114];
</code></pre>



<a name="0x4_inscription_updater_CURSE_UNRECOGNIZED_EVEN_FIELD"></a>



<pre><code><b>const</b> <a href="inscription_updater.md#0x4_inscription_updater_CURSE_UNRECOGNIZED_EVEN_FIELD">CURSE_UNRECOGNIZED_EVEN_FIELD</a>: <a href="">vector</a>&lt;u8&gt; = [85, 110, 114, 101, 99, 111, 103, 110, 105, 122, 101, 100, 69, 118, 101, 110, 70, 105, 101, 108, 100];
</code></pre>



<a name="0x4_inscription_updater_ErrorFlotsamNotProcessed"></a>



<pre><code><b>const</b> <a href="inscription_updater.md#0x4_inscription_updater_ErrorFlotsamNotProcessed">ErrorFlotsamNotProcessed</a>: u64 = 2;
</code></pre>



<a name="0x4_inscription_updater_ErrorUTXOBalanceNotMatch"></a>



<pre><code><b>const</b> <a href="inscription_updater.md#0x4_inscription_updater_ErrorUTXOBalanceNotMatch">ErrorUTXOBalanceNotMatch</a>: u64 = 1;
</code></pre>



<a name="0x4_inscription_updater_ORDINAL_GENESIS_HEIGHT"></a>



<pre><code><b>const</b> <a href="inscription_updater.md#0x4_inscription_updater_ORDINAL_GENESIS_HEIGHT">ORDINAL_GENESIS_HEIGHT</a>: u64 = 767430;
</code></pre>



<a name="0x4_inscription_updater_process_tx"></a>

## Function `process_tx`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="inscription_updater.md#0x4_inscription_updater_process_tx">process_tx</a>(<a href="pending_block.md#0x4_pending_block">pending_block</a>: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="pending_block.md#0x4_pending_block_PendingBlock">pending_block::PendingBlock</a>&gt;, tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>, input_utxos: &<b>mut</b> <a href="">vector</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;): <a href="">vector</a>&lt;<a href="utxo.md#0x4_utxo_SealOut">utxo::SealOut</a>&gt;
</code></pre>



<a name="0x4_inscription_updater_need_process_oridinals"></a>

## Function `need_process_oridinals`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="inscription_updater.md#0x4_inscription_updater_need_process_oridinals">need_process_oridinals</a>(block_height: u64): bool
</code></pre>



<a name="0x4_inscription_updater_curse_duplicate_field"></a>

## Function `curse_duplicate_field`



<pre><code><b>public</b> <b>fun</b> <a href="inscription_updater.md#0x4_inscription_updater_curse_duplicate_field">curse_duplicate_field</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_inscription_updater_curse_incompleted_field"></a>

## Function `curse_incompleted_field`



<pre><code><b>public</b> <b>fun</b> <a href="inscription_updater.md#0x4_inscription_updater_curse_incompleted_field">curse_incompleted_field</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_inscription_updater_curse_not_at_offset_zero"></a>

## Function `curse_not_at_offset_zero`



<pre><code><b>public</b> <b>fun</b> <a href="inscription_updater.md#0x4_inscription_updater_curse_not_at_offset_zero">curse_not_at_offset_zero</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_inscription_updater_curse_not_in_first_input"></a>

## Function `curse_not_in_first_input`



<pre><code><b>public</b> <b>fun</b> <a href="inscription_updater.md#0x4_inscription_updater_curse_not_in_first_input">curse_not_in_first_input</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_inscription_updater_curse_pointer"></a>

## Function `curse_pointer`



<pre><code><b>public</b> <b>fun</b> <a href="inscription_updater.md#0x4_inscription_updater_curse_pointer">curse_pointer</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_inscription_updater_curse_pushnum"></a>

## Function `curse_pushnum`



<pre><code><b>public</b> <b>fun</b> <a href="inscription_updater.md#0x4_inscription_updater_curse_pushnum">curse_pushnum</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_inscription_updater_curse_reinscription"></a>

## Function `curse_reinscription`



<pre><code><b>public</b> <b>fun</b> <a href="inscription_updater.md#0x4_inscription_updater_curse_reinscription">curse_reinscription</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_inscription_updater_curse_stutter"></a>

## Function `curse_stutter`



<pre><code><b>public</b> <b>fun</b> <a href="inscription_updater.md#0x4_inscription_updater_curse_stutter">curse_stutter</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_inscription_updater_curse_unrecognized_even_field"></a>

## Function `curse_unrecognized_even_field`



<pre><code><b>public</b> <b>fun</b> <a href="inscription_updater.md#0x4_inscription_updater_curse_unrecognized_even_field">curse_unrecognized_even_field</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>
