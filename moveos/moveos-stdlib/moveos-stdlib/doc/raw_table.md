
<a name="0x2_raw_table"></a>

# Module `0x2::raw_table`

Raw Key Value table. This is the basic of storage abstraction.
This type table doesn't care about the key and value types. We leave the data type checking to the Native implementation.
This type table if for design internal global storage, so all functions are friend.


-  [Resource `TableInfo`](#0x2_raw_table_TableInfo)
-  [Resource `Box`](#0x2_raw_table_Box)
-  [Constants](#@Constants_0)
-  [Function `add`](#0x2_raw_table_add)
-  [Function `borrow`](#0x2_raw_table_borrow)
-  [Function `borrow_with_default`](#0x2_raw_table_borrow_with_default)
-  [Function `borrow_mut`](#0x2_raw_table_borrow_mut)
-  [Function `borrow_mut_with_default`](#0x2_raw_table_borrow_mut_with_default)
-  [Function `upsert`](#0x2_raw_table_upsert)
-  [Function `remove`](#0x2_raw_table_remove)
-  [Function `contains`](#0x2_raw_table_contains)
-  [Function `destroy_empty`](#0x2_raw_table_destroy_empty)
-  [Function `new_empty_table_info`](#0x2_raw_table_new_empty_table_info)
-  [Function `length`](#0x2_raw_table_length)
-  [Function `unpack`](#0x2_raw_table_unpack)
-  [Function `new_table_handle`](#0x2_raw_table_new_table_handle)


<pre><code><b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_raw_table_TableInfo"></a>

## Resource `TableInfo`

TableInfo is a struct that contains the information of a table, include Table,TypeTable,ObjectStorage.


<pre><code><b>struct</b> <a href="raw_table.md#0x2_raw_table_TableInfo">TableInfo</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>state_root: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>length: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x2_raw_table_Box"></a>

## Resource `Box`

Wrapper for values. Required for making values appear as resources in the implementation.
Because the GlobalValue in MoveVM must be a resource.


<pre><code><b>struct</b> <a href="raw_table.md#0x2_raw_table_Box">Box</a>&lt;V&gt; <b>has</b> drop, store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>val: V</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x2_raw_table_SparseMerklePlaceHolderHash"></a>



<pre><code><b>const</b> <a href="raw_table.md#0x2_raw_table_SparseMerklePlaceHolderHash">SparseMerklePlaceHolderHash</a>: <b>address</b> = 5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000;
</code></pre>



<a name="0x2_raw_table_add"></a>

## Function `add`

Add a new entry to the table. Aborts if an entry for this
key already exists. The entry itself is not stored in the
table, and cannot be discovered from it.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_add">add</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: &<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K, val: V)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_add">add</a>&lt;K: <b>copy</b> + drop, V&gt;(table_handle: &ObjectID, key: K, val: V) {
    <a href="raw_table.md#0x2_raw_table_add_box">add_box</a>&lt;K, V, <a href="raw_table.md#0x2_raw_table_Box">Box</a>&lt;V&gt;&gt;(table_handle, key, <a href="raw_table.md#0x2_raw_table_Box">Box</a> {val} )
}
</code></pre>



</details>

<a name="0x2_raw_table_borrow"></a>

## Function `borrow`

Acquire an immutable reference to the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_borrow">borrow</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: &<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K): &V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_borrow">borrow</a>&lt;K: <b>copy</b> + drop, V&gt;(table_handle: &ObjectID, key: K): &V {
    &<a href="raw_table.md#0x2_raw_table_borrow_box">borrow_box</a>&lt;K, V, <a href="raw_table.md#0x2_raw_table_Box">Box</a>&lt;V&gt;&gt;(table_handle, key).val
}
</code></pre>



</details>

<a name="0x2_raw_table_borrow_with_default"></a>

## Function `borrow_with_default`

Acquire an immutable reference to the value which <code>key</code> maps to.
Returns specified default value if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_borrow_with_default">borrow_with_default</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: &<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K, default: &V): &V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_borrow_with_default">borrow_with_default</a>&lt;K: <b>copy</b> + drop, V&gt;(table_handle: &ObjectID, key: K, default: &V): &V {
    <b>if</b> (!<a href="raw_table.md#0x2_raw_table_contains">contains</a>&lt;K&gt;(table_handle, <b>copy</b> key)) {
        default
    } <b>else</b> {
        <a href="raw_table.md#0x2_raw_table_borrow">borrow</a>(table_handle, <b>copy</b> key)
    }
}
</code></pre>



</details>

<a name="0x2_raw_table_borrow_mut"></a>

## Function `borrow_mut`

Acquire a mutable reference to the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_borrow_mut">borrow_mut</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: &<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K): &<b>mut</b> V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_borrow_mut">borrow_mut</a>&lt;K: <b>copy</b> + drop, V&gt;(table_handle: &ObjectID, key: K): &<b>mut</b> V {
    &<b>mut</b> <a href="raw_table.md#0x2_raw_table_borrow_box_mut">borrow_box_mut</a>&lt;K, V, <a href="raw_table.md#0x2_raw_table_Box">Box</a>&lt;V&gt;&gt;(table_handle, key).val
}
</code></pre>



</details>

<a name="0x2_raw_table_borrow_mut_with_default"></a>

## Function `borrow_mut_with_default`

Acquire a mutable reference to the value which <code>key</code> maps to.
Insert the pair (<code>key</code>, <code>default</code>) first if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_borrow_mut_with_default">borrow_mut_with_default</a>&lt;K: <b>copy</b>, drop, V: drop&gt;(table_handle: &<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K, default: V): &<b>mut</b> V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_borrow_mut_with_default">borrow_mut_with_default</a>&lt;K: <b>copy</b> + drop, V: drop&gt;(table_handle: &ObjectID, key: K, default: V): &<b>mut</b> V {
    <b>if</b> (!<a href="raw_table.md#0x2_raw_table_contains">contains</a>&lt;K&gt;(table_handle, <b>copy</b> key)) {
        <a href="raw_table.md#0x2_raw_table_add">add</a>(table_handle, <b>copy</b> key, default)
    };
    <a href="raw_table.md#0x2_raw_table_borrow_mut">borrow_mut</a>(table_handle, key)
}
</code></pre>



</details>

<a name="0x2_raw_table_upsert"></a>

## Function `upsert`

Insert the pair (<code>key</code>, <code>value</code>) if there is no entry for <code>key</code>.
update the value of the entry for <code>key</code> to <code>value</code> otherwise


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_upsert">upsert</a>&lt;K: <b>copy</b>, drop, V: drop&gt;(table_handle: &<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K, value: V)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_upsert">upsert</a>&lt;K: <b>copy</b> + drop, V: drop&gt;(table_handle: &ObjectID, key: K, value: V) {
    <b>if</b> (!<a href="raw_table.md#0x2_raw_table_contains">contains</a>&lt;K&gt;(table_handle, <b>copy</b> key)) {
        <a href="raw_table.md#0x2_raw_table_add">add</a>(table_handle, <b>copy</b> key, value)
    } <b>else</b> {
        <b>let</b> ref = <a href="raw_table.md#0x2_raw_table_borrow_mut">borrow_mut</a>(table_handle, key);
        *ref = value;
    };
}
</code></pre>



</details>

<a name="0x2_raw_table_remove"></a>

## Function `remove`

Remove from <code><a href="table.md#0x2_table">table</a></code> and return the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_remove">remove</a>&lt;K: <b>copy</b>, drop, V&gt;(table_handle: &<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K): V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_remove">remove</a>&lt;K: <b>copy</b> + drop, V&gt;(table_handle: &ObjectID, key: K): V {
    <b>let</b> <a href="raw_table.md#0x2_raw_table_Box">Box</a> { val } = <a href="raw_table.md#0x2_raw_table_remove_box">remove_box</a>&lt;K, V, <a href="raw_table.md#0x2_raw_table_Box">Box</a>&lt;V&gt;&gt;(table_handle, key);
    val
}
</code></pre>



</details>

<a name="0x2_raw_table_contains"></a>

## Function `contains`

Returns true if <code><a href="table.md#0x2_table">table</a></code> contains an entry for <code>key</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_contains">contains</a>&lt;K: <b>copy</b>, drop&gt;(table_handle: &<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, key: K): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_contains">contains</a>&lt;K: <b>copy</b> + drop&gt;(table_handle: &ObjectID, key: K): bool {
    <a href="raw_table.md#0x2_raw_table_contains_box">contains_box</a>&lt;K&gt;(table_handle, key)
}
</code></pre>



</details>

<a name="0x2_raw_table_destroy_empty"></a>

## Function `destroy_empty`

Destroy a table. The table must be empty to succeed.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_destroy_empty">destroy_empty</a>(table_handle: &<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_destroy_empty">destroy_empty</a>(table_handle: &ObjectID) {
    <a href="raw_table.md#0x2_raw_table_destroy_empty_box">destroy_empty_box</a>(table_handle)
}
</code></pre>



</details>

<a name="0x2_raw_table_new_empty_table_info"></a>

## Function `new_empty_table_info`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_new_empty_table_info">new_empty_table_info</a>(): <a href="raw_table.md#0x2_raw_table_TableInfo">raw_table::TableInfo</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_new_empty_table_info">new_empty_table_info</a>(): <a href="raw_table.md#0x2_raw_table_TableInfo">TableInfo</a> {
    <a href="raw_table.md#0x2_raw_table_TableInfo">TableInfo</a> {
        state_root: <a href="raw_table.md#0x2_raw_table_SparseMerklePlaceHolderHash">SparseMerklePlaceHolderHash</a>,
        length: 0u64,
    }
}
</code></pre>



</details>

<a name="0x2_raw_table_length"></a>

## Function `length`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_length">length</a>(self: &<a href="raw_table.md#0x2_raw_table_TableInfo">raw_table::TableInfo</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_length">length</a>(self: &<a href="raw_table.md#0x2_raw_table_TableInfo">TableInfo</a>) : u64{
    self.length
}
</code></pre>



</details>

<a name="0x2_raw_table_unpack"></a>

## Function `unpack`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_unpack">unpack</a>(self: <a href="raw_table.md#0x2_raw_table_TableInfo">raw_table::TableInfo</a>): (<b>address</b>, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_unpack">unpack</a>(self: <a href="raw_table.md#0x2_raw_table_TableInfo">TableInfo</a>) : (<b>address</b>, u64){
    <b>let</b> <a href="raw_table.md#0x2_raw_table_TableInfo">TableInfo</a> { state_root, length } = self;
    (state_root, length)
}
</code></pre>



</details>

<a name="0x2_raw_table_new_table_handle"></a>

## Function `new_table_handle`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_new_table_handle">new_table_handle</a>(ctx: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="raw_table.md#0x2_raw_table_new_table_handle">new_table_handle</a>(ctx: &<b>mut</b> TxContext): ObjectID {
    <a href="tx_context.md#0x2_tx_context_fresh_object_id">tx_context::fresh_object_id</a>(ctx)
}
</code></pre>



</details>
