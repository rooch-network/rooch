
<a name="0x2_storage_context"></a>

# Module `0x2::storage_context`

StorageContext is part of the StorageAbstraction
TODO we need to redegin the StorageContext and AppStorageContext


-  [Struct `StorageContext`](#0x2_storage_context_StorageContext)
-  [Function `new_with_id`](#0x2_storage_context_new_with_id)


<pre><code><b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
</code></pre>



<a name="0x2_storage_context_StorageContext"></a>

## Struct `StorageContext`



<pre><code><b>struct</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a> <b>has</b> store
</code></pre>



<a name="0x2_storage_context_new_with_id"></a>

## Function `new_with_id`

Create a new StorageContext with a given ObjectID.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="storage_context.md#0x2_storage_context_new_with_id">new_with_id</a>(handle: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>
</code></pre>
