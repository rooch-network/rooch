
<a name="0x2_table_vec"></a>

# Module `0x2::table_vec`

A basic scalable vector library implemented using <code>Table</code>.


-  [Struct `TableVec`](#0x2_table_vec_TableVec)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x2_table_vec_new)
-  [Function `singleton`](#0x2_table_vec_singleton)
-  [Function `length`](#0x2_table_vec_length)
-  [Function `is_empty`](#0x2_table_vec_is_empty)
-  [Function `borrow`](#0x2_table_vec_borrow)
-  [Function `push_back`](#0x2_table_vec_push_back)
-  [Function `borrow_mut`](#0x2_table_vec_borrow_mut)
-  [Function `pop_back`](#0x2_table_vec_pop_back)
-  [Function `destroy_empty`](#0x2_table_vec_destroy_empty)
-  [Function `drop`](#0x2_table_vec_drop)
-  [Function `swap`](#0x2_table_vec_swap)
-  [Function `swap_remove`](#0x2_table_vec_swap_remove)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="table.md#0x2_table">0x2::table</a>;
</code></pre>



<a name="0x2_table_vec_TableVec"></a>

## Struct `TableVec`



<pre><code><b>struct</b> <a href="table_vec.md#0x2_table_vec_TableVec">TableVec</a>&lt;Element: store&gt; <b>has</b> store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_table_vec_ErrorIndexOutOfBound"></a>



<pre><code><b>const</b> <a href="table_vec.md#0x2_table_vec_ErrorIndexOutOfBound">ErrorIndexOutOfBound</a>: u64 = 1;
</code></pre>



<a name="0x2_table_vec_ErrorTableNonEmpty"></a>



<pre><code><b>const</b> <a href="table_vec.md#0x2_table_vec_ErrorTableNonEmpty">ErrorTableNonEmpty</a>: u64 = 2;
</code></pre>



<a name="0x2_table_vec_new"></a>

## Function `new`

Create an empty TableVec.


<pre><code><b>public</b> <b>fun</b> <a href="table_vec.md#0x2_table_vec_new">new</a>&lt;Element: store&gt;(id: <a href="object.md#0x2_object_UID">object::UID</a>): <a href="table_vec.md#0x2_table_vec_TableVec">table_vec::TableVec</a>&lt;Element&gt;
</code></pre>



<a name="0x2_table_vec_singleton"></a>

## Function `singleton`

Return a TableVec of size one containing element <code>e</code>.


<pre><code><b>public</b> <b>fun</b> <a href="table_vec.md#0x2_table_vec_singleton">singleton</a>&lt;Element: store&gt;(id: <a href="object.md#0x2_object_UID">object::UID</a>, e: Element): <a href="table_vec.md#0x2_table_vec_TableVec">table_vec::TableVec</a>&lt;Element&gt;
</code></pre>



<a name="0x2_table_vec_length"></a>

## Function `length`

Return the length of the TableVec.


<pre><code><b>public</b> <b>fun</b> <a href="table_vec.md#0x2_table_vec_length">length</a>&lt;Element: store&gt;(t: &<a href="table_vec.md#0x2_table_vec_TableVec">table_vec::TableVec</a>&lt;Element&gt;): u64
</code></pre>



<a name="0x2_table_vec_is_empty"></a>

## Function `is_empty`

Return if the TableVec is empty or not.


<pre><code><b>public</b> <b>fun</b> <a href="table_vec.md#0x2_table_vec_is_empty">is_empty</a>&lt;Element: store&gt;(t: &<a href="table_vec.md#0x2_table_vec_TableVec">table_vec::TableVec</a>&lt;Element&gt;): bool
</code></pre>



<a name="0x2_table_vec_borrow"></a>

## Function `borrow`

Acquire an immutable reference to the <code>i</code>th element of the TableVec <code>t</code>.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="table_vec.md#0x2_table_vec_borrow">borrow</a>&lt;Element: store&gt;(t: &<a href="table_vec.md#0x2_table_vec_TableVec">table_vec::TableVec</a>&lt;Element&gt;, i: u64): &Element
</code></pre>



<a name="0x2_table_vec_push_back"></a>

## Function `push_back`

Add element <code>e</code> to the end of the TableVec <code>t</code>.


<pre><code><b>public</b> <b>fun</b> <a href="table_vec.md#0x2_table_vec_push_back">push_back</a>&lt;Element: store&gt;(t: &<b>mut</b> <a href="table_vec.md#0x2_table_vec_TableVec">table_vec::TableVec</a>&lt;Element&gt;, e: Element)
</code></pre>



<a name="0x2_table_vec_borrow_mut"></a>

## Function `borrow_mut`

Return a mutable reference to the <code>i</code>th element in the TableVec <code>t</code>.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="table_vec.md#0x2_table_vec_borrow_mut">borrow_mut</a>&lt;Element: store&gt;(t: &<b>mut</b> <a href="table_vec.md#0x2_table_vec_TableVec">table_vec::TableVec</a>&lt;Element&gt;, i: u64): &<b>mut</b> Element
</code></pre>



<a name="0x2_table_vec_pop_back"></a>

## Function `pop_back`

Pop an element from the end of TableVec <code>t</code>.
Aborts if <code>t</code> is empty.


<pre><code><b>public</b> <b>fun</b> <a href="table_vec.md#0x2_table_vec_pop_back">pop_back</a>&lt;Element: store&gt;(t: &<b>mut</b> <a href="table_vec.md#0x2_table_vec_TableVec">table_vec::TableVec</a>&lt;Element&gt;): Element
</code></pre>



<a name="0x2_table_vec_destroy_empty"></a>

## Function `destroy_empty`

Destroy the TableVec <code>t</code>.
Aborts if <code>t</code> is not empty.


<pre><code><b>public</b> <b>fun</b> <a href="table_vec.md#0x2_table_vec_destroy_empty">destroy_empty</a>&lt;Element: store&gt;(t: <a href="table_vec.md#0x2_table_vec_TableVec">table_vec::TableVec</a>&lt;Element&gt;)
</code></pre>



<a name="0x2_table_vec_drop"></a>

## Function `drop`

Drop a possibly non-empty TableVec <code>t</code>.
Usable only if the value type <code>Element</code> has the <code>drop</code> ability


<pre><code><b>public</b> <b>fun</b> <a href="table_vec.md#0x2_table_vec_drop">drop</a>&lt;Element: drop, store&gt;(t: <a href="table_vec.md#0x2_table_vec_TableVec">table_vec::TableVec</a>&lt;Element&gt;)
</code></pre>



<a name="0x2_table_vec_swap"></a>

## Function `swap`

Swaps the elements at the <code>i</code>th and <code>j</code>th indices in the TableVec <code>t</code>.
Aborts if <code>i</code> or <code>j</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="table_vec.md#0x2_table_vec_swap">swap</a>&lt;Element: store&gt;(t: &<b>mut</b> <a href="table_vec.md#0x2_table_vec_TableVec">table_vec::TableVec</a>&lt;Element&gt;, i: u64, j: u64)
</code></pre>



<a name="0x2_table_vec_swap_remove"></a>

## Function `swap_remove`

Swap the <code>i</code>th element of the TableVec <code>t</code> with the last element and then pop the TableVec.
This is O(1), but does not preserve ordering of elements in the TableVec.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="table_vec.md#0x2_table_vec_swap_remove">swap_remove</a>&lt;Element: store&gt;(t: &<b>mut</b> <a href="table_vec.md#0x2_table_vec_TableVec">table_vec::TableVec</a>&lt;Element&gt;, i: u64): Element
</code></pre>
