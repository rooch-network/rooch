
<a name="0x3_onchain_config"></a>

# Module `0x3::onchain_config`



-  [Struct `GasEntry`](#0x3_onchain_config_GasEntry)
-  [Resource `GasSchedule`](#0x3_onchain_config_GasSchedule)
-  [Resource `OnchainConfig`](#0x3_onchain_config_OnchainConfig)
-  [Function `genesis_init`](#0x3_onchain_config_genesis_init)
-  [Function `sequencer`](#0x3_onchain_config_sequencer)
-  [Function `update_framework_version`](#0x3_onchain_config_update_framework_version)
-  [Function `framework_version`](#0x3_onchain_config_framework_version)
-  [Function `onchain_config`](#0x3_onchain_config_onchain_config)
-  [Function `onchain_gas_schedule`](#0x3_onchain_config_onchain_gas_schedule)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::object</a>;
</code></pre>



<a name="0x3_onchain_config_GasEntry"></a>

## Struct `GasEntry`



<pre><code>#[data_struct]
<b>struct</b> <a href="onchain_config.md#0x3_onchain_config_GasEntry">GasEntry</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_onchain_config_GasSchedule"></a>

## Resource `GasSchedule`



<pre><code>#[data_struct]
<b>struct</b> <a href="onchain_config.md#0x3_onchain_config_GasSchedule">GasSchedule</a> <b>has</b> <b>copy</b>, drop, store, key
</code></pre>



<a name="0x3_onchain_config_OnchainConfig"></a>

## Resource `OnchainConfig`

OnchainConfig is framework configurations stored on chain.


<pre><code><b>struct</b> <a href="onchain_config.md#0x3_onchain_config_OnchainConfig">OnchainConfig</a> <b>has</b> key
</code></pre>



<a name="0x3_onchain_config_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, _genesis_account: &<a href="">signer</a>, sequencer: <b>address</b>, gas_schedule_blob: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_onchain_config_sequencer"></a>

## Function `sequencer`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_sequencer">sequencer</a>(ctx: &<a href="_Context">context::Context</a>): <b>address</b>
</code></pre>



<a name="0x3_onchain_config_update_framework_version"></a>

## Function `update_framework_version`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_update_framework_version">update_framework_version</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>)
</code></pre>



<a name="0x3_onchain_config_framework_version"></a>

## Function `framework_version`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_framework_version">framework_version</a>(ctx: &<a href="_Context">context::Context</a>): u64
</code></pre>



<a name="0x3_onchain_config_onchain_config"></a>

## Function `onchain_config`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config">onchain_config</a>(ctx: &<a href="_Context">context::Context</a>): &<a href="onchain_config.md#0x3_onchain_config_OnchainConfig">onchain_config::OnchainConfig</a>
</code></pre>



<a name="0x3_onchain_config_onchain_gas_schedule"></a>

## Function `onchain_gas_schedule`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_onchain_gas_schedule">onchain_gas_schedule</a>(ctx: &<a href="_Context">context::Context</a>): &<a href="onchain_config.md#0x3_onchain_config_GasSchedule">onchain_config::GasSchedule</a>
</code></pre>
