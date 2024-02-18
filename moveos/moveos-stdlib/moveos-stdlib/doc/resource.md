
<a name="0x2_resource"></a>

# Module `0x2::resource`

ResourceObject is part of the StorageAbstraction
It is used to store the account's resources


-  [Resource `Resource`](#0x2_resource_Resource)
-  [Constants](#@Constants_0)
-  [Function `resource_object_id`](#0x2_resource_resource_object_id)
-  [Function `create_resource_object`](#0x2_resource_create_resource_object)
-  [Function `borrow_resource`](#0x2_resource_borrow_resource)
-  [Function `borrow_mut_resource`](#0x2_resource_borrow_mut_resource)
-  [Function `move_resource_to`](#0x2_resource_move_resource_to)
-  [Function `move_resource_from`](#0x2_resource_move_resource_from)
-  [Function `exists_resource`](#0x2_resource_exists_resource)
-  [Function `transfer`](#0x2_resource_transfer)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
<b>use</b> <a href="type_table.md#0x2_type_table">0x2::type_table</a>;
</code></pre>



<a name="0x2_resource_Resource"></a>

## Resource `Resource`



<pre><code><b>struct</b> <a href="resource.md#0x2_resource_Resource">Resource</a> <b>has</b> store, key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_resource_ErrorResourceAlreadyExists"></a>

The resource with the given type already exists


<pre><code><b>const</b> <a href="resource.md#0x2_resource_ErrorResourceAlreadyExists">ErrorResourceAlreadyExists</a>: u64 = 1;
</code></pre>



<a name="0x2_resource_ErrorResourceNotExists"></a>

The resource with the given type not exists


<pre><code><b>const</b> <a href="resource.md#0x2_resource_ErrorResourceNotExists">ErrorResourceNotExists</a>: u64 = 2;
</code></pre>



<a name="0x2_resource_resource_object_id"></a>

## Function `resource_object_id`



<pre><code><b>public</b> <b>fun</b> <a href="resource.md#0x2_resource_resource_object_id">resource_object_id</a>(account: <b>address</b>): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<a name="0x2_resource_create_resource_object"></a>

## Function `create_resource_object`

Create a new resource object space


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="resource.md#0x2_resource_create_resource_object">create_resource_object</a>(account: <b>address</b>)
</code></pre>



<a name="0x2_resource_borrow_resource"></a>

## Function `borrow_resource`



<pre><code><b>public</b> <b>fun</b> <a href="resource.md#0x2_resource_borrow_resource">borrow_resource</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="resource.md#0x2_resource_Resource">resource::Resource</a>&gt;): &T
</code></pre>



<a name="0x2_resource_borrow_mut_resource"></a>

## Function `borrow_mut_resource`



<pre><code><b>public</b> <b>fun</b> <a href="resource.md#0x2_resource_borrow_mut_resource">borrow_mut_resource</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="resource.md#0x2_resource_Resource">resource::Resource</a>&gt;): &<b>mut</b> T
</code></pre>



<a name="0x2_resource_move_resource_to"></a>

## Function `move_resource_to`



<pre><code><b>public</b> <b>fun</b> <a href="resource.md#0x2_resource_move_resource_to">move_resource_to</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="resource.md#0x2_resource_Resource">resource::Resource</a>&gt;, <a href="resource.md#0x2_resource">resource</a>: T)
</code></pre>



<a name="0x2_resource_move_resource_from"></a>

## Function `move_resource_from`



<pre><code><b>public</b> <b>fun</b> <a href="resource.md#0x2_resource_move_resource_from">move_resource_from</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="resource.md#0x2_resource_Resource">resource::Resource</a>&gt;): T
</code></pre>



<a name="0x2_resource_exists_resource"></a>

## Function `exists_resource`



<pre><code><b>public</b> <b>fun</b> <a href="resource.md#0x2_resource_exists_resource">exists_resource</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="resource.md#0x2_resource_Resource">resource::Resource</a>&gt;): bool
</code></pre>



<a name="0x2_resource_transfer"></a>

## Function `transfer`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="resource.md#0x2_resource_transfer">transfer</a>(obj: <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="resource.md#0x2_resource_Resource">resource::Resource</a>&gt;, account: <b>address</b>)
</code></pre>
