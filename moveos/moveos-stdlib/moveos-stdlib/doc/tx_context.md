
<a name="0x2_tx_context"></a>

# Module `0x2::tx_context`



-  [Struct `TxContext`](#0x2_tx_context_TxContext)
-  [Constants](#@Constants_0)
-  [Function `sender`](#0x2_tx_context_sender)
-  [Function `fresh_address`](#0x2_tx_context_fresh_address)
-  [Function `fresh_object_id`](#0x2_tx_context_fresh_object_id)
-  [Function `derive_id`](#0x2_tx_context_derive_id)
-  [Function `tx_hash`](#0x2_tx_context_tx_hash)
-  [Function `ids_created`](#0x2_tx_context_ids_created)
-  [Function `add`](#0x2_tx_context_add)
-  [Function `get`](#0x2_tx_context_get)


<pre><code><b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="copyable_any.md#0x2_copyable_any">0x2::copyable_any</a>;
<b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
<b>use</b> <a href="simple_map.md#0x2_simple_map">0x2::simple_map</a>;
<b>use</b> <a href="type_info.md#0x2_type_info">0x2::type_info</a>;
</code></pre>



<a name="0x2_tx_context_TxContext"></a>

## Struct `TxContext`

Information about the transaction currently being executed.
This cannot be constructed by a transaction--it is a privileged object created by
the VM and passed in to the entrypoint of the transaction as <code>&<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">TxContext</a></code>.


<pre><code><b>struct</b> <a href="tx_context.md#0x2_tx_context_TxContext">TxContext</a> <b>has</b> drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>sender: <b>address</b></code>
</dt>
<dd>
 The address of the user that signed the current transaction
</dd>
<dt>
<code>tx_hash: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>
 Hash of the current transaction
</dd>
<dt>
<code>ids_created: u64</code>
</dt>
<dd>
 Counter recording the number of fresh id's created while executing
 this transaction. Always 0 at the start of a transaction
</dd>
<dt>
<code>map: <a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="copyable_any.md#0x2_copyable_any_Any">copyable_any::Any</a>&gt;</code>
</dt>
<dd>
 A Key-Value map that can be used to store context information
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x2_tx_context_EBadTxHashLength"></a>

Expected an tx hash of length 32, but found a different length


<pre><code><b>const</b> <a href="tx_context.md#0x2_tx_context_EBadTxHashLength">EBadTxHashLength</a>: u64 = 0;
</code></pre>



<a name="0x2_tx_context_TX_HASH_LENGTH"></a>

Number of bytes in an tx hash (which will be the transaction digest)


<pre><code><b>const</b> <a href="tx_context.md#0x2_tx_context_TX_HASH_LENGTH">TX_HASH_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x2_tx_context_sender"></a>

## Function `sender`

Return the address of the user that signed the current
transaction


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_sender">sender</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_sender">sender</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">TxContext</a>): <b>address</b> {
    self.sender
}
</code></pre>



</details>

<a name="0x2_tx_context_fresh_address"></a>

## Function `fresh_address`

Generate a new unique address,


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_fresh_address">fresh_address</a>(ctx: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_fresh_address">fresh_address</a>(ctx: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">TxContext</a>): <b>address</b> {
    <b>let</b> addr = <a href="tx_context.md#0x2_tx_context_derive_id">derive_id</a>(ctx.tx_hash, ctx.ids_created);
    ctx.ids_created = ctx.ids_created + 1;
    addr
}
</code></pre>



</details>

<a name="0x2_tx_context_fresh_object_id"></a>

## Function `fresh_object_id`

Generate a new unique object ID


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_fresh_object_id">fresh_object_id</a>(ctx: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_fresh_object_id">fresh_object_id</a>(ctx: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">TxContext</a>): ObjectID {
    <a href="object_id.md#0x2_object_id_address_to_object_id">object_id::address_to_object_id</a>(<a href="tx_context.md#0x2_tx_context_fresh_address">fresh_address</a>(ctx))
}
</code></pre>



</details>

<a name="0x2_tx_context_derive_id"></a>

## Function `derive_id`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_derive_id">derive_id</a>(<a href="">hash</a>: <a href="">vector</a>&lt;u8&gt;, index: u64): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_derive_id">derive_id</a>(<a href="">hash</a>: <a href="">vector</a>&lt;u8&gt;, index: u64): <b>address</b> {
    <b>let</b> bytes = <a href="">hash</a>;
    <a href="_append">vector::append</a>(&<b>mut</b> bytes, <a href="../doc/bcs.md#0x1_bcs_to_bytes">bcs::to_bytes</a>(&index));
    //TODO change <b>return</b> type <b>to</b> h256 and <b>use</b> h256 <b>to</b> replace <b>address</b>?
    <b>let</b> id = <a href="_sha3_256">hash::sha3_256</a>(bytes);
    bcs::to_address(id)
}
</code></pre>



</details>

<a name="0x2_tx_context_tx_hash"></a>

## Function `tx_hash`

Return the hash of the current transaction


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_tx_hash">tx_hash</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_tx_hash">tx_hash</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">TxContext</a>): <a href="">vector</a>&lt;u8&gt; {
    self.tx_hash
}
</code></pre>



</details>

<a name="0x2_tx_context_ids_created"></a>

## Function `ids_created`

Return the number of id's created by the current transaction.
Hidden for now, but may expose later


<pre><code><b>fun</b> <a href="tx_context.md#0x2_tx_context_ids_created">ids_created</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="tx_context.md#0x2_tx_context_ids_created">ids_created</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">TxContext</a>): u64 {
    self.ids_created
}
</code></pre>



</details>

<a name="0x2_tx_context_add"></a>

## Function `add`

Add a value to the context map


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_add">add</a>&lt;T: <b>copy</b>, drop, store&gt;(self: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>, value: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_add">add</a>&lt;T: drop + store + <b>copy</b>&gt;(self: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">TxContext</a>, value: T) {
    <b>let</b> <a href="any.md#0x2_any">any</a> = <a href="copyable_any.md#0x2_copyable_any_pack">copyable_any::pack</a>(value);
    <b>let</b> <a href="">type_name</a> = *<a href="copyable_any.md#0x2_copyable_any_type_name">copyable_any::type_name</a>(&<a href="any.md#0x2_any">any</a>);
    <a href="simple_map.md#0x2_simple_map_add">simple_map::add</a>(&<b>mut</b> self.map, <a href="">type_name</a>, <a href="any.md#0x2_any">any</a>)
}
</code></pre>



</details>

<a name="0x2_tx_context_get"></a>

## Function `get`

Get a value from the context map


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_get">get</a>&lt;T: <b>copy</b>, drop, store&gt;(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="_Option">option::Option</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_get">get</a>&lt;T: drop + store + <b>copy</b>&gt;(self: &<a href="tx_context.md#0x2_tx_context_TxContext">TxContext</a>): Option&lt;T&gt; {
    <b>let</b> <a href="">type_name</a> = <a href="type_info.md#0x2_type_info_type_name">type_info::type_name</a>&lt;T&gt;();
    <b>if</b> (<a href="simple_map.md#0x2_simple_map_contains_key">simple_map::contains_key</a>(&self.map, &<a href="">type_name</a>)) {
        <b>let</b> <a href="any.md#0x2_any">any</a> = <a href="simple_map.md#0x2_simple_map_borrow">simple_map::borrow</a>(&self.map, &<a href="">type_name</a>);
        <a href="_some">option::some</a>(<a href="copyable_any.md#0x2_copyable_any_unpack">copyable_any::unpack</a>(*<a href="any.md#0x2_any">any</a>))
    }<b>else</b>{
        <a href="_none">option::none</a>()
    }
}
</code></pre>



</details>
