
<a name="0x2_object_dynamic_field"></a>

# Module `0x2::object_dynamic_field`



-  [Function `add_field`](#0x2_object_dynamic_field_add_field)
-  [Function `borrow_field`](#0x2_object_dynamic_field_borrow_field)
-  [Function `borrow_field_with_default`](#0x2_object_dynamic_field_borrow_field_with_default)
-  [Function `borrow_mut_field`](#0x2_object_dynamic_field_borrow_mut_field)
-  [Function `borrow_mut_field_with_default`](#0x2_object_dynamic_field_borrow_mut_field_with_default)
-  [Function `upsert_field`](#0x2_object_dynamic_field_upsert_field)
-  [Function `remove_field`](#0x2_object_dynamic_field_remove_field)
-  [Function `contains_field`](#0x2_object_dynamic_field_contains_field)
-  [Function `field_size`](#0x2_object_dynamic_field_field_size)


<pre><code><b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
</code></pre>



<a name="0x2_object_dynamic_field_add_field"></a>

## Function `add_field`

Add a dynamic filed to the object. Aborts if an entry for this
key already exists. The entry itself is not stored in the
table, and cannot be discovered from it.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object_dynamic_field.md#0x2_object_dynamic_field_add_field">add_field</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K, val: V)
</code></pre>



<a name="0x2_object_dynamic_field_borrow_field"></a>

## Function `borrow_field`

Acquire an immutable reference to the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object_dynamic_field.md#0x2_object_dynamic_field_borrow_field">borrow_field</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): &V
</code></pre>



<a name="0x2_object_dynamic_field_borrow_field_with_default"></a>

## Function `borrow_field_with_default`

Acquire an immutable reference to the value which <code>key</code> maps to.
Returns specified default value if there is no entry for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object_dynamic_field.md#0x2_object_dynamic_field_borrow_field_with_default">borrow_field_with_default</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K, default: &V): &V
</code></pre>



<a name="0x2_object_dynamic_field_borrow_mut_field"></a>

## Function `borrow_mut_field`

Acquire a mutable reference to the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object_dynamic_field.md#0x2_object_dynamic_field_borrow_mut_field">borrow_mut_field</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): &<b>mut</b> V
</code></pre>



<a name="0x2_object_dynamic_field_borrow_mut_field_with_default"></a>

## Function `borrow_mut_field_with_default`

Acquire a mutable reference to the value which <code>key</code> maps to.
Insert the pair (<code>key</code>, <code>default</code>) first if there is no entry for <code>key</code>.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object_dynamic_field.md#0x2_object_dynamic_field_borrow_mut_field_with_default">borrow_mut_field_with_default</a>&lt;T: key, K: <b>copy</b>, drop, V: drop&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K, default: V): &<b>mut</b> V
</code></pre>



<a name="0x2_object_dynamic_field_upsert_field"></a>

## Function `upsert_field`

Insert the pair (<code>key</code>, <code>value</code>) if there is no entry for <code>key</code>.
update the value of the entry for <code>key</code> to <code>value</code> otherwise


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object_dynamic_field.md#0x2_object_dynamic_field_upsert_field">upsert_field</a>&lt;T: key, K: <b>copy</b>, drop, V: drop&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K, value: V)
</code></pre>



<a name="0x2_object_dynamic_field_remove_field"></a>

## Function `remove_field`

Remove from <code><a href="table.md#0x2_table">table</a></code> and return the value which <code>key</code> maps to.
Aborts if there is no entry for <code>key</code>.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="object_dynamic_field.md#0x2_object_dynamic_field_remove_field">remove_field</a>&lt;T: key, K: <b>copy</b>, drop, V&gt;(obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): V
</code></pre>



<a name="0x2_object_dynamic_field_contains_field"></a>

## Function `contains_field`

Returns true if <code><a href="table.md#0x2_table">table</a></code> contains an entry for <code>key</code>.


<pre><code><b>public</b> <b>fun</b> <a href="object_dynamic_field.md#0x2_object_dynamic_field_contains_field">contains_field</a>&lt;T: key, K: <b>copy</b>, drop&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;, key: K): bool
</code></pre>



<a name="0x2_object_dynamic_field_field_size"></a>

## Function `field_size`

Returns the size of the table, the number of key-value pairs


<pre><code><b>public</b> <b>fun</b> <a href="object_dynamic_field.md#0x2_object_dynamic_field_field_size">field_size</a>&lt;T: key&gt;(obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;): u64
</code></pre>
