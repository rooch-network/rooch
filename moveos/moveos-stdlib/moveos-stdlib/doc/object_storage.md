
<a name="0x2_object_storage"></a>

# Module `0x2::object_storage`

AccountStorage is part of the StorageAbstraction
It is used to store the objects


-  [Struct `ObjectStorage`](#0x2_object_storage_ObjectStorage)
-  [Constants](#@Constants_0)
-  [Function `new_with_id`](#0x2_object_storage_new_with_id)
-  [Function `global_object_storage_handle`](#0x2_object_storage_global_object_storage_handle)
-  [Function `borrow`](#0x2_object_storage_borrow)
-  [Function `borrow_mut`](#0x2_object_storage_borrow_mut)
-  [Function `remove`](#0x2_object_storage_remove)
-  [Function `add`](#0x2_object_storage_add)
-  [Function `contains`](#0x2_object_storage_contains)


<pre><code><b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
<b>use</b> <a href="raw_table.md#0x2_raw_table">0x2::raw_table</a>;
</code></pre>



<a name="0x2_object_storage_ObjectStorage"></a>

## Struct `ObjectStorage`



<pre><code><b>struct</b> <a href="object_storage.md#0x2_object_storage_ObjectStorage">ObjectStorage</a> <b>has</b> store
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


<a name="0x2_object_storage_GlobalObjectStorageHandle"></a>



<pre><code><b>const</b> <a href="object_storage.md#0x2_object_storage_GlobalObjectStorageHandle">GlobalObjectStorageHandle</a>: <b>address</b> = 0;
</code></pre>



<a name="0x2_object_storage_new_with_id"></a>

## Function `new_with_id`

Create a new ObjectStorage with a given handle.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_new_with_id">new_with_id</a>(handle: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): <a href="object_storage.md#0x2_object_storage_ObjectStorage">object_storage::ObjectStorage</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_new_with_id">new_with_id</a>(handle: ObjectID): <a href="object_storage.md#0x2_object_storage_ObjectStorage">ObjectStorage</a> {
    <a href="object_storage.md#0x2_object_storage_ObjectStorage">ObjectStorage</a> {
        handle,
    }
}
</code></pre>



</details>

<a name="0x2_object_storage_global_object_storage_handle"></a>

## Function `global_object_storage_handle`

The global object storage's table handle should be 0x0


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_global_object_storage_handle">global_object_storage_handle</a>(): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_global_object_storage_handle">global_object_storage_handle</a>(): ObjectID {
    <a href="object_id.md#0x2_object_id_address_to_object_id">object_id::address_to_object_id</a>(<a href="object_storage.md#0x2_object_storage_GlobalObjectStorageHandle">GlobalObjectStorageHandle</a>)
}
</code></pre>



</details>

<a name="0x2_object_storage_borrow"></a>

## Function `borrow`

Borrow Object from object store with object_id


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_borrow">borrow</a>&lt;T: key&gt;(self: &<a href="object_storage.md#0x2_object_storage_ObjectStorage">object_storage::ObjectStorage</a>, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_borrow">borrow</a>&lt;T: key&gt;(self: &<a href="object_storage.md#0x2_object_storage_ObjectStorage">ObjectStorage</a>, <a href="object_id.md#0x2_object_id">object_id</a>: ObjectID): &Object&lt;T&gt; {
    <a href="raw_table.md#0x2_raw_table_borrow">raw_table::borrow</a>&lt;ObjectID, Object&lt;T&gt;&gt;(&self.handle, <a href="object_id.md#0x2_object_id">object_id</a>)
}
</code></pre>



</details>

<a name="0x2_object_storage_borrow_mut"></a>

## Function `borrow_mut`

Borrow mut Object from object store with object_id


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_borrow_mut">borrow_mut</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object_storage.md#0x2_object_storage_ObjectStorage">object_storage::ObjectStorage</a>, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_borrow_mut">borrow_mut</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object_storage.md#0x2_object_storage_ObjectStorage">ObjectStorage</a>, <a href="object_id.md#0x2_object_id">object_id</a>: ObjectID): &<b>mut</b> Object&lt;T&gt; {
    <a href="raw_table.md#0x2_raw_table_borrow_mut">raw_table::borrow_mut</a>&lt;ObjectID, Object&lt;T&gt;&gt;(&self.handle, <a href="object_id.md#0x2_object_id">object_id</a>)
}
</code></pre>



</details>

<a name="0x2_object_storage_remove"></a>

## Function `remove`

Remove object from object store


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_remove">remove</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object_storage.md#0x2_object_storage_ObjectStorage">object_storage::ObjectStorage</a>, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_remove">remove</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object_storage.md#0x2_object_storage_ObjectStorage">ObjectStorage</a>, <a href="object_id.md#0x2_object_id">object_id</a>: ObjectID): Object&lt;T&gt; {
    <a href="raw_table.md#0x2_raw_table_remove">raw_table::remove</a>&lt;ObjectID, Object&lt;T&gt;&gt;(&self.handle, <a href="object_id.md#0x2_object_id">object_id</a>)
}
</code></pre>



</details>

<a name="0x2_object_storage_add"></a>

## Function `add`

Add object to object store


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_add">add</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object_storage.md#0x2_object_storage_ObjectStorage">object_storage::ObjectStorage</a>, obj: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_add">add</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object_storage.md#0x2_object_storage_ObjectStorage">ObjectStorage</a>, obj: Object&lt;T&gt;) {
    <a href="raw_table.md#0x2_raw_table_add">raw_table::add</a>&lt;ObjectID, Object&lt;T&gt;&gt;(&self.handle, <a href="object.md#0x2_object_id">object::id</a>(&obj), obj);
}
</code></pre>



</details>

<a name="0x2_object_storage_contains"></a>

## Function `contains`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_contains">contains</a>(self: &<a href="object_storage.md#0x2_object_storage_ObjectStorage">object_storage::ObjectStorage</a>, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_storage.md#0x2_object_storage_contains">contains</a>(self: &<a href="object_storage.md#0x2_object_storage_ObjectStorage">ObjectStorage</a>, <a href="object_id.md#0x2_object_id">object_id</a>: ObjectID): bool {
    <a href="raw_table.md#0x2_raw_table_contains">raw_table::contains</a>&lt;ObjectID&gt;(&self.handle, <a href="object_id.md#0x2_object_id">object_id</a>)
}
</code></pre>



</details>
