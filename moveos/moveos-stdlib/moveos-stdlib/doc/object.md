
<a name="0x2_object"></a>

# Module `0x2::object`

Move Object
For more details, please refer to https://rooch.network/docs/developer-guides/object


For more details please refer https://rooch.network/docs/developer-guides/object
The Object is a box style Object
The differents with the Object in [Sui](https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/object.move#L75):
1. The Object is a struct in Move
2. The Object is a use case of the Hot Potato pattern in Move. Objects do not have any ability, so they cannot be drop, copy, or store, and can only be handled by StorageContext API after creation.


-  [Struct `ObjectEntity`](#0x2_object_ObjectEntity)
-  [Resource `Object`](#0x2_object_Object)
-  [Struct `ObjectID`](#0x2_object_ObjectID)
-  [Constants](#@Constants_0)
-  [Function `address_to_object_id`](#0x2_object_address_to_object_id)
-  [Function `object_id_to_table_handle`](#0x2_object_object_id_to_table_handle)
-  [Function `singleton_object_id`](#0x2_object_singleton_object_id)
-  [Function `new`](#0x2_object_new)
-  [Function `new_singleton`](#0x2_object_new_singleton)
-  [Function `borrow`](#0x2_object_borrow)
-  [Function `borrow_mut`](#0x2_object_borrow_mut)
-  [Function `remove`](#0x2_object_remove)
-  [Function `to_permanent`](#0x2_object_to_permanent)
-  [Function `to_shared`](#0x2_object_to_shared)
-  [Function `to_frozen`](#0x2_object_to_frozen)
-  [Function `transfer`](#0x2_object_transfer)
-  [Function `transfer_extend`](#0x2_object_transfer_extend)
-  [Function `transfer_to_system`](#0x2_object_transfer_to_system)
-  [Function `id`](#0x2_object_id)
-  [Function `owner`](#0x2_object_owner)
-  [Function `owner_internal`](#0x2_object_owner_internal)
-  [Function `is_shared`](#0x2_object_is_shared)
-  [Function `is_shared_internal`](#0x2_object_is_shared_internal)
-  [Function `is_frozen`](#0x2_object_is_frozen)
-  [Function `is_frozen_internal`](#0x2_object_is_frozen_internal)
-  [Function `as_ref`](#0x2_object_as_ref)
-  [Function `as_mut_ref`](#0x2_object_as_mut_ref)
-  [Function `global_object_storage_handle`](#0x2_object_global_object_storage_handle)
-  [Function `add_to_global`](#0x2_object_add_to_global)
-  [Function `borrow_from_global`](#0x2_object_borrow_from_global)
-  [Function `borrow_mut_from_global`](#0x2_object_borrow_mut_from_global)
-  [Function `remove_from_global`](#0x2_object_remove_from_global)
-  [Function `contains_global`](#0x2_object_contains_global)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="address.md#0x2_address">0x2::address</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="raw_table.md#0x2_raw_table">0x2::raw_table</a>;
<b>use</b> <a href="type_info.md#0x2_type_info">0x2::type_info</a>;
</code></pre>



<a name="0x2_object_ObjectEntity"></a>

## Struct `ObjectEntity`

ObjectEntity<T> is a box of the value of T
It does not have any ability, so it can not be <code>drop</code>, <code><b>copy</b></code>, or <code>store</code>, and can only be handled by storage API after creation.


<pre><code><b>struct</b> <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a></code>
</dt>
<dd>

</dd>
<dt>
<code>owner: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>value: T</code>
</dt>
<dd>
 A flag to indicate whether the object is shared or frozen
</dd>
</dl>


</details>

<a name="0x2_object_Object"></a>

## Resource `Object`

Object<T> is a pointer to the ObjectEntity<T>, It has <code>key</code> and <code>store</code> ability.
It has the same lifetime as the ObjectEntity<T>
Developers only need to use Object<T> related APIs and do not need to know the ObjectEntity<T>.


<pre><code><b>struct</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt; <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x2_object_ObjectID"></a>

## Struct `ObjectID`

ObjectID is a unique identifier for the Object


<pre><code><b>struct</b> <a href="object.md#0x2_object_ObjectID">ObjectID</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x2_object_ErrorInvalidOwnerAddress"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorInvalidOwnerAddress">ErrorInvalidOwnerAddress</a>: u64 = 2;
</code></pre>



<a name="0x2_object_ErrorObjectFrozen"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectFrozen">ErrorObjectFrozen</a>: u64 = 1;
</code></pre>



<a name="0x2_object_GlobalObjectStorageHandle"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_GlobalObjectStorageHandle">GlobalObjectStorageHandle</a>: <b>address</b> = 0;
</code></pre>



<a name="0x2_object_SYSTEM_OWNER_ADDRESS"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_SYSTEM_OWNER_ADDRESS">SYSTEM_OWNER_ADDRESS</a>: <b>address</b> = 0;
</code></pre>



<a name="0x2_object_address_to_object_id"></a>

## Function `address_to_object_id`

Generate a new ObjectID from an address


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_address_to_object_id">address_to_object_id</a>(<b>address</b>: <b>address</b>): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_address_to_object_id">address_to_object_id</a>(<b>address</b>: <b>address</b>): <a href="object.md#0x2_object_ObjectID">ObjectID</a> {
    <a href="object.md#0x2_object_ObjectID">ObjectID</a> { id: <b>address</b> }
}
</code></pre>



</details>

<a name="0x2_object_object_id_to_table_handle"></a>

## Function `object_id_to_table_handle`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_object_id_to_table_handle">object_id_to_table_handle</a>(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_object_id_to_table_handle">object_id_to_table_handle</a>(object_id: <a href="object.md#0x2_object_ObjectID">ObjectID</a>): TableHandle {
    <a href="raw_table.md#0x2_raw_table_new_table_handle">raw_table::new_table_handle</a>(object_id.id)
}
</code></pre>



</details>

<a name="0x2_object_singleton_object_id"></a>

## Function `singleton_object_id`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_singleton_object_id">singleton_object_id</a>&lt;T&gt;(): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_singleton_object_id">singleton_object_id</a>&lt;T&gt;(): <a href="object.md#0x2_object_ObjectID">ObjectID</a> {
    <a href="object.md#0x2_object_address_to_object_id">address_to_object_id</a>(
        <a href="address.md#0x2_address_from_bytes">address::from_bytes</a>(
            <a href="_sha3_256">hash::sha3_256</a>(
                <a href="_to_bytes">bcs::to_bytes</a>(
                    &<a href="type_info.md#0x2_type_info_type_of">type_info::type_of</a>&lt;T&gt;()
                )
            )
        )
    )
}
</code></pre>



</details>

<a name="0x2_object_new"></a>

## Function `new`

Create a new object, the object is owned by <code>System</code> by default.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new">new</a>&lt;T: key&gt;(id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new">new</a>&lt;T: key&gt;(id: <a href="object.md#0x2_object_ObjectID">ObjectID</a>, value: T): <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt; {
    <b>let</b> owner = <a href="object.md#0x2_object_SYSTEM_OWNER_ADDRESS">SYSTEM_OWNER_ADDRESS</a>;
    <b>let</b> entity = <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;{id, value, owner};
    <a href="object.md#0x2_object_add_to_global">add_to_global</a>(entity);
    <a href="object.md#0x2_object_Object">Object</a>{id}
}
</code></pre>



</details>

<a name="0x2_object_new_singleton"></a>

## Function `new_singleton`

Create a new singleton object, singleton object is always owned by <code>System</code> and is p
Singleton object means the object of <code>T</code> is only one instance in the Object Storage.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new_singleton">new_singleton</a>&lt;T: key&gt;(value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new_singleton">new_singleton</a>&lt;T: key&gt;(value: T): <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt; {
    <b>let</b> id = <a href="object.md#0x2_object_singleton_object_id">singleton_object_id</a>&lt;T&gt;();
    <a href="object.md#0x2_object_new">new</a>(id, value)
}
</code></pre>



</details>

<a name="0x2_object_borrow"></a>

## Function `borrow`

Borrow the object value


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow">borrow</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow">borrow</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): &T {
    <b>let</b> obj_enitty = <a href="object.md#0x2_object_borrow_from_global">borrow_from_global</a>&lt;T&gt;(self.id);
    &obj_enitty.value
}
</code></pre>



</details>

<a name="0x2_object_borrow_mut"></a>

## Function `borrow_mut`

Borrow the object mutable value


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut">borrow_mut</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &<b>mut</b> T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut">borrow_mut</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): &<b>mut</b> T {
    <b>let</b> obj_entity = <a href="object.md#0x2_object_borrow_mut_from_global">borrow_mut_from_global</a>&lt;T&gt;(self.id);
    &<b>mut</b> obj_entity.value
}
</code></pre>



</details>

<a name="0x2_object_remove"></a>

## Function `remove`

Remove the object from the global storage, and return the object value
This function is only can be called by the module of <code>T</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_remove">remove</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_remove">remove</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;) : T {
    <b>let</b> <a href="object.md#0x2_object_Object">Object</a>{id} = self;
    <b>let</b> object_entity = <a href="object.md#0x2_object_remove_from_global">remove_from_global</a>&lt;T&gt;(id);
    <b>let</b> <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>{id:_, owner:_, value} = object_entity;
    value
}
</code></pre>



</details>

<a name="0x2_object_to_permanent"></a>

## Function `to_permanent`

Directly drop the Object, and make the Object permanent, the object will can not be removed from the object storage.
If you want to remove the object, please use <code>remove</code> function.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_to_permanent">to_permanent</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_to_permanent">to_permanent</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;) {
    <b>let</b> <a href="object.md#0x2_object_Object">Object</a>{id:_} = self;
}
</code></pre>



</details>

<a name="0x2_object_to_shared"></a>

## Function `to_shared`

Make the Object shared, Any one can get the &mut Object<T> from shared object
The shared object also can be removed from the object storage.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_to_shared">to_shared</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_to_shared">to_shared</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;) {
    <b>let</b> obj_entity = <a href="object.md#0x2_object_borrow_mut_from_global">borrow_mut_from_global</a>&lt;T&gt;(self.id);
    // TODO set the flag
    <a href="object.md#0x2_object_transfer_to_system">transfer_to_system</a>(obj_entity);
    <a href="object.md#0x2_object_to_permanent">to_permanent</a>(self);
}
</code></pre>



</details>

<a name="0x2_object_to_frozen"></a>

## Function `to_frozen`

Make the Object frozen, Any one can not get the &mut Object<T> from frozen object


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_to_frozen">to_frozen</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_to_frozen">to_frozen</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;) {
    <b>let</b> obj_entity = <a href="object.md#0x2_object_borrow_mut_from_global">borrow_mut_from_global</a>&lt;T&gt;(self.id);
    // TODO set the flag
    <a href="object.md#0x2_object_transfer_to_system">transfer_to_system</a>(obj_entity);
    <a href="object.md#0x2_object_to_permanent">to_permanent</a>(self);
}
</code></pre>



</details>

<a name="0x2_object_transfer"></a>

## Function `transfer`

Transfer the object to the new owner
Only the <code>T</code> with <code>store</code> can be directly transferred.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_transfer">transfer</a>&lt;T: store, key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, new_owner: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_transfer">transfer</a>&lt;T: key + store&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;, new_owner: <b>address</b>) {
    <b>let</b> obj_entity = <a href="object.md#0x2_object_borrow_mut_from_global">borrow_mut_from_global</a>&lt;T&gt;(self.id);
    <a href="object.md#0x2_object_transer_internal">transer_internal</a>(obj_entity, new_owner);
}
</code></pre>



</details>

<a name="0x2_object_transfer_extend"></a>

## Function `transfer_extend`

Transfer the object to the new owner
This function is for the module of <code>T</code> to extend the <code>transfer</code> function.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_transfer_extend">transfer_extend</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, new_owner: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_transfer_extend">transfer_extend</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;, new_owner: <b>address</b>) {
    <b>let</b> obj = <a href="object.md#0x2_object_borrow_mut_from_global">borrow_mut_from_global</a>&lt;T&gt;(self.id);
    <a href="object.md#0x2_object_transer_internal">transer_internal</a>(obj, new_owner);
}
</code></pre>



</details>

<a name="0x2_object_transfer_to_system"></a>

## Function `transfer_to_system`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_transfer_to_system">transfer_to_system</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_transfer_to_system">transfer_to_system</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;){
    self.owner = <a href="object.md#0x2_object_SYSTEM_OWNER_ADDRESS">SYSTEM_OWNER_ADDRESS</a>;
}
</code></pre>



</details>

<a name="0x2_object_id"></a>

## Function `id`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_id">id</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_id">id</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): <a href="object.md#0x2_object_ObjectID">ObjectID</a> {
    self.id
}
</code></pre>



</details>

<a name="0x2_object_owner"></a>

## Function `owner`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_owner">owner</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_owner">owner</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): <b>address</b> {
    <b>let</b> obj_enitty = <a href="object.md#0x2_object_borrow_from_global">borrow_from_global</a>&lt;T&gt;(self.id);
    obj_enitty.owner
}
</code></pre>



</details>

<a name="0x2_object_owner_internal"></a>

## Function `owner_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_owner_internal">owner_internal</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_owner_internal">owner_internal</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;): <b>address</b> {
    self.owner
}
</code></pre>



</details>

<a name="0x2_object_is_shared"></a>

## Function `is_shared`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_shared">is_shared</a>&lt;T&gt;(_self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_shared">is_shared</a>&lt;T&gt;(_self: &<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;) : bool {
    // TODO check the flag
    <b>false</b>
}
</code></pre>



</details>

<a name="0x2_object_is_shared_internal"></a>

## Function `is_shared_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_is_shared_internal">is_shared_internal</a>&lt;T&gt;(_self: &<a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_is_shared_internal">is_shared_internal</a>&lt;T&gt;(_self: &<a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;) : bool {
    // TODO check the flag
    <b>false</b>
}
</code></pre>



</details>

<a name="0x2_object_is_frozen"></a>

## Function `is_frozen`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_frozen">is_frozen</a>&lt;T&gt;(_self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_frozen">is_frozen</a>&lt;T&gt;(_self: &<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;) : bool {
    // TODO check the flag
    <b>false</b>
}
</code></pre>



</details>

<a name="0x2_object_is_frozen_internal"></a>

## Function `is_frozen_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_is_frozen_internal">is_frozen_internal</a>&lt;T&gt;(_self: &<a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_is_frozen_internal">is_frozen_internal</a>&lt;T&gt;(_self: &<a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;) : bool {
    // TODO check the flag
    <b>false</b>
}
</code></pre>



</details>

<a name="0x2_object_as_ref"></a>

## Function `as_ref`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_as_ref">as_ref</a>&lt;T: key&gt;(object_entity: &<a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_as_ref">as_ref</a>&lt;T: key&gt;(object_entity: &<a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;) : &<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;{
    <a href="object.md#0x2_object_as_ref_inner">as_ref_inner</a>&lt;<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;&gt;(object_entity.id)
}
</code></pre>



</details>

<a name="0x2_object_as_mut_ref"></a>

## Function `as_mut_ref`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_as_mut_ref">as_mut_ref</a>&lt;T: key&gt;(object_entity: &<b>mut</b> <a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_as_mut_ref">as_mut_ref</a>&lt;T: key&gt;(object_entity: &<b>mut</b> <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;) : &<b>mut</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;{
    <b>assert</b>!(!<a href="object.md#0x2_object_is_frozen_internal">is_frozen_internal</a>(object_entity), <a href="_permission_denied">error::permission_denied</a>(<a href="object.md#0x2_object_ErrorObjectFrozen">ErrorObjectFrozen</a>));
    <a href="object.md#0x2_object_as_mut_ref_inner">as_mut_ref_inner</a>&lt;<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;&gt;(object_entity.id)
}
</code></pre>



</details>

<a name="0x2_object_global_object_storage_handle"></a>

## Function `global_object_storage_handle`

The global object storage's table handle should be <code>0x0</code>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_global_object_storage_handle">global_object_storage_handle</a>(): <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_global_object_storage_handle">global_object_storage_handle</a>(): TableHandle {
    <a href="raw_table.md#0x2_raw_table_new_table_handle">raw_table::new_table_handle</a>(<a href="object.md#0x2_object_GlobalObjectStorageHandle">GlobalObjectStorageHandle</a>)
}
</code></pre>



</details>

<a name="0x2_object_add_to_global"></a>

## Function `add_to_global`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_add_to_global">add_to_global</a>&lt;T: key&gt;(obj: <a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_add_to_global">add_to_global</a>&lt;T: key&gt;(obj: <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;) {
    <a href="raw_table.md#0x2_raw_table_add">raw_table::add</a>&lt;<a href="object.md#0x2_object_ObjectID">ObjectID</a>, <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;&gt;(<a href="object.md#0x2_object_global_object_storage_handle">global_object_storage_handle</a>(), obj.id, obj);
}
</code></pre>



</details>

<a name="0x2_object_borrow_from_global"></a>

## Function `borrow_from_global`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_from_global">borrow_from_global</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_from_global">borrow_from_global</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">ObjectID</a>): &<a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt; {
    <a href="raw_table.md#0x2_raw_table_borrow">raw_table::borrow</a>&lt;<a href="object.md#0x2_object_ObjectID">ObjectID</a>, <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;&gt;(<a href="object.md#0x2_object_global_object_storage_handle">global_object_storage_handle</a>(), object_id)
}
</code></pre>



</details>

<a name="0x2_object_borrow_mut_from_global"></a>

## Function `borrow_mut_from_global`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_mut_from_global">borrow_mut_from_global</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_mut_from_global">borrow_mut_from_global</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt; {
    <a href="raw_table.md#0x2_raw_table_borrow_mut">raw_table::borrow_mut</a>&lt;<a href="object.md#0x2_object_ObjectID">ObjectID</a>, <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;&gt;(<a href="object.md#0x2_object_global_object_storage_handle">global_object_storage_handle</a>(), object_id)
}
</code></pre>



</details>

<a name="0x2_object_remove_from_global"></a>

## Function `remove_from_global`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_remove_from_global">remove_from_global</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): <a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_remove_from_global">remove_from_global</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">ObjectID</a>): <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt; {
    <a href="raw_table.md#0x2_raw_table_remove">raw_table::remove</a>&lt;<a href="object.md#0x2_object_ObjectID">ObjectID</a>, <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;&gt;(<a href="object.md#0x2_object_global_object_storage_handle">global_object_storage_handle</a>(), object_id)
}
</code></pre>



</details>

<a name="0x2_object_contains_global"></a>

## Function `contains_global`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_contains_global">contains_global</a>(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_contains_global">contains_global</a>(object_id: <a href="object.md#0x2_object_ObjectID">ObjectID</a>): bool {
    <a href="raw_table.md#0x2_raw_table_contains">raw_table::contains</a>&lt;<a href="object.md#0x2_object_ObjectID">ObjectID</a>&gt;(<a href="object.md#0x2_object_global_object_storage_handle">global_object_storage_handle</a>(), object_id)
}
</code></pre>



</details>
