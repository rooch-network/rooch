
<a name="0x2_object"></a>

# Module `0x2::object`

Move Object
For more details, please refer to https://rooch.network/docs/developer-guides/object


-  [Struct `ObjectID`](#0x2_object_ObjectID)
-  [Resource `Object`](#0x2_object_Object)
-  [Resource `DynamicField`](#0x2_object_DynamicField)
-  [Constants](#@Constants_0)
-  [Function `has_parent`](#0x2_object_has_parent)
-  [Function `parent_id`](#0x2_object_parent_id)
-  [Function `child_id`](#0x2_object_child_id)
-  [Function `is_parent`](#0x2_object_is_parent)
-  [Function `is_root`](#0x2_object_is_root)
-  [Function `address_to_object_id`](#0x2_object_address_to_object_id)
-  [Function `named_object_id`](#0x2_object_named_object_id)
-  [Function `account_named_object_id`](#0x2_object_account_named_object_id)
-  [Function `custom_object_id`](#0x2_object_custom_object_id)
-  [Function `custom_object_id_with_parent`](#0x2_object_custom_object_id_with_parent)
-  [Function `new`](#0x2_object_new)
-  [Function `new_with_id`](#0x2_object_new_with_id)
-  [Function `new_named_object`](#0x2_object_new_named_object)
-  [Function `new_account_named_object`](#0x2_object_new_account_named_object)
-  [Function `new_with_object_id`](#0x2_object_new_with_object_id)
-  [Function `new_with_parent`](#0x2_object_new_with_parent)
-  [Function `new_with_parent_and_id`](#0x2_object_new_with_parent_and_id)
-  [Function `new_with_parent_and_key`](#0x2_object_new_with_parent_and_key)
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
-  [Function `transfer`](#0x2_object_transfer)
-  [Function `transfer_extend`](#0x2_object_transfer_extend)
-  [Function `id`](#0x2_object_id)
-  [Function `owner`](#0x2_object_owner)
-  [Function `is_system_owned`](#0x2_object_is_system_owned)
-  [Function `is_user_owned`](#0x2_object_is_user_owned)
-  [Function `add_field`](#0x2_object_add_field)
-  [Function `add_field_internal`](#0x2_object_add_field_internal)
-  [Function `borrow_field`](#0x2_object_borrow_field)
-  [Function `borrow_field_internal`](#0x2_object_borrow_field_internal)
-  [Function `borrow_field_with_default`](#0x2_object_borrow_field_with_default)
-  [Function `borrow_mut_field`](#0x2_object_borrow_mut_field)
-  [Function `borrow_mut_field_internal`](#0x2_object_borrow_mut_field_internal)
-  [Function `borrow_mut_field_with_default`](#0x2_object_borrow_mut_field_with_default)
-  [Function `upsert_field`](#0x2_object_upsert_field)
-  [Function `remove_field`](#0x2_object_remove_field)
-  [Function `remove_field_internal`](#0x2_object_remove_field_internal)
-  [Function `contains_field`](#0x2_object_contains_field)
-  [Function `contains_field_internal`](#0x2_object_contains_field_internal)
-  [Function `contains_field_with_type`](#0x2_object_contains_field_with_type)
-  [Function `field_size`](#0x2_object_field_size)


<pre><code><b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="address.md#0x2_address">0x2::address</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="signer.md#0x2_signer">0x2::signer</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
<b>use</b> <a href="type_info.md#0x2_type_info">0x2::type_info</a>;
</code></pre>



<a name="0x2_object_ObjectID"></a>

## Struct `ObjectID`

ObjectID is a unique identifier for the Object


<pre><code>#[data_struct]
<b>struct</b> <a href="object.md#0x2_object_ObjectID">ObjectID</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x2_object_Object"></a>

## Resource `Object`

Object<T> is a pointer type to the Object in storage, It has <code>key</code> and <code>store</code> ability.


<pre><code><b>struct</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt; <b>has</b> store, key
</code></pre>



<a name="0x2_object_DynamicField"></a>

## Resource `DynamicField`

The dynamic field


<pre><code><b>struct</b> <a href="object.md#0x2_object_DynamicField">DynamicField</a>&lt;Name, Value&gt; <b>has</b> store, key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_object_ErrorTypeMismatch"></a>

The type of the object or field is mismatch


<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorTypeMismatch">ErrorTypeMismatch</a>: u64 = 10;
</code></pre>



<a name="0x2_object_ErrorAlreadyExists"></a>

The Object or dynamic field already exists


<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorAlreadyExists">ErrorAlreadyExists</a>: u64 = 1;
</code></pre>



<a name="0x2_object_ErrorChildObjectTooDeep"></a>

The child object level is too deep


<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorChildObjectTooDeep">ErrorChildObjectTooDeep</a>: u64 = 11;
</code></pre>



<a name="0x2_object_ErrorFieldsNotEmpty"></a>

The dynamic fields is not empty


<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorFieldsNotEmpty">ErrorFieldsNotEmpty</a>: u64 = 8;
</code></pre>



<a name="0x2_object_ErrorInvalidOwnerAddress"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorInvalidOwnerAddress">ErrorInvalidOwnerAddress</a>: u64 = 3;
</code></pre>



<a name="0x2_object_ErrorNotFound"></a>

Can not found the Object or dynamic field


<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorNotFound">ErrorNotFound</a>: u64 = 2;
</code></pre>



<a name="0x2_object_ErrorObjectAlreadyBorrowed"></a>

The object or field is already borrowed


<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectAlreadyBorrowed">ErrorObjectAlreadyBorrowed</a>: u64 = 7;
</code></pre>



<a name="0x2_object_ErrorObjectAlreadyTakenOutOrEmbeded"></a>

The object or field is already taken out or embedded in other struct


<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectAlreadyTakenOutOrEmbeded">ErrorObjectAlreadyTakenOutOrEmbeded</a>: u64 = 15;
</code></pre>



<a name="0x2_object_ErrorObjectFrozen"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectFrozen">ErrorObjectFrozen</a>: u64 = 9;
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



<a name="0x2_object_ErrorObjectRuntimeError"></a>

The object runtime error


<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectRuntimeError">ErrorObjectRuntimeError</a>: u64 = 14;
</code></pre>



<a name="0x2_object_ErrorParentNotMatch"></a>

The parent object is not match


<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorParentNotMatch">ErrorParentNotMatch</a>: u64 = 13;
</code></pre>



<a name="0x2_object_ErrorWithoutParent"></a>

The object has no parent


<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorWithoutParent">ErrorWithoutParent</a>: u64 = 12;
</code></pre>



<a name="0x2_object_FROZEN_OBJECT_FLAG_MASK"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_FROZEN_OBJECT_FLAG_MASK">FROZEN_OBJECT_FLAG_MASK</a>: u8 = 2;
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



<a name="0x2_object_has_parent"></a>

## Function `has_parent`

Check if the object_id has parent
The object_id has parent means the object_id is not the root object_id


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_has_parent">has_parent</a>(object_id: &<a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): bool
</code></pre>



<a name="0x2_object_parent_id"></a>

## Function `parent_id`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_parent_id">parent_id</a>(object_id: &<a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_object_child_id"></a>

## Function `child_id`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_child_id">child_id</a>(parent_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, key: <b>address</b>): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_object_is_parent"></a>

## Function `is_parent`

Check if the <code>parent</code> is the parent of the <code>child</code>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_parent">is_parent</a>(parent: &<a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, child: &<a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): bool
</code></pre>



<a name="0x2_object_is_root"></a>

## Function `is_root`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_root">is_root</a>(object_id: &<a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): bool
</code></pre>



<a name="0x2_object_address_to_object_id"></a>

## Function `address_to_object_id`

Generate a new ObjectID from an address


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_address_to_object_id">address_to_object_id</a>(<b>address</b>: <b>address</b>): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_object_named_object_id"></a>

## Function `named_object_id`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_named_object_id">named_object_id</a>&lt;T&gt;(): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_object_account_named_object_id"></a>

## Function `account_named_object_id`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_account_named_object_id">account_named_object_id</a>&lt;T: key&gt;(<a href="account.md#0x2_account">account</a>: <b>address</b>): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_object_custom_object_id"></a>

## Function `custom_object_id`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_custom_object_id">custom_object_id</a>&lt;ID: <b>copy</b>, drop, store, T: key&gt;(id: ID): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_object_custom_object_id_with_parent"></a>

## Function `custom_object_id_with_parent`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_custom_object_id_with_parent">custom_object_id_with_parent</a>&lt;ID: <b>copy</b>, drop, store, T: key&gt;(parent_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, id: ID): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_object_new"></a>

## Function `new`

Create a new Object, Add the Object to the global object storage and return the Object


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_new">new</a>&lt;T: key&gt;(value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_new_with_id"></a>

## Function `new_with_id`

Create a new object with custom ID, the ObjectID is generated by the <code>id</code> and type_name of <code>T</code>
The caller must ensure that the <code>id</code> is unique


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_new_with_id">new_with_id</a>&lt;ID: <b>copy</b>, drop, store, T: key&gt;(id: ID, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
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



<a name="0x2_object_new_with_object_id"></a>

## Function `new_with_object_id`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new_with_object_id">new_with_object_id</a>&lt;T: key&gt;(id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_new_with_parent"></a>

## Function `new_with_parent`

Create a new object under the parent object


<pre><code>#[private_generics(#[P], #[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_new_with_parent">new_with_parent</a>&lt;P: key, T: key&gt;(parent: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;P&gt;, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_new_with_parent_and_id"></a>

## Function `new_with_parent_and_id`

Create a new object under the parent object with custom ID, the ObjectID is generated by <code>custom_object_id_with_parent</code>


<pre><code>#[private_generics(#[P], #[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_new_with_parent_and_id">new_with_parent_and_id</a>&lt;P: key, ID: <b>copy</b>, drop, store, T: key&gt;(parent: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;P&gt;, id: ID, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_new_with_parent_and_key"></a>

## Function `new_with_parent_and_key`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new_with_parent_and_key">new_with_parent_and_key</a>&lt;P: key, T: key&gt;(parent: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;P&gt;, child_key: <b>address</b>, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
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

Check if the object with <code>object_id</code> exists in the global object storage


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_exists_object">exists_object</a>(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): bool
</code></pre>



<a name="0x2_object_exists_object_with_type"></a>

## Function `exists_object_with_type`

Check if the object exists in the global object storage and the type of the object is <code>T</code>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_exists_object_with_type">exists_object_with_type</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): bool
</code></pre>



<a name="0x2_object_borrow_object"></a>

## Function `borrow_object`

Borrow Object from object store by object_id
Any one can borrow an <code>&<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;</code> from the global object storage
Except the object is embedded in other struct


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_object">borrow_object</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_borrow_mut_object"></a>

## Function `borrow_mut_object`

Borrow mut Object by <code>owner</code> and <code>object_id</code>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_object">borrow_mut_object</a>&lt;T: key&gt;(owner: &<a href="">signer</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_borrow_mut_object_extend"></a>

## Function `borrow_mut_object_extend`

Borrow mut Object by <code>object_id</code>, Only the module of <code>T</code> can borrow the <code><a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;</code> with object_id.
Except the object is frozen or is embedded in other struct


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_object_extend">borrow_mut_object_extend</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_take_object"></a>

## Function `take_object`

Take out the Object by <code>owner</code> and <code>object_id</code>
The <code>T</code> must have <code>key + store</code> ability.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_take_object">take_object</a>&lt;T: store, key&gt;(owner: &<a href="">signer</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_take_object_extend"></a>

## Function `take_object_extend`

Take out the Object by <code>object_id</code>
This function is for developer to extend, Only the module of <code>T</code> can call this function.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_take_object_extend">take_object_extend</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_borrow_mut_object_shared"></a>

## Function `borrow_mut_object_shared`

Borrow mut Shared Object by object_id


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_object_shared">borrow_mut_object_shared</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
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
The module of <code>T</code> can call <code>take_object_extend</code> to take out the shared object, then remove the shared object.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_to_shared">to_shared</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<a name="0x2_object_is_shared"></a>

## Function `is_shared`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_shared">is_shared</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): bool
</code></pre>



<a name="0x2_object_to_frozen"></a>

## Function `to_frozen`

Make the Object frozen, No one can not get the &mut Object<T> from frozen object


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_to_frozen">to_frozen</a>&lt;T: key&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<a name="0x2_object_is_frozen"></a>

## Function `is_frozen`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_frozen">is_frozen</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): bool
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



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_id">id</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_object_owner"></a>

## Function `owner`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_owner">owner</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): <b>address</b>
</code></pre>



<a name="0x2_object_is_system_owned"></a>

## Function `is_system_owned`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_system_owned">is_system_owned</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): bool
</code></pre>



<a name="0x2_object_is_user_owned"></a>

## Function `is_user_owned`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_is_user_owned">is_user_owned</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): bool
</code></pre>



<a name="0x2_object_add_field"></a>

## Function `add_field`

Add a dynamic field to the object. Aborts if an field for this
key already exists. The field itself is not stored in the
object, and cannot be discovered from it.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_add_field">add_field</a>&lt;T: key, Name: <b>copy</b>, drop, store, Value: store&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, name: Name, val: Value)
</code></pre>



<a name="0x2_object_add_field_internal"></a>

## Function `add_field_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_add_field_internal">add_field_internal</a>&lt;Name: <b>copy</b>, drop, store, Value&gt;(obj_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, name: Name, value: Value)
</code></pre>



<a name="0x2_object_borrow_field"></a>

## Function `borrow_field`

Acquire an immutable reference to the value which <code>key</code> maps to.
Aborts if there is no field for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_field">borrow_field</a>&lt;T: key, Name: <b>copy</b>, drop, store, Value: store&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, name: Name): &Value
</code></pre>



<a name="0x2_object_borrow_field_internal"></a>

## Function `borrow_field_internal`

Borrow FieldValue and return the val of FieldValue


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_field_internal">borrow_field_internal</a>&lt;Name: <b>copy</b>, drop, store, Value&gt;(obj_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, name: Name): &Value
</code></pre>



<a name="0x2_object_borrow_field_with_default"></a>

## Function `borrow_field_with_default`

Acquire an immutable reference to the value which <code>key</code> maps to.
Returns specified default value if there is no field for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_field_with_default">borrow_field_with_default</a>&lt;T: key, Name: <b>copy</b>, drop, store, Value: store&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, name: Name, default: &Value): &Value
</code></pre>



<a name="0x2_object_borrow_mut_field"></a>

## Function `borrow_mut_field`

Acquire a mutable reference to the value which <code>key</code> maps to.
Aborts if there is no field for <code>key</code>.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_field">borrow_mut_field</a>&lt;T: key, Name: <b>copy</b>, drop, store, Value: store&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, name: Name): &<b>mut</b> Value
</code></pre>



<a name="0x2_object_borrow_mut_field_internal"></a>

## Function `borrow_mut_field_internal`

Acquire a mutable reference to the value which <code>key</code> maps to.
Aborts if there is no field for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_mut_field_internal">borrow_mut_field_internal</a>&lt;Name: <b>copy</b>, drop, store, Value&gt;(obj_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, name: Name): &<b>mut</b> Value
</code></pre>



<a name="0x2_object_borrow_mut_field_with_default"></a>

## Function `borrow_mut_field_with_default`

Acquire a mutable reference to the value which <code>key</code> maps to.
Insert the pair (<code>key</code>, <code>default</code>) first if there is no field for <code>key</code>.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_field_with_default">borrow_mut_field_with_default</a>&lt;T: key, Name: <b>copy</b>, drop, store, Value: drop, store&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, name: Name, default: Value): &<b>mut</b> Value
</code></pre>



<a name="0x2_object_upsert_field"></a>

## Function `upsert_field`

Insert the pair (<code>key</code>, <code>value</code>) if there is no field for <code>key</code>.
update the value of the field for <code>key</code> to <code>value</code> otherwise


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_upsert_field">upsert_field</a>&lt;T: key, Name: <b>copy</b>, drop, store, Value: drop, store&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, name: Name, value: Value)
</code></pre>



<a name="0x2_object_remove_field"></a>

## Function `remove_field`

Remove from <code><a href="object.md#0x2_object">object</a></code> and return the value which <code>key</code> maps to.
Aborts if there is no field for <code>key</code>.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_remove_field">remove_field</a>&lt;T: key, Name: <b>copy</b>, drop, store, Value: store&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, name: Name): Value
</code></pre>



<a name="0x2_object_remove_field_internal"></a>

## Function `remove_field_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_remove_field_internal">remove_field_internal</a>&lt;T: key, Name: <b>copy</b>, drop, store, Value&gt;(obj_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, name: Name): Value
</code></pre>



<a name="0x2_object_contains_field"></a>

## Function `contains_field`

Returns true if <code><a href="object.md#0x2_object">object</a></code> contains an field for <code>key</code>, include normal field and object field


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_contains_field">contains_field</a>&lt;T: key, Name: <b>copy</b>, drop, store&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, name: Name): bool
</code></pre>



<a name="0x2_object_contains_field_internal"></a>

## Function `contains_field_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_contains_field_internal">contains_field_internal</a>&lt;Name: <b>copy</b>, drop, store&gt;(obj_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, name: Name): bool
</code></pre>



<a name="0x2_object_contains_field_with_type"></a>

## Function `contains_field_with_type`

Returns true if <code><a href="object.md#0x2_object">object</a></code> contains an field for <code>key</code> and the value type is <code>Value</code>. only for normal field


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_contains_field_with_type">contains_field_with_type</a>&lt;T: key, Name: <b>copy</b>, drop, store, Value: store&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, name: Name): bool
</code></pre>



<a name="0x2_object_field_size"></a>

## Function `field_size`

Returns the size of the object fields, the number of key-value pairs


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_field_size">field_size</a>&lt;T: key&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): u64
</code></pre>
