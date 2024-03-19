
<a name="0x2_table"></a>

# Module `0x2::table`

Type of large-scale storage tables.
source: https://github.com/move-language/move/blob/1b6b7513dcc1a5c866f178ca5c1e74beb2ce181e/language/extensions/move-table-extension/sources/Table.move#L1

It implements the Table type which supports individual table items to be represented by
separate global state items. The number of items and a unique handle are tracked on the table
struct itself, while the operations are implemented as native functions. No traversal is provided.


-  [Resource `TablePlaceholder`](#0x2_table_TablePlaceholder)
-  [Struct `Table`](#0x2_table_Table)
-  [Function `new`](#0x2_table_new)
-  [Function `add`](#0x2_table_add)
-  [Function `borrow`](#0x2_table_borrow)
-  [Function `borrow_with_default`](#0x2_table_borrow_with_default)
-  [Function `borrow_mut`](#0x2_table_borrow_mut)
-  [Function `borrow_mut_with_default`](#0x2_table_borrow_mut_with_default)
-  [Function `upsert`](#0x2_table_upsert)
-  [Function `remove`](#0x2_table_remove)
-  [Function `contains`](#0x2_table_contains)
-  [Function `destroy_empty`](#0x2_table_destroy_empty)
-  [Function `length`](#0x2_table_length)
-  [Function `is_empty`](#0x2_table_is_empty)
-  [Function `drop`](#0x2_table_drop)
-  [Function `handle`](#0x2_table_handle)


<pre><code><b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
</code></pre>



<a name="0x2_table_TablePlaceholder"></a>

## Resource `TablePlaceholder`



<pre><code><b>struct</b> <a href="table.md#0x2_table_TablePlaceholder">TablePlaceholder</a> <b>has</b> key
</code></pre>



<a name="0x2_table_Table"></a>

## Struct `Table`

Type of tables


<pre><code><b>struct</b> <a href="table.md#0x2_table_Table">Table</a>&lt;K: <b>copy</b>, drop, V&gt; <b>has</b> store
</code></pre>



<a name="0x2_table_new"></a>

## Function `new`

Create a new Table.


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_new">new</a>&lt;K: <b>copy</b>, drop, V: store&gt;(): <a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;
</code></pre>



<a name="0x2_table_add"></a>

## Function `add`

Add a new entry to the table. Aborts if an entry for this
key already exists. The entry itself is not stored in the
table, and cannot be discovered from it.


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_add">add</a>&lt;K: <b>copy</b>, drop, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;, key: K, val: V)
</code></pre>



<a name="0x2_table_borrow"></a>

## Function `borrow`

Acquire an immutable reference to the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_borrow">borrow</a>&lt;K: <b>copy</b>, drop, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;, key: K): &V
</code></pre>



<a name="0x2_table_borrow_with_default"></a>

## Function `borrow_with_default`

Acquire an immutable reference to the value which <code>key</code> maps to.
Returns specified default value if there is no entry for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_borrow_with_default">borrow_with_default</a>&lt;K: <b>copy</b>, drop, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;, key: K, default: &V): &V
</code></pre>



<a name="0x2_table_borrow_mut"></a>

## Function `borrow_mut`

Acquire a mutable reference to the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_borrow_mut">borrow_mut</a>&lt;K: <b>copy</b>, drop, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;, key: K): &<b>mut</b> V
</code></pre>



<a name="0x2_table_borrow_mut_with_default"></a>

## Function `borrow_mut_with_default`

Acquire a mutable reference to the value which <code>key</code> maps to.
Insert the pair (<code>key</code>, <code>default</code>) first if there is no entry for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_borrow_mut_with_default">borrow_mut_with_default</a>&lt;K: <b>copy</b>, drop, V: drop, store&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;, key: K, default: V): &<b>mut</b> V
</code></pre>



<a name="0x2_table_upsert"></a>

## Function `upsert`

Insert the pair (<code>key</code>, <code>value</code>) if there is no entry for <code>key</code>.
update the value of the entry for <code>key</code> to <code>value</code> otherwise


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_upsert">upsert</a>&lt;K: <b>copy</b>, drop, V: drop, store&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;, key: K, value: V)
</code></pre>



<a name="0x2_table_remove"></a>

## Function `remove`

Remove from <code><a href="table.md#0x2_table">table</a></code> and return the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_remove">remove</a>&lt;K: <b>copy</b>, drop, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;, key: K): V
</code></pre>



<a name="0x2_table_contains"></a>

## Function `contains`

Returns true if <code><a href="table.md#0x2_table">table</a></code> contains an entry for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_contains">contains</a>&lt;K: <b>copy</b>, drop, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;, key: K): bool
</code></pre>



<a name="0x2_table_destroy_empty"></a>

## Function `destroy_empty`

Destroy a table. Aborts if the table is not empty.


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_destroy_empty">destroy_empty</a>&lt;K: <b>copy</b>, drop, V: store&gt;(<a href="table.md#0x2_table">table</a>: <a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;)
</code></pre>



<a name="0x2_table_length"></a>

## Function `length`

Returns the size of the table, the number of key-value pairs


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_length">length</a>&lt;K: <b>copy</b>, drop, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;): u64
</code></pre>



<a name="0x2_table_is_empty"></a>

## Function `is_empty`

Returns true iff the table is empty (if <code>length</code> returns <code>0</code>)


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_is_empty">is_empty</a>&lt;K: <b>copy</b>, drop, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;): bool
</code></pre>



<a name="0x2_table_drop"></a>

## Function `drop`

Drop a possibly non-empty table.
Usable only if the value type <code>V</code> has the <code>drop</code> ability


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_drop">drop</a>&lt;K: <b>copy</b>, drop, V: drop&gt;(<a href="table.md#0x2_table">table</a>: <a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;)
</code></pre>



<a name="0x2_table_handle"></a>

## Function `handle`

Returns table handle of <code><a href="table.md#0x2_table">table</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="table.md#0x2_table_handle">handle</a>&lt;K: <b>copy</b>, drop, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="table.md#0x2_table_Table">table::Table</a>&lt;K, V&gt;): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>
