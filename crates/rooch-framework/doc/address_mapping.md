
<a name="0x3_address_mapping"></a>

# Module `0x3::address_mapping`



-  [Struct `MultiChainAddress`](#0x3_address_mapping_MultiChainAddress)
-  [Resource `AddressMapping`](#0x3_address_mapping_AddressMapping)
-  [Constants](#@Constants_0)
-  [Function `is_rooch_address`](#0x3_address_mapping_is_rooch_address)
-  [Function `resolve`](#0x3_address_mapping_resolve)
-  [Function `resolve_or_generate`](#0x3_address_mapping_resolve_or_generate)
-  [Function `exists_mapping`](#0x3_address_mapping_exists_mapping)
-  [Function `bind`](#0x3_address_mapping_bind)
-  [Function `bind_no_check`](#0x3_address_mapping_bind_no_check)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="core_addresses.md#0x3_core_addresses">0x3::core_addresses</a>;
<b>use</b> <a href="hash.md#0x3_hash">0x3::hash</a>;
</code></pre>



<a name="0x3_address_mapping_MultiChainAddress"></a>

## Struct `MultiChainAddress`



<pre><code><b>struct</b> <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">MultiChainAddress</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>multichain_id: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>raw_address: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_address_mapping_AddressMapping"></a>

## Resource `AddressMapping`



<pre><code><b>struct</b> <a href="address_mapping.md#0x3_address_mapping_AddressMapping">AddressMapping</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mapping: <a href="_Table">table::Table</a>&lt;<a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">address_mapping::MultiChainAddress</a>, <b>address</b>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_address_mapping_MULTICHAIN_ID_BITCOIN"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_MULTICHAIN_ID_BITCOIN">MULTICHAIN_ID_BITCOIN</a>: u64 = 0;
</code></pre>



<a name="0x3_address_mapping_MULTICHAIN_ID_ETHER"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_MULTICHAIN_ID_ETHER">MULTICHAIN_ID_ETHER</a>: u64 = 60;
</code></pre>



<a name="0x3_address_mapping_MULTICHAIN_ID_NOSTR"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_MULTICHAIN_ID_NOSTR">MULTICHAIN_ID_NOSTR</a>: u64 = 1237;
</code></pre>



<a name="0x3_address_mapping_MULTICHAIN_ID_ROOCH"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_MULTICHAIN_ID_ROOCH">MULTICHAIN_ID_ROOCH</a>: u64 = 20230101;
</code></pre>



<a name="0x3_address_mapping_is_rooch_address"></a>

## Function `is_rooch_address`



<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_is_rooch_address">is_rooch_address</a>(maddress: &<a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">address_mapping::MultiChainAddress</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_is_rooch_address">is_rooch_address</a>(maddress: &<a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">MultiChainAddress</a>) : bool{
    maddress.multichain_id == <a href="address_mapping.md#0x3_address_mapping_MULTICHAIN_ID_ROOCH">MULTICHAIN_ID_ROOCH</a>
}
</code></pre>



</details>

<a name="0x3_address_mapping_resolve"></a>

## Function `resolve`

Resolve a multi-chain address to a rooch address


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve">resolve</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, maddress: <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">address_mapping::MultiChainAddress</a>): <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve">resolve</a>(ctx: &StorageContext, maddress: <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">MultiChainAddress</a>): Option&lt;<b>address</b>&gt; {
    <b>if</b> (<a href="address_mapping.md#0x3_address_mapping_is_rooch_address">is_rooch_address</a>(&maddress)) {
        <b>return</b> <a href="_some">option::some</a>(moveos_std::bcs::to_address(maddress.raw_address))
    };
    <b>let</b> am = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="address_mapping.md#0x3_address_mapping_AddressMapping">AddressMapping</a>&gt;(ctx, @rooch_framework);
    <b>if</b>(<a href="_contains">table::contains</a>(&am.mapping, maddress)){
        <b>let</b> addr = <a href="_borrow">table::borrow</a>(&am.mapping, maddress);
        <a href="_some">option::some</a>(*addr)
    }<b>else</b>{
        <a href="_none">option::none</a>()
    }
}
</code></pre>



</details>

<a name="0x3_address_mapping_resolve_or_generate"></a>

## Function `resolve_or_generate`

Resolve a multi-chain address to a rooch address, if not exists, generate a new rooch address


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve_or_generate">resolve_or_generate</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, maddress: <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">address_mapping::MultiChainAddress</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_resolve_or_generate">resolve_or_generate</a>(ctx: &StorageContext, maddress: <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">MultiChainAddress</a>): <b>address</b> {
    <b>let</b> addr = <a href="address_mapping.md#0x3_address_mapping_resolve">resolve</a>(ctx, maddress);
    <b>if</b>(<a href="_is_none">option::is_none</a>(&addr)){
        <a href="address_mapping.md#0x3_address_mapping_generate_rooch_address">generate_rooch_address</a>(maddress)
    }<b>else</b>{
        <a href="_extract">option::extract</a>(&<b>mut</b> addr)
    }
}
</code></pre>



</details>

<a name="0x3_address_mapping_exists_mapping"></a>

## Function `exists_mapping`

Check if a multi-chain address is bound to a rooch address


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_exists_mapping">exists_mapping</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, maddress: <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">address_mapping::MultiChainAddress</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_exists_mapping">exists_mapping</a>(ctx: &StorageContext, maddress: <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">MultiChainAddress</a>): bool {
    <b>if</b> (<a href="address_mapping.md#0x3_address_mapping_is_rooch_address">is_rooch_address</a>(&maddress)) {
        <b>return</b> <b>true</b>
    };
    <b>let</b> am = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="address_mapping.md#0x3_address_mapping_AddressMapping">AddressMapping</a>&gt;(ctx, @rooch_framework);
    <a href="_contains">table::contains</a>(&am.mapping, maddress)
}
</code></pre>



</details>

<a name="0x3_address_mapping_bind"></a>

## Function `bind`

Bind a multi-chain address to the sender's rooch address
The caller need to ensure the relationship between the multi-chain address and the rooch address


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_bind">bind</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, sender: &<a href="">signer</a>, maddress: <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">address_mapping::MultiChainAddress</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_bind">bind</a>(ctx: &<b>mut</b> StorageContext, sender: &<a href="">signer</a>, maddress: <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">MultiChainAddress</a>) {
    <a href="address_mapping.md#0x3_address_mapping_bind_no_check">bind_no_check</a>(ctx, <a href="_address_of">signer::address_of</a>(sender), maddress);
}
</code></pre>



</details>

<a name="0x3_address_mapping_bind_no_check"></a>

## Function `bind_no_check`

Bind a rooch address to a multi-chain address


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_bind_no_check">bind_no_check</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, rooch_address: <b>address</b>, maddress: <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">address_mapping::MultiChainAddress</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_bind_no_check">bind_no_check</a>(ctx: &<b>mut</b> StorageContext, rooch_address: <b>address</b>, maddress: <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">MultiChainAddress</a>) {
    <b>if</b>(<a href="address_mapping.md#0x3_address_mapping_is_rooch_address">is_rooch_address</a>(&maddress)){
        //Do nothing <b>if</b> the multi-chain <b>address</b> is a rooch <b>address</b>
        <b>return</b>
    };
    <b>let</b> am = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="address_mapping.md#0x3_address_mapping_AddressMapping">AddressMapping</a>&gt;(ctx, @rooch_framework);
    <a href="_add">table::add</a>(&<b>mut</b> am.mapping, maddress, rooch_address);
    //TODO matienance the reverse mapping rooch_address -&gt; <a href="">vector</a>&lt;<a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">MultiChainAddress</a>&gt;
}
</code></pre>



</details>
