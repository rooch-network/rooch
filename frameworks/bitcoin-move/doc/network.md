
<a name="0x4_network"></a>

# Module `0x4::network`



-  [Constants](#@Constants_0)
-  [Function `network_bitcoin`](#0x4_network_network_bitcoin)
-  [Function `network_testnet`](#0x4_network_network_testnet)
-  [Function `network_signet`](#0x4_network_network_signet)
-  [Function `network_regtest`](#0x4_network_network_regtest)
-  [Function `is_mainnet`](#0x4_network_is_mainnet)
-  [Function `default_network`](#0x4_network_default_network)
-  [Function `network`](#0x4_network_network)
-  [Function `from_str`](#0x4_network_from_str)
-  [Function `network_name`](#0x4_network_network_name)
-  [Function `bech32_hrp`](#0x4_network_bech32_hrp)
-  [Function `network_magic`](#0x4_network_network_magic)


<pre><code><b>use</b> <a href="">0x1::string</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_network_ErrorUnknownNetwork"></a>



<pre><code><b>const</b> <a href="network.md#0x4_network_ErrorUnknownNetwork">ErrorUnknownNetwork</a>: u64 = 1;
</code></pre>



<a name="0x4_network_NETWORK_BITCOIN"></a>

Currently, Move does not support enum types, so we use constants to represent the network type.
Mainnet Bitcoin.


<pre><code><b>const</b> <a href="network.md#0x4_network_NETWORK_BITCOIN">NETWORK_BITCOIN</a>: u8 = 1;
</code></pre>



<a name="0x4_network_NETWORK_REGTEST"></a>

Bitcoin's regtest network.


<pre><code><b>const</b> <a href="network.md#0x4_network_NETWORK_REGTEST">NETWORK_REGTEST</a>: u8 = 4;
</code></pre>



<a name="0x4_network_NETWORK_SIGNET"></a>

Bitcoin's signet network.


<pre><code><b>const</b> <a href="network.md#0x4_network_NETWORK_SIGNET">NETWORK_SIGNET</a>: u8 = 3;
</code></pre>



<a name="0x4_network_NETWORK_TESTNET"></a>

Bitcoin's testnet network.


<pre><code><b>const</b> <a href="network.md#0x4_network_NETWORK_TESTNET">NETWORK_TESTNET</a>: u8 = 2;
</code></pre>



<a name="0x4_network_network_bitcoin"></a>

## Function `network_bitcoin`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_network_bitcoin">network_bitcoin</a>(): u8
</code></pre>



<a name="0x4_network_network_testnet"></a>

## Function `network_testnet`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_network_testnet">network_testnet</a>(): u8
</code></pre>



<a name="0x4_network_network_signet"></a>

## Function `network_signet`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_network_signet">network_signet</a>(): u8
</code></pre>



<a name="0x4_network_network_regtest"></a>

## Function `network_regtest`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_network_regtest">network_regtest</a>(): u8
</code></pre>



<a name="0x4_network_is_mainnet"></a>

## Function `is_mainnet`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_is_mainnet">is_mainnet</a>(<a href="network.md#0x4_network">network</a>: u8): bool
</code></pre>



<a name="0x4_network_default_network"></a>

## Function `default_network`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_default_network">default_network</a>(): u8
</code></pre>



<a name="0x4_network_network"></a>

## Function `network`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network">network</a>(): u8
</code></pre>



<a name="0x4_network_from_str"></a>

## Function `from_str`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_from_str">from_str</a>(<a href="network.md#0x4_network">network</a>: &<a href="_String">string::String</a>): u8
</code></pre>



<a name="0x4_network_network_name"></a>

## Function `network_name`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_network_name">network_name</a>(<a href="network.md#0x4_network">network</a>: u8): <a href="_String">string::String</a>
</code></pre>



<a name="0x4_network_bech32_hrp"></a>

## Function `bech32_hrp`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_bech32_hrp">bech32_hrp</a>(<a href="network.md#0x4_network">network</a>: u8): <a href="_String">string::String</a>
</code></pre>



<a name="0x4_network_network_magic"></a>

## Function `network_magic`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_network_magic">network_magic</a>(<a href="network.md#0x4_network">network</a>: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>
