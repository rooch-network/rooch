
<a name="0x3_address_mapping"></a>

# Module `0x3::address_mapping`



-  [Struct `MultiChainAddress`](#0x3_address_mapping_MultiChainAddress)
-  [Resource `AddressMapping`](#0x3_address_mapping_AddressMapping)
-  [Constants](#@Constants_0)
-  [Function `is_rooch_address`](#0x3_address_mapping_is_rooch_address)
-  [Function `resolve`](#0x3_address_mapping_resolve)
-  [Function `binding`](#0x3_address_mapping_binding)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::bcd</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="core_addresses.md#0x3_core_addresses">0x3::core_addresses</a>;
</code></pre>



<a name="0x3_address_mapping_MultiChainAddress"></a>

## Struct `MultiChainAddress`



<pre><code><b>struct</b> <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">MultiChainAddress</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_id: u32</code>
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


<a name="0x3_address_mapping_COIN_TYPE_BTC"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_COIN_TYPE_BTC">COIN_TYPE_BTC</a>: u32 = 0;
</code></pre>



<a name="0x3_address_mapping_COIN_TYPE_ETH"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_COIN_TYPE_ETH">COIN_TYPE_ETH</a>: u32 = 60;
</code></pre>



<a name="0x3_address_mapping_COIN_TYPE_ROH"></a>



<pre><code><b>const</b> <a href="address_mapping.md#0x3_address_mapping_COIN_TYPE_ROH">COIN_TYPE_ROH</a>: u32 = 20230101;
</code></pre>



<a name="0x3_address_mapping_is_rooch_address"></a>

## Function `is_rooch_address`



<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_is_rooch_address">is_rooch_address</a>(maddress: &<a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">address_mapping::MultiChainAddress</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_is_rooch_address">is_rooch_address</a>(maddress: &<a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">MultiChainAddress</a>) : bool{
    maddress.coin_id == <a href="address_mapping.md#0x3_address_mapping_COIN_TYPE_ROH">COIN_TYPE_ROH</a>
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
        <b>return</b> <a href="_some">option::some</a>(moveos_std::bcd::to_address(maddress.raw_address))
    };
    <b>let</b> am = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="address_mapping.md#0x3_address_mapping_AddressMapping">AddressMapping</a>&gt;(ctx, @rooch_framework);
    <b>if</b>(<a href="_contains">table::contains</a>(&am.mapping, maddress)){
        <b>let</b> addr = <a href="_borrow">table::borrow</a>(&am.mapping, maddress);
        <a href="_some">option::some</a>(*addr)
    }<b>else</b>{
        <a href="_none">option::none</a>&lt;<b>address</b>&gt;()
    }
}
</code></pre>



</details>

<a name="0x3_address_mapping_binding"></a>

## Function `binding`

Binding a multi-chain address to a rooch address
The caller need to ensure the relationship between the multi-chain address and the rooch address


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_binding">binding</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, sender: &<a href="">signer</a>, maddress: <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">address_mapping::MultiChainAddress</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="address_mapping.md#0x3_address_mapping_binding">binding</a>(ctx: &<b>mut</b> StorageContext, sender: &<a href="">signer</a>, maddress: <a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">MultiChainAddress</a>) {
    <b>let</b> am = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="address_mapping.md#0x3_address_mapping_AddressMapping">AddressMapping</a>&gt;(ctx, @rooch_framework);
    <b>let</b> sender_addr = <a href="_address_of">signer::address_of</a>(sender);
    <a href="_add">table::add</a>(&<b>mut</b> am.mapping, maddress, sender_addr);
    //TODO matienance the reverse mapping rooch_address -&gt; <a href="">vector</a>&lt;<a href="address_mapping.md#0x3_address_mapping_MultiChainAddress">MultiChainAddress</a>&gt;
}
</code></pre>



</details>
