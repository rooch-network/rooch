
<a name="0x4_bbn_updater"></a>

# Module `0x4::bbn_updater`



-  [Constants](#@Constants_0)
-  [Function `is_possible_bbn_tx`](#0x4_bbn_updater_is_possible_bbn_tx)
-  [Function `process_bbn_tx_entry`](#0x4_bbn_updater_process_bbn_tx_entry)
-  [Function `is_expired`](#0x4_bbn_updater_is_expired)
-  [Function `clear_unbonded_stakes`](#0x4_bbn_updater_clear_unbonded_stakes)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="bbn.md#0x4_bbn">0x4::bbn</a>;
<b>use</b> <a href="bitcoin.md#0x4_bitcoin">0x4::bitcoin</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
<b>use</b> <a href="utxo.md#0x4_utxo">0x4::utxo</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_bbn_updater_ErrorTransactionNotFound"></a>



<pre><code><b>const</b> <a href="bbn_updater.md#0x4_bbn_updater_ErrorTransactionNotFound">ErrorTransactionNotFound</a>: u64 = 1;
</code></pre>



<a name="0x4_bbn_updater_is_possible_bbn_tx"></a>

## Function `is_possible_bbn_tx`

Check if the transaction is a possible Babylon transaction
If the transaction contains an OP_RETURN output with the correct tag, it is considered a possible Babylon transaction


<pre><code><b>public</b> <b>fun</b> <a href="bbn_updater.md#0x4_bbn_updater_is_possible_bbn_tx">is_possible_bbn_tx</a>(txid: <b>address</b>): bool
</code></pre>



<a name="0x4_bbn_updater_process_bbn_tx_entry"></a>

## Function `process_bbn_tx_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="bbn_updater.md#0x4_bbn_updater_process_bbn_tx_entry">process_bbn_tx_entry</a>(txid: <b>address</b>)
</code></pre>



<a name="0x4_bbn_updater_is_expired"></a>

## Function `is_expired`



<pre><code><b>public</b> <b>fun</b> <a href="bbn_updater.md#0x4_bbn_updater_is_expired">is_expired</a>(stake: &<a href="bbn.md#0x4_bbn_BBNStakeSeal">bbn::BBNStakeSeal</a>): bool
</code></pre>



<a name="0x4_bbn_updater_clear_unbonded_stakes"></a>

## Function `clear_unbonded_stakes`



<pre><code><b>public</b> entry <b>fun</b> <a href="bbn_updater.md#0x4_bbn_updater_clear_unbonded_stakes">clear_unbonded_stakes</a>(seal_obj_id: <a href="_ObjectID">object::ObjectID</a>)
</code></pre>
