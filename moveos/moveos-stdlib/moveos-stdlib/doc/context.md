
<a name="0x2_context"></a>

# Module `0x2::context`

Context is part of the StorageAbstraction
It is used to provide a context for the storage operations, make the storage abstraction,
and let developers customize the storage


-  [Struct `Context`](#0x2_context_Context)
-  [Constants](#@Constants_0)
-  [Function `tx_context`](#0x2_context_tx_context)
-  [Function `tx_context_mut`](#0x2_context_tx_context_mut)
-  [Function `sender`](#0x2_context_sender)
-  [Function `sequence_number`](#0x2_context_sequence_number)
-  [Function `max_gas_amount`](#0x2_context_max_gas_amount)
-  [Function `fresh_address`](#0x2_context_fresh_address)
-  [Function `fresh_object_id`](#0x2_context_fresh_object_id)
-  [Function `fresh_uid`](#0x2_context_fresh_uid)
-  [Function `tx_hash`](#0x2_context_tx_hash)
-  [Function `add`](#0x2_context_add)
-  [Function `get`](#0x2_context_get)
-  [Function `tx_meta`](#0x2_context_tx_meta)
-  [Function `tx_gas_payment_account`](#0x2_context_tx_gas_payment_account)
-  [Function `tx_result`](#0x2_context_tx_result)
-  [Function `new_object`](#0x2_context_new_object)
-  [Function `new_object_with_id`](#0x2_context_new_object_with_id)
-  [Function `new_singleton`](#0x2_context_new_singleton)
-  [Function `borrow_object`](#0x2_context_borrow_object)
-  [Function `borrow_mut_object`](#0x2_context_borrow_mut_object)
-  [Function `take_object`](#0x2_context_take_object)
-  [Function `borrow_mut_object_shared`](#0x2_context_borrow_mut_object_shared)
-  [Function `borrow_mut_object_extend`](#0x2_context_borrow_mut_object_extend)
-  [Function `exist_object`](#0x2_context_exist_object)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="signer.md#0x2_signer">0x2::signer</a>;
<b>use</b> <a href="storage_context.md#0x2_storage_context">0x2::storage_context</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
<b>use</b> <a href="tx_meta.md#0x2_tx_meta">0x2::tx_meta</a>;
<b>use</b> <a href="tx_result.md#0x2_tx_result">0x2::tx_result</a>;
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



<a name="@Constants_0"></a>

## Constants


<a name="0x2_context_ErrorObjectNotShared"></a>



<pre><code><b>const</b> <a href="context.md#0x2_context_ErrorObjectNotShared">ErrorObjectNotShared</a>: u64 = 2;
</code></pre>



<a name="0x2_context_ErrorObjectOwnerNotMatch"></a>



<pre><code><b>const</b> <a href="context.md#0x2_context_ErrorObjectOwnerNotMatch">ErrorObjectOwnerNotMatch</a>: u64 = 1;
</code></pre>



<a name="0x2_context_tx_context"></a>

## Function `tx_context`

Get an immutable reference to the transaction context from the storage context


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context">tx_context</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>
</code></pre>



<a name="0x2_context_tx_context_mut"></a>

## Function `tx_context_mut`

Get a mutable reference to the transaction context from the storage context


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="context.md#0x2_context_tx_context_mut">tx_context_mut</a>(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>): &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>
</code></pre>



<a name="0x2_context_sender"></a>

## Function `sender`

Return the address of the user that signed the current transaction


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_sender">sender</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): <b>address</b>
</code></pre>



<a name="0x2_context_sequence_number"></a>

## Function `sequence_number`

Return the sequence number of the current transaction


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_sequence_number">sequence_number</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): u64
</code></pre>



<a name="0x2_context_max_gas_amount"></a>

## Function `max_gas_amount`

Return the maximum gas amount that can be used by the current transaction


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_max_gas_amount">max_gas_amount</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): u64
</code></pre>



<a name="0x2_context_fresh_address"></a>

## Function `fresh_address`

Generate a new unique address


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_fresh_address">fresh_address</a>(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>): <b>address</b>
</code></pre>



<a name="0x2_context_fresh_object_id"></a>

## Function `fresh_object_id`

Generate a new unique ObjectID


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_fresh_object_id">fresh_object_id</a>(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_context_fresh_uid"></a>

## Function `fresh_uid`

Generate a new unique ID


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_fresh_uid">fresh_uid</a>(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>): <a href="object.md#0x2_object_UID">object::UID</a>
</code></pre>



<a name="0x2_context_tx_hash"></a>

## Function `tx_hash`

Return the hash of the current transaction


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_tx_hash">tx_hash</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_context_add"></a>

## Function `add`

Add a value to the context map


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_add">add</a>&lt;T: <b>copy</b>, drop, store&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, value: T)
</code></pre>



<a name="0x2_context_get"></a>

## Function `get`

Get a value from the context map


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_get">get</a>&lt;T: <b>copy</b>, drop, store&gt;(self: &<a href="context.md#0x2_context_Context">context::Context</a>): <a href="_Option">option::Option</a>&lt;T&gt;
</code></pre>



<a name="0x2_context_tx_meta"></a>

## Function `tx_meta`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta">tx_meta</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): <a href="tx_meta.md#0x2_tx_meta_TxMeta">tx_meta::TxMeta</a>
</code></pre>



<a name="0x2_context_tx_gas_payment_account"></a>

## Function `tx_gas_payment_account`



<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_tx_gas_payment_account">tx_gas_payment_account</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): <b>address</b>
</code></pre>



<a name="0x2_context_tx_result"></a>

## Function `tx_result`



<pre><code><b>public</b> <b>fun</b> <a href="tx_result.md#0x2_tx_result">tx_result</a>(self: &<a href="context.md#0x2_context_Context">context::Context</a>): <a href="tx_result.md#0x2_tx_result_TxResult">tx_result::TxResult</a>
</code></pre>



<a name="0x2_context_new_object"></a>

## Function `new_object`

Create a new Object, Add the Object to the global object storage and return the Object
Note: the default owner is the <code>System</code>, the caller should explicitly transfer the Object to the owner.
The owner can get the <code>&<b>mut</b> Object</code> by <code>borrow_mut_object</code>
TODO should we still keep this function?


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_new_object">new_object</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_context_new_object_with_id"></a>

## Function `new_object_with_id`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="context.md#0x2_context_new_object_with_id">new_object_with_id</a>&lt;T: key&gt;(_self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, value: T): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_context_new_singleton"></a>

## Function `new_singleton`

Create a new singleton object, the object is owned by <code>System</code> by default.
Singleton object means the object of <code>T</code> is only one instance in the Object Storage.


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_new_singleton">new_singleton</a>&lt;T: key&gt;(_self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, value: T): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_context_borrow_object"></a>

## Function `borrow_object`

Borrow Object from object store by object_id
Any one can borrow an <code>&Object&lt;T&gt;</code> from the global object storage


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_borrow_object">borrow_object</a>&lt;T: key&gt;(_self: &<a href="context.md#0x2_context_Context">context::Context</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_context_borrow_mut_object"></a>

## Function `borrow_mut_object`

Borrow mut Object by <code>owner</code> and <code>object_id</code>


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_borrow_mut_object">borrow_mut_object</a>&lt;T: key&gt;(_self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, owner: &<a href="">signer</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_context_take_object"></a>

## Function `take_object`

Take out the Object by <code>owner</code> and <code>object_id</code>
Note: When the Object is taken out, the Object will auto become <code>SystemOwned</code> Object.
TODO find a better name.


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_take_object">take_object</a>&lt;T: key&gt;(_self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, owner: &<a href="">signer</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_context_borrow_mut_object_shared"></a>

## Function `borrow_mut_object_shared`

Borrow mut Shared Object by object_id


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_borrow_mut_object_shared">borrow_mut_object_shared</a>&lt;T: key&gt;(_self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_context_borrow_mut_object_extend"></a>

## Function `borrow_mut_object_extend`

The module of T can borrow mut Object from object store by any object_id


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_borrow_mut_object_extend">borrow_mut_object_extend</a>&lt;T: key&gt;(_self: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_context_exist_object"></a>

## Function `exist_object`

Check if the object exists in the global object storage


<pre><code><b>public</b> <b>fun</b> <a href="context.md#0x2_context_exist_object">exist_object</a>&lt;T: key&gt;(_self: &<a href="context.md#0x2_context_Context">context::Context</a>, object_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): bool
</code></pre>
