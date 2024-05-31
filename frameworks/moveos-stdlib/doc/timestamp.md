
<a name="0x2_timestamp"></a>

# Module `0x2::timestamp`

This module keeps a global wall clock that stores the current Unix time in milliseconds.
It interacts with the other modules in the following ways:
* genesis: to initialize the timestamp
* L1 block: update the timestamp via L1s block header timestamp
* L2 transactions: update the timestamp via L2 transaction's timestamp


-  [Constants](#@Constants_0)
-  [Function `try_update_global_time`](#0x2_timestamp_try_update_global_time)
-  [Function `timestamp`](#0x2_timestamp_timestamp)
-  [Function `milliseconds`](#0x2_timestamp_milliseconds)
-  [Function `seconds`](#0x2_timestamp_seconds)
-  [Function `now_milliseconds`](#0x2_timestamp_now_milliseconds)
-  [Function `now_seconds`](#0x2_timestamp_now_seconds)
-  [Function `seconds_to_milliseconds`](#0x2_timestamp_seconds_to_milliseconds)
-  [Function `fast_forward_seconds_by_system`](#0x2_timestamp_fast_forward_seconds_by_system)


<pre><code><b>use</b> <a href="core_addresses.md#0x2_core_addresses">0x2::core_addresses</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="signer.md#0x2_signer">0x2::signer</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_timestamp_ErrorInvalidTimestamp"></a>

An invalid timestamp was provided


<pre><code><b>const</b> <a href="timestamp.md#0x2_timestamp_ErrorInvalidTimestamp">ErrorInvalidTimestamp</a>: u64 = 1;
</code></pre>



<a name="0x2_timestamp_ErrorNotGenesisAddress"></a>



<pre><code><b>const</b> <a href="timestamp.md#0x2_timestamp_ErrorNotGenesisAddress">ErrorNotGenesisAddress</a>: u64 = 2;
</code></pre>



<a name="0x2_timestamp_MILLI_CONVERSION_FACTOR"></a>

Conversion factor between seconds and milliseconds


<pre><code><b>const</b> <a href="timestamp.md#0x2_timestamp_MILLI_CONVERSION_FACTOR">MILLI_CONVERSION_FACTOR</a>: u64 = 1000;
</code></pre>



<a name="0x2_timestamp_try_update_global_time"></a>

## Function `try_update_global_time`

Tries to update the global clock time, if the new time is smaller than the current time, ignores the update, and returns false.
Only the framework genesis account can update the global clock time.


<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x2_timestamp_try_update_global_time">try_update_global_time</a>(genesis_account: &<a href="">signer</a>, timestamp_milliseconds: u64): bool
</code></pre>



<a name="0x2_timestamp_timestamp"></a>

## Function `timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x2_timestamp">timestamp</a>(): &<a href="object.md#0x2_object_Timestamp">object::Timestamp</a>
</code></pre>



<a name="0x2_timestamp_milliseconds"></a>

## Function `milliseconds`



<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x2_timestamp_milliseconds">milliseconds</a>(self: &<a href="object.md#0x2_object_Timestamp">object::Timestamp</a>): u64
</code></pre>



<a name="0x2_timestamp_seconds"></a>

## Function `seconds`



<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x2_timestamp_seconds">seconds</a>(self: &<a href="object.md#0x2_object_Timestamp">object::Timestamp</a>): u64
</code></pre>



<a name="0x2_timestamp_now_milliseconds"></a>

## Function `now_milliseconds`

Gets the current time in milliseconds.


<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x2_timestamp_now_milliseconds">now_milliseconds</a>(): u64
</code></pre>



<a name="0x2_timestamp_now_seconds"></a>

## Function `now_seconds`

Gets the current time in seconds.


<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x2_timestamp_now_seconds">now_seconds</a>(): u64
</code></pre>



<a name="0x2_timestamp_seconds_to_milliseconds"></a>

## Function `seconds_to_milliseconds`



<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x2_timestamp_seconds_to_milliseconds">seconds_to_milliseconds</a>(seconds: u64): u64
</code></pre>



<a name="0x2_timestamp_fast_forward_seconds_by_system"></a>

## Function `fast_forward_seconds_by_system`



<pre><code><b>public</b> <b>fun</b> <a href="timestamp.md#0x2_timestamp_fast_forward_seconds_by_system">fast_forward_seconds_by_system</a>(genesis_account: &<a href="">signer</a>, timestamp_seconds: u64)
</code></pre>
