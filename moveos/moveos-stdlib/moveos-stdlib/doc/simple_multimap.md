
<a name="0x2_simple_multimap"></a>

# Module `0x2::simple_multimap`

A simple map that stores key/value pairs in a vector, and support multi values for one key.


-  [Struct `SimpleMultiMap`](#0x2_simple_multimap_SimpleMultiMap)
-  [Struct `Element`](#0x2_simple_multimap_Element)
-  [Constants](#@Constants_0)
-  [Function `length`](#0x2_simple_multimap_length)
-  [Function `new`](#0x2_simple_multimap_new)
-  [Function `borrow`](#0x2_simple_multimap_borrow)
-  [Function `borrow_mut`](#0x2_simple_multimap_borrow_mut)
-  [Function `borrow_first`](#0x2_simple_multimap_borrow_first)
-  [Function `borrow_first_mut`](#0x2_simple_multimap_borrow_first_mut)
-  [Function `borrow_first_with_default`](#0x2_simple_multimap_borrow_first_with_default)
-  [Function `contains_key`](#0x2_simple_multimap_contains_key)
-  [Function `destroy_empty`](#0x2_simple_multimap_destroy_empty)
-  [Function `drop`](#0x2_simple_multimap_drop)
-  [Function `add`](#0x2_simple_multimap_add)
-  [Function `keys`](#0x2_simple_multimap_keys)
-  [Function `values`](#0x2_simple_multimap_values)
-  [Function `to_vec_pair`](#0x2_simple_multimap_to_vec_pair)
-  [Function `remove`](#0x2_simple_multimap_remove)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x2_simple_multimap_SimpleMultiMap"></a>

## Struct `SimpleMultiMap`



<pre><code><b>struct</b> <a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">SimpleMultiMap</a>&lt;Key, Value&gt; <b>has</b> store
</code></pre>



<a name="0x2_simple_multimap_Element"></a>

## Struct `Element`



<pre><code><b>struct</b> <a href="simple_multimap.md#0x2_simple_multimap_Element">Element</a>&lt;Key, Value&gt; <b>has</b> store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_simple_multimap_ErrorKeyNotFound"></a>

Map key is not found


<pre><code><b>const</b> <a href="simple_multimap.md#0x2_simple_multimap_ErrorKeyNotFound">ErrorKeyNotFound</a>: u64 = 1;
</code></pre>



<a name="0x2_simple_multimap_length"></a>

## Function `length`



<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_length">length</a>&lt;Key: store, Value: store&gt;(map: &<a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;): u64
</code></pre>



<a name="0x2_simple_multimap_new"></a>

## Function `new`

Create an empty SimpleMultiMap.


<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_new">new</a>&lt;Key: store, Value: store&gt;(): <a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;
</code></pre>



<a name="0x2_simple_multimap_borrow"></a>

## Function `borrow`



<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_borrow">borrow</a>&lt;Key: store, Value: store&gt;(map: &<a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;, key: &Key): &<a href="">vector</a>&lt;Value&gt;
</code></pre>



<a name="0x2_simple_multimap_borrow_mut"></a>

## Function `borrow_mut`



<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_borrow_mut">borrow_mut</a>&lt;Key: store, Value: store&gt;(map: &<b>mut</b> <a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;, key: &Key): &<b>mut</b> <a href="">vector</a>&lt;Value&gt;
</code></pre>



<a name="0x2_simple_multimap_borrow_first"></a>

## Function `borrow_first`



<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_borrow_first">borrow_first</a>&lt;Key: store, Value: store&gt;(map: &<a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;, key: &Key): &Value
</code></pre>



<a name="0x2_simple_multimap_borrow_first_mut"></a>

## Function `borrow_first_mut`



<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_borrow_first_mut">borrow_first_mut</a>&lt;Key: store, Value: store&gt;(map: &<b>mut</b> <a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;, key: &Key): &<b>mut</b> Value
</code></pre>



<a name="0x2_simple_multimap_borrow_first_with_default"></a>

## Function `borrow_first_with_default`



<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_borrow_first_with_default">borrow_first_with_default</a>&lt;Key: store, Value: store&gt;(map: &<a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;, key: &Key, default: &Value): &Value
</code></pre>



<a name="0x2_simple_multimap_contains_key"></a>

## Function `contains_key`



<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_contains_key">contains_key</a>&lt;Key: store, Value: store&gt;(map: &<a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;, key: &Key): bool
</code></pre>



<a name="0x2_simple_multimap_destroy_empty"></a>

## Function `destroy_empty`



<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_destroy_empty">destroy_empty</a>&lt;Key: store, Value: store&gt;(map: <a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;)
</code></pre>



<a name="0x2_simple_multimap_drop"></a>

## Function `drop`

Drop all keys and values in the map. This requires keys and values to be dropable.


<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_drop">drop</a>&lt;Key: <b>copy</b>, drop, Value: drop&gt;(map: <a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;)
</code></pre>



<a name="0x2_simple_multimap_add"></a>

## Function `add`



<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_add">add</a>&lt;Key: drop, store, Value: store&gt;(map: &<b>mut</b> <a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;, key: Key, value: Value)
</code></pre>



<a name="0x2_simple_multimap_keys"></a>

## Function `keys`

Return all keys in the map. This requires keys to be copyable.


<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_keys">keys</a>&lt;Key: <b>copy</b>, Value&gt;(map: &<a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;): <a href="">vector</a>&lt;Key&gt;
</code></pre>



<a name="0x2_simple_multimap_values"></a>

## Function `values`

Return all values in the map. This requires values to be copyable.
This function flatten the vector<vector<Value>> to vector<Value>


<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_values">values</a>&lt;Key, Value: <b>copy</b>&gt;(map: &<a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;): <a href="">vector</a>&lt;Value&gt;
</code></pre>



<a name="0x2_simple_multimap_to_vec_pair"></a>

## Function `to_vec_pair`

Transform the map into two vectors with the keys and values respectively
Primarily used to destroy a map
Note: Do not assume the key's order


<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_to_vec_pair">to_vec_pair</a>&lt;Key: store, Value: store&gt;(map: <a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;): (<a href="">vector</a>&lt;Key&gt;, <a href="">vector</a>&lt;<a href="">vector</a>&lt;Value&gt;&gt;)
</code></pre>



<a name="0x2_simple_multimap_remove"></a>

## Function `remove`



<pre><code><b>public</b> <b>fun</b> <a href="simple_multimap.md#0x2_simple_multimap_remove">remove</a>&lt;Key: store, Value: store&gt;(map: &<b>mut</b> <a href="simple_multimap.md#0x2_simple_multimap_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;Key, Value&gt;, key: &Key): (Key, <a href="">vector</a>&lt;Value&gt;)
</code></pre>
