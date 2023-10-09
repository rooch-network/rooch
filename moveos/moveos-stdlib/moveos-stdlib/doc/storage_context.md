
<a name="0x2_storage_context"></a>

# Module `0x2::storage_context`

StorageContext is part of the StorageAbstraction
It is used to store objects


-  [Struct `StorageContext`](#0x2_storage_context_StorageContext)
-  [Constants](#@Constants_0)
-  [Function `new_with_id`](#0x2_storage_context_new_with_id)
-  [Function `global_object_storage_handle`](#0x2_storage_context_global_object_storage_handle)
-  [Function `borrow`](#0x2_storage_context_borrow)
-  [Function `borrow_mut`](#0x2_storage_context_borrow_mut)
-  [Function `remove`](#0x2_storage_context_remove)
-  [Function `add`](#0x2_storage_context_add)
-  [Function `contains`](#0x2_storage_context_contains)


<pre><code><b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
<b>use</b> <a href="raw_table.md#0x2_raw_table">0x2::raw_table</a>;
</code></pre>



<a name="0x2_storage_context_StorageContext"></a>

## Struct `StorageContext`



<pre><code><b>struct</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>handle: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x2_storage_context_GlobalObjectStorageHandle"></a>



<pre><code><b>const</b> <a href="storage_context.md#0x2_storage_context_GlobalObjectStorageHandle">GlobalObjectStorageHandle</a>: <b>address</b> = 0;
</code></pre>



<a name="0x2_storage_context_new_with_id"></a>

## Function `new_with_id`

Create a new StorageContext with a given ObjectID.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_new_with_id">new_with_id</a>(handle: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_new_with_id">new_with_id</a>(handle: ObjectID): <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a> {
    <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a> {
        handle,
    }
}
</code></pre>



</details>

<a name="0x2_storage_context_global_object_storage_handle"></a>

## Function `global_object_storage_handle`

The global object storage's table handle should be <code>0x0</code>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_global_object_storage_handle">global_object_storage_handle</a>(): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_global_object_storage_handle">global_object_storage_handle</a>(): ObjectID {
    <a href="object_id.md#0x2_object_id_address_to_object_id">object_id::address_to_object_id</a>(<a href="storage_context.md#0x2_storage_context_GlobalObjectStorageHandle">GlobalObjectStorageHandle</a>)
}
</code></pre>



</details>

<a name="0x2_storage_context_borrow"></a>

## Function `borrow`

Borrow object from storage context with object_id


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_borrow">borrow</a>&lt;T: key&gt;(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_borrow">borrow</a>&lt;T: key&gt;(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>, <a href="object_id.md#0x2_object_id">object_id</a>: ObjectID): &Object&lt;T&gt; {
    <a href="raw_table.md#0x2_raw_table_borrow">raw_table::borrow</a>&lt;ObjectID, Object&lt;T&gt;&gt;(&self.handle, <a href="object_id.md#0x2_object_id">object_id</a>)
}
</code></pre>



</details>

<a name="0x2_storage_context_borrow_mut"></a>

## Function `borrow_mut`

Borrow mut object from storage context with object_id


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_borrow_mut">borrow_mut</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_borrow_mut">borrow_mut</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>, <a href="object_id.md#0x2_object_id">object_id</a>: ObjectID): &<b>mut</b> Object&lt;T&gt; {
    <a href="raw_table.md#0x2_raw_table_borrow_mut">raw_table::borrow_mut</a>&lt;ObjectID, Object&lt;T&gt;&gt;(&self.handle, <a href="object_id.md#0x2_object_id">object_id</a>)
}
</code></pre>



</details>

<a name="0x2_storage_context_remove"></a>

## Function `remove`

Remove object from storage context


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_remove">remove</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_remove">remove</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>, <a href="object_id.md#0x2_object_id">object_id</a>: ObjectID): Object&lt;T&gt; {
    <a href="raw_table.md#0x2_raw_table_remove">raw_table::remove</a>&lt;ObjectID, Object&lt;T&gt;&gt;(&self.handle, <a href="object_id.md#0x2_object_id">object_id</a>)
}
</code></pre>



</details>

<a name="0x2_storage_context_add"></a>

## Function `add`

Add object to storage context


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_add">add</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, obj: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_add">add</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>, obj: Object&lt;T&gt;) {
    <a href="raw_table.md#0x2_raw_table_add">raw_table::add</a>&lt;ObjectID, Object&lt;T&gt;&gt;(&self.handle, <a href="object.md#0x2_object_id">object::id</a>(&obj), obj);
}
</code></pre>



</details>

<a name="0x2_storage_context_contains"></a>

## Function `contains`

Returns true if the object exixts


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_contains">contains</a>(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_contains">contains</a>(self: &<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>, <a href="object_id.md#0x2_object_id">object_id</a>: ObjectID): bool {
    <a href="raw_table.md#0x2_raw_table_contains">raw_table::contains</a>&lt;ObjectID&gt;(&self.handle, <a href="object_id.md#0x2_object_id">object_id</a>)
}
</code></pre>



</details>
