
<a name="0x3_timestamp"></a>

# Module `0x3::timestamp`



-  [Resource `TimestampPlaceholder`](#0x3_timestamp_TimestampPlaceholder)
-  [Constants](#@Constants_0)
-  [Function `fast_forward_seconds_for_local`](#0x3_timestamp_fast_forward_seconds_for_local)


<pre><code><b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::timestamp</a>;
<b>use</b> <a href="chain_id.md#0x3_chain_id">0x3::chain_id</a>;
</code></pre>



<a name="0x3_timestamp_TimestampPlaceholder"></a>

## Resource `TimestampPlaceholder`

Just using to get module signer


<pre><code><b>struct</b> <a href="timestamp.md#0x3_timestamp_TimestampPlaceholder">TimestampPlaceholder</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_timestamp_ErrorInvalidTimestamp"></a>

An invalid timestamp was provided


<pre><code><b>const</b> <a href="timestamp.md#0x3_timestamp_ErrorInvalidTimestamp">ErrorInvalidTimestamp</a>: u64 = 1;
</code></pre>



<a name="0x3_timestamp_fast_forward_seconds_for_local"></a>

## Function `fast_forward_seconds_for_local`

Fast forwards the clock by the given number of seconds, but only if the chain is in local mode.


<pre><code><b>public</b> entry <b>fun</b> <a href="timestamp.md#0x3_timestamp_fast_forward_seconds_for_local">fast_forward_seconds_for_local</a>(timestamp_seconds: u64)
</code></pre>
