
<a name="0x4_inscription_updater"></a>

# Module `0x4::inscription_updater`

The move version inscription_updater
https://github.com/ordinals/ord/blob/e59bd3e73d30ed9bc0b252ba2084bba670d6b0db/src/index/updater/inscription_updater.rs


-  [Struct `FlotsamNew`](#0x4_inscription_updater_FlotsamNew)
-  [Struct `Flotsam`](#0x4_inscription_updater_Flotsam)
-  [Struct `Envelope`](#0x4_inscription_updater_Envelope)
-  [Struct `InscriptionRecord`](#0x4_inscription_updater_InscriptionRecord)
-  [Struct `InscriptionCreatedEvent`](#0x4_inscription_updater_InscriptionCreatedEvent)
-  [Struct `InscriptionTransferredEvent`](#0x4_inscription_updater_InscriptionTransferredEvent)
-  [Struct `InscriptionUpdater`](#0x4_inscription_updater_InscriptionUpdater)
-  [Struct `Location`](#0x4_inscription_updater_Location)
-  [Struct `Range`](#0x4_inscription_updater_Range)
-  [Constants](#@Constants_0)
-  [Function `process_tx`](#0x4_inscription_updater_process_tx)
-  [Function `need_process_oridinals`](#0x4_inscription_updater_need_process_oridinals)
-  [Function `parse_inscription_from_tx`](#0x4_inscription_updater_parse_inscription_from_tx)


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



<a name="0x4_inscription_updater_Envelope"></a>

## Struct `Envelope`



<pre><code><b>struct</b> <a href="inscription_updater.md#0x4_inscription_updater_Envelope">Envelope</a>&lt;T&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_inscription_updater_InscriptionRecord"></a>

## Struct `InscriptionRecord`



<pre><code><b>struct</b> <a href="inscription_updater.md#0x4_inscription_updater_InscriptionRecord">InscriptionRecord</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_inscription_updater_InscriptionCreatedEvent"></a>

## Struct `InscriptionCreatedEvent`



<pre><code><b>struct</b> <a href="inscription_updater.md#0x4_inscription_updater_InscriptionCreatedEvent">InscriptionCreatedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_inscription_updater_InscriptionTransferredEvent"></a>

## Struct `InscriptionTransferredEvent`



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



<a name="@Constants_0"></a>

## Constants


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



<a name="0x4_inscription_updater_parse_inscription_from_tx"></a>

## Function `parse_inscription_from_tx`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="inscription_updater.md#0x4_inscription_updater_parse_inscription_from_tx">parse_inscription_from_tx</a>(_tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>): <a href="">vector</a>&lt;<a href="inscription_updater.md#0x4_inscription_updater_Envelope">inscription_updater::Envelope</a>&lt;<a href="inscription_updater.md#0x4_inscription_updater_InscriptionRecord">inscription_updater::InscriptionRecord</a>&gt;&gt;
</code></pre>
