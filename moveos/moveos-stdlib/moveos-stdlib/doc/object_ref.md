
<a name="0x2_object_ref"></a>

# Module `0x2::object_ref`



-  [Resource `ObjectRef`](#0x2_object_ref_ObjectRef)
-  [Function `new_internal`](#0x2_object_ref_new_internal)
-  [Function `to_user_owner`](#0x2_object_ref_to_user_owner)
-  [Function `borrow`](#0x2_object_ref_borrow)
-  [Function `borrow_extend`](#0x2_object_ref_borrow_extend)
-  [Function `borrow_mut`](#0x2_object_ref_borrow_mut)
-  [Function `borrow_mut_extend`](#0x2_object_ref_borrow_mut_extend)
-  [Function `remove`](#0x2_object_ref_remove)
-  [Function `remove_extend`](#0x2_object_ref_remove_extend)
-  [Function `id`](#0x2_object_ref_id)
-  [Function `owner`](#0x2_object_ref_owner)


<pre><code><b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="raw_table.md#0x2_raw_table">0x2::raw_table</a>;
</code></pre>



<a name="0x2_object_ref_ObjectRef"></a>

## Resource `ObjectRef`

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

<a name="0x2_object_ref_to_user_owner"></a>

## Function `to_user_owner`



<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_to_user_owner">to_user_owner</a>&lt;T: key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;, owner: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_to_user_owner">to_user_owner</a>&lt;T: key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;, owner: <b>address</b>){
    <b>let</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>{id} = self;
    <b>let</b> obj = <a href="raw_table.md#0x2_raw_table_borrow_mut_from_global">raw_table::borrow_mut_from_global</a>&lt;T&gt;(&id);
    <a href="object.md#0x2_object_set_owner">object::set_owner</a>(obj, owner);
}
</code></pre>



</details>

<a name="0x2_object_ref_borrow"></a>

## Function `borrow`

Borrow the object value


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_borrow">borrow</a>&lt;T: store, key&gt;(self: &<a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;): &T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_borrow">borrow</a>&lt;T: key + store&gt;(self: &<a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;): &T {
    <b>let</b> obj = <a href="raw_table.md#0x2_raw_table_borrow_from_global">raw_table::borrow_from_global</a>&lt;T&gt;(&self.id);
    <a href="object.md#0x2_object_internal_borrow">object::internal_borrow</a>(obj)
}
</code></pre>



</details>

<a name="0x2_object_ref_borrow_extend"></a>

## Function `borrow_extend`



<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_borrow_extend">borrow_extend</a>&lt;T: key&gt;(self: &<a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;): &T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_borrow_extend">borrow_extend</a>&lt;T: key&gt;(self: &<a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;): &T {
    <b>let</b> obj = <a href="raw_table.md#0x2_raw_table_borrow_from_global">raw_table::borrow_from_global</a>&lt;T&gt;(&self.id);
    <a href="object.md#0x2_object_internal_borrow">object::internal_borrow</a>(obj)
}
</code></pre>



</details>

<a name="0x2_object_ref_borrow_mut"></a>

## Function `borrow_mut`

Borrow the object mutable value


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_borrow_mut">borrow_mut</a>&lt;T: store, key&gt;(self: &<b>mut</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;): &<b>mut</b> T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_borrow_mut">borrow_mut</a>&lt;T: key + store&gt;(self: &<b>mut</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;): &<b>mut</b> T {
    <b>let</b> obj = <a href="raw_table.md#0x2_raw_table_borrow_mut_from_global">raw_table::borrow_mut_from_global</a>&lt;T&gt;(&self.id);
    <a href="object.md#0x2_object_internal_borrow_mut">object::internal_borrow_mut</a>(obj)
}
</code></pre>



</details>

<a name="0x2_object_ref_borrow_mut_extend"></a>

## Function `borrow_mut_extend`



<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_borrow_mut_extend">borrow_mut_extend</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;): &<b>mut</b> T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_borrow_mut_extend">borrow_mut_extend</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;): &<b>mut</b> T {
    <b>let</b> obj = <a href="raw_table.md#0x2_raw_table_borrow_mut_from_global">raw_table::borrow_mut_from_global</a>&lt;T&gt;(&self.id);
    <a href="object.md#0x2_object_internal_borrow_mut">object::internal_borrow_mut</a>(obj)
}
</code></pre>



</details>

<a name="0x2_object_ref_remove"></a>

## Function `remove`

Remove the object from the global storage, and return the object value


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_remove">remove</a>&lt;T: store, key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_remove">remove</a>&lt;T: key + store&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;) : T {
    <b>let</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>{id} = self;
    <b>let</b> <a href="object.md#0x2_object">object</a> = <a href="raw_table.md#0x2_raw_table_remove_from_global">raw_table::remove_from_global</a>(&id);
    <b>let</b> (_id, _owner, value) = <a href="object.md#0x2_object_unpack_internal">object::unpack_internal</a>(<a href="object.md#0x2_object">object</a>);
    value
}
</code></pre>



</details>

<a name="0x2_object_ref_remove_extend"></a>

## Function `remove_extend`



<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_remove_extend">remove_extend</a>&lt;T: key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_ref.md#0x2_object_ref_remove_extend">remove_extend</a>&lt;T: key&gt;(self: <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>&lt;T&gt;) : T {
    <b>let</b> <a href="object_ref.md#0x2_object_ref_ObjectRef">ObjectRef</a>{id} = self;
    <b>let</b> <a href="object.md#0x2_object">object</a> = <a href="raw_table.md#0x2_raw_table_remove_from_global">raw_table::remove_from_global</a>(&id);
    <b>let</b> (_id, _owner, value) = <a href="object.md#0x2_object_unpack_internal">object::unpack_internal</a>(<a href="object.md#0x2_object">object</a>);
    value
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
