
<a name="0x3_upgrade"></a>

# Module `0x3::upgrade`



-  [Struct `FrameworkUpgradeEvent`](#0x3_upgrade_FrameworkUpgradeEvent)
-  [Constants](#@Constants_0)
-  [Function `upgrade_entry`](#0x3_upgrade_upgrade_entry)


<pre><code><b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="account.md#0x3_account">0x3::account</a>;
<b>use</b> <a href="onchain_config.md#0x3_onchain_config">0x3::onchain_config</a>;
</code></pre>



<a name="0x3_upgrade_FrameworkUpgradeEvent"></a>

## Struct `FrameworkUpgradeEvent`

Event for framework upgrades


<pre><code><b>struct</b> <a href="upgrade.md#0x3_upgrade_FrameworkUpgradeEvent">FrameworkUpgradeEvent</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_upgrade_ErrorNotSequencer"></a>



<pre><code><b>const</b> <a href="upgrade.md#0x3_upgrade_ErrorNotSequencer">ErrorNotSequencer</a>: u64 = 1;
</code></pre>



<a name="0x3_upgrade_upgrade_entry"></a>

## Function `upgrade_entry`



<pre><code>entry <b>fun</b> <a href="upgrade.md#0x3_upgrade_upgrade_entry">upgrade_entry</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, move_std_bundles: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, moveos_std_bundles: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, rooch_framework_bundles: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;)
</code></pre>
