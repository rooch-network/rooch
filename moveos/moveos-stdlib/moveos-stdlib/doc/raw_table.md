
<a name="0x2_raw_table"></a>

# Module `0x2::raw_table`

Raw Key Value table. This is the basic of storage abstraction.
This type table doesn't care about the key and value types. We leave the data type checking to the Native implementation.
This type table is for internal global storage, so all functions are friend.


-  [Resource `TableInfo`](#0x2_raw_table_TableInfo)
-  [Resource `Box`](#0x2_raw_table_Box)
-  [Struct `TableHandle`](#0x2_raw_table_TableHandle)
-  [Constants](#@Constants_0)
-  [Function `add`](#0x2_raw_table_add)
-  [Function `borrow`](#0x2_raw_table_borrow)
-  [Function `borrow_with_default`](#0x2_raw_table_borrow_with_default)
-  [Function `borrow_mut`](#0x2_raw_table_borrow_mut)
-  [Function `borrow_mut_with_default`](#0x2_raw_table_borrow_mut_with_default)
-  [Function `upsert`](#0x2_raw_table_upsert)
-  [Function `remove`](#0x2_raw_table_remove)
-  [Function `contains`](#0x2_raw_table_contains)
-  [Function `length`](#0x2_raw_table_length)
-  [Function `is_empty`](#0x2_raw_table_is_empty)
-  [Function `drop_unchecked`](#0x2_raw_table_drop_unchecked)
-  [Function `destroy_empty`](#0x2_raw_table_destroy_empty)
-  [Function `new_table_handle`](#0x2_raw_table_new_table_handle)


<pre><code></code></pre>



<a name="0x2_raw_table_TableInfo"></a>

## Resource `TableInfo`



<pre><code><b>struct</b> <a href="raw_table.md#0x2_raw_table_TableInfo">TableInfo</a> <b>has</b> key
</code></pre>



<a name="0x2_raw_table_Box"></a>

## Resource `Box`

Wrapper for values. Required for making values appear as resources in the implementation.
Because the GlobalValue in MoveVM must be a resource.


<pre><code><b>struct</b> <a href="raw_table.md#0x2_raw_table_Box">Box</a>&lt;V&gt; <b>has</b> drop, store, key
</code></pre>



<a name="0x2_raw_table_TableHandle"></a>

## Struct `TableHandle`

The TableHandle has the same layout as the ObjectID
We can convert the ObjectID to TableHandle
Define a TableHandle is for remove the dependency of ObjectID, and object module


<pre><code><b>struct</b> <a href="raw_table.md#0x2_raw_table_TableHandle">TableHandle</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_raw_table_ErrorAlreadyExists"></a>

The key already exists in the table


<pre><code><b>const</b> <a href="raw_table.md#0x2_raw_table_ErrorAlreadyExists">ErrorAlreadyExists</a>: u64 = 1;
</code></pre>



<a name="0x2_raw_table_ErrorDuplicateOperation"></a>

Duplicate operation on the table


<pre><code><b>const</b> <a href="raw_table.md#0x2_raw_table_ErrorDuplicateOperation">ErrorDuplicateOperation</a>: u64 = 3;
</code></pre>



<a name="0x2_raw_table_ErrorNotEmpty"></a>

The table is not empty


<pre><code><b>const</b> <a href="raw_table.md#0x2_raw_table_ErrorNotEmpty">ErrorNotEmpty</a>: u64 = 4;
</code></pre>



<a name="0x2_raw_table_ErrorNotFound"></a>

Can not found the key in the table


<pre><code><b>const</b> <a href="raw_table.md#0x2_raw_table_ErrorNotFound">ErrorNotFound</a>: u64 = 2;
</code></pre>



<a name="0x2_raw_table_add"></a>

## Function `add`

Add a new entry to the table. Aborts if an entry for this
key already exists. The entry itself is not stored in the
table, and cannot be discovered from it.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_add">add</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>, key: K, val: V)
</code></pre>



<a name="0x2_raw_table_borrow"></a>

## Function `borrow`

Acquire an immutable reference to the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_borrow">borrow</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>, key: K): &V
</code></pre>



<a name="0x2_raw_table_borrow_with_default"></a>

## Function `borrow_with_default`

Acquire an immutable reference to the value which <code>key</code> maps to.
Returns specified default value if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_borrow_with_default">borrow_with_default</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>, key: K, default: &V): &V
</code></pre>



<a name="0x2_raw_table_borrow_mut"></a>

## Function `borrow_mut`

Acquire a mutable reference to the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_borrow_mut">borrow_mut</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>, key: K): &<b>mut</b> V
</code></pre>



<a name="0x2_raw_table_borrow_mut_with_default"></a>

## Function `borrow_mut_with_default`

Acquire a mutable reference to the value which <code>key</code> maps to.
Insert the pair (<code>key</code>, <code>default</code>) first if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_borrow_mut_with_default">borrow_mut_with_default</a>&lt;K: <b>copy</b>, drop, V: drop&gt;(table_handle: <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>, key: K, default: V): &<b>mut</b> V
</code></pre>



<a name="0x2_raw_table_upsert"></a>

## Function `upsert`

Insert the pair (<code>key</code>, <code>value</code>) if there is no entry for <code>key</code>.
update the value of the entry for <code>key</code> to <code>value</code> otherwise


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_upsert">upsert</a>&lt;K: <b>copy</b>, drop, V: drop&gt;(table_handle: <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>, key: K, value: V)
</code></pre>



<a name="0x2_raw_table_remove"></a>

## Function `remove`

Remove from <code><a href="table.md#0x2_table">table</a></code> and return the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_remove">remove</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>, key: K): V
</code></pre>



<a name="0x2_raw_table_contains"></a>

## Function `contains`

Returns true if <code><a href="table.md#0x2_table">table</a></code> contains an entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_contains">contains</a>&lt;K: <b>copy</b>, drop&gt;(table_handle: <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>, key: K): bool
</code></pre>



<a name="0x2_raw_table_length"></a>

## Function `length`

Returns the size of the table, the number of key-value pairs


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_length">length</a>(table_handle: <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>): u64
</code></pre>



<a name="0x2_raw_table_is_empty"></a>

## Function `is_empty`

Returns true if the table is empty (if <code>length</code> returns <code>0</code>)


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_is_empty">is_empty</a>(table_handle: <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>): bool
</code></pre>



<a name="0x2_raw_table_drop_unchecked"></a>

## Function `drop_unchecked`

Drop a table even if it is not empty.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_drop_unchecked">drop_unchecked</a>(table_handle: <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>)
</code></pre>



<a name="0x2_raw_table_destroy_empty"></a>

## Function `destroy_empty`

Destroy a table. Aborts if the table is not empty


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_destroy_empty">destroy_empty</a>(table_handle: <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>)
</code></pre>



<a name="0x2_raw_table_new_table_handle"></a>

## Function `new_table_handle`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_new_table_handle">new_table_handle</a>(id: <b>address</b>): <a href="raw_table.md#0x2_raw_table_TableHandle">raw_table::TableHandle</a>
</code></pre>
