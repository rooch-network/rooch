
<a name="0x2_object_ref"></a>

# Module `0x2::object_ref`



-  [Resource `ObjectRef`](#0x2_object_ref_ObjectRef)
-  [Constants](#@Constants_0)
-  [Function `new_internal`](#0x2_object_ref_new_internal)
-  [Function `as_ref`](#0x2_object_ref_as_ref)
-  [Function `as_mut_ref`](#0x2_object_ref_as_mut_ref)
-  [Function `borrow`](#0x2_object_ref_borrow)
-  [Function `borrow_mut`](#0x2_object_ref_borrow_mut)
-  [Function `remove`](#0x2_object_ref_remove)
-  [Function `to_permanent`](#0x2_object_ref_to_permanent)
-  [Function `to_shared`](#0x2_object_ref_to_shared)
-  [Function `to_frozen`](#0x2_object_ref_to_frozen)
-  [Function `transfer`](#0x2_object_ref_transfer)
-  [Function `transfer_extend`](#0x2_object_ref_transfer_extend)
-  [Function `id`](#0x2_object_ref_id)
-  [Function `owner`](#0x2_object_ref_owner)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="raw_table.md#0x2_raw_table">0x2::raw_table</a>;
</code></pre>



<a name="0x2_object_ref_ObjectRef"></a>

## Resource `ObjectRef`

TODO rename to Object
ObjectRef<T> is a reference of the Object<T>
It likes ObjectID, but it contains the type information of the object.


<pre><code><b>struct</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt; <b>has</b> store, key
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

<a name="@Constants_0"></a>

## Constants


<a name="0x2_object_ref_ErrorObjectFrozen"></a>



<pre><code><b>const</b> <a href="object_ref.md#0x2_object_ref_ErrorObjectFrozen">ErrorObjectFrozen</a>: u64 = 1;
</code></pre>



<a name="0x2_object_ref_new_internal"></a>

## Function `new_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_ref.md#0x2_object_ref_new_internal">new_internal</a>&lt;T: key&gt;(<a href="object.md#0x2_object">object</a>: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_ref.md#0x2_object_ref_new_internal">new_internal</a>&lt;T: key&gt;(<a href="object.md#0x2_object">object</a>: &<b>mut</b> Object&lt;T&gt;) : <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt; {
    <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a> {
        id: <a href="object.md#0x2_object_id">object::id</a>(<a href="object.md#0x2_object">object</a>),
    }
}
</code></pre>



</details>

<a name="0x2_object_ref_as_ref"></a>

## Function `as_ref`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_ref.md#0x2_object_ref_as_ref">as_ref</a>&lt;T: key&gt;(<a href="object.md#0x2_object">object</a>: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &<a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_ref.md#0x2_object_ref_as_ref">as_ref</a>&lt;T: key&gt;(<a href="object.md#0x2_object">object</a>: &Object&lt;T&gt;) : &<a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;{
    <a href="object_ref.md#0x2_object_ref_as_ref_inner">as_ref_inner</a>&lt;<a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;&gt;(<a href="object.md#0x2_object_id">object::id</a>(<a href="object.md#0x2_object">object</a>))
}
</code></pre>



</details>

<a name="0x2_object_ref_as_mut_ref"></a>

## Function `as_mut_ref`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_ref.md#0x2_object_ref_as_mut_ref">as_mut_ref</a>&lt;T: key&gt;(<a href="object.md#0x2_object">object</a>: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &<b>mut</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_ref.md#0x2_object_ref_as_mut_ref">as_mut_ref</a>&lt;T: key&gt;(<a href="object.md#0x2_object">object</a>: &<b>mut</b> Object&lt;T&gt;) : &<b>mut</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;{
    <b>assert</b>!(!<a href="object.md#0x2_object_is_frozen">object::is_frozen</a>(<a href="object.md#0x2_object">object</a>), <a href="_permission_denied">error::permission_denied</a>(<a href="object_ref.md#0x2_object_ref_ErrorObjectFrozen">ErrorObjectFrozen</a>));
    <a href="object_ref.md#0x2_object_ref_as_mut_ref_inner">as_mut_ref_inner</a>&lt;<a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;&gt;(<a href="object.md#0x2_object_id">object::id</a>(<a href="object.md#0x2_object">object</a>))
}
</code></pre>



</details>

<a name="0x2_object_ref_borrow"></a>

## Function `borrow`

Borrow the object value


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_borrow">borrow</a>&lt;T: key&gt;(self: &<a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;): &T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_borrow">borrow</a>&lt;T: key&gt;(self: &<a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;): &T {
    <b>let</b> obj = <a href="raw_table.md#0x2_raw_table_borrow_from_global">raw_table::borrow_from_global</a>&lt;T&gt;(&self.id);
    <a href="object.md#0x2_object_borrow">object::borrow</a>(obj)
}
</code></pre>



</details>

<a name="0x2_object_ref_borrow_mut"></a>

## Function `borrow_mut`

Borrow the object mutable value


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_borrow_mut">borrow_mut</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;): &<b>mut</b> T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_borrow_mut">borrow_mut</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;): &<b>mut</b> T {
    <b>let</b> obj = <a href="raw_table.md#0x2_raw_table_borrow_mut_from_global">raw_table::borrow_mut_from_global</a>&lt;T&gt;(&self.id);
    <a href="object.md#0x2_object_borrow_mut">object::borrow_mut</a>(obj)
}
</code></pre>



</details>

<a name="0x2_object_ref_remove"></a>

## Function `remove`

Remove the object from the global storage, and return the object value
This function is only can be called by the module of <code>T</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_remove">remove</a>&lt;T: key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_remove">remove</a>&lt;T: key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;) : T {
    <b>let</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>{id} = self;
    <b>let</b> <a href="object.md#0x2_object">object</a> = <a href="raw_table.md#0x2_raw_table_remove_from_global">raw_table::remove_from_global</a>(&id);
    <b>let</b> (_id, _owner, value) = <a href="object.md#0x2_object_unpack">object::unpack</a>(<a href="object.md#0x2_object">object</a>);
    value
}
</code></pre>



</details>

<a name="0x2_object_ref_to_permanent"></a>

## Function `to_permanent`

Directly drop the ObjectRef, and make the Object permanent, the object will can not be removed from the object storage.
If you want to remove the object, please use <code>remove</code> function.


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_to_permanent">to_permanent</a>&lt;T: key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_to_permanent">to_permanent</a>&lt;T: key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;) {
    <b>let</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>{id:_} = self;
}
</code></pre>



</details>

<a name="0x2_object_ref_to_shared"></a>

## Function `to_shared`

Make the Object shared, Any one can get the &mut ObjectRef<T> from shared object
The shared object also can be removed from the object storage.


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_to_shared">to_shared</a>&lt;T: key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_to_shared">to_shared</a>&lt;T: key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;) {
    <b>let</b> obj = <a href="raw_table.md#0x2_raw_table_borrow_mut_from_global">raw_table::borrow_mut_from_global</a>&lt;T&gt;(&self.id);
    <a href="object.md#0x2_object_to_shared">object::to_shared</a>(obj);
    <a href="object.md#0x2_object_transfer_to_system">object::transfer_to_system</a>(obj);
    <a href="object_ref.md#0x2_object_ref_to_permanent">to_permanent</a>(self);
}
</code></pre>



</details>

<a name="0x2_object_ref_to_frozen"></a>

## Function `to_frozen`

Make the Object frozen, Any one can not get the &mut ObjectRef<T> from frozen object


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_to_frozen">to_frozen</a>&lt;T: key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_to_frozen">to_frozen</a>&lt;T: key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;) {
    <b>let</b> obj = <a href="raw_table.md#0x2_raw_table_borrow_mut_from_global">raw_table::borrow_mut_from_global</a>&lt;T&gt;(&self.id);
    <a href="object.md#0x2_object_to_frozen">object::to_frozen</a>(obj);
    <a href="object.md#0x2_object_transfer_to_system">object::transfer_to_system</a>(obj);
    <a href="object_ref.md#0x2_object_ref_to_permanent">to_permanent</a>(self);
}
</code></pre>



</details>

<a name="0x2_object_ref_transfer"></a>

## Function `transfer`

Transfer the object to the new owner
Only the <code>T</code> with <code>store</code> can be directly transferred.


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_transfer">transfer</a>&lt;T: store, key&gt;(self: &<b>mut</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;, new_owner: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_transfer">transfer</a>&lt;T: key + store&gt;(self: &<b>mut</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;, new_owner: <b>address</b>) {
    <b>let</b> obj = <a href="raw_table.md#0x2_raw_table_borrow_mut_from_global">raw_table::borrow_mut_from_global</a>&lt;T&gt;(&self.id);
    <a href="object.md#0x2_object_transfer">object::transfer</a>(obj, new_owner);
}
</code></pre>



</details>

<a name="0x2_object_ref_transfer_extend"></a>

## Function `transfer_extend`

Transfer the object to the new owner
This function is for the module of <code>T</code> to extend the <code>transfer</code> function.


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_transfer_extend">transfer_extend</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;, new_owner: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_transfer_extend">transfer_extend</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;, new_owner: <b>address</b>) {
    <b>let</b> obj = <a href="raw_table.md#0x2_raw_table_borrow_mut_from_global">raw_table::borrow_mut_from_global</a>&lt;T&gt;(&self.id);
    <a href="object.md#0x2_object_transfer">object::transfer</a>(obj, new_owner);
}
</code></pre>



</details>

<a name="0x2_object_ref_id"></a>

## Function `id`



<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_id">id</a>&lt;T&gt;(self: &<a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_id">id</a>&lt;T&gt;(self: &<a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;): ObjectID {
    self.id
}
</code></pre>



</details>

<a name="0x2_object_ref_owner"></a>

## Function `owner`



<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_owner">owner</a>&lt;T: key&gt;(self: &<a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_owner">owner</a>&lt;T: key&gt;(self: &<a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;): <b>address</b> {
    <b>let</b> obj = <a href="raw_table.md#0x2_raw_table_borrow_from_global">raw_table::borrow_from_global</a>&lt;T&gt;(&self.id);
    <a href="object.md#0x2_object_owner">object::owner</a>(obj)
}
</code></pre>



</details>
