
<a name="0x2_linked_table"></a>

# Module `0x2::linked_table`

Similar to <code>moveos_std::table</code> but the values are linked together, allowing for ordered insertion and
removal


-  [Resource `TablePlaceholder`](#0x2_linked_table_TablePlaceholder)
-  [Resource `LinkedTable`](#0x2_linked_table_LinkedTable)
-  [Struct `Node`](#0x2_linked_table_Node)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x2_linked_table_new)
-  [Function `front`](#0x2_linked_table_front)
-  [Function `back`](#0x2_linked_table_back)
-  [Function `push_front`](#0x2_linked_table_push_front)
-  [Function `push_back`](#0x2_linked_table_push_back)
-  [Function `borrow`](#0x2_linked_table_borrow)
-  [Function `borrow_mut`](#0x2_linked_table_borrow_mut)
-  [Function `prev`](#0x2_linked_table_prev)
-  [Function `next`](#0x2_linked_table_next)
-  [Function `remove`](#0x2_linked_table_remove)
-  [Function `pop_front`](#0x2_linked_table_pop_front)
-  [Function `pop_back`](#0x2_linked_table_pop_back)
-  [Function `contains`](#0x2_linked_table_contains)
-  [Function `length`](#0x2_linked_table_length)
-  [Function `is_empty`](#0x2_linked_table_is_empty)
-  [Function `destroy_empty`](#0x2_linked_table_destroy_empty)
-  [Function `drop`](#0x2_linked_table_drop)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
</code></pre>



<a name="0x2_linked_table_TablePlaceholder"></a>

## Resource `TablePlaceholder`



<pre><code><b>struct</b> <a href="linked_table.md#0x2_linked_table_TablePlaceholder">TablePlaceholder</a> <b>has</b> key
</code></pre>



<a name="0x2_linked_table_LinkedTable"></a>

## Resource `LinkedTable`



<pre><code><b>struct</b> <a href="linked_table.md#0x2_linked_table_LinkedTable">LinkedTable</a>&lt;K: <b>copy</b>, drop, store, V: store&gt; <b>has</b> store, key
</code></pre>



<a name="0x2_linked_table_Node"></a>

## Struct `Node`



<pre><code><b>struct</b> <a href="linked_table.md#0x2_linked_table_Node">Node</a>&lt;K: <b>copy</b>, drop, store, V: store&gt; <b>has</b> store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_linked_table_ErrorTableIsEmpty"></a>



<pre><code><b>const</b> <a href="linked_table.md#0x2_linked_table_ErrorTableIsEmpty">ErrorTableIsEmpty</a>: u64 = 1;
</code></pre>



<a name="0x2_linked_table_ErrorTableNotEmpty"></a>



<pre><code><b>const</b> <a href="linked_table.md#0x2_linked_table_ErrorTableNotEmpty">ErrorTableNotEmpty</a>: u64 = 0;
</code></pre>



<a name="0x2_linked_table_new"></a>

## Function `new`

Creates a new, empty table


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_new">new</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(): <a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;
</code></pre>



<a name="0x2_linked_table_front"></a>

## Function `front`

Returns the key for the first element in the table, or None if the table is empty


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_front">front</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;): &<a href="_Option">option::Option</a>&lt;K&gt;
</code></pre>



<a name="0x2_linked_table_back"></a>

## Function `back`

Returns the key for the last element in the table, or None if the table is empty


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_back">back</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;): &<a href="_Option">option::Option</a>&lt;K&gt;
</code></pre>



<a name="0x2_linked_table_push_front"></a>

## Function `push_front`

Inserts a key-value pair at the front of the table, i.e. the newly inserted pair will be
the first element in the table
Aborts with if the table already has an entry with
that key <code>k: K</code>.


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_push_front">push_front</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;, k: K, value: V)
</code></pre>



<a name="0x2_linked_table_push_back"></a>

## Function `push_back`

Inserts a key-value pair at the back of the table, i.e. the newly inserted pair will be
the last element in the table
Aborts if the table already has an entry with
that key <code>k: K</code>.


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_push_back">push_back</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;, k: K, value: V)
</code></pre>



<a name="0x2_linked_table_borrow"></a>

## Function `borrow`

Immutable borrows the value associated with the key in the table <code><a href="table.md#0x2_table">table</a>: &<a href="linked_table.md#0x2_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code>.
Aborts if the table does not have an entry with
that key <code>k: K</code>.


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_borrow">borrow</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;, k: K): &V
</code></pre>



<a name="0x2_linked_table_borrow_mut"></a>

## Function `borrow_mut`

Mutably borrows the value associated with the key in the table <code><a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="linked_table.md#0x2_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code>.
Aborts if the table does not have an entry with
that key <code>k: K</code>.


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_borrow_mut">borrow_mut</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;, k: K): &<b>mut</b> V
</code></pre>



<a name="0x2_linked_table_prev"></a>

## Function `prev`

Borrows the key for the previous entry of the specified key <code>k: K</code> in the table
<code><a href="table.md#0x2_table">table</a>: &<a href="linked_table.md#0x2_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code>. Returns None if the entry does not have a predecessor.
Aborts if the table does not have an entry with
that key <code>k: K</code>


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_prev">prev</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;, k: K): &<a href="_Option">option::Option</a>&lt;K&gt;
</code></pre>



<a name="0x2_linked_table_next"></a>

## Function `next`

Borrows the key for the next entry of the specified key <code>k: K</code> in the table
<code><a href="table.md#0x2_table">table</a>: &<a href="linked_table.md#0x2_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code>. Returns None if the entry does not have a predecessor.
Aborts if the table does not have an entry with
that key <code>k: K</code>


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_next">next</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;, k: K): &<a href="_Option">option::Option</a>&lt;K&gt;
</code></pre>



<a name="0x2_linked_table_remove"></a>

## Function `remove`

Removes the key-value pair in the table <code><a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="linked_table.md#0x2_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code> and returns the value.
This splices the element out of the ordering.
Aborts if the table does not have an entry with
that key <code>k: K</code>. Note: this is also what happens when the table is empty.


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_remove">remove</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;, k: K): V
</code></pre>



<a name="0x2_linked_table_pop_front"></a>

## Function `pop_front`

Removes the front of the table <code><a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="linked_table.md#0x2_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code> and returns the value.
Aborts with <code>ETableIsEmpty</code> if the table is empty


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_pop_front">pop_front</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;): (K, V)
</code></pre>



<a name="0x2_linked_table_pop_back"></a>

## Function `pop_back`

Removes the back of the table <code><a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="linked_table.md#0x2_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code> and returns the value.
Aborts with <code>ETableIsEmpty</code> if the table is empty


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_pop_back">pop_back</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<b>mut</b> <a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;): (K, V)
</code></pre>



<a name="0x2_linked_table_contains"></a>

## Function `contains`

Returns true iff there is a value associated with the key <code>k: K</code> in table
<code><a href="table.md#0x2_table">table</a>: &<a href="linked_table.md#0x2_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code>


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_contains">contains</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;, k: K): bool
</code></pre>



<a name="0x2_linked_table_length"></a>

## Function `length`

Returns the size of the table, the number of key-value pairs


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_length">length</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;): u64
</code></pre>



<a name="0x2_linked_table_is_empty"></a>

## Function `is_empty`

Returns true if the table is empty (if <code>length</code> returns <code>0</code>)


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_is_empty">is_empty</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: &<a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;): bool
</code></pre>



<a name="0x2_linked_table_destroy_empty"></a>

## Function `destroy_empty`

Destroys an empty table
Aborts with <code>ETableNotEmpty</code> if the table still contains values


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_destroy_empty">destroy_empty</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="table.md#0x2_table">table</a>: <a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;)
</code></pre>



<a name="0x2_linked_table_drop"></a>

## Function `drop`

Drop a possibly non-empty table.
Usable only if the value type <code>V</code> has the <code>drop</code> ability


<pre><code><b>public</b> <b>fun</b> <a href="linked_table.md#0x2_linked_table_drop">drop</a>&lt;K: <b>copy</b>, drop, store, V: drop, store&gt;(table_obj: <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="linked_table.md#0x2_linked_table_LinkedTable">linked_table::LinkedTable</a>&lt;K, V&gt;&gt;)
</code></pre>
