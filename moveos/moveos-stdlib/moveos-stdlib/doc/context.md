
<a name="0x2_context"></a>

# Module `0x2::context`

Context is part of the StorageAbstraction
It is used to provide a context for the storage operations, make the storage abstraction,
and let developers customize the storage


-  [Struct `Context`](#0x2_context_Context)
-  [Function `tx_context`](#0x2_context_tx_context)
-  [Function `tx_context_mut`](#0x2_context_tx_context_mut)
-  [Function `sender`](#0x2_context_sender)
-  [Function `sequence_number`](#0x2_context_sequence_number)
-  [Function `max_gas_amount`](#0x2_context_max_gas_amount)
-  [Function `fresh_address`](#0x2_context_fresh_address)
-  [Function `fresh_object_id`](#0x2_context_fresh_object_id)
-  [Function `tx_hash`](#0x2_context_tx_hash)
-  [Function `add`](#0x2_context_add)
-  [Function `get`](#0x2_context_get)
-  [Function `tx_meta`](#0x2_context_tx_meta)
-  [Function `tx_gas_payment_account`](#0x2_context_tx_gas_payment_account)
-  [Function `tx_result`](#0x2_context_tx_result)
-  [Function `borrow_object`](#0x2_context_borrow_object)
-  [Function `borrow_object_mut`](#0x2_context_borrow_object_mut)
-  [Function `remove_object`](#0x2_context_remove_object)
-  [Function `exist_object`](#0x2_context_exist_object)
-  [Function `new_object`](#0x2_context_new_object)
-  [Function `new_object_with_owner`](#0x2_context_new_object_with_owner)
-  [Function `new_object_with_id`](#0x2_context_new_object_with_id)
-  [Function `new_singleton_object`](#0x2_context_new_singleton_object)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="object_ref.md#0x2_object_ref">0x2::object_ref</a>;
<b>use</b> <a href="storage_context.md#0x2_storage_context">0x2::storage_context</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
<b>use</b> <a href="tx_meta.md#0x2_tx_meta">0x2::tx_meta</a>;
<b>use</b> <a href="tx_result.md#0x2_tx_result">0x2::tx_result</a>;
<b>use</b> <a href="type_info.md#0x2_type_info">0x2::type_info</a>;
</code></pre>



<a name="0x2_context_Context"></a>

## Struct `Context`

Information about the global context include TxContext and StorageContext
We can not put the StorageContext to TxContext, because object module depends on tx_context module,
and storage_context module depends on object module.
We put both TxContext and StorageContext to Context, for convenience of developers.
The Context can not be <code>drop</code> or <code>store</code>, so developers need to pass the <code>&<a href="context.md#0x2_context_Context">Context</a></code> or <code>&<b>mut</b> <a href="context.md#0x2_context_Context">Context</a></code> to the <code>entry</code> function.


<pre><code><b>struct</b> <a href="context.md#0x2_context_Context">Context</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="tx_context.md#0x2_tx_context">tx_context</a>: <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a></code>
</dt>
<dd>

</dd>
<dt>
<code><a href="storage_context.md#0x2_storage_context">storage_context</a>: <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a></code>
</dt>
<dd>
 The Global Object Storage
</dd>
</dl>


</details>

<a name="0x2_context_tx_context"></a>

## Function `tx_context`

Get an immutable reference to the transaction context from the storage context


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context">tx_context</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context">tx_context</a>(self: &<a href="context.md#0x2_context_Context">Context</a>): &TxContext {
    &self.<a href="tx_context.md#0x2_tx_context">tx_context</a>
}
</code></pre>



</details>

<a name="0x2_context_tx_context_mut"></a>

## Function `tx_context_mut`

Get a mutable reference to the transaction context from the storage context


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="context.md#0x2_context_tx_context_mut">tx_context_mut</a>(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>): &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="context.md#0x2_context_tx_context_mut">tx_context_mut</a>(self: &<b>mut</b> <a href="context.md#0x2_context_Context">Context</a>): &<b>mut</b> TxContext {
    &<b>mut</b> self.<a href="tx_context.md#0x2_tx_context">tx_context</a>
}
</code></pre>



</details>

<a name="0x2_context_sender"></a>

## Function `sender`

Return the address of the user that signed the current transaction


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_sender">sender</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_sender">sender</a>(self: &<a href="context.md#0x2_context_Context">Context</a>): <b>address</b> {
    <a href="tx_context.md#0x2_tx_context_sender">tx_context::sender</a>(&self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_context_sequence_number"></a>

## Function `sequence_number`

Return the sequence number of the current transaction


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_sequence_number">sequence_number</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_sequence_number">sequence_number</a>(self: &<a href="context.md#0x2_context_Context">Context</a>): u64 {
    <a href="tx_context.md#0x2_tx_context_sequence_number">tx_context::sequence_number</a>(&self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_context_max_gas_amount"></a>

## Function `max_gas_amount`

Return the maximum gas amount that can be used by the current transaction


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_max_gas_amount">max_gas_amount</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_max_gas_amount">max_gas_amount</a>(self: &<a href="context.md#0x2_context_Context">Context</a>): u64 {
    <a href="tx_context.md#0x2_tx_context_max_gas_amount">tx_context::max_gas_amount</a>(&self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_context_fresh_address"></a>

## Function `fresh_address`

Generate a new unique address


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_fresh_address">fresh_address</a>(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_fresh_address">fresh_address</a>(self: &<b>mut</b> <a href="context.md#0x2_context_Context">Context</a>): <b>address</b> {
    <a href="tx_context.md#0x2_tx_context_fresh_address">tx_context::fresh_address</a>(&<b>mut</b> self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_context_fresh_object_id"></a>

## Function `fresh_object_id`

Generate a new unique object ID


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_fresh_object_id">fresh_object_id</a>(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_fresh_object_id">fresh_object_id</a>(self: &<b>mut</b> <a href="context.md#0x2_context_Context">Context</a>): ObjectID {
    <a href="object.md#0x2_object_address_to_object_id">object::address_to_object_id</a>(<a href="tx_context.md#0x2_tx_context_fresh_address">tx_context::fresh_address</a>(&<b>mut</b> self.<a href="tx_context.md#0x2_tx_context">tx_context</a>))
}
</code></pre>



</details>

<a name="0x2_context_tx_hash"></a>

## Function `tx_hash`

Return the hash of the current transaction


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_tx_hash">tx_hash</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_tx_hash">tx_hash</a>(self: &<a href="context.md#0x2_context_Context">Context</a>): <a href="">vector</a>&lt;u8&gt; {
    <a href="tx_context.md#0x2_tx_context_tx_hash">tx_context::tx_hash</a>(&self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_context_add"></a>

## Function `add`

Add a value to the context map


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_add">add</a>&lt;T: <b>copy</b>, drop, store&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, value: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_add">add</a>&lt;T: drop + store + <b>copy</b>&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">Context</a>, value: T) {
    <a href="tx_context.md#0x2_tx_context_add">tx_context::add</a>(&<b>mut</b> self.<a href="tx_context.md#0x2_tx_context">tx_context</a>, value);
}
</code></pre>



</details>

<a name="0x2_context_get"></a>

## Function `get`

Get a value from the context map


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_get">get</a>&lt;T: <b>copy</b>, drop, store&gt;(self: &<a href="context.md#0x2_context_Context">context::Context</a>): <a href="_Option">option::Option</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_get">get</a>&lt;T: drop + store + <b>copy</b>&gt;(self: &<a href="context.md#0x2_context_Context">Context</a>): Option&lt;T&gt; {
    <a href="tx_context.md#0x2_tx_context_get">tx_context::get</a>(&self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_context_tx_meta"></a>

## Function `tx_meta`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta">tx_meta</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): <a href="tx_meta.md#0x2_tx_meta_TxMeta">tx_meta::TxMeta</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta">tx_meta</a>(self: &<a href="context.md#0x2_context_Context">Context</a>): TxMeta {
    <a href="tx_context.md#0x2_tx_context_tx_meta">tx_context::tx_meta</a>(&self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_context_tx_gas_payment_account"></a>

## Function `tx_gas_payment_account`



<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_tx_gas_payment_account">tx_gas_payment_account</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_tx_gas_payment_account">tx_gas_payment_account</a>(self: &<a href="context.md#0x2_context_Context">Context</a>): <b>address</b> {
    <a href="tx_context.md#0x2_tx_context_tx_gas_payment_account">tx_context::tx_gas_payment_account</a>(&self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_context_tx_result"></a>

## Function `tx_result`



<pre><code><b>public</b> <b>fun</b> <a href="tx_result.md#0x2_tx_result">tx_result</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): <a href="tx_result.md#0x2_tx_result_TxResult">tx_result::TxResult</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_result.md#0x2_tx_result">tx_result</a>(self: &<a href="context.md#0x2_context_Context">Context</a>): TxResult {
    <a href="tx_context.md#0x2_tx_context_tx_result">tx_context::tx_result</a>(&self.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_context_borrow_object"></a>

## Function `borrow_object`

Borrow Object from object store with object_id


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_borrow_object">borrow_object</a>&lt;T: key&gt;(self: &<a href="context.md#0x2_context_Context">context::Context</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_borrow_object">borrow_object</a>&lt;T: key&gt;(self: &<a href="context.md#0x2_context_Context">Context</a>, object_id: ObjectID): &Object&lt;T&gt; {
    <a href="storage_context.md#0x2_storage_context_borrow">storage_context::borrow</a>&lt;T&gt;(&self.<a href="storage_context.md#0x2_storage_context">storage_context</a>, object_id)
}
</code></pre>



</details>

<a name="0x2_context_borrow_object_mut"></a>

## Function `borrow_object_mut`

Borrow mut Object from object store with object_id


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_borrow_object_mut">borrow_object_mut</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_borrow_object_mut">borrow_object_mut</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">Context</a>, object_id: ObjectID): &<b>mut</b> Object&lt;T&gt; {
    <a href="storage_context.md#0x2_storage_context_borrow_mut">storage_context::borrow_mut</a>&lt;T&gt;(&<b>mut</b> self.<a href="storage_context.md#0x2_storage_context">storage_context</a>, object_id)
}
</code></pre>



</details>

<a name="0x2_context_remove_object"></a>

## Function `remove_object`

Remove object from object store, and unpack the Object


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_remove_object">remove_object</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): (<a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, <b>address</b>, T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_remove_object">remove_object</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">Context</a>, object_id: ObjectID): (ObjectID, <b>address</b>, T) {
    <b>let</b> obj = <a href="storage_context.md#0x2_storage_context_remove">storage_context::remove</a>&lt;T&gt;(&<b>mut</b> self.<a href="storage_context.md#0x2_storage_context">storage_context</a>, object_id);
    <a href="object.md#0x2_object_unpack_internal">object::unpack_internal</a>(obj)
}
</code></pre>



</details>

<a name="0x2_context_exist_object"></a>

## Function `exist_object`



<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_exist_object">exist_object</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_exist_object">exist_object</a>(self: &<a href="context.md#0x2_context_Context">Context</a>, object_id: ObjectID): bool {
    <a href="storage_context.md#0x2_storage_context_contains">storage_context::contains</a>(&self.<a href="storage_context.md#0x2_storage_context">storage_context</a>, object_id)
}
</code></pre>



</details>

<a name="0x2_context_new_object"></a>

## Function `new_object`

Create a new Object, the owner is the <code>sender</code>
Add the Object to the global object storage and return the ObjectRef


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_new_object">new_object</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, value: T): <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_new_object">new_object</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">Context</a>, value: T): ObjectRef&lt;T&gt; {
    <b>let</b> id = <a href="context.md#0x2_context_fresh_object_id">fresh_object_id</a>(self);
    <b>let</b> owner = <a href="context.md#0x2_context_sender">sender</a>(self);
    <a href="context.md#0x2_context_new_object_with_id">new_object_with_id</a>(self, id, owner, value)
}
</code></pre>



</details>

<a name="0x2_context_new_object_with_owner"></a>

## Function `new_object_with_owner`

Create a new Object with owner
Add the Object to the global object storage and return the ObjectRef


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_new_object_with_owner">new_object_with_owner</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, owner: <b>address</b>, value: T): <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_new_object_with_owner">new_object_with_owner</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">Context</a>, owner: <b>address</b>, value: T): ObjectRef&lt;T&gt; {
    <b>let</b> object_id = <a href="context.md#0x2_context_fresh_object_id">fresh_object_id</a>(self);
    <a href="context.md#0x2_context_new_object_with_id">new_object_with_id</a>(self, object_id, owner, value)
}
</code></pre>



</details>

<a name="0x2_context_new_object_with_id"></a>

## Function `new_object_with_id`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="context.md#0x2_context_new_object_with_id">new_object_with_id</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, owner: <b>address</b>, value: T): <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="context.md#0x2_context_new_object_with_id">new_object_with_id</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">Context</a>, id: ObjectID, owner: <b>address</b>, value: T) : ObjectRef&lt;T&gt; {
    <b>let</b> obj = <a href="object.md#0x2_object_new">object::new</a>(id, owner, value);
    <b>let</b> obj_ref = <a href="object_ref.md#0x2_object_ref_new_internal">object_ref::new_internal</a>(&<b>mut</b> obj);
    <a href="storage_context.md#0x2_storage_context_add">storage_context::add</a>(&<b>mut</b> self.<a href="storage_context.md#0x2_storage_context">storage_context</a>, obj);
    obj_ref
}
</code></pre>



</details>

<a name="0x2_context_new_singleton_object"></a>

## Function `new_singleton_object`



<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_new_singleton_object">new_singleton_object</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, value: T): <a href="object_ref.md#0x2_object_ref_ObjectRef">object_ref::ObjectRef</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_new_singleton_object">new_singleton_object</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">Context</a>, value: T): ObjectRef&lt;T&gt; {
    <b>let</b> object_id = <a href="object.md#0x2_object_singleton_object_id">object::singleton_object_id</a>&lt;T&gt;();
    <a href="context.md#0x2_context_new_object_with_id">new_object_with_id</a>(self, object_id, <a href="type_info.md#0x2_type_info_account_address">type_info::account_address</a>(&<a href="type_info.md#0x2_type_info_type_of">type_info::type_of</a>&lt;T&gt;()), value)
}
</code></pre>



</details>
