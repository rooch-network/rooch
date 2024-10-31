
<a name="0xa_ton_address_mapping"></a>

# Module `0xa::ton_address_mapping`



-  [Resource `RoochToTonAddressMapping`](#0xa_ton_address_mapping_RoochToTonAddressMapping)
-  [Constants](#@Constants_0)
-  [Function `resolve_to_ton_address`](#0xa_ton_address_mapping_resolve_to_ton_address)
-  [Function `binding_ton_address`](#0xa_ton_address_mapping_binding_ton_address)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x3::bitcoin_address</a>;
<b>use</b> <a href="ton_address.md#0xa_ton_address">0xa::ton_address</a>;
<b>use</b> <a href="ton_proof.md#0xa_ton_proof">0xa::ton_proof</a>;
</code></pre>



<a name="0xa_ton_address_mapping_RoochToTonAddressMapping"></a>

## Resource `RoochToTonAddressMapping`

Mapping from rooch address to ton address
The mapping record is the object field, key is the rooch address, value is the ton address


<pre><code><b>struct</b> <a href="ton_address_mapping.md#0xa_ton_address_mapping_RoochToTonAddressMapping">RoochToTonAddressMapping</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_ton_address_mapping_ErrorInvalidBindingAddress"></a>



<pre><code><b>const</b> <a href="ton_address_mapping.md#0xa_ton_address_mapping_ErrorInvalidBindingAddress">ErrorInvalidBindingAddress</a>: u64 = 2;
</code></pre>



<a name="0xa_ton_address_mapping_ErrorInvalidBindingProof"></a>



<pre><code><b>const</b> <a href="ton_address_mapping.md#0xa_ton_address_mapping_ErrorInvalidBindingProof">ErrorInvalidBindingProof</a>: u64 = 1;
</code></pre>



<a name="0xa_ton_address_mapping_resolve_to_ton_address"></a>

## Function `resolve_to_ton_address`



<pre><code><b>public</b> <b>fun</b> <a href="ton_address_mapping.md#0xa_ton_address_mapping_resolve_to_ton_address">resolve_to_ton_address</a>(sender: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="ton_address.md#0xa_ton_address_TonAddress">ton_address::TonAddress</a>&gt;
</code></pre>



<a name="0xa_ton_address_mapping_binding_ton_address"></a>

## Function `binding_ton_address`



<pre><code><b>public</b> <b>fun</b> <a href="ton_address_mapping.md#0xa_ton_address_mapping_binding_ton_address">binding_ton_address</a>(proof: <a href="ton_proof.md#0xa_ton_proof_TonProof">ton_proof::TonProof</a>, <a href="ton_address.md#0xa_ton_address">ton_address</a>: <a href="ton_address.md#0xa_ton_address_TonAddress">ton_address::TonAddress</a>)
</code></pre>
