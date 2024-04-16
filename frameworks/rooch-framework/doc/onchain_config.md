
<a name="0x3_onchain_config"></a>

# Module `0x3::onchain_config`



-  [Struct `GasScheduleUpdated`](#0x3_onchain_config_GasScheduleUpdated)
-  [Struct `GasEntry`](#0x3_onchain_config_GasEntry)
-  [Resource `GasSchedule`](#0x3_onchain_config_GasSchedule)
-  [Struct `GasScheduleConfig`](#0x3_onchain_config_GasScheduleConfig)
-  [Resource `OnchainConfig`](#0x3_onchain_config_OnchainConfig)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_onchain_config_genesis_init)
-  [Function `new_gas_schedule_config`](#0x3_onchain_config_new_gas_schedule_config)
-  [Function `new_gas_entry`](#0x3_onchain_config_new_gas_entry)
-  [Function `sequencer`](#0x3_onchain_config_sequencer)
-  [Function `update_framework_version`](#0x3_onchain_config_update_framework_version)
-  [Function `framework_version`](#0x3_onchain_config_framework_version)
-  [Function `onchain_config`](#0x3_onchain_config_onchain_config)
-  [Function `update_onchain_gas_schedule_entry`](#0x3_onchain_config_update_onchain_gas_schedule_entry)
-  [Function `onchain_gas_schedule`](#0x3_onchain_config_onchain_gas_schedule)
-  [Function `gas_schedule_version`](#0x3_onchain_config_gas_schedule_version)
-  [Function `gas_schedule_entries`](#0x3_onchain_config_gas_schedule_entries)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::features</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="chain_id.md#0x3_chain_id">0x3::chain_id</a>;
</code></pre>



<a name="0x3_onchain_config_GasScheduleUpdated"></a>

## Struct `GasScheduleUpdated`



<pre><code><b>struct</b> <a href="onchain_config.md#0x3_onchain_config_GasScheduleUpdated">GasScheduleUpdated</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_onchain_config_GasEntry"></a>

## Struct `GasEntry`



<pre><code>#[data_struct]
<b>struct</b> <a href="onchain_config.md#0x3_onchain_config_GasEntry">GasEntry</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_onchain_config_GasSchedule"></a>

## Resource `GasSchedule`



<pre><code><b>struct</b> <a href="onchain_config.md#0x3_onchain_config_GasSchedule">GasSchedule</a> <b>has</b> key
</code></pre>



<a name="0x3_onchain_config_GasScheduleConfig"></a>

## Struct `GasScheduleConfig`



<pre><code>#[data_struct]
<b>struct</b> <a href="onchain_config.md#0x3_onchain_config_GasScheduleConfig">GasScheduleConfig</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_onchain_config_OnchainConfig"></a>

## Resource `OnchainConfig`

OnchainConfig is framework configurations stored on chain.


<pre><code><b>struct</b> <a href="onchain_config.md#0x3_onchain_config_OnchainConfig">OnchainConfig</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_onchain_config_ErrorInvalidGasScheduleEntries"></a>



<pre><code><b>const</b> <a href="onchain_config.md#0x3_onchain_config_ErrorInvalidGasScheduleEntries">ErrorInvalidGasScheduleEntries</a>: u64 = 2;
</code></pre>



<a name="0x3_onchain_config_ErrorNotSequencer"></a>



<pre><code><b>const</b> <a href="onchain_config.md#0x3_onchain_config_ErrorNotSequencer">ErrorNotSequencer</a>: u64 = 1;
</code></pre>



<a name="0x3_onchain_config_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_genesis_init">genesis_init</a>(genesis_account: &<a href="">signer</a>, sequencer: <b>address</b>, gas_schedule_config: <a href="onchain_config.md#0x3_onchain_config_GasScheduleConfig">onchain_config::GasScheduleConfig</a>)
</code></pre>



<a name="0x3_onchain_config_new_gas_schedule_config"></a>

## Function `new_gas_schedule_config`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_new_gas_schedule_config">new_gas_schedule_config</a>(entries: <a href="">vector</a>&lt;<a href="onchain_config.md#0x3_onchain_config_GasEntry">onchain_config::GasEntry</a>&gt;): <a href="onchain_config.md#0x3_onchain_config_GasScheduleConfig">onchain_config::GasScheduleConfig</a>
</code></pre>



<a name="0x3_onchain_config_new_gas_entry"></a>

## Function `new_gas_entry`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_new_gas_entry">new_gas_entry</a>(key: <a href="_String">ascii::String</a>, val: u64): <a href="onchain_config.md#0x3_onchain_config_GasEntry">onchain_config::GasEntry</a>
</code></pre>



<a name="0x3_onchain_config_sequencer"></a>

## Function `sequencer`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_sequencer">sequencer</a>(): <b>address</b>
</code></pre>



<a name="0x3_onchain_config_update_framework_version"></a>

## Function `update_framework_version`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_update_framework_version">update_framework_version</a>()
</code></pre>



<a name="0x3_onchain_config_framework_version"></a>

## Function `framework_version`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_framework_version">framework_version</a>(): u64
</code></pre>



<a name="0x3_onchain_config_onchain_config"></a>

## Function `onchain_config`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config">onchain_config</a>(): &<a href="onchain_config.md#0x3_onchain_config_OnchainConfig">onchain_config::OnchainConfig</a>
</code></pre>



<a name="0x3_onchain_config_update_onchain_gas_schedule_entry"></a>

## Function `update_onchain_gas_schedule_entry`



<pre><code>entry <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_update_onchain_gas_schedule_entry">update_onchain_gas_schedule_entry</a>(<a href="">account</a>: &<a href="">signer</a>, gas_schedule_config: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_onchain_config_onchain_gas_schedule"></a>

## Function `onchain_gas_schedule`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_onchain_gas_schedule">onchain_gas_schedule</a>(): &<a href="onchain_config.md#0x3_onchain_config_GasSchedule">onchain_config::GasSchedule</a>
</code></pre>



<a name="0x3_onchain_config_gas_schedule_version"></a>

## Function `gas_schedule_version`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_gas_schedule_version">gas_schedule_version</a>(schedule: &<a href="onchain_config.md#0x3_onchain_config_GasSchedule">onchain_config::GasSchedule</a>): u64
</code></pre>



<a name="0x3_onchain_config_gas_schedule_entries"></a>

## Function `gas_schedule_entries`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_gas_schedule_entries">gas_schedule_entries</a>(schedule: &<a href="onchain_config.md#0x3_onchain_config_GasSchedule">onchain_config::GasSchedule</a>): &<a href="">vector</a>&lt;<a href="onchain_config.md#0x3_onchain_config_GasEntry">onchain_config::GasEntry</a>&gt;
</code></pre>
