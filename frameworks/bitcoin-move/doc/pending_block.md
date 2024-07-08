
<a name="0x4_pending_block"></a>

# Module `0x4::pending_block`

PendingStore is used to store the pending blocks and txs, and handle the reorg


-  [Resource `PendingBlock`](#0x4_pending_block_PendingBlock)
-  [Resource `PendingStore`](#0x4_pending_block_PendingStore)
-  [Struct `InprocessBlock`](#0x4_pending_block_InprocessBlock)
-  [Struct `ReorgEvent`](#0x4_pending_block_ReorgEvent)
-  [Struct `PendingTxs`](#0x4_pending_block_PendingTxs)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_pending_block_genesis_init)
-  [Function `add_pending_block`](#0x4_pending_block_add_pending_block)
-  [Function `process_pending_tx`](#0x4_pending_block_process_pending_tx)
-  [Function `finish_pending_tx`](#0x4_pending_block_finish_pending_tx)
-  [Function `finish_pending_block`](#0x4_pending_block_finish_pending_block)
-  [Function `inprocess_block_flotsams_mut`](#0x4_pending_block_inprocess_block_flotsams_mut)
-  [Function `inprocess_block_flotsams`](#0x4_pending_block_inprocess_block_flotsams)
-  [Function `inprocess_block_tx`](#0x4_pending_block_inprocess_block_tx)
-  [Function `inprocess_block_header`](#0x4_pending_block_inprocess_block_header)
-  [Function `inprocess_block_height`](#0x4_pending_block_inprocess_block_height)
-  [Function `get_ready_pending_txs`](#0x4_pending_block_get_ready_pending_txs)
-  [Function `get_best_block`](#0x4_pending_block_get_best_block)
-  [Function `get_reorg_block_count`](#0x4_pending_block_get_reorg_block_count)
-  [Function `update_reorg_block_count_for_local`](#0x4_pending_block_update_reorg_block_count_for_local)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x3::chain_id</a>;
<b>use</b> <a href="ord.md#0x4_ord">0x4::ord</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
</code></pre>



<a name="0x4_pending_block_PendingBlock"></a>

## Resource `PendingBlock`



<pre><code><b>struct</b> <a href="pending_block.md#0x4_pending_block_PendingBlock">PendingBlock</a> <b>has</b> key
</code></pre>



<a name="0x4_pending_block_PendingStore"></a>

## Resource `PendingStore`



<pre><code><b>struct</b> <a href="pending_block.md#0x4_pending_block_PendingStore">PendingStore</a> <b>has</b> key
</code></pre>



<a name="0x4_pending_block_InprocessBlock"></a>

## Struct `InprocessBlock`

InprocessBlock is used to store the block and txs that are being processed
This is a hot potato struct, can not be store and drop


<pre><code><b>struct</b> <a href="pending_block.md#0x4_pending_block_InprocessBlock">InprocessBlock</a>
</code></pre>



<a name="0x4_pending_block_ReorgEvent"></a>

## Struct `ReorgEvent`



<pre><code><b>struct</b> <a href="pending_block.md#0x4_pending_block_ReorgEvent">ReorgEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="0x4_pending_block_PendingTxs"></a>

## Struct `PendingTxs`



<pre><code><b>struct</b> <a href="pending_block.md#0x4_pending_block_PendingTxs">PendingTxs</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_pending_block_BLOCK_FLOTSAM_KEY"></a>



<pre><code><b>const</b> <a href="pending_block.md#0x4_pending_block_BLOCK_FLOTSAM_KEY">BLOCK_FLOTSAM_KEY</a>: <a href="">vector</a>&lt;u8&gt; = [98, 108, 111, 99, 107, 95, 102, 108, 111, 116, 115, 97, 109];
</code></pre>



<a name="0x4_pending_block_ErrorBlockAlreadyProcessed"></a>



<pre><code><b>const</b> <a href="pending_block.md#0x4_pending_block_ErrorBlockAlreadyProcessed">ErrorBlockAlreadyProcessed</a>: u64 = 1;
</code></pre>



<a name="0x4_pending_block_ErrorNeedToWaitMoreBlocks"></a>



<pre><code><b>const</b> <a href="pending_block.md#0x4_pending_block_ErrorNeedToWaitMoreBlocks">ErrorNeedToWaitMoreBlocks</a>: u64 = 5;
</code></pre>



<a name="0x4_pending_block_ErrorPendingBlockNotFinished"></a>



<pre><code><b>const</b> <a href="pending_block.md#0x4_pending_block_ErrorPendingBlockNotFinished">ErrorPendingBlockNotFinished</a>: u64 = 6;
</code></pre>



<a name="0x4_pending_block_ErrorPendingBlockNotFound"></a>



<pre><code><b>const</b> <a href="pending_block.md#0x4_pending_block_ErrorPendingBlockNotFound">ErrorPendingBlockNotFound</a>: u64 = 2;
</code></pre>



<a name="0x4_pending_block_ErrorPendingTxNotFound"></a>



<pre><code><b>const</b> <a href="pending_block.md#0x4_pending_block_ErrorPendingTxNotFound">ErrorPendingTxNotFound</a>: u64 = 3;
</code></pre>



<a name="0x4_pending_block_ErrorReorgFailed"></a>



<pre><code><b>const</b> <a href="pending_block.md#0x4_pending_block_ErrorReorgFailed">ErrorReorgFailed</a>: u64 = 4;
</code></pre>



<a name="0x4_pending_block_ErrorUnsupportedChain"></a>



<pre><code><b>const</b> <a href="pending_block.md#0x4_pending_block_ErrorUnsupportedChain">ErrorUnsupportedChain</a>: u64 = 7;
</code></pre>



<a name="0x4_pending_block_TX_IDS_KEY"></a>



<pre><code><b>const</b> <a href="pending_block.md#0x4_pending_block_TX_IDS_KEY">TX_IDS_KEY</a>: <a href="">vector</a>&lt;u8&gt; = [116, 120, 95, 105, 100, 115];
</code></pre>



<a name="0x4_pending_block_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="pending_block.md#0x4_pending_block_genesis_init">genesis_init</a>(reorg_block_count: u64)
</code></pre>



<a name="0x4_pending_block_add_pending_block"></a>

## Function `add_pending_block`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="pending_block.md#0x4_pending_block_add_pending_block">add_pending_block</a>(block_height: u64, block_hash: <b>address</b>, block: <a href="types.md#0x4_types_Block">types::Block</a>)
</code></pre>



<a name="0x4_pending_block_process_pending_tx"></a>

## Function `process_pending_tx`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="pending_block.md#0x4_pending_block_process_pending_tx">process_pending_tx</a>(block_hash: <b>address</b>, txid: <b>address</b>): <a href="pending_block.md#0x4_pending_block_InprocessBlock">pending_block::InprocessBlock</a>
</code></pre>



<a name="0x4_pending_block_finish_pending_tx"></a>

## Function `finish_pending_tx`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="pending_block.md#0x4_pending_block_finish_pending_tx">finish_pending_tx</a>(inprocess_block: <a href="pending_block.md#0x4_pending_block_InprocessBlock">pending_block::InprocessBlock</a>)
</code></pre>



<a name="0x4_pending_block_finish_pending_block"></a>

## Function `finish_pending_block`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="pending_block.md#0x4_pending_block_finish_pending_block">finish_pending_block</a>(inprocess_block: <a href="pending_block.md#0x4_pending_block_InprocessBlock">pending_block::InprocessBlock</a>): <a href="types.md#0x4_types_Header">types::Header</a>
</code></pre>



<a name="0x4_pending_block_inprocess_block_flotsams_mut"></a>

## Function `inprocess_block_flotsams_mut`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="pending_block.md#0x4_pending_block_inprocess_block_flotsams_mut">inprocess_block_flotsams_mut</a>(inprocess_block: &<b>mut</b> <a href="pending_block.md#0x4_pending_block_InprocessBlock">pending_block::InprocessBlock</a>): &<b>mut</b> <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_Flotsam">ord::Flotsam</a>&gt;
</code></pre>



<a name="0x4_pending_block_inprocess_block_flotsams"></a>

## Function `inprocess_block_flotsams`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="pending_block.md#0x4_pending_block_inprocess_block_flotsams">inprocess_block_flotsams</a>(inprocess_block: &<a href="pending_block.md#0x4_pending_block_InprocessBlock">pending_block::InprocessBlock</a>): <a href="">vector</a>&lt;<a href="ord.md#0x4_ord_Flotsam">ord::Flotsam</a>&gt;
</code></pre>



<a name="0x4_pending_block_inprocess_block_tx"></a>

## Function `inprocess_block_tx`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="pending_block.md#0x4_pending_block_inprocess_block_tx">inprocess_block_tx</a>(inprocess_block: &<a href="pending_block.md#0x4_pending_block_InprocessBlock">pending_block::InprocessBlock</a>): &<a href="types.md#0x4_types_Transaction">types::Transaction</a>
</code></pre>



<a name="0x4_pending_block_inprocess_block_header"></a>

## Function `inprocess_block_header`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="pending_block.md#0x4_pending_block_inprocess_block_header">inprocess_block_header</a>(inprocess_block: &<a href="pending_block.md#0x4_pending_block_InprocessBlock">pending_block::InprocessBlock</a>): &<a href="types.md#0x4_types_Header">types::Header</a>
</code></pre>



<a name="0x4_pending_block_inprocess_block_height"></a>

## Function `inprocess_block_height`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="pending_block.md#0x4_pending_block_inprocess_block_height">inprocess_block_height</a>(inprocess_block: &<a href="pending_block.md#0x4_pending_block_InprocessBlock">pending_block::InprocessBlock</a>): u64
</code></pre>



<a name="0x4_pending_block_get_ready_pending_txs"></a>

## Function `get_ready_pending_txs`

Get the pending txs which are ready to be processed


<pre><code><b>public</b> <b>fun</b> <a href="pending_block.md#0x4_pending_block_get_ready_pending_txs">get_ready_pending_txs</a>(): <a href="_Option">option::Option</a>&lt;<a href="pending_block.md#0x4_pending_block_PendingTxs">pending_block::PendingTxs</a>&gt;
</code></pre>



<a name="0x4_pending_block_get_best_block"></a>

## Function `get_best_block`



<pre><code><b>public</b> <b>fun</b> <a href="pending_block.md#0x4_pending_block_get_best_block">get_best_block</a>(): <a href="_Option">option::Option</a>&lt;<a href="types.md#0x4_types_BlockHeightHash">types::BlockHeightHash</a>&gt;
</code></pre>



<a name="0x4_pending_block_get_reorg_block_count"></a>

## Function `get_reorg_block_count`



<pre><code><b>public</b> <b>fun</b> <a href="pending_block.md#0x4_pending_block_get_reorg_block_count">get_reorg_block_count</a>(): u64
</code></pre>



<a name="0x4_pending_block_update_reorg_block_count_for_local"></a>

## Function `update_reorg_block_count_for_local`

Update the <code>reorg_block_count</code> config for local env to testing


<pre><code><b>public</b> entry <b>fun</b> <a href="pending_block.md#0x4_pending_block_update_reorg_block_count_for_local">update_reorg_block_count_for_local</a>(count: u64)
</code></pre>
