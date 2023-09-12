
<a name="0x3_timestamp"></a>

# Module `0x3::timestamp`

This module keeps a global wall clock that stores the current Unix time in microseconds.
It interacts with the other modules in the following ways:
* genesis: to initialize the timestamp
* L1 block: update the timestamp via L1s block header timestamp
* TickTransaction: update the timestamp via the time offset in the TickTransaction(TODO)


-  [Resource `CurrentTimeMicroseconds`](#0x3_timestamp_CurrentTimeMicroseconds)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_timestamp_genesis_init)
-  [Function `update_global_time`](#0x3_timestamp_update_global_time)
-  [Function `try_update_global_time`](#0x3_timestamp_try_update_global_time)
-  [Function `now_microseconds`](#0x3_timestamp_now_microseconds)
-  [Function `now_seconds`](#0x3_timestamp_now_seconds)
-  [Function `seconds_to_microseconds`](#0x3_timestamp_seconds_to_microseconds)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
</code></pre>



<a name="0x3_timestamp_CurrentTimeMicroseconds"></a>

## Resource `CurrentTimeMicroseconds`

A singleton resource holding the current Unix time in microseconds


<pre><code><b>struct</b> <a href="timestamp.md#0x3_timestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>microseconds: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_timestamp_ErrorInvalidTimestamp"></a>

An invalid timestamp was provided


<pre><code><b>const</b> <a href="timestamp.md#0x3_timestamp_ErrorInvalidTimestamp">ErrorInvalidTimestamp</a>: u64 = 1;
</code></pre>



<a name="0x3_timestamp_MICRO_CONVERSION_FACTOR"></a>

Conversion factor between seconds and microseconds


<pre><code><b>const</b> <a href="timestamp.md#0x3_timestamp_MICRO_CONVERSION_FACTOR">MICRO_CONVERSION_FACTOR</a>: u64 = 1000000;
</code></pre>



<a name="0x3_timestamp_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timestamp.md#0x3_timestamp_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, genesis_account: &<a href="">signer</a>, initial_time_microseconds: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timestamp.md#0x3_timestamp_genesis_init">genesis_init</a>(ctx: &<b>mut</b> StorageContext, genesis_account: &<a href="">signer</a>, initial_time_microseconds: u64) {
    <b>let</b> current_time = <a href="timestamp.md#0x3_timestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> { microseconds: initial_time_microseconds };
    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, genesis_account, current_time);
}
</code></pre>



</details>

<a name="0x3_timestamp_update_global_time"></a>

## Function `update_global_time`

Updates the wall clock time, if the new time is smaller than the current time, aborts.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timestamp.md#0x3_timestamp_update_global_time">update_global_time</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="timestamp.md#0x3_timestamp">timestamp</a>: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timestamp.md#0x3_timestamp_update_global_time">update_global_time</a>(ctx: &<b>mut</b> StorageContext,<a href="timestamp.md#0x3_timestamp">timestamp</a>: u64) {
    <b>let</b> global_timer = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="timestamp.md#0x3_timestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(ctx, @rooch_framework);
    <b>let</b> now = global_timer.microseconds;
    <b>assert</b>!(now &lt; <a href="timestamp.md#0x3_timestamp">timestamp</a>, <a href="_invalid_argument">error::invalid_argument</a>(<a href="timestamp.md#0x3_timestamp_ErrorInvalidTimestamp">ErrorInvalidTimestamp</a>));
    global_timer.microseconds = <a href="timestamp.md#0x3_timestamp">timestamp</a>;
}
</code></pre>



</details>

<a name="0x3_timestamp_try_update_global_time"></a>

## Function `try_update_global_time`

Tries to update the wall clock time, if the new time is smaller than the current time, ignores the update, and returns false.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timestamp.md#0x3_timestamp_try_update_global_time">try_update_global_time</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="timestamp.md#0x3_timestamp">timestamp</a>: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timestamp.md#0x3_timestamp_try_update_global_time">try_update_global_time</a>(ctx: &<b>mut</b> StorageContext, <a href="timestamp.md#0x3_timestamp">timestamp</a>: u64) : bool {
    <b>let</b> global_timer = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="timestamp.md#0x3_timestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(ctx, @rooch_framework);
    <b>let</b> now = global_timer.microseconds;
    <b>if</b>(now &lt; <a href="timestamp.md#0x3_timestamp">timestamp</a>) {
        global_timer.microseconds = <a href="timestamp.md#0x3_timestamp">timestamp</a>;
        <b>true</b>
    }<b>else</b>{
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x3_timestamp_now_microseconds"></a>

## Function `now_microseconds`

Gets the current time in microseconds.


<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x3_timestamp_now_microseconds">now_microseconds</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x3_timestamp_now_microseconds">now_microseconds</a>(ctx: &StorageContext): u64 {
    <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="timestamp.md#0x3_timestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(ctx, @rooch_framework).microseconds
}
</code></pre>



</details>

<a name="0x3_timestamp_now_seconds"></a>

## Function `now_seconds`

Gets the current time in seconds.


<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x3_timestamp_now_seconds">now_seconds</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x3_timestamp_now_seconds">now_seconds</a>(ctx: &StorageContext): u64 {
    <a href="timestamp.md#0x3_timestamp_now_microseconds">now_microseconds</a>(ctx) / <a href="timestamp.md#0x3_timestamp_MICRO_CONVERSION_FACTOR">MICRO_CONVERSION_FACTOR</a>
}
</code></pre>



</details>

<a name="0x3_timestamp_seconds_to_microseconds"></a>

## Function `seconds_to_microseconds`



<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x3_timestamp_seconds_to_microseconds">seconds_to_microseconds</a>(seconds: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x3_timestamp_seconds_to_microseconds">seconds_to_microseconds</a>(seconds: u64): u64 {
    seconds * <a href="timestamp.md#0x3_timestamp_MICRO_CONVERSION_FACTOR">MICRO_CONVERSION_FACTOR</a>
}
</code></pre>



</details>
