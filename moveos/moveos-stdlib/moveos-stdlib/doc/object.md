
<a name="0x2_object"></a>

# Module `0x2::object`

Move Object
For more details, please refer to https://rooch.network/docs/developer-guides/object


-  [Resource `Root`](#0x2_object_Root)
-  [Struct `ObjectEntity`](#0x2_object_ObjectEntity)
-  [Resource `Object`](#0x2_object_Object)
-  [Struct `TestStructID`](#0x2_object_TestStructID)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x2_object_new)
-  [Function `new_named_object`](#0x2_object_new_named_object)
-  [Function `new_account_named_object`](#0x2_object_new_account_named_object)
-  [Function `new_custom_object`](#0x2_object_new_custom_object)
-  [Function `new_with_id`](#0x2_object_new_with_id)
-  [Function `borrow`](#0x2_object_borrow)
-  [Function `borrow_mut`](#0x2_object_borrow_mut)
-  [Function `exists_object`](#0x2_object_exists_object)
-  [Function `exists_object_with_type`](#0x2_object_exists_object_with_type)
-  [Function `borrow_object`](#0x2_object_borrow_object)
-  [Function `borrow_mut_object`](#0x2_object_borrow_mut_object)
-  [Function `borrow_mut_object_extend`](#0x2_object_borrow_mut_object_extend)
-  [Function `take_object`](#0x2_object_take_object)
-  [Function `take_object_extend`](#0x2_object_take_object_extend)
-  [Function `borrow_mut_object_shared`](#0x2_object_borrow_mut_object_shared)
-  [Function `remove`](#0x2_object_remove)
-  [Function `remove_unchecked`](#0x2_object_remove_unchecked)
-  [Function `to_shared`](#0x2_object_to_shared)
-  [Function `is_shared`](#0x2_object_is_shared)
-  [Function `to_frozen`](#0x2_object_to_frozen)
-  [Function `is_frozen`](#0x2_object_is_frozen)
-  [Function `is_bound`](#0x2_object_is_bound)
-  [Function `is_bound_internal`](#0x2_object_is_bound_internal)
-  [Function `to_user_owned`](#0x2_object_to_user_owned)
-  [Function `to_system_owned`](#0x2_object_to_system_owned)
-  [Function `to_system_owned_internal`](#0x2_object_to_system_owned_internal)
-  [Function `transfer`](#0x2_object_transfer)
-  [Function `transfer_extend`](#0x2_object_transfer_extend)
-  [Function `id`](#0x2_object_id)
-  [Function `owner`](#0x2_object_owner)
-  [Function `owner_internal`](#0x2_object_owner_internal)
-  [Function `is_system_owned`](#0x2_object_is_system_owned)
-  [Function `is_user_owned_internal`](#0x2_object_is_user_owned_internal)
-  [Function `is_user_owned`](#0x2_object_is_user_owned)
-  [Function `as_ref`](#0x2_object_as_ref)
-  [Function `as_mut_ref`](#0x2_object_as_mut_ref)
-  [Function `mut_entity_as_object`](#0x2_object_mut_entity_as_object)
-  [Function `global_object_storage_handle`](#0x2_object_global_object_storage_handle)
-  [Function `add_to_global`](#0x2_object_add_to_global)
-  [Function `borrow_root_object`](#0x2_object_borrow_root_object)
-  [Function `borrow_from_global`](#0x2_object_borrow_from_global)
-  [Function `borrow_mut_root_object`](#0x2_object_borrow_mut_root_object)
-  [Function `borrow_mut_from_global`](#0x2_object_borrow_mut_from_global)
-  [Function `remove_from_global`](#0x2_object_remove_from_global)
-  [Function `contains_global`](#0x2_object_contains_global)
-  [Function `add_field`](#0x2_object_add_field)
-  [Function `add_field_internal`](#0x2_object_add_field_internal)
-  [Function `borrow_field`](#0x2_object_borrow_field)
-  [Function `borrow_field_internal`](#0x2_object_borrow_field_internal)
-  [Function `borrow_field_with_default`](#0x2_object_borrow_field_with_default)
-  [Function `borrow_field_with_default_internal`](#0x2_object_borrow_field_with_default_internal)
-  [Function `borrow_mut_field`](#0x2_object_borrow_mut_field)
-  [Function `borrow_mut_field_internal`](#0x2_object_borrow_mut_field_internal)
-  [Function `borrow_mut_field_with_default`](#0x2_object_borrow_mut_field_with_default)
-  [Function `borrow_mut_field_with_default_internal`](#0x2_object_borrow_mut_field_with_default_internal)
-  [Function `upsert_field`](#0x2_object_upsert_field)
-  [Function `upsert_field_internal`](#0x2_object_upsert_field_internal)
-  [Function `remove_field`](#0x2_object_remove_field)
-  [Function `remove_field_internal`](#0x2_object_remove_field_internal)
-  [Function `contains_field`](#0x2_object_contains_field)
-  [Function `contains_field_internal`](#0x2_object_contains_field_internal)
-  [Function `field_size`](#0x2_object_field_size)
-  [Function `field_size_internal`](#0x2_object_field_size_internal)


<pre><code><b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
<b>use</b> <a href="raw_table.md#0x2_raw_table">0x2::raw_table</a>;
<b>use</b> <a href="signer.md#0x2_signer">0x2::signer</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_object_Root"></a>

## Resource `Root`



<pre><code><b>struct</b> <a href="object.md#0x2_object_Root">Root</a> <b>has</b> key
</code></pre>



<a name="0x2_object_ObjectEntity"></a>

## Struct `ObjectEntity`

ObjectEntity<T> is a box of the value of T
It does not have any ability, so it can not be <code>drop</code>, <code><b>copy</b></code>, or <code>store</code>, and can only be handled by storage API after creation.


<pre><code><b>struct</b> <a href="object.md#0x2_object_ObjectEntity">ObjectEntity</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_Object"></a>

## Resource `Object`

Object<T> is a pointer to the ObjectEntity<T>, It has <code>key</code> and <code>store</code> ability.
It has the same lifetime as the ObjectEntity<T>
Developers only need to use Object<T> related APIs and do not need to know the ObjectEntity<T>.


<pre><code><b>struct</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt; <b>has</b> store, key
</code></pre>



<a name="0x2_object_TestStructID"></a>

## Struct `TestStructID`



<pre><code><b>struct</b> <a href="object.md#0x2_object_TestStructID">TestStructID</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_object_BOUND_OBJECT_FLAG_MASK"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_BOUND_OBJECT_FLAG_MASK">BOUND_OBJECT_FLAG_MASK</a>: u8 = 4;
</code></pre>



<a name="0x2_object_ErrorInvalidOwnerAddress"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorInvalidOwnerAddress">ErrorInvalidOwnerAddress</a>: u64 = 3;
</code></pre>



<a name="0x2_object_ErrorObjectAlreadyBorrowed"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectAlreadyBorrowed">ErrorObjectAlreadyBorrowed</a>: u64 = 7;
</code></pre>



<a name="0x2_object_ErrorObjectAlreadyExist"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectAlreadyExist">ErrorObjectAlreadyExist</a>: u64 = 1;
</code></pre>



<a name="0x2_object_ErrorObjectContainsDynamicFields"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectContainsDynamicFields">ErrorObjectContainsDynamicFields</a>: u64 = 8;
</code></pre>



<a name="0x2_object_ErrorObjectFrozen"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectFrozen">ErrorObjectFrozen</a>: u64 = 2;
</code></pre>



<a name="0x2_object_ErrorObjectIsBound"></a>

Can not take out the object which is bound to the account


<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectIsBound">ErrorObjectIsBound</a>: u64 = 6;
</code></pre>



<a name="0x2_object_ErrorObjectNotShared"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectNotShared">ErrorObjectNotShared</a>: u64 = 5;
</code></pre>



<a name="0x2_object_ErrorObjectOwnerNotMatch"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectOwnerNotMatch">ErrorObjectOwnerNotMatch</a>: u64 = 4;
</code></pre>



<a name="0x2_object_FROZEN_OBJECT_FLAG_MASK"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_FROZEN_OBJECT_FLAG_MASK">FROZEN_OBJECT_FLAG_MASK</a>: u8 = 2;
</code></pre>



<a name="0x2_object_GlobalObjectStorageHandleID"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_GlobalObjectStorageHandleID">GlobalObjectStorageHandleID</a>: <b>address</b> = 0x0;
</code></pre>



<a name="0x2_object_SHARED_OBJECT_FLAG_MASK"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_SHARED_OBJECT_FLAG_MASK">SHARED_OBJECT_FLAG_MASK</a>: u8 = 1;
</code></pre>



<a name="0x2_object_SPARSE_MERKLE_PLACEHOLDER_HASH"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_SPARSE_MERKLE_PLACEHOLDER_HASH">SPARSE_MERKLE_PLACEHOLDER_HASH</a>: <b>address</b> = 0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000;
</code></pre>



<a name="0x2_object_SYSTEM_OWNER_ADDRESS"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_SYSTEM_OWNER_ADDRESS">SYSTEM_OWNER_ADDRESS</a>: <b>address</b> = 0x0;
</code></pre>



<a name="0x2_object_new"></a>

## Function `new`

Create a new Object, Add the Object to the global object storage and return the Object


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_new">new</a>&lt;T: key&gt;(value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_new_named_object"></a>

## Function `new_named_object`

Create a new named Object, the ObjectID is generated by the type_name of <code>T</code>


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_new_named_object">new_named_object</a>&lt;T: key&gt;(value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_new_account_named_object"></a>

## Function `new_account_named_object`

Create a new account named object, the ObjectID is generated by the account address and type_name of <code>T</code>


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_new_account_named_object">new_account_named_object</a>&lt;T: key&gt;(<a href="account.md#0x2_account">account</a>: <b>address</b>, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_new_custom_object"></a>

## Function `new_custom_object`

Create a new custom object, the ObjectID is generated by the <code>id</code> and type_name of <code>T</code>


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_new_custom_object">new_custom_object</a>&lt;ID: drop, T: key&gt;(id: ID, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_new_with_id"></a>

## Function `new_with_id`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new_with_id">new_with_id</a>&lt;T: key&gt;(id: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_borrow"></a>

## Function `borrow`

Borrow the object value


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow">borrow</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &T
</code></pre>



<a name="0x2_object_borrow_mut"></a>

## Function `borrow_mut`

Borrow the object mutable value


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut">borrow_mut</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &<b>mut</b> T
</code></pre>



<a name="0x2_object_exists_object"></a>

## Function `exists_object`

Check if the object with <code><a href="object_id.md#0x2_object_id">object_id</a></code> exists in the global object storage


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_exists_object">exists_object</a>(<a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): bool
</code></pre>



<a name="0x2_object_exists_object_with_type"></a>

## Function `exists_object_with_type`

Check if the object exists in the global object storage and the type of the object is <code>T</code>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_exists_object_with_type">exists_object_with_type</a>&lt;T: key&gt;(<a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): bool
</code></pre>



<a name="0x2_object_borrow_object"></a>

## Function `borrow_object`

Borrow Object from object store by object_id
Any one can borrow an <code>&<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;</code> from the global object storage


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_object">borrow_object</a>&lt;T: key&gt;(<a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_borrow_mut_object"></a>

## Function `borrow_mut_object`

Borrow mut Object by <code>owner</code> and <code><a href="object_id.md#0x2_object_id">object_id</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_object">borrow_mut_object</a>&lt;T: key&gt;(owner: &<a href="">signer</a>, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_borrow_mut_object_extend"></a>

## Function `borrow_mut_object_extend`

Borrow mut Object by <code><a href="object_id.md#0x2_object_id">object_id</a></code>


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_object_extend">borrow_mut_object_extend</a>&lt;T: key&gt;(<a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_take_object"></a>

## Function `take_object`

Take out the UserOwnedObject by <code>owner</code> and <code><a href="object_id.md#0x2_object_id">object_id</a></code>
The <code>T</code> must have <code>key + store</code> ability.
Note: When the Object is taken out, the Object will auto become <code>SystemOwned</code> Object.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_take_object">take_object</a>&lt;T: store, key&gt;(owner: &<a href="">signer</a>, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_take_object_extend"></a>

## Function `take_object_extend`

Take out the UserOwnedObject by <code><a href="object_id.md#0x2_object_id">object_id</a></code>, return the owner and Object
This function is for developer to extend, Only the module of <code>T</code> can take out the <code>UserOwnedObject</code> with object_id.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_take_object_extend">take_object_extend</a>&lt;T: key&gt;(<a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): (<b>address</b>, <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<a name="0x2_object_borrow_mut_object_shared"></a>

## Function `borrow_mut_object_shared`

Borrow mut Shared Object by object_id


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_object_shared">borrow_mut_object_shared</a>&lt;T: key&gt;(<a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_remove"></a>

## Function `remove`

Remove the object from the global storage, and return the object value
This function is only can be called by the module of <code>T</code>.
The caller must ensure that the dynamic fields are empty before delete the Object


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_remove">remove</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): T
</code></pre>



<a name="0x2_object_remove_unchecked"></a>

## Function `remove_unchecked`

Remove the object from the global storage, and return the object value
Do not check if the dynamic fields are empty


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_remove_unchecked">remove_unchecked</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): T
</code></pre>



<a name="0x2_object_to_shared"></a>

## Function `to_shared`

Make the Object shared, Any one can get the &mut Object<T> from shared object
The shared object also can be removed from the object storage.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_to_shared">to_shared</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<a name="0x2_object_is_shared"></a>

## Function `is_shared`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_shared">is_shared</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): bool
</code></pre>



<a name="0x2_object_to_frozen"></a>

## Function `to_frozen`

Make the Object frozen, Any one can not get the &mut Object<T> from frozen object


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_to_frozen">to_frozen</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<a name="0x2_object_is_frozen"></a>

## Function `is_frozen`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_frozen">is_frozen</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): bool
</code></pre>



<a name="0x2_object_is_bound"></a>

## Function `is_bound`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_bound">is_bound</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): bool
</code></pre>



<a name="0x2_object_is_bound_internal"></a>

## Function `is_bound_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_is_bound_internal">is_bound_internal</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;): bool
</code></pre>



<a name="0x2_object_to_user_owned"></a>

## Function `to_user_owned`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_to_user_owned">to_user_owned</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, new_owner: <b>address</b>)
</code></pre>



<a name="0x2_object_to_system_owned"></a>

## Function `to_system_owned`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_to_system_owned">to_system_owned</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<a name="0x2_object_to_system_owned_internal"></a>

## Function `to_system_owned_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_to_system_owned_internal">to_system_owned_internal</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;)
</code></pre>



<a name="0x2_object_transfer"></a>

## Function `transfer`

Transfer the object to the new owner
Only the <code>T</code> with <code>store</code> can be directly transferred.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_transfer">transfer</a>&lt;T: store, key&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, new_owner: <b>address</b>)
</code></pre>



<a name="0x2_object_transfer_extend"></a>

## Function `transfer_extend`

Transfer the object to the new owner
This function is for the module of <code>T</code> to extend the <code>transfer</code> function.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_transfer_extend">transfer_extend</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, new_owner: <b>address</b>)
</code></pre>



<a name="0x2_object_id"></a>

## Function `id`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_id">id</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<a name="0x2_object_owner"></a>

## Function `owner`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_owner">owner</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): <b>address</b>
</code></pre>



<a name="0x2_object_owner_internal"></a>

## Function `owner_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_owner_internal">owner_internal</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;): <b>address</b>
</code></pre>



<a name="0x2_object_is_system_owned"></a>

## Function `is_system_owned`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_system_owned">is_system_owned</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): bool
</code></pre>



<a name="0x2_object_is_user_owned_internal"></a>

## Function `is_user_owned_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_is_user_owned_internal">is_user_owned_internal</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;): bool
</code></pre>



<a name="0x2_object_is_user_owned"></a>

## Function `is_user_owned`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_user_owned">is_user_owned</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): bool
</code></pre>



<a name="0x2_object_as_ref"></a>

## Function `as_ref`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_as_ref">as_ref</a>&lt;T: key&gt;(object_entity: &<a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_as_mut_ref"></a>

## Function `as_mut_ref`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_as_mut_ref">as_mut_ref</a>&lt;T: key&gt;(object_entity: &<b>mut</b> <a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_mut_entity_as_object"></a>

## Function `mut_entity_as_object`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_mut_entity_as_object">mut_entity_as_object</a>&lt;T: key&gt;(object_entity: &<b>mut</b> <a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_global_object_storage_handle"></a>

## Function `global_object_storage_handle`

The global object storage's table handle should be <code>0x0</code>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_global_object_storage_handle">global_object_storage_handle</a>(): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<a name="0x2_object_add_to_global"></a>

## Function `add_to_global`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_add_to_global">add_to_global</a>&lt;T: key&gt;(obj: <a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;)
</code></pre>



<a name="0x2_object_borrow_root_object"></a>

## Function `borrow_root_object`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_root_object">borrow_root_object</a>(): &<a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;<a href="object.md#0x2_object_Root">object::Root</a>&gt;
</code></pre>



<a name="0x2_object_borrow_from_global"></a>

## Function `borrow_from_global`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_from_global">borrow_from_global</a>&lt;T: key&gt;(<a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): &<a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_borrow_mut_root_object"></a>

## Function `borrow_mut_root_object`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_mut_root_object">borrow_mut_root_object</a>(): &<b>mut</b> <a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;<a href="object.md#0x2_object_Root">object::Root</a>&gt;
</code></pre>



<a name="0x2_object_borrow_mut_from_global"></a>

## Function `borrow_mut_from_global`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_mut_from_global">borrow_mut_from_global</a>&lt;T: key&gt;(<a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_remove_from_global"></a>

## Function `remove_from_global`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_remove_from_global">remove_from_global</a>&lt;T: key&gt;(<a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): <a href="object.md#0x2_object_ObjectEntity">object::ObjectEntity</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_contains_global"></a>

## Function `contains_global`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_contains_global">contains_global</a>(<a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): bool
</code></pre>



<a name="0x2_object_add_field"></a>

## Function `add_field`

Add a dynamic filed to the object. Aborts if an entry for this
key already exists. The entry itself is not stored in the
table, and cannot be discovered from it.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_add_field">add_field</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K, val: V)
</code></pre>



<a name="0x2_object_add_field_internal"></a>

## Function `add_field_internal`

Add a new entry to the table. Aborts if an entry for this
key already exists. The entry itself is not stored in the
table, and cannot be discovered from it.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_add_field_internal">add_field_internal</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(table_handle: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K, val: V)
</code></pre>



<a name="0x2_object_borrow_field"></a>

## Function `borrow_field`

Acquire an immutable reference to the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_field">borrow_field</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): &V
</code></pre>



<a name="0x2_object_borrow_field_internal"></a>

## Function `borrow_field_internal`

Acquire an immutable reference to the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_field_internal">borrow_field_internal</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K): &V
</code></pre>



<a name="0x2_object_borrow_field_with_default"></a>

## Function `borrow_field_with_default`

Acquire an immutable reference to the value which <code>key</code> maps to.
Returns specified default value if there is no entry for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_field_with_default">borrow_field_with_default</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K, default: &V): &V
</code></pre>



<a name="0x2_object_borrow_field_with_default_internal"></a>

## Function `borrow_field_with_default_internal`

Acquire an immutable reference to the value which <code>key</code> maps to.
Returns specified default value if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_field_with_default_internal">borrow_field_with_default_internal</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K, default: &V): &V
</code></pre>



<a name="0x2_object_borrow_mut_field"></a>

## Function `borrow_mut_field`

Acquire a mutable reference to the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_field">borrow_mut_field</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): &<b>mut</b> V
</code></pre>



<a name="0x2_object_borrow_mut_field_internal"></a>

## Function `borrow_mut_field_internal`

Acquire a mutable reference to the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_mut_field_internal">borrow_mut_field_internal</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K): &<b>mut</b> V
</code></pre>



<a name="0x2_object_borrow_mut_field_with_default"></a>

## Function `borrow_mut_field_with_default`

Acquire a mutable reference to the value which <code>key</code> maps to.
Insert the pair (<code>key</code>, <code>default</code>) first if there is no entry for <code>key</code>.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_field_with_default">borrow_mut_field_with_default</a>&lt;T: key, K: <b>copy</b>, drop, V: drop&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K, default: V): &<b>mut</b> V
</code></pre>



<a name="0x2_object_borrow_mut_field_with_default_internal"></a>

## Function `borrow_mut_field_with_default_internal`

Acquire a mutable reference to the value which <code>key</code> maps to.
Insert the pair (<code>key</code>, <code>default</code>) first if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_mut_field_with_default_internal">borrow_mut_field_with_default_internal</a>&lt;T: key, K: <b>copy</b>, drop, V: drop&gt;(table_handle: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K, default: V): &<b>mut</b> V
</code></pre>



<a name="0x2_object_upsert_field"></a>

## Function `upsert_field`

Insert the pair (<code>key</code>, <code>value</code>) if there is no entry for <code>key</code>.
update the value of the entry for <code>key</code> to <code>value</code> otherwise


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_upsert_field">upsert_field</a>&lt;T: key, K: <b>copy</b>, drop, V: drop&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K, value: V)
</code></pre>



<a name="0x2_object_upsert_field_internal"></a>

## Function `upsert_field_internal`

Insert the pair (<code>key</code>, <code>value</code>) if there is no entry for <code>key</code>.
update the value of the entry for <code>key</code> to <code>value</code> otherwise


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_upsert_field_internal">upsert_field_internal</a>&lt;T: key, K: <b>copy</b>, drop, V: drop&gt;(table_handle: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K, value: V)
</code></pre>



<a name="0x2_object_remove_field"></a>

## Function `remove_field`

Remove from <code><a href="table.md#0x2_table">table</a></code> and return the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_remove_field">remove_field</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): V
</code></pre>



<a name="0x2_object_remove_field_internal"></a>

## Function `remove_field_internal`

Remove from <code><a href="table.md#0x2_table">table</a></code> and return the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_remove_field_internal">remove_field_internal</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(table_handle: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K): V
</code></pre>



<a name="0x2_object_contains_field"></a>

## Function `contains_field`

Returns true if <code><a href="table.md#0x2_table">table</a></code> contains an entry for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_contains_field">contains_field</a>&lt;T: key, K: <b>copy</b>, drop&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): bool
</code></pre>



<a name="0x2_object_contains_field_internal"></a>

## Function `contains_field_internal`

Returns true if <code><a href="table.md#0x2_table">table</a></code> contains an entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_contains_field_internal">contains_field_internal</a>&lt;K: <b>copy</b>, drop&gt;(table_handle: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K): bool
</code></pre>



<a name="0x2_object_field_size"></a>

## Function `field_size`

Returns the size of the table, the number of key-value pairs


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_field_size">field_size</a>&lt;T: key&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): u64
</code></pre>



<a name="0x2_object_field_size_internal"></a>

## Function `field_size_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_field_size_internal">field_size_internal</a>&lt;T: key&gt;(<a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): u64
</code></pre>
