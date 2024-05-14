
<a name="0x2_gas_schedule"></a>

# Module `0x2::gas_schedule`



-  [Struct `GasScheduleUpdated`](#0x2_gas_schedule_GasScheduleUpdated)
-  [Struct `GasEntry`](#0x2_gas_schedule_GasEntry)
-  [Resource `GasSchedule`](#0x2_gas_schedule_GasSchedule)
-  [Struct `GasScheduleConfig`](#0x2_gas_schedule_GasScheduleConfig)
-  [Constants](#@Constants_0)
-  [Function `initial_max_gas_amount`](#0x2_gas_schedule_initial_max_gas_amount)
-  [Function `genesis_init`](#0x2_gas_schedule_genesis_init)
-  [Function `new_gas_schedule_config`](#0x2_gas_schedule_new_gas_schedule_config)
-  [Function `new_gas_entry`](#0x2_gas_schedule_new_gas_entry)
-  [Function `update_gas_schedule`](#0x2_gas_schedule_update_gas_schedule)
-  [Function `gas_schedule`](#0x2_gas_schedule_gas_schedule)
-  [Function `gas_schedule_max_gas_amount`](#0x2_gas_schedule_gas_schedule_max_gas_amount)
-  [Function `gas_schedule_version`](#0x2_gas_schedule_gas_schedule_version)
-  [Function `gas_schedule_entries`](#0x2_gas_schedule_gas_schedule_entries)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="core_addresses.md#0x2_core_addresses">0x2::core_addresses</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="signer.md#0x2_signer">0x2::signer</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_gas_schedule_GasScheduleUpdated"></a>

## Struct `GasScheduleUpdated`



<pre><code><b>struct</b> <a href="gas_schedule.md#0x2_gas_schedule_GasScheduleUpdated">GasScheduleUpdated</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x2_gas_schedule_GasEntry"></a>

## Struct `GasEntry`



<pre><code>#[data_struct]
<b>struct</b> <a href="gas_schedule.md#0x2_gas_schedule_GasEntry">GasEntry</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x2_gas_schedule_GasSchedule"></a>

## Resource `GasSchedule`



<pre><code><b>struct</b> <a href="gas_schedule.md#0x2_gas_schedule_GasSchedule">GasSchedule</a> <b>has</b> key
</code></pre>



<a name="0x2_gas_schedule_GasScheduleConfig"></a>

## Struct `GasScheduleConfig`



<pre><code>#[data_struct]
<b>struct</b> <a href="gas_schedule.md#0x2_gas_schedule_GasScheduleConfig">GasScheduleConfig</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_gas_schedule_ErrorInvalidGasScheduleEntries"></a>



<pre><code><b>const</b> <a href="gas_schedule.md#0x2_gas_schedule_ErrorInvalidGasScheduleEntries">ErrorInvalidGasScheduleEntries</a>: u64 = 1;
</code></pre>



<a name="0x2_gas_schedule_InitialMaxGasAmount"></a>



<pre><code><b>const</b> <a href="gas_schedule.md#0x2_gas_schedule_InitialMaxGasAmount">InitialMaxGasAmount</a>: u64 = 1000000000;
</code></pre>



<a name="0x2_gas_schedule_initial_max_gas_amount"></a>

## Function `initial_max_gas_amount`



<pre><code><b>public</b> <b>fun</b> <a href="gas_schedule.md#0x2_gas_schedule_initial_max_gas_amount">initial_max_gas_amount</a>(): u64
</code></pre>



<a name="0x2_gas_schedule_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_schedule.md#0x2_gas_schedule_genesis_init">genesis_init</a>(gas_schedule_config: <a href="gas_schedule.md#0x2_gas_schedule_GasScheduleConfig">gas_schedule::GasScheduleConfig</a>)
</code></pre>



<a name="0x2_gas_schedule_new_gas_schedule_config"></a>

## Function `new_gas_schedule_config`



<pre><code><b>public</b> <b>fun</b> <a href="gas_schedule.md#0x2_gas_schedule_new_gas_schedule_config">new_gas_schedule_config</a>(max_gas_amount: u64, entries: <a href="">vector</a>&lt;<a href="gas_schedule.md#0x2_gas_schedule_GasEntry">gas_schedule::GasEntry</a>&gt;): <a href="gas_schedule.md#0x2_gas_schedule_GasScheduleConfig">gas_schedule::GasScheduleConfig</a>
</code></pre>



<a name="0x2_gas_schedule_new_gas_entry"></a>

## Function `new_gas_entry`



<pre><code><b>public</b> <b>fun</b> <a href="gas_schedule.md#0x2_gas_schedule_new_gas_entry">new_gas_entry</a>(key: <a href="_String">ascii::String</a>, val: u64): <a href="gas_schedule.md#0x2_gas_schedule_GasEntry">gas_schedule::GasEntry</a>
</code></pre>



<a name="0x2_gas_schedule_update_gas_schedule"></a>

## Function `update_gas_schedule`



<pre><code><b>public</b> <b>fun</b> <a href="gas_schedule.md#0x2_gas_schedule_update_gas_schedule">update_gas_schedule</a>(<a href="account.md#0x2_account">account</a>: &<a href="">signer</a>, gas_schedule_config: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x2_gas_schedule_gas_schedule"></a>

## Function `gas_schedule`



<pre><code><b>public</b> <b>fun</b> <a href="gas_schedule.md#0x2_gas_schedule">gas_schedule</a>(): &<a href="gas_schedule.md#0x2_gas_schedule_GasSchedule">gas_schedule::GasSchedule</a>
</code></pre>



<a name="0x2_gas_schedule_gas_schedule_max_gas_amount"></a>

## Function `gas_schedule_max_gas_amount`



<pre><code><b>public</b> <b>fun</b> <a href="gas_schedule.md#0x2_gas_schedule_gas_schedule_max_gas_amount">gas_schedule_max_gas_amount</a>(schedule: &<a href="gas_schedule.md#0x2_gas_schedule_GasSchedule">gas_schedule::GasSchedule</a>): u64
</code></pre>



<a name="0x2_gas_schedule_gas_schedule_version"></a>

## Function `gas_schedule_version`



<pre><code><b>public</b> <b>fun</b> <a href="gas_schedule.md#0x2_gas_schedule_gas_schedule_version">gas_schedule_version</a>(schedule: &<a href="gas_schedule.md#0x2_gas_schedule_GasSchedule">gas_schedule::GasSchedule</a>): u64
</code></pre>



<a name="0x2_gas_schedule_gas_schedule_entries"></a>

## Function `gas_schedule_entries`



<pre><code><b>public</b> <b>fun</b> <a href="gas_schedule.md#0x2_gas_schedule_gas_schedule_entries">gas_schedule_entries</a>(schedule: &<a href="gas_schedule.md#0x2_gas_schedule_GasSchedule">gas_schedule::GasSchedule</a>): &<a href="">vector</a>&lt;<a href="gas_schedule.md#0x2_gas_schedule_GasEntry">gas_schedule::GasEntry</a>&gt;
</code></pre>
