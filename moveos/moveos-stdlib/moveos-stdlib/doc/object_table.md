
<a name="0x2_object_table"></a>

# Module `0x2::object_table`



-  [Resource `ObjectTable`](#0x2_object_table_ObjectTable)
-  [Function `new`](#0x2_object_table_new)
-  [Function `add`](#0x2_object_table_add)
-  [Function `borrow`](#0x2_object_table_borrow)
-  [Function `borrow_mut`](#0x2_object_table_borrow_mut)
-  [Function `remove`](#0x2_object_table_remove)
-  [Function `contains`](#0x2_object_table_contains)
-  [Function `destroy_empty`](#0x2_object_table_destroy_empty)
-  [Function `length`](#0x2_object_table_length)
-  [Function `is_empty`](#0x2_object_table_is_empty)
-  [Function `handle`](#0x2_object_table_handle)


<pre><code><b>use</b> <a href="context.md#0x2_context">0x2::context</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
</code></pre>



<a name="0x2_object_table_ObjectTable"></a>

## Resource `ObjectTable`

A Table for storing objects


<pre><code><b>struct</b> <a href="object_table.md#0x2_object_table_ObjectTable">ObjectTable</a>&lt;T&gt; <b>has</b> store, key
</code></pre>



<a name="0x2_object_table_new"></a>

## Function `new`

Create a new Table.


<pre><code><b>public</b> <b>fun</b> <a href="object_table.md#0x2_object_table_new">new</a>&lt;T: key&gt;(ctx: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>): <a href="object_table.md#0x2_object_table_ObjectTable">object_table::ObjectTable</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_table_add"></a>

## Function `add`

Add a new Object to the table.


<pre><code><b>public</b> <b>fun</b> <a href="object_table.md#0x2_object_table_add">add</a>&lt;T&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="object_table.md#0x2_object_table_ObjectTable">object_table::ObjectTable</a>&lt;T&gt;, obj: <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<a name="0x2_object_table_borrow"></a>

## Function `borrow`

Acquire an immutable reference to the Object<T> with <code><a href="object_id.md#0x2_object_id">object_id</a></code>.
Aborts if there is no entry for <code><a href="object_id.md#0x2_object_id">object_id</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="object_table.md#0x2_object_table_borrow">borrow</a>&lt;T&gt;(<a href="table.md#0x2_table">table</a>: &<a href="object_table.md#0x2_object_table_ObjectTable">object_table::ObjectTable</a>&lt;T&gt;, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_table_borrow_mut"></a>

## Function `borrow_mut`

Acquire a mutable reference to the Object<T> with <code><a href="object_id.md#0x2_object_id">object_id</a></code>.
Aborts if there is no entry for <code><a href="object_id.md#0x2_object_id">object_id</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="object_table.md#0x2_object_table_borrow_mut">borrow_mut</a>&lt;T&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="object_table.md#0x2_object_table_ObjectTable">object_table::ObjectTable</a>&lt;T&gt;, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_table_remove"></a>

## Function `remove`

Remove from <code><a href="table.md#0x2_table">table</a></code> and return the Object<T>  with <code><a href="object_id.md#0x2_object_id">object_id</a></code>.
Aborts if there is no entry for <code><a href="object_id.md#0x2_object_id">object_id</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="object_table.md#0x2_object_table_remove">remove</a>&lt;T&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="object_table.md#0x2_object_table_ObjectTable">object_table::ObjectTable</a>&lt;T&gt;, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;
</code></pre>



<a name="0x2_object_table_contains"></a>

## Function `contains`

Returns true if <code><a href="table.md#0x2_table">table</a></code> contains an entry for <code><a href="object_id.md#0x2_object_id">object_id</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="object_table.md#0x2_object_table_contains">contains</a>&lt;T&gt;(<a href="table.md#0x2_table">table</a>: &<a href="object_table.md#0x2_object_table_ObjectTable">object_table::ObjectTable</a>&lt;T&gt;, <a href="object_id.md#0x2_object_id">object_id</a>: <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>): bool
</code></pre>



<a name="0x2_object_table_destroy_empty"></a>

## Function `destroy_empty`

Destroy a table. Aborts if the table is not empty.


<pre><code><b>public</b> <b>fun</b> <a href="object_table.md#0x2_object_table_destroy_empty">destroy_empty</a>&lt;T&gt;(<a href="table.md#0x2_table">table</a>: <a href="object_table.md#0x2_object_table_ObjectTable">object_table::ObjectTable</a>&lt;T&gt;)
</code></pre>



<a name="0x2_object_table_length"></a>

## Function `length`

Returns the size of the table


<pre><code><b>public</b> <b>fun</b> <a href="object_table.md#0x2_object_table_length">length</a>&lt;T&gt;(<a href="table.md#0x2_table">table</a>: &<a href="object_table.md#0x2_object_table_ObjectTable">object_table::ObjectTable</a>&lt;T&gt;): u64
</code></pre>



<a name="0x2_object_table_is_empty"></a>

## Function `is_empty`

Returns true iff the table is empty (if <code>length</code> returns <code>0</code>)


<pre><code><b>public</b> <b>fun</b> <a href="object_table.md#0x2_object_table_is_empty">is_empty</a>&lt;T&gt;(<a href="table.md#0x2_table">table</a>: &<a href="object_table.md#0x2_object_table_ObjectTable">object_table::ObjectTable</a>&lt;T&gt;): bool
</code></pre>



<a name="0x2_object_table_handle"></a>

## Function `handle`

Returns table handle of <code><a href="table.md#0x2_table">table</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="object_table.md#0x2_object_table_handle">handle</a>&lt;T&gt;(<a href="table.md#0x2_table">table</a>: &<a href="object_table.md#0x2_object_table_ObjectTable">object_table::ObjectTable</a>&lt;T&gt;): &<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>
