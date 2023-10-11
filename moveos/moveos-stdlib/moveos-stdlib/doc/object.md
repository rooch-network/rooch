
<a name="0x2_object"></a>

# Module `0x2::object`

Move Object
The Object is a box style Object
The differents with the Object in [Sui](https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/object.move#L75):
1. The Object is a struct in Move
2. The Object is a use case of the Hot Potato pattern in Move. Objects do not have any ability, so they cannot be drop, copy, or store, and can only be handled by StorageContext API after creation.


-  [Struct `Object`](#0x2_object_Object)
-  [Struct `ObjectID`](#0x2_object_ObjectID)
-  [Function `address_to_object_id`](#0x2_object_address_to_object_id)
-  [Function `new`](#0x2_object_new)
-  [Function `borrow`](#0x2_object_borrow)
-  [Function `internal_borrow`](#0x2_object_internal_borrow)
-  [Function `borrow_mut`](#0x2_object_borrow_mut)
-  [Function `internal_borrow_mut`](#0x2_object_internal_borrow_mut)
-  [Function `transfer`](#0x2_object_transfer)
-  [Function `id`](#0x2_object_id)
-  [Function `owner`](#0x2_object_owner)
-  [Function `unpack`](#0x2_object_unpack)
-  [Function `unpack_internal`](#0x2_object_unpack_internal)


<pre><code><b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
</code></pre>



<a name="0x2_object_Object"></a>

## Struct `Object`

Box style object
The object can not be copied, droped and stored. It only can be consumed by StorageContext API.


<pre><code><b>struct</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;
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

</dd>
</dl>


</details>

<a name="0x2_object_ObjectID"></a>

## Struct `ObjectID`

An object ID


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

<a name="0x2_object_new"></a>

## Function `new`

Create a new object, the object is owned by <code>owner</code>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new">new</a>&lt;T: key&gt;(id: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, owner: <b>address</b>, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new">new</a>&lt;T: key&gt;(id: ObjectID, owner: <b>address</b>, value: T): <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt; {
    <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;{id, value, owner}
}
</code></pre>



</details>

<a name="0x2_object_borrow"></a>

## Function `borrow`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow">borrow</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow">borrow</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): &T {
    &self.value
}
</code></pre>



</details>

<a name="0x2_object_internal_borrow"></a>

## Function `internal_borrow`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_internal_borrow">internal_borrow</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_internal_borrow">internal_borrow</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): &T {
    &self.value
}
</code></pre>



</details>

<a name="0x2_object_borrow_mut"></a>

## Function `borrow_mut`

Borrow the mutable object value


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut">borrow_mut</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &<b>mut</b> T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut">borrow_mut</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): &<b>mut</b> T {
    &<b>mut</b> self.value
}
</code></pre>



</details>

<a name="0x2_object_internal_borrow_mut"></a>

## Function `internal_borrow_mut`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_internal_borrow_mut">internal_borrow_mut</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &<b>mut</b> T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_internal_borrow_mut">internal_borrow_mut</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): &<b>mut</b> T {
    &<b>mut</b> self.value
}
</code></pre>



</details>

<a name="0x2_object_transfer"></a>

## Function `transfer`

Transfer object to recipient


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_transfer">transfer</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, recipient: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_transfer">transfer</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;, recipient: <b>address</b>) {
    self.owner = recipient;
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



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_owner">owner</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_owner">owner</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): <b>address</b> {
    self.owner
}
</code></pre>



</details>

<a name="0x2_object_unpack"></a>

## Function `unpack`

Unpack the object, return the id, owner, and value


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_unpack">unpack</a>&lt;T&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): (<a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, <b>address</b>, T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_unpack">unpack</a>&lt;T&gt;(self: <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): (<a href="object.md#0x2_object_ObjectID">ObjectID</a>, <b>address</b>, T) {
    <a href="object.md#0x2_object_unpack_internal">unpack_internal</a>(self)
}
</code></pre>



</details>

<a name="0x2_object_unpack_internal"></a>

## Function `unpack_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_unpack_internal">unpack_internal</a>&lt;T&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): (<a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, <b>address</b>, T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_unpack_internal">unpack_internal</a>&lt;T&gt;(self: <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): (<a href="object.md#0x2_object_ObjectID">ObjectID</a>, <b>address</b>, T) {
    <b>let</b> <a href="object.md#0x2_object_Object">Object</a>{id, owner, value} = self;
    (id, owner, value)
}
</code></pre>



</details>
