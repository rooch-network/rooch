
<a name="0x3_upgrade"></a>

# Module `0x3::upgrade`



-  [Struct `GasUpgradeEvent`](#0x3_upgrade_GasUpgradeEvent)
-  [Constants](#@Constants_0)
-  [Function `upgrade_gas_schedule`](#0x3_upgrade_upgrade_gas_schedule)


<pre><code><b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::gas_schedule</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="onchain_config.md#0x3_onchain_config">0x3::onchain_config</a>;
</code></pre>



<a name="0x3_upgrade_GasUpgradeEvent"></a>

## Struct `GasUpgradeEvent`

Event for framework upgrades


<pre><code><b>struct</b> <a href="upgrade.md#0x3_upgrade_GasUpgradeEvent">GasUpgradeEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_upgrade_MoveosStdAccount"></a>



<pre><code><b>const</b> <a href="upgrade.md#0x3_upgrade_MoveosStdAccount">MoveosStdAccount</a>: <b>address</b> = 0x2;
</code></pre>



<a name="0x3_upgrade_upgrade_gas_schedule"></a>

## Function `upgrade_gas_schedule`



<pre><code>entry <b>fun</b> <a href="upgrade.md#0x3_upgrade_upgrade_gas_schedule">upgrade_gas_schedule</a>(<a href="">account</a>: &<a href="">signer</a>, gas_schedule_config: <a href="">vector</a>&lt;u8&gt;)
</code></pre>
