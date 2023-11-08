
<a name="0x2_type_table"></a>

# Module `0x2::type_table`

TypeTable is a table use struct Type as Key, struct as Value


-  [Struct `TypeTable`](#0x2_type_table_TypeTable)
-  [Function `new`](#0x2_type_table_new)
-  [Function `new_with_id`](#0x2_type_table_new_with_id)
-  [Function `add`](#0x2_type_table_add)
-  [Function `borrow`](#0x2_type_table_borrow)
-  [Function `borrow_mut`](#0x2_type_table_borrow_mut)
-  [Function `remove`](#0x2_type_table_remove)
-  [Function `contains`](#0x2_type_table_contains)
-  [Function `handle`](#0x2_type_table_handle)
-  [Function `destroy_empty`](#0x2_type_table_destroy_empty)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x1::type_name</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="raw_table.md#0x2_raw_table">0x2::raw_table</a>;
</code></pre>



<a name="0x2_type_table_TypeTable"></a>

## Struct `TypeTable`



<pre><code><b>struct</b> <a href="type_table.md#0x2_type_table_TypeTable">TypeTable</a> <b>has</b> store
</code></pre>



<a name="0x2_type_table_new"></a>

## Function `new`

Create a new Table.


<pre><code><b>public</b> <b>fun</b> <a href="type_table.md#0x2_type_table_new">new</a>(id: <a href="object.md#0x2_object_UID">object::UID</a>): <a href="type_table.md#0x2_type_table_TypeTable">type_table::TypeTable</a>
</code></pre>



<a name="0x2_type_table_new_with_id"></a>

## Function `new_with_id`

Create a new Table with a given handle.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="type_table.md#0x2_type_table_new_with_id">new_with_id</a>(handle: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>): <a href="type_table.md#0x2_type_table_TypeTable">type_table::TypeTable</a>
</code></pre>



<a name="0x2_type_table_add"></a>

## Function `add`

Add a new entry of <code>V</code> to the table. Aborts if an entry for
entry of <code>V</code> type already exists.


<pre><code><b>public</b> <b>fun</b> <a href="type_table.md#0x2_type_table_add">add</a>&lt;V: key&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="type_table.md#0x2_type_table_TypeTable">type_table::TypeTable</a>, val: V)
</code></pre>



<a name="0x2_type_table_borrow"></a>

## Function `borrow`

Acquire an immutable reference to the value which type is <code>V</code>.
Aborts if there is no entry for <code>V</code>.


<pre><code><b>public</b> <b>fun</b> <a href="type_table.md#0x2_type_table_borrow">borrow</a>&lt;V: key&gt;(<a href="table.md#0x2_table">table</a>: &<a href="type_table.md#0x2_type_table_TypeTable">type_table::TypeTable</a>): &V
</code></pre>



<a name="0x2_type_table_borrow_mut"></a>

## Function `borrow_mut`

Acquire a mutable reference to the value which type is <code>V</code>.
Aborts if there is no entry for <code>V</code>.


<pre><code><b>public</b> <b>fun</b> <a href="type_table.md#0x2_type_table_borrow_mut">borrow_mut</a>&lt;V: key&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="type_table.md#0x2_type_table_TypeTable">type_table::TypeTable</a>): &<b>mut</b> V
</code></pre>



<a name="0x2_type_table_remove"></a>

## Function `remove`

Remove from <code><a href="table.md#0x2_table">table</a></code> and return the value which type is <code>V</code>.
Aborts if there is no entry for <code>V</code>.


<pre><code><b>public</b> <b>fun</b> <a href="type_table.md#0x2_type_table_remove">remove</a>&lt;V: key&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="type_table.md#0x2_type_table_TypeTable">type_table::TypeTable</a>): V
</code></pre>



<a name="0x2_type_table_contains"></a>

## Function `contains`

Returns true if <code><a href="table.md#0x2_table">table</a></code> contains an entry for type <code>V</code>.


<pre><code><b>public</b> <b>fun</b> <a href="type_table.md#0x2_type_table_contains">contains</a>&lt;V: key&gt;(<a href="table.md#0x2_table">table</a>: &<a href="type_table.md#0x2_type_table_TypeTable">type_table::TypeTable</a>): bool
</code></pre>



<a name="0x2_type_table_handle"></a>

## Function `handle`

Returns table handle of <code><a href="table.md#0x2_table">table</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="type_table.md#0x2_type_table_handle">handle</a>(<a href="table.md#0x2_table">table</a>: &<a href="type_table.md#0x2_type_table_TypeTable">type_table::TypeTable</a>): &<a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_type_table_destroy_empty"></a>

## Function `destroy_empty`

Destroy a table. The table must be empty to succeed.


<pre><code><b>public</b> <b>fun</b> <a href="type_table.md#0x2_type_table_destroy_empty">destroy_empty</a>(<a href="table.md#0x2_table">table</a>: <a href="type_table.md#0x2_type_table_TypeTable">type_table::TypeTable</a>)
</code></pre>
