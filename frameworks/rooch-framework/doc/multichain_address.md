
<a name="0x3_multichain_address"></a>

# Module `0x3::multichain_address`



-  [Struct `MultiChainAddress`](#0x3_multichain_address_MultiChainAddress)
-  [Constants](#@Constants_0)
-  [Function `multichain_id_bitcoin`](#0x3_multichain_address_multichain_id_bitcoin)
-  [Function `multichain_id_ether`](#0x3_multichain_address_multichain_id_ether)
-  [Function `multichain_id_nostr`](#0x3_multichain_address_multichain_id_nostr)
-  [Function `multichain_id_rooch`](#0x3_multichain_address_multichain_id_rooch)
-  [Function `get_length`](#0x3_multichain_address_get_length)
-  [Function `new`](#0x3_multichain_address_new)
-  [Function `from_bytes`](#0x3_multichain_address_from_bytes)
-  [Function `from_eth`](#0x3_multichain_address_from_eth)
-  [Function `from_bitcoin`](#0x3_multichain_address_from_bitcoin)
-  [Function `multichain_id`](#0x3_multichain_address_multichain_id)
-  [Function `raw_address`](#0x3_multichain_address_raw_address)
-  [Function `is_rooch_address`](#0x3_multichain_address_is_rooch_address)
-  [Function `is_eth_address`](#0x3_multichain_address_is_eth_address)
-  [Function `is_bitcoin_address`](#0x3_multichain_address_is_bitcoin_address)
-  [Function `into_rooch_address`](#0x3_multichain_address_into_rooch_address)
-  [Function `into_eth_address`](#0x3_multichain_address_into_eth_address)
-  [Function `into_bitcoin_address`](#0x3_multichain_address_into_bitcoin_address)
-  [Function `mapping_to_rooch_address`](#0x3_multichain_address_mapping_to_rooch_address)


<pre><code><b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::hash</a>;
<b>use</b> <a href="bitcoin_address.md#0x3_bitcoin_address">0x3::bitcoin_address</a>;
<b>use</b> <a href="ethereum_address.md#0x3_ethereum_address">0x3::ethereum_address</a>;
</code></pre>



<a name="0x3_multichain_address_MultiChainAddress"></a>

## Struct `MultiChainAddress`



<pre><code>#[data_struct]
<b>struct</b> <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">MultiChainAddress</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_multichain_address_LENGTH"></a>



<pre><code><b>const</b> <a href="multichain_address.md#0x3_multichain_address_LENGTH">LENGTH</a>: u64 = 31;
</code></pre>



<a name="0x3_multichain_address_ErrorMultiChainIDMismatch"></a>



<pre><code><b>const</b> <a href="multichain_address.md#0x3_multichain_address_ErrorMultiChainIDMismatch">ErrorMultiChainIDMismatch</a>: u64 = 1;
</code></pre>



<a name="0x3_multichain_address_MULTICHAIN_ID_BITCOIN"></a>



<pre><code><b>const</b> <a href="multichain_address.md#0x3_multichain_address_MULTICHAIN_ID_BITCOIN">MULTICHAIN_ID_BITCOIN</a>: u64 = 0;
</code></pre>



<a name="0x3_multichain_address_MULTICHAIN_ID_ETHER"></a>



<pre><code><b>const</b> <a href="multichain_address.md#0x3_multichain_address_MULTICHAIN_ID_ETHER">MULTICHAIN_ID_ETHER</a>: u64 = 60;
</code></pre>



<a name="0x3_multichain_address_MULTICHAIN_ID_NOSTR"></a>



<pre><code><b>const</b> <a href="multichain_address.md#0x3_multichain_address_MULTICHAIN_ID_NOSTR">MULTICHAIN_ID_NOSTR</a>: u64 = 1237;
</code></pre>



<a name="0x3_multichain_address_MULTICHAIN_ID_ROOCH"></a>



<pre><code><b>const</b> <a href="multichain_address.md#0x3_multichain_address_MULTICHAIN_ID_ROOCH">MULTICHAIN_ID_ROOCH</a>: u64 = 20230101;
</code></pre>



<a name="0x3_multichain_address_multichain_id_bitcoin"></a>

## Function `multichain_id_bitcoin`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_multichain_id_bitcoin">multichain_id_bitcoin</a>(): u64
</code></pre>



<a name="0x3_multichain_address_multichain_id_ether"></a>

## Function `multichain_id_ether`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_multichain_id_ether">multichain_id_ether</a>(): u64
</code></pre>



<a name="0x3_multichain_address_multichain_id_nostr"></a>

## Function `multichain_id_nostr`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_multichain_id_nostr">multichain_id_nostr</a>(): u64
</code></pre>



<a name="0x3_multichain_address_multichain_id_rooch"></a>

## Function `multichain_id_rooch`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_multichain_id_rooch">multichain_id_rooch</a>(): u64
</code></pre>



<a name="0x3_multichain_address_get_length"></a>

## Function `get_length`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_get_length">get_length</a>(): u64
</code></pre>



<a name="0x3_multichain_address_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_new">new</a>(multichain_id: u64, raw_address: <a href="">vector</a>&lt;u8&gt;): <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>
</code></pre>



<a name="0x3_multichain_address_from_bytes"></a>

## Function `from_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_from_bytes">from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>
</code></pre>



<a name="0x3_multichain_address_from_eth"></a>

## Function `from_eth`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_from_eth">from_eth</a>(eth_address: <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ethereum_address::ETHAddress</a>): <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>
</code></pre>



<a name="0x3_multichain_address_from_bitcoin"></a>

## Function `from_bitcoin`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_from_bitcoin">from_bitcoin</a>(<a href="bitcoin_address.md#0x3_bitcoin_address">bitcoin_address</a>: <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>): <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>
</code></pre>



<a name="0x3_multichain_address_multichain_id"></a>

## Function `multichain_id`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_multichain_id">multichain_id</a>(self: &<a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): u64
</code></pre>



<a name="0x3_multichain_address_raw_address"></a>

## Function `raw_address`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_raw_address">raw_address</a>(self: &<a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_multichain_address_is_rooch_address"></a>

## Function `is_rooch_address`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_is_rooch_address">is_rooch_address</a>(maddress: &<a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): bool
</code></pre>



<a name="0x3_multichain_address_is_eth_address"></a>

## Function `is_eth_address`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_is_eth_address">is_eth_address</a>(maddress: &<a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): bool
</code></pre>



<a name="0x3_multichain_address_is_bitcoin_address"></a>

## Function `is_bitcoin_address`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_is_bitcoin_address">is_bitcoin_address</a>(maddress: &<a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): bool
</code></pre>



<a name="0x3_multichain_address_into_rooch_address"></a>

## Function `into_rooch_address`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_into_rooch_address">into_rooch_address</a>(maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): <b>address</b>
</code></pre>



<a name="0x3_multichain_address_into_eth_address"></a>

## Function `into_eth_address`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_into_eth_address">into_eth_address</a>(maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ethereum_address::ETHAddress</a>
</code></pre>



<a name="0x3_multichain_address_into_bitcoin_address"></a>

## Function `into_bitcoin_address`



<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_into_bitcoin_address">into_bitcoin_address</a>(maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>
</code></pre>



<a name="0x3_multichain_address_mapping_to_rooch_address"></a>

## Function `mapping_to_rooch_address`

Mapping from MultiChainAddress to rooch address
If the MultiChainAddress is not rooch address, it will generate a new rooch address based on the MultiChainAddress


<pre><code><b>public</b> <b>fun</b> <a href="multichain_address.md#0x3_multichain_address_mapping_to_rooch_address">mapping_to_rooch_address</a>(maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): <b>address</b>
</code></pre>
