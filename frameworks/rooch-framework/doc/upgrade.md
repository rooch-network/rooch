
<a name="0x3_upgrade"></a>

# Module `0x3::upgrade`



-  [Struct `StdlibPackage`](#0x3_upgrade_StdlibPackage)
-  [Struct `Stdlib`](#0x3_upgrade_Stdlib)
-  [Resource `UpgradeCap`](#0x3_upgrade_UpgradeCap)
-  [Struct `FrameworkUpgradeEvent`](#0x3_upgrade_FrameworkUpgradeEvent)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_upgrade_genesis_init)
-  [Function `new_upgrade_cap_for_upgrade`](#0x3_upgrade_new_upgrade_cap_for_upgrade)
-  [Function `upgrade_entry`](#0x3_upgrade_upgrade_entry)
-  [Function `upgrade_v2_entry`](#0x3_upgrade_upgrade_v2_entry)
-  [Function `upgrade_gas_schedule`](#0x3_upgrade_upgrade_gas_schedule)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::gas_schedule</a>;
<b>use</b> <a href="">0x2::module_store</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="onchain_config.md#0x3_onchain_config">0x3::onchain_config</a>;
</code></pre>



<a name="0x3_upgrade_StdlibPackage"></a>

## Struct `StdlibPackage`



<pre><code>#[data_struct]
<b>struct</b> <a href="upgrade.md#0x3_upgrade_StdlibPackage">StdlibPackage</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_upgrade_Stdlib"></a>

## Struct `Stdlib`

Collection of framework packages. The struct must keep the same with the Rust definition.


<pre><code>#[data_struct]
<b>struct</b> <a href="upgrade.md#0x3_upgrade_Stdlib">Stdlib</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_upgrade_UpgradeCap"></a>

## Resource `UpgradeCap`

Upgrade capability


<pre><code><b>struct</b> <a href="upgrade.md#0x3_upgrade_UpgradeCap">UpgradeCap</a> <b>has</b> store, key
</code></pre>



<a name="0x3_upgrade_FrameworkUpgradeEvent"></a>

## Struct `FrameworkUpgradeEvent`

Event for framework upgrades


<pre><code><b>struct</b> <a href="upgrade.md#0x3_upgrade_FrameworkUpgradeEvent">FrameworkUpgradeEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_upgrade_ErrorNotSequencer"></a>



<pre><code><b>const</b> <a href="upgrade.md#0x3_upgrade_ErrorNotSequencer">ErrorNotSequencer</a>: u64 = 1;
</code></pre>



<a name="0x3_upgrade_BitcoinMoveAccount"></a>



<pre><code><b>const</b> <a href="upgrade.md#0x3_upgrade_BitcoinMoveAccount">BitcoinMoveAccount</a>: <b>address</b> = 0x4;
</code></pre>



<a name="0x3_upgrade_ErrorCapabilityAlreadyExists"></a>



<pre><code><b>const</b> <a href="upgrade.md#0x3_upgrade_ErrorCapabilityAlreadyExists">ErrorCapabilityAlreadyExists</a>: u64 = 2;
</code></pre>



<a name="0x3_upgrade_ErrorNoAccess"></a>



<pre><code><b>const</b> <a href="upgrade.md#0x3_upgrade_ErrorNoAccess">ErrorNoAccess</a>: u64 = 3;
</code></pre>



<a name="0x3_upgrade_MoveStdAccount"></a>



<pre><code><b>const</b> <a href="upgrade.md#0x3_upgrade_MoveStdAccount">MoveStdAccount</a>: <b>address</b> = 0x1;
</code></pre>



<a name="0x3_upgrade_MoveosStdAccount"></a>



<pre><code><b>const</b> <a href="upgrade.md#0x3_upgrade_MoveosStdAccount">MoveosStdAccount</a>: <b>address</b> = 0x2;
</code></pre>



<a name="0x3_upgrade_RoochFrameworkAccount"></a>



<pre><code><b>const</b> <a href="upgrade.md#0x3_upgrade_RoochFrameworkAccount">RoochFrameworkAccount</a>: <b>address</b> = 0x3;
</code></pre>



<a name="0x3_upgrade_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="upgrade.md#0x3_upgrade_genesis_init">genesis_init</a>(sequencer: <b>address</b>)
</code></pre>



<a name="0x3_upgrade_new_upgrade_cap_for_upgrade"></a>

## Function `new_upgrade_cap_for_upgrade`

Aqcuires the upgrade capability for the sequencer.
Only used for framework upgrading.
TODO: remove this function when reset genesis.


<pre><code><b>public</b> <b>fun</b> <a href="upgrade.md#0x3_upgrade_new_upgrade_cap_for_upgrade">new_upgrade_cap_for_upgrade</a>(sequencer: &<a href="">signer</a>)
</code></pre>



<a name="0x3_upgrade_upgrade_entry"></a>

## Function `upgrade_entry`



<pre><code>entry <b>fun</b> <a href="upgrade.md#0x3_upgrade_upgrade_entry">upgrade_entry</a>(<a href="">account</a>: &<a href="">signer</a>, move_std_bundles: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, moveos_std_bundles: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, rooch_framework_bundles: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, bitcoin_move_bundles: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;)
</code></pre>



<a name="0x3_upgrade_upgrade_v2_entry"></a>

## Function `upgrade_v2_entry`

Upgrade the framework package
<code>package_bytes</code> is the serialized <code><a href="upgrade.md#0x3_upgrade_StdlibPackage">StdlibPackage</a></code>


<pre><code>entry <b>fun</b> <a href="upgrade.md#0x3_upgrade_upgrade_v2_entry">upgrade_v2_entry</a>(<a href="">account</a>: &<a href="">signer</a>, package_bytes: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_upgrade_upgrade_gas_schedule"></a>

## Function `upgrade_gas_schedule`



<pre><code>entry <b>fun</b> <a href="upgrade.md#0x3_upgrade_upgrade_gas_schedule">upgrade_gas_schedule</a>(<a href="">account</a>: &<a href="">signer</a>, gas_schedule_config: <a href="">vector</a>&lt;u8&gt;)
</code></pre>
