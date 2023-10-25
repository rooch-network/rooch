
<a name="0x2_object"></a>

# Module `0x2::object`

Move Object
The Object is a box style Object
The differents with the Object in [Sui](https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/object.move#L75):
1. The Object is a struct in Move
2. The Object is a use case of the Hot Potato pattern in Move. Objects do not have any ability, so they cannot be drop, copy, or store, and can only be handled by StorageContext API after creation.


-  [Struct `Object`](#0x2_object_Object)
-  [Struct `ObjectID`](#0x2_object_ObjectID)
-  [Constants](#@Constants_0)
-  [Function `address_to_object_id`](#0x2_object_address_to_object_id)
-  [Function `singleton_object_id`](#0x2_object_singleton_object_id)
-  [Function `code_owner_address`](#0x2_object_code_owner_address)
-  [Function `new`](#0x2_object_new)
-  [Function `borrow`](#0x2_object_borrow)
-  [Function `borrow_mut`](#0x2_object_borrow_mut)
-  [Function `set_owner`](#0x2_object_set_owner)
-  [Function `set_external`](#0x2_object_set_external)
-  [Function `id`](#0x2_object_id)
-  [Function `owner`](#0x2_object_owner)
-  [Function `unpack`](#0x2_object_unpack)


<pre><code><b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="address.md#0x2_address">0x2::address</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="type_info.md#0x2_type_info">0x2::type_info</a>;
</code></pre>



<a name="0x2_object_Object"></a>

## Struct `Object`

TODO rename to ObjectEntity
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
 Whether the object is external
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

<a name="@Constants_0"></a>

## Constants


<a name="0x2_object_CODE_OWNER_ADDRESS"></a>



<pre><code><b>const</b> <a href="object.md#0x2_object_CODE_OWNER_ADDRESS">CODE_OWNER_ADDRESS</a>: <b>address</b> = 0;
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

<a name="0x2_object_code_owner_address"></a>

## Function `code_owner_address`



<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_code_owner_address">code_owner_address</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object.md#0x2_object_code_owner_address">code_owner_address</a>(): <b>address</b> {
    <a href="object.md#0x2_object_CODE_OWNER_ADDRESS">CODE_OWNER_ADDRESS</a>
}
</code></pre>



</details>

<a name="0x2_object_new"></a>

## Function `new`

Create a new object, the object is owned by <code>owner</code>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new">new</a>&lt;T: key&gt;(id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_new">new</a>&lt;T: key&gt;(id: <a href="object.md#0x2_object_ObjectID">ObjectID</a>, value: T): <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt; {
    <b>let</b> owner = <a href="object.md#0x2_object_CODE_OWNER_ADDRESS">CODE_OWNER_ADDRESS</a>;
    <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;{id, value, owner}
}
</code></pre>



</details>

<a name="0x2_object_borrow"></a>

## Function `borrow`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow">borrow</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow">borrow</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): &T {
    &self.value
}
</code></pre>



</details>

<a name="0x2_object_borrow_mut"></a>

## Function `borrow_mut`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_mut">borrow_mut</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): &<b>mut</b> T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_borrow_mut">borrow_mut</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): &<b>mut</b> T {
    &<b>mut</b> self.value
}
</code></pre>



</details>

<a name="0x2_object_set_owner"></a>

## Function `set_owner`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_set_owner">set_owner</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, owner: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_set_owner">set_owner</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;, owner: <b>address</b>) {
    self.owner = owner;
}
</code></pre>



</details>

<a name="0x2_object_set_external"></a>

## Function `set_external`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_set_external">set_external</a>&lt;T&gt;(_self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, _external: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_set_external">set_external</a>&lt;T&gt;(_self: &<b>mut</b> <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;, _external: bool) {
    //self.external = external;
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


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_unpack">unpack</a>&lt;T&gt;(self: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): (<a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, <b>address</b>, T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object.md#0x2_object_unpack">unpack</a>&lt;T&gt;(self: <a href="object.md#0x2_object_Object">Object</a>&lt;T&gt;): (<a href="object.md#0x2_object_ObjectID">ObjectID</a>, <b>address</b>, T) {
    <b>let</b> <a href="object.md#0x2_object_Object">Object</a>{id, owner, value} = self;
    (id, owner, value)
}
</code></pre>



</details>
