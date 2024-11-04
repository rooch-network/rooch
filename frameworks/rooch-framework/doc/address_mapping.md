
<a name="0x3_address_mapping"></a>

# Module `0x3::address_mapping`



-  [Resource `MultiChainAddressMapping`](#0x3_address_mapping_MultiChainAddressMapping)
-  [Resource `RoochToBitcoinAddressMapping`](#0x3_address_mapping_RoochToBitcoinAddressMapping)
-  [Resource `RoochToTonAddressMapping`](#0x3_address_mapping_RoochToTonAddressMapping)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_address_mapping_genesis_init)
-  [Function `init_ton_mapping`](#0x3_address_mapping_init_ton_mapping)
-  [Function `resolve`](#0x3_address_mapping_resolve)
-  [Function `resolve_bitcoin`](#0x3_address_mapping_resolve_bitcoin)
-  [Function `exists_mapping`](#0x3_address_mapping_exists_mapping)
-  [Function `bind_bitcoin_address_internal`](#0x3_address_mapping_bind_bitcoin_address_internal)
-  [Function `bind_bitcoin_address_by_system`](#0x3_address_mapping_bind_bitcoin_address_by_system)
-  [Function `bind_bitcoin_address`](#0x3_address_mapping_bind_bitcoin_address)
-  [Function `resolve_to_ton_address`](#0x3_address_mapping_resolve_to_ton_address)
-  [Function `resolve_via_ton_address`](#0x3_address_mapping_resolve_via_ton_address)
-  [Function `resolve_via_ton_address_str`](#0x3_address_mapping_resolve_via_ton_address_str)
-  [Function `bind_ton_address`](#0x3_address_mapping_bind_ton_address)
-  [Function `bind_ton_address_entry`](#0x3_address_mapping_bind_ton_address_entry)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::core_addresses</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="bitcoin_address.md#0x3_bitcoin_address">0x3::bitcoin_address</a>;
<b>use</b> <a href="multichain_address.md#0x3_multichain_address">0x3::multichain_address</a>;
<b>use</b> <a href="ton_address.md#0x3_ton_address">0x3::ton_address</a>;
<b>use</b> <a href="ton_proof.md#0x3_ton_proof">0x3::ton_proof</a>;
</code></pre>



<a name="0x3_address_mapping_MultiChainAddressMapping"></a>

## Resource `MultiChainAddressMapping`

Mapping from multi-chain address to rooch address
Not including Bitcoin address, because Bitcoin address can directly hash to rooch address
The mapping record is the object field, key is the multi-chain address, value is the rooch address


<pre><code><b>struct</b> <a href="address_mapping.md#0x3_address_mapping_MultiChainAddressMapping">MultiChainAddressMapping</a> <b>has</b> key
</code></pre>



<a name="0x3_address_mapping_RoochToBitcoinAddressMapping"></a>

## Resource `RoochToBitcoinAddressMapping`

Mapping from rooch address to bitcoin address, other chain can use new table
The mapping record is the object field, key is the rooch address, value is the Bitcoin address


<pre><code><b>struct</b> <a href="address_mapping.md#0x3_address_mapping_RoochToBitcoinAddressMapping">RoochToBitcoinAddressMapping</a> <b>has</b> key
</code></pre>



<a name="0x3_address_mapping_RoochToTonAddressMapping"></a>

## Resource `RoochToTonAddressMapping`

Mapping from rooch address to ton address
The mapping record is the object field, key is the rooch address, value is the ton address


<pre><code><b>struct</b> <a href="address_mapping.md#0x3_address_mapping_RoochToTonAddressMapping">RoochToTonAddressMapping</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_address_mapping_ErrorInvalidBindingAddress"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_ErrorInvalidBindingAddress">ErrorInvalidBindingAddress</a>: u64 = 4;
</code></pre>



<a name="0x3_address_mapping_ErrorInvalidBindingProof"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_ErrorInvalidBindingProof">ErrorInvalidBindingProof</a>: u64 = 3;
</code></pre>



<a name="0x3_address_mapping_ErrorMultiChainAddressInvalid"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_ErrorMultiChainAddressInvalid">ErrorMultiChainAddressInvalid</a>: u64 = 1;
</code></pre>



<a name="0x3_address_mapping_ErrorUnsupportedAddress"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_ErrorUnsupportedAddress">ErrorUnsupportedAddress</a>: u64 = 2;
</code></pre>



<a name="0x3_address_mapping_NAMED_MAPPING_INDEX"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_NAMED_MAPPING_INDEX">NAMED_MAPPING_INDEX</a>: u64 = 0;
</code></pre>



<a name="0x3_address_mapping_NAMED_REVERSE_MAPPING_INDEX"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_NAMED_REVERSE_MAPPING_INDEX">NAMED_REVERSE_MAPPING_INDEX</a>: u64 = 1;
</code></pre>



<a name="0x3_address_mapping_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_genesis_init">genesis_init</a>()
</code></pre>



<a name="0x3_address_mapping_init_ton_mapping"></a>

## Function `init_ton_mapping`



<pre><code><b>public</b> entry <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_init_ton_mapping">init_ton_mapping</a>()
</code></pre>



<a name="0x3_address_mapping_resolve"></a>

## Function `resolve`

Resolve a multi-chain address to a rooch address


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve">resolve</a>(maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;
</code></pre>



<a name="0x3_address_mapping_resolve_bitcoin"></a>

## Function `resolve_bitcoin`

Resolve a rooch address to a bitcoin address


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve_bitcoin">resolve_bitcoin</a>(rooch_address: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>&gt;
</code></pre>



<a name="0x3_address_mapping_exists_mapping"></a>

## Function `exists_mapping`

Check if a multi-chain address is bound to a rooch address


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_exists_mapping">exists_mapping</a>(maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): bool
</code></pre>



<a name="0x3_address_mapping_bind_bitcoin_address_internal"></a>

## Function `bind_bitcoin_address_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_bind_bitcoin_address_internal">bind_bitcoin_address_internal</a>(rooch_address: <b>address</b>, btc_address: <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>)
</code></pre>



<a name="0x3_address_mapping_bind_bitcoin_address_by_system"></a>

## Function `bind_bitcoin_address_by_system`



<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_bind_bitcoin_address_by_system">bind_bitcoin_address_by_system</a>(system: &<a href="">signer</a>, rooch_address: <b>address</b>, btc_address: <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>)
</code></pre>



<a name="0x3_address_mapping_bind_bitcoin_address"></a>

## Function `bind_bitcoin_address`

Bind a bitcoin address to a rooch address
We can calculate the rooch address from bitcoin address
So we call this function for record rooch address to bitcoin address mapping


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_bind_bitcoin_address">bind_bitcoin_address</a>(btc_address: <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>)
</code></pre>



<a name="0x3_address_mapping_resolve_to_ton_address"></a>

## Function `resolve_to_ton_address`



<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve_to_ton_address">resolve_to_ton_address</a>(sender: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="ton_address.md#0x3_ton_address_TonAddress">ton_address::TonAddress</a>&gt;
</code></pre>



<a name="0x3_address_mapping_resolve_via_ton_address"></a>

## Function `resolve_via_ton_address`



<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve_via_ton_address">resolve_via_ton_address</a>(<a href="ton_address.md#0x3_ton_address">ton_address</a>: <a href="ton_address.md#0x3_ton_address_TonAddress">ton_address::TonAddress</a>): <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;
</code></pre>



<a name="0x3_address_mapping_resolve_via_ton_address_str"></a>

## Function `resolve_via_ton_address_str`



<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve_via_ton_address_str">resolve_via_ton_address_str</a>(ton_address_str: <a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;
</code></pre>



<a name="0x3_address_mapping_bind_ton_address"></a>

## Function `bind_ton_address`

Bind a ton address to a rooch address
The user needs to provide a valid ton proof and the ton address he wants to bind


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_bind_ton_address">bind_ton_address</a>(proof_data: <a href="ton_proof.md#0x3_ton_proof_TonProofData">ton_proof::TonProofData</a>, <a href="ton_address.md#0x3_ton_address">ton_address</a>: <a href="ton_address.md#0x3_ton_address_TonAddress">ton_address::TonAddress</a>)
</code></pre>



<a name="0x3_address_mapping_bind_ton_address_entry"></a>

## Function `bind_ton_address_entry`



<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_bind_ton_address_entry">bind_ton_address_entry</a>(proof_data_bytes: <a href="">vector</a>&lt;u8&gt;, ton_address_str: <a href="_String">string::String</a>)
</code></pre>
