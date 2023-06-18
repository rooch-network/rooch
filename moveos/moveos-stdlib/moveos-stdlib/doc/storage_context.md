
<a name="0x2_storage_context"></a>

# Module `0x2::storage_context`

StorageContext is part of the StorageAbstraction
It is used to provide a context for the storage operations, make the storage abstraction,
and let developers can customize the storage


-  [Struct `StorageContext`](#0x2_storage_context_StorageContext)
-  [Function `tx_context`](#0x2_storage_context_tx_context)
-  [Function `tx_context_mut`](#0x2_storage_context_tx_context_mut)
-  [Function `object_storage`](#0x2_storage_context_object_storage)
-  [Function `object_storage_mut`](#0x2_storage_context_object_storage_mut)
-  [Function `sender`](#0x2_storage_context_sender)
-  [Function `fresh_address`](#0x2_storage_context_fresh_address)
-  [Function `fresh_object_id`](#0x2_storage_context_fresh_object_id)
-  [Function `tx_hash`](#0x2_storage_context_tx_hash)


<pre><code><b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
<b>use</b> <a href="object_storage.md#0x2_object_storage">0x2::object_storage</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_storage_context_StorageContext"></a>

## Struct `StorageContext`

Information about the global storage context
We can not put the StorageContext to TxContext, because object module depends on tx_context module,
and storage_context module depends on object module.
We put TxContext to StorageContext, for convenience of developers.
The StorageContext can not be <code>drop</code> or <code>store</code>, so developers need to pass the <code>&<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a></code> or <code>&<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a></code> to the <code>entry</code> function.


<pre><code><b>struct</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>
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
<code><a href="object_storage.md#0x2_object_storage">object_storage</a>: <a href="object_storage.md#0x2_object_storage_ObjectStorage">object_storage::ObjectStorage</a></code>
</dt>
<dd>
 The Global Object Storage
</dd>
</dl>


</details>

<a name="0x2_storage_context_tx_context"></a>

## Function `tx_context`



<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context">tx_context</a>(this: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context">tx_context</a>(this: &<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): &TxContext {
    &this.<a href="tx_context.md#0x2_tx_context">tx_context</a>
}
</code></pre>



</details>

<a name="0x2_storage_context_tx_context_mut"></a>

## Function `tx_context_mut`



<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_tx_context_mut">tx_context_mut</a>(this: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_tx_context_mut">tx_context_mut</a>(this: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): &<b>mut</b> TxContext {
    &<b>mut</b> this.<a href="tx_context.md#0x2_tx_context">tx_context</a>
}
</code></pre>



</details>

<a name="0x2_storage_context_object_storage"></a>

## Function `object_storage`



<pre><code><b>public</b> <b>fun</b> <a href="object_storage.md#0x2_object_storage">object_storage</a>(this: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): &<a href="object_storage.md#0x2_object_storage_ObjectStorage">object_storage::ObjectStorage</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="object_storage.md#0x2_object_storage">object_storage</a>(this: &<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): &ObjectStorage {
    &this.<a href="object_storage.md#0x2_object_storage">object_storage</a>
}
</code></pre>



</details>

<a name="0x2_storage_context_object_storage_mut"></a>

## Function `object_storage_mut`



<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_object_storage_mut">object_storage_mut</a>(this: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): &<b>mut</b> <a href="object_storage.md#0x2_object_storage_ObjectStorage">object_storage::ObjectStorage</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_object_storage_mut">object_storage_mut</a>(this: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): &<b>mut</b> ObjectStorage {
    &<b>mut</b> this.<a href="object_storage.md#0x2_object_storage">object_storage</a>
}
</code></pre>



</details>

<a name="0x2_storage_context_sender"></a>

## Function `sender`

Wrap functions for TxContext


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_sender">sender</a>(this: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_sender">sender</a>(this: &<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): <b>address</b> {
    <a href="tx_context.md#0x2_tx_context_sender">tx_context::sender</a>(&this.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_storage_context_fresh_address"></a>

## Function `fresh_address`



<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_fresh_address">fresh_address</a>(this: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_fresh_address">fresh_address</a>(this: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): <b>address</b> {
    <a href="tx_context.md#0x2_tx_context_fresh_address">tx_context::fresh_address</a>(&<b>mut</b> this.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_storage_context_fresh_object_id"></a>

## Function `fresh_object_id`



<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_fresh_object_id">fresh_object_id</a>(this: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_fresh_object_id">fresh_object_id</a>(this: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): ObjectID {
    <a href="tx_context.md#0x2_tx_context_fresh_object_id">tx_context::fresh_object_id</a>(&<b>mut</b> this.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>

<a name="0x2_storage_context_tx_hash"></a>

## Function `tx_hash`



<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_tx_hash">tx_hash</a>(this: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="storage_context.md#0x2_storage_context_tx_hash">tx_hash</a>(this: &<a href="storage_context.md#0x2_storage_context_StorageContext">StorageContext</a>): <a href="">vector</a>&lt;u8&gt; {
    <a href="tx_context.md#0x2_tx_context_tx_hash">tx_context::tx_hash</a>(&this.<a href="tx_context.md#0x2_tx_context">tx_context</a>)
}
</code></pre>



</details>
