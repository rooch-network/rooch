
<a name="0x2_storage_context"></a>

# Module `0x2::storage_context`

StorageContext is part of the StorageAbstraction
It is used to provide a context for the storage operations, make the storage abstraction,
and let developers can customize the storage


-  [Struct `StorageContext`](#0x2_storage_context_StorageContext)
-  [Function `tx_context`](#0x2_storage_context_tx_context)
-  [Function `tx_context_mut`](#0x2_storage_context_tx_context_mut)
-  [Function `object_storage`](#0x2_storage_context_object_storage)
-  [Function `object_storage_mut`](#0x2_storage_context_object_storage_mut)
-  [Function `new_table`](#0x2_storage_context_new_table)
-  [Function `destroy_empty_table`](#0x2_storage_context_destroy_empty_table)
-  [Function `sender`](#0x2_storage_context_sender)
-  [Function `fresh_address`](#0x2_storage_context_fresh_address)
-  [Function `fresh_object_id`](#0x2_storage_context_fresh_object_id)
-  [Function `tx_hash`](#0x2_storage_context_tx_hash)
-  [Function `add`](#0x2_storage_context_add)
-  [Function `get`](#0x2_storage_context_get)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
<b>use</b> <a href="object_storage.md#0x2_object_storage">0x2::object_storage</a>;
<b>use</b> <a href="raw_table.md#0x2_raw_table">0x2::raw_table</a>;
<b>use</b> <a href="table.md#0x2_table">0x2::table</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_storage_context_StorageContext"></a>

## Struct `StorageContext`

Information about the global storage context
We can not put the StorageContext to TxContext, because object module depends on tx_context module,
and storage_context module depends on object module.
We put TxContext to StorageContext, for convenience of developers.
The StorageContext can not be <code>drop</code> or <code>store</code>, so developers need to pass the <code>&<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a></code> or <code>&<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a></code> to the <code>entry</code> function.


<pre><code><b>struct</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="tx_context.md#0x2_tx_context">tx_context</a>: <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a></code>
</dt>
<dd>

</dd>
<dt>
<code><a href="object_storage.md#0x2_object_storage">object_storage</a>: <a href="object_storage.md#0x2_object_storage_ObjectStorage">object_storage::ObjectStorage</a></code>
</dt>
<dd>
 The Global Object Storage
</dd>
</dl>


</details>

<a name="0x2_storage_context_tx_context"></a>

## Function `tx_context`



<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context">tx_context</a>(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context">tx_context</a>(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): &TxContext {
    &self.<a href="tx_context.md#0x2_tx_context">tx_context</a>
}
</code></pre>



</details>

<a name="0x2_storage_context_tx_context_mut"></a>

## Function `tx_context_mut`



<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_tx_context_mut">tx_context_mut</a>(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_tx_context_mut">tx_context_mut</a>(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): &<b>mut</b> TxContext {
    &<b>mut</b> self.<a href="tx_context.md#0x2_tx_context">tx_context</a>
}
</code></pre>



</details>

<a name="0x2_storage_context_object_storage"></a>

## Function `object_storage`



<pre><code><b>public</b> <b>fun</b> <a href="object_storage.md#0x2_object_storage">object_storage</a>(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): &<a href="object_storage.md#0x2_object_storage_ObjectStorage">object_storage::ObjectStorage</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_storage.md#0x2_object_storage">object_storage</a>(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): &ObjectStorage {
    &self.<a href="object_storage.md#0x2_object_storage">object_storage</a>
}
</code></pre>



</details>

<a name="0x2_storage_context_object_storage_mut"></a>

## Function `object_storage_mut`



<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_object_storage_mut">object_storage_mut</a>(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): &<b>mut</b> <a href="object_storage.md#0x2_object_storage_ObjectStorage">object_storage::ObjectStorage</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_object_storage_mut">object_storage_mut</a>(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): &<b>mut</b> ObjectStorage {
    &<b>mut</b> self.<a href="object_storage.md#0x2_object_storage">object_storage</a>
}
</code></pre>



</details>

<a name="0x2_storage_context_new_table"></a>

## Function `new_table`

Table create function


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_new_table">new_table</a>&lt;K: <b>copy</b>, drop, V: store&gt;(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): <a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_new_table">new_table</a>&lt;K: <b>copy</b> + drop, V: store&gt;(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): Table&lt;K,V&gt; {
    <b>let</b> sender = <a href="storage_context.md#0x2_storage_context_sender">sender</a>(self);
    <b>let</b> table_handle = <a href="storage_context.md#0x2_storage_context_fresh_object_id">fresh_object_id</a>(self);
    <b>let</b> table_info = <a href="raw_table.md#0x2_raw_table_new_empty_table_info">raw_table::new_empty_table_info</a>();
    <b>let</b> table_info_object = <a href="object.md#0x2_object_new_with_id">object::new_with_id</a>(table_handle, sender, table_info);
    <a href="object_storage.md#0x2_object_storage_add_internal">object_storage::add_internal</a>(&<b>mut</b> self.<a href="object_storage.md#0x2_object_storage">object_storage</a>, table_info_object);
    <a href="table.md#0x2_table_new_with_id">table::new_with_id</a>(table_handle)
}
</code></pre>



</details>

<a name="0x2_storage_context_destroy_empty_table"></a>

## Function `destroy_empty_table`



<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_destroy_empty_table">destroy_empty_table</a>&lt;K: <b>copy</b>, drop, V: store&gt;(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, <a href="table.md#0x2_table">table</a>: <a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_destroy_empty_table">destroy_empty_table</a>&lt;K: <b>copy</b> + drop, V: store&gt;(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>, <a href="table.md#0x2_table">table</a>: Table&lt;K,V&gt;) {
    <b>let</b> handle = <a href="table.md#0x2_table_handle">table::handle</a>(&<a href="table.md#0x2_table">table</a>);
    <b>let</b> table_info_object = <a href="object_storage.md#0x2_object_storage_remove_internal">object_storage::remove_internal</a>(&<b>mut</b> self.<a href="object_storage.md#0x2_object_storage">object_storage</a>, handle);
    <b>let</b> (_, _, table_info) = <a href="object.md#0x2_object_unpack_internal">object::unpack_internal</a>&lt;TableInfo&gt;(table_info_object);
    <b>let</b> (_state_root, length) = <a href="raw_table.md#0x2_raw_table_unpack">raw_table::unpack</a>(table_info);
    <b>assert</b>!(length == 0, 1000);
    <a href="table.md#0x2_table_destroy_empty">table::destroy_empty</a>(<a href="table.md#0x2_table">table</a>)
}
</code></pre>



</details>

<a name="0x2_storage_context_sender"></a>

## Function `sender`

Wrap functions for TxContext


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_sender">sender</a>(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_sender">sender</a>(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): <b>address</b> {
    <a href="tx_context.md#0x2_tx_context_sender">tx_context::sender</a>(&self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_storage_context_fresh_address"></a>

## Function `fresh_address`



<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_fresh_address">fresh_address</a>(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_fresh_address">fresh_address</a>(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): <b>address</b> {
    <a href="tx_context.md#0x2_tx_context_fresh_address">tx_context::fresh_address</a>(&<b>mut</b> self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_storage_context_fresh_object_id"></a>

## Function `fresh_object_id`



<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_fresh_object_id">fresh_object_id</a>(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_fresh_object_id">fresh_object_id</a>(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): ObjectID {
    <a href="tx_context.md#0x2_tx_context_fresh_object_id">tx_context::fresh_object_id</a>(&<b>mut</b> self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_storage_context_tx_hash"></a>

## Function `tx_hash`



<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_tx_hash">tx_hash</a>(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_tx_hash">tx_hash</a>(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): <a href="">vector</a>&lt;u8&gt; {
    <a href="tx_context.md#0x2_tx_context_tx_hash">tx_context::tx_hash</a>(&self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_storage_context_add"></a>

## Function `add`

Add a value to the context map


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_add">add</a>&lt;T: <b>copy</b>, drop, store&gt;(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, value: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_add">add</a>&lt;T: drop + store + <b>copy</b>&gt;(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>, value: T) {
    <a href="tx_context.md#0x2_tx_context_add">tx_context::add</a>(&<b>mut</b> self.<a href="tx_context.md#0x2_tx_context">tx_context</a>, value);
}
</code></pre>



</details>

<a name="0x2_storage_context_get"></a>

## Function `get`

Get a value from the context map


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_get">get</a>&lt;T: <b>copy</b>, drop, store&gt;(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): <a href="_Option">option::Option</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_get">get</a>&lt;T: drop + store + <b>copy</b>&gt;(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): Option&lt;T&gt; {
    <a href="tx_context.md#0x2_tx_context_get">tx_context::get</a>(&self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>
