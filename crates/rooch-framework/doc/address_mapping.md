
<a name="0x3_address_mapping"></a>

# Module `0x3::address_mapping`



-  [Resource `AddressMapping`](#0x3_address_mapping_AddressMapping)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_address_mapping_genesis_init)
-  [Function `borrow`](#0x3_address_mapping_borrow)
-  [Function `resolve_address`](#0x3_address_mapping_resolve_address)
-  [Function `resolve_or_generate_address`](#0x3_address_mapping_resolve_or_generate_address)
-  [Function `exists_mapping_address`](#0x3_address_mapping_exists_mapping_address)
-  [Function `resolve`](#0x3_address_mapping_resolve)
-  [Function `resolve_or_generate`](#0x3_address_mapping_resolve_or_generate)
-  [Function `exists_mapping`](#0x3_address_mapping_exists_mapping)
-  [Function `bind`](#0x3_address_mapping_bind)
-  [Function `bind_no_check`](#0x3_address_mapping_bind_no_check)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="multichain_address.md#0x3_multichain_address">0x3::multichain_address</a>;
</code></pre>



<a name="0x3_address_mapping_AddressMapping"></a>

## Resource `AddressMapping`



<pre><code><b>struct</b> <a href="address_mapping.md#0x3_address_mapping_AddressMapping">AddressMapping</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_address_mapping_ErrorMultiChainAddressInvalid"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_ErrorMultiChainAddressInvalid">ErrorMultiChainAddressInvalid</a>: u64 = 1;
</code></pre>



<a name="0x3_address_mapping_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, _genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x3_address_mapping_borrow"></a>

## Function `borrow`

Borrow the address mapping object


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_borrow">borrow</a>(ctx: &<a href="_Context">context::Context</a>): &<a href="_Object">object::Object</a>&lt;<a href="address_mapping.md#0x3_address_mapping_AddressMapping">address_mapping::AddressMapping</a>&gt;
</code></pre>



<a name="0x3_address_mapping_resolve_address"></a>

## Function `resolve_address`



<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve_address">resolve_address</a>(obj: &<a href="_Object">object::Object</a>&lt;<a href="address_mapping.md#0x3_address_mapping_AddressMapping">address_mapping::AddressMapping</a>&gt;, maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;
</code></pre>



<a name="0x3_address_mapping_resolve_or_generate_address"></a>

## Function `resolve_or_generate_address`



<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve_or_generate_address">resolve_or_generate_address</a>(obj: &<a href="_Object">object::Object</a>&lt;<a href="address_mapping.md#0x3_address_mapping_AddressMapping">address_mapping::AddressMapping</a>&gt;, maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): <b>address</b>
</code></pre>



<a name="0x3_address_mapping_exists_mapping_address"></a>

## Function `exists_mapping_address`



<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_exists_mapping_address">exists_mapping_address</a>(obj: &<a href="_Object">object::Object</a>&lt;<a href="address_mapping.md#0x3_address_mapping_AddressMapping">address_mapping::AddressMapping</a>&gt;, maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): bool
</code></pre>



<a name="0x3_address_mapping_resolve"></a>

## Function `resolve`

Resolve a multi-chain address to a rooch address


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve">resolve</a>(ctx: &<a href="_Context">context::Context</a>, maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;
</code></pre>



<a name="0x3_address_mapping_resolve_or_generate"></a>

## Function `resolve_or_generate`

Resolve a multi-chain address to a rooch address, if not exists, generate a new rooch address


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve_or_generate">resolve_or_generate</a>(ctx: &<a href="_Context">context::Context</a>, maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): <b>address</b>
</code></pre>



<a name="0x3_address_mapping_exists_mapping"></a>

## Function `exists_mapping`

Check if a multi-chain address is bound to a rooch address


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_exists_mapping">exists_mapping</a>(ctx: &<a href="_Context">context::Context</a>, maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>): bool
</code></pre>



<a name="0x3_address_mapping_bind"></a>

## Function `bind`

Bind a multi-chain address to the sender's rooch address
The caller need to ensure the relationship between the multi-chain address and the rooch address


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_bind">bind</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, sender: &<a href="">signer</a>, maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>)
</code></pre>



<a name="0x3_address_mapping_bind_no_check"></a>

## Function `bind_no_check`

Bind a rooch address to a multi-chain address


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_bind_no_check">bind_no_check</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, rooch_address: <b>address</b>, maddress: <a href="multichain_address.md#0x3_multichain_address_MultiChainAddress">multichain_address::MultiChainAddress</a>)
</code></pre>
