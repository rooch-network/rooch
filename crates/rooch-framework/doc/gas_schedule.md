
<a name="0x3_gas_schedule"></a>

# Module `0x3::gas_schedule`



-  [Struct `GasEntry`](#0x3_gas_schedule_GasEntry)
-  [Resource `GasSchedule`](#0x3_gas_schedule_GasSchedule)
-  [Function `gas_schedule_init`](#0x3_gas_schedule_gas_schedule_init)
-  [Function `get_gas_schedule`](#0x3_gas_schedule_get_gas_schedule)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::object</a>;
</code></pre>



<a name="0x3_gas_schedule_GasEntry"></a>

## Struct `GasEntry`



<pre><code>#[data_struct]
<b>struct</b> <a href="gas_schedule.md#0x3_gas_schedule_GasEntry">GasEntry</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_gas_schedule_GasSchedule"></a>

## Resource `GasSchedule`



<pre><code>#[data_struct]
<b>struct</b> <a href="gas_schedule.md#0x3_gas_schedule_GasSchedule">GasSchedule</a> <b>has</b> <b>copy</b>, drop, key
</code></pre>



<a name="0x3_gas_schedule_gas_schedule_init"></a>

## Function `gas_schedule_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="gas_schedule.md#0x3_gas_schedule_gas_schedule_init">gas_schedule_init</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, _genesis_account: &<a href="">signer</a>, gas_schedule_blob: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_gas_schedule_get_gas_schedule"></a>

## Function `get_gas_schedule`



<pre><code><b>public</b> <b>fun</b> <a href="gas_schedule.md#0x3_gas_schedule_get_gas_schedule">get_gas_schedule</a>(ctx: &<a href="_Context">context::Context</a>): &<a href="gas_schedule.md#0x3_gas_schedule_GasSchedule">gas_schedule::GasSchedule</a>
</code></pre>
