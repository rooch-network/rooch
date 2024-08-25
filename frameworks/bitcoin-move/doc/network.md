
<a name="0x4_network"></a>

# Module `0x4::network`



-  [Resource `BitcoinNetwork`](#0x4_network_BitcoinNetwork)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_network_genesis_init)
-  [Function `network`](#0x4_network_network)
-  [Function `network_bitcoin`](#0x4_network_network_bitcoin)
-  [Function `network_testnet`](#0x4_network_network_testnet)
-  [Function `network_signet`](#0x4_network_network_signet)
-  [Function `network_regtest`](#0x4_network_network_regtest)
-  [Function `is_mainnet`](#0x4_network_is_mainnet)
-  [Function `is_testnet`](#0x4_network_is_testnet)
-  [Function `is_signet`](#0x4_network_is_signet)
-  [Function `from_str`](#0x4_network_from_str)
-  [Function `network_name`](#0x4_network_network_name)
-  [Function `bech32_hrp`](#0x4_network_bech32_hrp)
-  [Function `jubilee_height`](#0x4_network_jubilee_height)
-  [Function `first_inscription_height`](#0x4_network_first_inscription_height)
-  [Function `subsidy_by_height`](#0x4_network_subsidy_by_height)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::object</a>;
</code></pre>



<a name="0x4_network_BitcoinNetwork"></a>

## Resource `BitcoinNetwork`

Bitcoin network onchain configuration.


<pre><code><b>struct</b> <a href="network.md#0x4_network_BitcoinNetwork">BitcoinNetwork</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_network_COIN_VALUE"></a>

How many satoshis are in "one bitcoin".


<pre><code><b>const</b> <a href="network.md#0x4_network_COIN_VALUE">COIN_VALUE</a>: u64 = 100000000;
</code></pre>



<a name="0x4_network_ErrorUnknownNetwork"></a>



<pre><code><b>const</b> <a href="network.md#0x4_network_ErrorUnknownNetwork">ErrorUnknownNetwork</a>: u64 = 1;
</code></pre>



<a name="0x4_network_FIRST_POST_SUBSIDY_EPOCH"></a>



<pre><code><b>const</b> <a href="network.md#0x4_network_FIRST_POST_SUBSIDY_EPOCH">FIRST_POST_SUBSIDY_EPOCH</a>: u32 = 33;
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



<a name="0x4_network_SUBSIDY_HALVING_INTERVAL"></a>

How may blocks between halvings.


<pre><code><b>const</b> <a href="network.md#0x4_network_SUBSIDY_HALVING_INTERVAL">SUBSIDY_HALVING_INTERVAL</a>: u32 = 210000;
</code></pre>



<a name="0x4_network_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="network.md#0x4_network_genesis_init">genesis_init</a>(<a href="network.md#0x4_network">network</a>: u8)
</code></pre>



<a name="0x4_network_network"></a>

## Function `network`

Get the current network from the onchain configuration.


<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network">network</a>(): u8
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



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_is_mainnet">is_mainnet</a>(): bool
</code></pre>



<a name="0x4_network_is_testnet"></a>

## Function `is_testnet`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_is_testnet">is_testnet</a>(): bool
</code></pre>



<a name="0x4_network_is_signet"></a>

## Function `is_signet`



<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_is_signet">is_signet</a>(): bool
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



<a name="0x4_network_jubilee_height"></a>

## Function `jubilee_height`

Ordinals jubilee height.
https://github.com/ordinals/ord/blob/75bf04b22107155f8f8ab6c77f6eefa8117d9ace/src/chain.rs#L49-L56


<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_jubilee_height">jubilee_height</a>(): u64
</code></pre>



<a name="0x4_network_first_inscription_height"></a>

## Function `first_inscription_height`

Ordinals first inscription height.
https://github.com/ordinals/ord/blob/75bf04b22107155f8f8ab6c77f6eefa8117d9ace/src/chain.rs#L36-L43


<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_first_inscription_height">first_inscription_height</a>(): u64
</code></pre>



<a name="0x4_network_subsidy_by_height"></a>

## Function `subsidy_by_height`

Block Rewards


<pre><code><b>public</b> <b>fun</b> <a href="network.md#0x4_network_subsidy_by_height">subsidy_by_height</a>(height: u64): u64
</code></pre>
