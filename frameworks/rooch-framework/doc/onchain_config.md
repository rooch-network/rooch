
<a name="0x3_onchain_config"></a>

# Module `0x3::onchain_config`



-  [Resource `OnchainConfig`](#0x3_onchain_config_OnchainConfig)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_onchain_config_genesis_init)
-  [Function `sequencer`](#0x3_onchain_config_sequencer)
-  [Function `update_framework_version`](#0x3_onchain_config_update_framework_version)
-  [Function `framework_version`](#0x3_onchain_config_framework_version)
-  [Function `onchain_config`](#0x3_onchain_config_onchain_config)
-  [Function `add_to_publishing_allowlist`](#0x3_onchain_config_add_to_publishing_allowlist)
-  [Function `remove_from_publishing_allowlist`](#0x3_onchain_config_remove_from_publishing_allowlist)


<pre><code><b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x2::features</a>;
<b>use</b> <a href="">0x2::move_module</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="chain_id.md#0x3_chain_id">0x3::chain_id</a>;
</code></pre>



<a name="0x3_onchain_config_OnchainConfig"></a>

## Resource `OnchainConfig`

OnchainConfig is framework configurations stored on chain.


<pre><code><b>struct</b> <a href="onchain_config.md#0x3_onchain_config_OnchainConfig">OnchainConfig</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_onchain_config_ErrorNotSequencer"></a>



<pre><code><b>const</b> <a href="onchain_config.md#0x3_onchain_config_ErrorNotSequencer">ErrorNotSequencer</a>: u64 = 1;
</code></pre>



<a name="0x3_onchain_config_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_genesis_init">genesis_init</a>(genesis_account: &<a href="">signer</a>, sequencer: <b>address</b>)
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



<a name="0x3_onchain_config_add_to_publishing_allowlist"></a>

## Function `add_to_publishing_allowlist`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_add_to_publishing_allowlist">add_to_publishing_allowlist</a>(<a href="">account</a>: &<a href="">signer</a>, publisher: <b>address</b>)
</code></pre>



<a name="0x3_onchain_config_remove_from_publishing_allowlist"></a>

## Function `remove_from_publishing_allowlist`



<pre><code><b>public</b> <b>fun</b> <a href="onchain_config.md#0x3_onchain_config_remove_from_publishing_allowlist">remove_from_publishing_allowlist</a>(<a href="">account</a>: &<a href="">signer</a>, publisher: <b>address</b>)
</code></pre>
