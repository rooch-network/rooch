
<a name="0x3_timestamp"></a>

# Module `0x3::timestamp`

This module keeps a global wall clock that stores the current Unix time in milliseconds.
It interacts with the other modules in the following ways:
* genesis: to initialize the timestamp
* L1 block: update the timestamp via L1s block header timestamp
* TickTransaction: update the timestamp via the time offset in the TickTransaction(TODO)


-  [Resource `Timestamp`](#0x3_timestamp_Timestamp)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_timestamp_genesis_init)
-  [Function `update_global_time`](#0x3_timestamp_update_global_time)
-  [Function `try_update_global_time`](#0x3_timestamp_try_update_global_time)
-  [Function `timestamp`](#0x3_timestamp_timestamp)
-  [Function `milliseconds`](#0x3_timestamp_milliseconds)
-  [Function `seconds`](#0x3_timestamp_seconds)
-  [Function `now_milliseconds`](#0x3_timestamp_now_milliseconds)
-  [Function `now_seconds`](#0x3_timestamp_now_seconds)
-  [Function `seconds_to_milliseconds`](#0x3_timestamp_seconds_to_milliseconds)
-  [Function `fast_forward_seconds_for_local`](#0x3_timestamp_fast_forward_seconds_for_local)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="chain_id.md#0x3_chain_id">0x3::chain_id</a>;
</code></pre>



<a name="0x3_timestamp_Timestamp"></a>

## Resource `Timestamp`

A singleton object holding the current Unix time in milliseconds


<pre><code><b>struct</b> <a href="timestamp.md#0x3_timestamp_Timestamp">Timestamp</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_timestamp_ErrorInvalidTimestamp"></a>

An invalid timestamp was provided


<pre><code><b>const</b> <a href="timestamp.md#0x3_timestamp_ErrorInvalidTimestamp">ErrorInvalidTimestamp</a>: u64 = 1;
</code></pre>



<a name="0x3_timestamp_MILLI_CONVERSION_FACTOR"></a>

Conversion factor between seconds and milliseconds


<pre><code><b>const</b> <a href="timestamp.md#0x3_timestamp_MILLI_CONVERSION_FACTOR">MILLI_CONVERSION_FACTOR</a>: u64 = 1000;
</code></pre>



<a name="0x3_timestamp_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timestamp.md#0x3_timestamp_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, _genesis_account: &<a href="">signer</a>, initial_time_milliseconds: u64)
</code></pre>



<a name="0x3_timestamp_update_global_time"></a>

## Function `update_global_time`

Updates the global clock time, if the new time is smaller than the current time, aborts.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timestamp.md#0x3_timestamp_update_global_time">update_global_time</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, timestamp_milliseconds: u64)
</code></pre>



<a name="0x3_timestamp_try_update_global_time"></a>

## Function `try_update_global_time`

Tries to update the global clock time, if the new time is smaller than the current time, ignores the update, and returns false.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timestamp.md#0x3_timestamp_try_update_global_time">try_update_global_time</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, timestamp_milliseconds: u64): bool
</code></pre>



<a name="0x3_timestamp_timestamp"></a>

## Function `timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x3_timestamp">timestamp</a>(ctx: &<a href="_Context">context::Context</a>): &<a href="timestamp.md#0x3_timestamp_Timestamp">timestamp::Timestamp</a>
</code></pre>



<a name="0x3_timestamp_milliseconds"></a>

## Function `milliseconds`



<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x3_timestamp_milliseconds">milliseconds</a>(self: &<a href="timestamp.md#0x3_timestamp_Timestamp">timestamp::Timestamp</a>): u64
</code></pre>



<a name="0x3_timestamp_seconds"></a>

## Function `seconds`



<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x3_timestamp_seconds">seconds</a>(self: &<a href="timestamp.md#0x3_timestamp_Timestamp">timestamp::Timestamp</a>): u64
</code></pre>



<a name="0x3_timestamp_now_milliseconds"></a>

## Function `now_milliseconds`

Gets the current time in milliseconds.


<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x3_timestamp_now_milliseconds">now_milliseconds</a>(ctx: &<a href="_Context">context::Context</a>): u64
</code></pre>



<a name="0x3_timestamp_now_seconds"></a>

## Function `now_seconds`

Gets the current time in seconds.


<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x3_timestamp_now_seconds">now_seconds</a>(ctx: &<a href="_Context">context::Context</a>): u64
</code></pre>



<a name="0x3_timestamp_seconds_to_milliseconds"></a>

## Function `seconds_to_milliseconds`



<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x3_timestamp_seconds_to_milliseconds">seconds_to_milliseconds</a>(seconds: u64): u64
</code></pre>



<a name="0x3_timestamp_fast_forward_seconds_for_local"></a>

## Function `fast_forward_seconds_for_local`

Fast forwards the clock by the given number of seconds, but only if the chain is in local mode.


<pre><code><b>public</b> entry <b>fun</b> <a href="timestamp.md#0x3_timestamp_fast_forward_seconds_for_local">fast_forward_seconds_for_local</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, timestamp_seconds: u64)
</code></pre>
