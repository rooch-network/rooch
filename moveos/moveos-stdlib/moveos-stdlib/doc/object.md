
<a name="0x2_object"></a>

# Module `0x2::object`

Move Object
The Object is a box style Object
The differents with the Object in [Sui](https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/object.move#L75):
1. The Object is a struct in Move
2. The Object is a use case for the Hot Potato pattern in Move. Objects do not have any ability, so they cannot be drop, copy, or store, and can only be handled by ObjectStorage API after creation.
More details about the Object can be found in [Storage Abstraction](https://github.com/rooch-network/rooch/blob/main/docs/design/storage_abstraction.md)


-  [Struct `Object`](#0x2_object_Object)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x2_object_new)
-  [Function `new_with_id`](#0x2_object_new_with_id)
-  [Function `borrow`](#0x2_object_borrow)
-  [Function `borrow_mut`](#0x2_object_borrow_mut)
-  [Function `transfer`](#0x2_object_transfer)
-  [Function `id`](#0x2_object_id)
-  [Function `owner`](#0x2_object_owner)
-  [Function `unpack`](#0x2_object_unpack)
-  [Function `unpack_internal`](#0x2_object_unpack_internal)


<pre><code><b>use</b> <a href="">0x1::debug</a>;
<b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_object_Object"></a>

## Struct `Object`

Box style object
The object can not be copied, droped, only can be consumed by ObjectStorage API.


<pre><code><b>struct</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a></code>
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

<a name="@Constants_0"></a>

## Constants


<a name="0x2_object_EInvalidAccess"></a>

Invalid access of object, the object is not owned by the signer or the object is not shared or immutable


<pre><code><b>const</b> <a href="object.md#0x2_object_EInvalidAccess">EInvalidAccess</a>: u64 = 0;
</code></pre>



<a name="0x2_object_new"></a>

## Function `new`

Create a new object, the object is owned by <code>owner</code>
The private generic is indicate the T should be defined in the same module as the caller. This is ensured by the verifier.


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_new">new</a>&lt;T: key&gt;(ctx: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>, owner: <b>address</b>, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_new">new</a>&lt;T: key&gt;(ctx: &<b>mut</b> TxContext, owner: <b>address</b>, value: T): <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt; {
    <b>let</b> id = <a href="tx_context.md#0x2_tx_context_fresh_object_id">tx_context::fresh_object_id</a>(ctx);
    <b>let</b> obj = <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;{id, value, owner};
    //TODO after add event, then remove the <a href="">debug</a> info
    <a href="_print">debug::print</a>(&obj);
    obj
}
</code></pre>



</details>

<a name="0x2_object_new_with_id"></a>

## Function `new_with_id`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new_with_id">new_with_id</a>&lt;T: key&gt;(id: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, owner: <b>address</b>, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new_with_id">new_with_id</a>&lt;T: key&gt;(id: ObjectID, owner: <b>address</b>, value: T): <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt; {
    <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;{id, owner, value}
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

<a name="0x2_object_borrow_mut"></a>

## Function `borrow_mut`

Borrow the object mutable value


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut">borrow_mut</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &<b>mut</b> T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_borrow_mut">borrow_mut</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): &<b>mut</b> T {
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



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_id">id</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_id">id</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): ObjectID {
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



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_unpack">unpack</a>&lt;T&gt;(obj: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): (<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, <b>address</b>, T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_unpack">unpack</a>&lt;T&gt;(obj: <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): (ObjectID, <b>address</b>, T) {
    <a href="object.md#0x2_object_unpack_internal">unpack_internal</a>(obj)
}
</code></pre>



</details>

<a name="0x2_object_unpack_internal"></a>

## Function `unpack_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_unpack_internal">unpack_internal</a>&lt;T&gt;(obj: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): (<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, <b>address</b>, T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_unpack_internal">unpack_internal</a>&lt;T&gt;(obj: <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): (ObjectID, <b>address</b>, T) {
    <b>let</b> <a href="object.md#0x2_object_Object">Object</a>{id, owner, value} = obj;
    (id, owner, value)
}
</code></pre>



</details>
