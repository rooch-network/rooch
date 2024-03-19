
<a name="0x2_object"></a>

# Module `0x2::object`

Move Object
For more details, please refer to https://rooch.network/docs/developer-guides/object


-  [Struct `ObjectID`](#0x2_object_ObjectID)
-  [Resource `Root`](#0x2_object_Root)
-  [Struct `ObjectEntity`](#0x2_object_ObjectEntity)
-  [Resource `Object`](#0x2_object_Object)
-  [Resource `Box`](#0x2_object_Box)
-  [Struct `TestStructID`](#0x2_object_TestStructID)
-  [Constants](#@Constants_0)
-  [Function `has_parent`](#0x2_object_has_parent)
-  [Function `parent_id`](#0x2_object_parent_id)
-  [Function `is_parent`](#0x2_object_is_parent)
-  [Function `is_root`](#0x2_object_is_root)
-  [Function `address_to_object_id`](#0x2_object_address_to_object_id)
-  [Function `named_object_id`](#0x2_object_named_object_id)
-  [Function `account_named_object_id`](#0x2_object_account_named_object_id)
-  [Function `custom_object_id`](#0x2_object_custom_object_id)
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
-  [Function `add_field`](#0x2_object_add_field)
-  [Function `add_object_field`](#0x2_object_add_object_field)
-  [Function `add_field_internal`](#0x2_object_add_field_internal)
-  [Function `borrow_field`](#0x2_object_borrow_field)
-  [Function `borrow_object_field`](#0x2_object_borrow_object_field)
-  [Function `borrow_field_with_default`](#0x2_object_borrow_field_with_default)
-  [Function `borrow_mut_field`](#0x2_object_borrow_mut_field)
-  [Function `borrow_mut_object_field`](#0x2_object_borrow_mut_object_field)
-  [Function `borrow_mut_field_with_default`](#0x2_object_borrow_mut_field_with_default)
-  [Function `upsert_field`](#0x2_object_upsert_field)
-  [Function `remove_field`](#0x2_object_remove_field)
-  [Function `remove_object_field`](#0x2_object_remove_object_field)
-  [Function `contains_field`](#0x2_object_contains_field)
-  [Function `contains_object_field`](#0x2_object_contains_object_field)
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


<pre><code><b>struct</b> <a href="object.md#0x2_object_ObjectID">ObjectID</a> <b>has</b> <b>copy</b>, drop, store
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



<a name="0x2_object_Box"></a>

## Resource `Box`

Wrapper for values. Required for making values appear as resources in the implementation.
Because the GlobalValue in MoveVM must be a resource.


<pre><code><b>struct</b> <a href="object.md#0x2_object_Box">Box</a>&lt;V&gt; <b>has</b> drop, store, key
</code></pre>



<a name="0x2_object_TestStructID"></a>

## Struct `TestStructID`



<pre><code><b>struct</b> <a href="object.md#0x2_object_TestStructID">TestStructID</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_object_ErrorTypeMismatch"></a>

The type of the object or field is mismatch


<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorTypeMismatch">ErrorTypeMismatch</a>: u64 = 10;
</code></pre>



<a name="0x2_object_BOUND_OBJECT_FLAG_MASK"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_BOUND_OBJECT_FLAG_MASK">BOUND_OBJECT_FLAG_MASK</a>: u8 = 4;
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



<pre><code><b>const</b> <a href="object.md#0x2_object_ErrorObjectAlreadyBorrowed">ErrorObjectAlreadyBorrowed</a>: u64 = 7;
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



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_account_named_object_id">account_named_object_id</a>&lt;T&gt;(<a href="account.md#0x2_account">account</a>: <b>address</b>): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_object_custom_object_id"></a>

## Function `custom_object_id`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_custom_object_id">custom_object_id</a>&lt;ID: drop, T&gt;(id: ID): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
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



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new_with_id">new_with_id</a>&lt;T: key&gt;(id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
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


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_object">borrow_object</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_borrow_mut_object"></a>

## Function `borrow_mut_object`

Borrow mut Object by <code>owner</code> and <code>object_id</code>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_object">borrow_mut_object</a>&lt;T: key&gt;(owner: &<a href="">signer</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_borrow_mut_object_extend"></a>

## Function `borrow_mut_object_extend`

Borrow mut Object by <code>object_id</code>


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_object_extend">borrow_mut_object_extend</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_take_object"></a>

## Function `take_object`

Take out the UserOwnedObject by <code>owner</code> and <code>object_id</code>
The <code>T</code> must have <code>key + store</code> ability.
Note: When the Object is taken out, the Object will auto become <code>SystemOwned</code> Object.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_take_object">take_object</a>&lt;T: store, key&gt;(owner: &<a href="">signer</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_take_object_extend"></a>

## Function `take_object_extend`

Take out the UserOwnedObject by <code>object_id</code>, return the owner and Object
This function is for developer to extend, Only the module of <code>T</code> can take out the <code>UserOwnedObject</code> with object_id.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_take_object_extend">take_object_extend</a>&lt;T: key&gt;(object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): (<b>address</b>, <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
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



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_id">id</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
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



<a name="0x2_object_add_field"></a>

## Function `add_field`

Add a dynamic filed to the object. Aborts if an field for this
key already exists. The field itself is not stored in the
object, and cannot be discovered from it.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_add_field">add_field</a>&lt;T: key, K: <b>copy</b>, drop, V: store&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K, val: V)
</code></pre>



<a name="0x2_object_add_object_field"></a>

## Function `add_object_field`

Add a object field to the object. return the child object
The parent object must be a shared object


<pre><code>#[private_generics(#[T], #[V])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_add_object_field">add_object_field</a>&lt;T: key, V: key&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, v: V): <a href="object.md#0x2_object_Object">object::Object</a>&lt;V&gt;
</code></pre>



<a name="0x2_object_add_field_internal"></a>

## Function `add_field_internal`

Add a new field to the object. Aborts if an field for this
key already exists. The field itself is not stored in the
object, and cannot be discovered from it.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_add_field_internal">add_field_internal</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(obj_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, key: K, val: V)
</code></pre>



<a name="0x2_object_borrow_field"></a>

## Function `borrow_field`

Acquire an immutable reference to the value which <code>key</code> maps to.
Aborts if there is no field for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_field">borrow_field</a>&lt;T: key, K: <b>copy</b>, drop, V: store&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): &V
</code></pre>



<a name="0x2_object_borrow_object_field"></a>

## Function `borrow_object_field`

Borrow the child object by <code>key</code>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_object_field">borrow_object_field</a>&lt;T: key, V: key&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;V&gt;
</code></pre>



<a name="0x2_object_borrow_field_with_default"></a>

## Function `borrow_field_with_default`

Acquire an immutable reference to the value which <code>key</code> maps to.
Returns specified default value if there is no field for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_field_with_default">borrow_field_with_default</a>&lt;T: key, K: <b>copy</b>, drop, V: store&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K, default: &V): &V
</code></pre>



<a name="0x2_object_borrow_mut_field"></a>

## Function `borrow_mut_field`

Acquire a mutable reference to the value which <code>key</code> maps to.
Aborts if there is no field for <code>key</code>.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_field">borrow_mut_field</a>&lt;T: key, K: <b>copy</b>, drop, V: store&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): &<b>mut</b> V
</code></pre>



<a name="0x2_object_borrow_mut_object_field"></a>

## Function `borrow_mut_object_field`

Borrow the child object by <code>key</code>
Because the parent object must be a shared object, so we do not require the #[private_generics(T)] here


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_object_field">borrow_mut_object_field</a>&lt;T: key, V: key&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;V&gt;
</code></pre>



<a name="0x2_object_borrow_mut_field_with_default"></a>

## Function `borrow_mut_field_with_default`

Acquire a mutable reference to the value which <code>key</code> maps to.
Insert the pair (<code>key</code>, <code>default</code>) first if there is no field for <code>key</code>.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut_field_with_default">borrow_mut_field_with_default</a>&lt;T: key, K: <b>copy</b>, drop, V: drop, store&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K, default: V): &<b>mut</b> V
</code></pre>



<a name="0x2_object_upsert_field"></a>

## Function `upsert_field`

Insert the pair (<code>key</code>, <code>value</code>) if there is no field for <code>key</code>.
update the value of the field for <code>key</code> to <code>value</code> otherwise


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_upsert_field">upsert_field</a>&lt;T: key, K: <b>copy</b>, drop, V: drop, store&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K, value: V)
</code></pre>



<a name="0x2_object_remove_field"></a>

## Function `remove_field`

Remove from <code><a href="object.md#0x2_object">object</a></code> and return the value which <code>key</code> maps to.
Aborts if there is no field for <code>key</code>.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_remove_field">remove_field</a>&lt;T: key, K: <b>copy</b>, drop, V: store&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): V
</code></pre>



<a name="0x2_object_remove_object_field"></a>

## Function `remove_object_field`



<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object.md#0x2_object_remove_object_field">remove_object_field</a>&lt;T: key, V: key&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, child: <a href="object.md#0x2_object_Object">object::Object</a>&lt;V&gt;): V
</code></pre>



<a name="0x2_object_contains_field"></a>

## Function `contains_field`

Returns true if <code><a href="object.md#0x2_object">object</a></code> contains an field for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_contains_field">contains_field</a>&lt;T: key, K: <b>copy</b>, drop&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): bool
</code></pre>



<a name="0x2_object_contains_object_field"></a>

## Function `contains_object_field`

Returns true if <code><a href="object.md#0x2_object">object</a></code> contains an Object field for <code>key</code> and the value type is <code>V</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_contains_object_field">contains_object_field</a>&lt;T: key, V: key&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): bool
</code></pre>



<a name="0x2_object_contains_field_with_type"></a>

## Function `contains_field_with_type`

Returns true if <code><a href="object.md#0x2_object">object</a></code> contains an field for <code>key</code> and the value type is <code>V</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_contains_field_with_type">contains_field_with_type</a>&lt;T: key, K: <b>copy</b>, drop, V: store&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): bool
</code></pre>



<a name="0x2_object_field_size"></a>

## Function `field_size`

Returns the size of the object fields, the number of key-value pairs


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_field_size">field_size</a>&lt;T: key&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): u64
</code></pre>
