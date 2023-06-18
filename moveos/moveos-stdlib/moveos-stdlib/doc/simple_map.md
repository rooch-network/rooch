
<a name="0x2_simple_map"></a>

# Module `0x2::simple_map`

Source from https://github.com/aptos-labs/aptos-core/blob/d50af4db34a6929642603c3896a0af17984b3054/aptos-move/framework/aptos-stdlib/sources/simple_map.move
Do some refator because we do not support inline and lambda yet.
This module provides a solution for unsorted maps, that is it has the properties that
1) Keys point to Values
2) Each Key must be unique
3) A Key can be found within O(N) time
4) The keys are unsorted.
5) Adds and removals take O(N) time


-  [Struct `SimpleMap`](#0x2_simple_map_SimpleMap)
-  [Struct `Element`](#0x2_simple_map_Element)
-  [Constants](#@Constants_0)
-  [Function `length`](#0x2_simple_map_length)
-  [Function `create`](#0x2_simple_map_create)
-  [Function `borrow`](#0x2_simple_map_borrow)
-  [Function `borrow_mut`](#0x2_simple_map_borrow_mut)
-  [Function `contains_key`](#0x2_simple_map_contains_key)
-  [Function `destroy_empty`](#0x2_simple_map_destroy_empty)
-  [Function `add`](#0x2_simple_map_add)
-  [Function `upsert`](#0x2_simple_map_upsert)
-  [Function `keys`](#0x2_simple_map_keys)
-  [Function `values`](#0x2_simple_map_values)
-  [Function `to_vec_pair`](#0x2_simple_map_to_vec_pair)
-  [Function `remove`](#0x2_simple_map_remove)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x2_simple_map_SimpleMap"></a>

## Struct `SimpleMap`



<pre><code><b>struct</b> <a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>data: <a href="">vector</a>&lt;<a href="simple_map.md#0x2_simple_map_Element">simple_map::Element</a>&lt;Key, Value&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x2_simple_map_Element"></a>

## Struct `Element`



<pre><code><b>struct</b> <a href="simple_map.md#0x2_simple_map_Element">Element</a>&lt;Key, Value&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>key: Key</code>
</dt>
<dd>

</dd>
<dt>
<code>value: Value</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x2_simple_map_EKEY_ALREADY_EXISTS"></a>

Map key already exists


<pre><code><b>const</b> <a href="simple_map.md#0x2_simple_map_EKEY_ALREADY_EXISTS">EKEY_ALREADY_EXISTS</a>: u64 = 1;
</code></pre>



<a name="0x2_simple_map_EKEY_NOT_FOUND"></a>

Map key is not found


<pre><code><b>const</b> <a href="simple_map.md#0x2_simple_map_EKEY_NOT_FOUND">EKEY_NOT_FOUND</a>: u64 = 2;
</code></pre>



<a name="0x2_simple_map_length"></a>

## Function `length`



<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_length">length</a>&lt;Key: store, Value: store&gt;(map: &<a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;Key, Value&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_length">length</a>&lt;Key: store, Value: store&gt;(map: &<a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt;): u64 {
    <a href="_length">vector::length</a>(&map.data)
}
</code></pre>



</details>

<a name="0x2_simple_map_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_create">create</a>&lt;Key: store, Value: store&gt;(): <a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;Key, Value&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_create">create</a>&lt;Key: store, Value: store&gt;(): <a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt; {
    <a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a> {
        data: <a href="_empty">vector::empty</a>(),
    }
}
</code></pre>



</details>

<a name="0x2_simple_map_borrow"></a>

## Function `borrow`



<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_borrow">borrow</a>&lt;Key: store, Value: store&gt;(map: &<a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;Key, Value&gt;, key: &Key): &Value
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_borrow">borrow</a>&lt;Key: store, Value: store&gt;(
    map: &<a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt;,
    key: &Key,
): &Value {
    <b>let</b> maybe_idx = <a href="simple_map.md#0x2_simple_map_find">find</a>(map, key);
    <b>assert</b>!(<a href="_is_some">option::is_some</a>(&maybe_idx), <a href="_invalid_argument">error::invalid_argument</a>(<a href="simple_map.md#0x2_simple_map_EKEY_NOT_FOUND">EKEY_NOT_FOUND</a>));
    <b>let</b> idx = <a href="_extract">option::extract</a>(&<b>mut</b> maybe_idx);
    &<a href="_borrow">vector::borrow</a>(&map.data, idx).value
}
</code></pre>



</details>

<a name="0x2_simple_map_borrow_mut"></a>

## Function `borrow_mut`



<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_borrow_mut">borrow_mut</a>&lt;Key: store, Value: store&gt;(map: &<b>mut</b> <a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;Key, Value&gt;, key: &Key): &<b>mut</b> Value
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_borrow_mut">borrow_mut</a>&lt;Key: store, Value: store&gt;(
    map: &<b>mut</b> <a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt;,
    key: &Key,
): &<b>mut</b> Value {
    <b>let</b> maybe_idx = <a href="simple_map.md#0x2_simple_map_find">find</a>(map, key);
    <b>assert</b>!(<a href="_is_some">option::is_some</a>(&maybe_idx), <a href="_invalid_argument">error::invalid_argument</a>(<a href="simple_map.md#0x2_simple_map_EKEY_NOT_FOUND">EKEY_NOT_FOUND</a>));
    <b>let</b> idx = <a href="_extract">option::extract</a>(&<b>mut</b> maybe_idx);
    &<b>mut</b> <a href="_borrow_mut">vector::borrow_mut</a>(&<b>mut</b> map.data, idx).value
}
</code></pre>



</details>

<a name="0x2_simple_map_contains_key"></a>

## Function `contains_key`



<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_contains_key">contains_key</a>&lt;Key: store, Value: store&gt;(map: &<a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;Key, Value&gt;, key: &Key): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_contains_key">contains_key</a>&lt;Key: store, Value: store&gt;(
    map: &<a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt;,
    key: &Key,
): bool {
    <b>let</b> maybe_idx = <a href="simple_map.md#0x2_simple_map_find">find</a>(map, key);
    <a href="_is_some">option::is_some</a>(&maybe_idx)
}
</code></pre>



</details>

<a name="0x2_simple_map_destroy_empty"></a>

## Function `destroy_empty`



<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_destroy_empty">destroy_empty</a>&lt;Key: store, Value: store&gt;(map: <a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;Key, Value&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_destroy_empty">destroy_empty</a>&lt;Key: store, Value: store&gt;(map: <a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt;) {
    <b>let</b> <a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a> { data } = map;
    <a href="_destroy_empty">vector::destroy_empty</a>(data);
}
</code></pre>



</details>

<a name="0x2_simple_map_add"></a>

## Function `add`



<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_add">add</a>&lt;Key: store, Value: store&gt;(map: &<b>mut</b> <a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;Key, Value&gt;, key: Key, value: Value)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_add">add</a>&lt;Key: store, Value: store&gt;(
    map: &<b>mut</b> <a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt;,
    key: Key,
    value: Value,
) {
    <b>let</b> maybe_idx = <a href="simple_map.md#0x2_simple_map_find">find</a>(map, &key);
    <b>assert</b>!(<a href="_is_none">option::is_none</a>(&maybe_idx), <a href="_invalid_argument">error::invalid_argument</a>(<a href="simple_map.md#0x2_simple_map_EKEY_ALREADY_EXISTS">EKEY_ALREADY_EXISTS</a>));

    <a href="_push_back">vector::push_back</a>(&<b>mut</b> map.data, <a href="simple_map.md#0x2_simple_map_Element">Element</a> { key, value });
}
</code></pre>



</details>

<a name="0x2_simple_map_upsert"></a>

## Function `upsert`

Insert key/value pair or update an existing key to a new value


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_upsert">upsert</a>&lt;Key: store, Value: store&gt;(map: &<b>mut</b> <a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;Key, Value&gt;, key: Key, value: Value): (<a href="_Option">option::Option</a>&lt;Key&gt;, <a href="_Option">option::Option</a>&lt;Value&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_upsert">upsert</a>&lt;Key: store, Value: store&gt;(
    map: &<b>mut</b> <a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt;,
    key: Key,
    value: Value
): (std::option::Option&lt;Key&gt;, std::option::Option&lt;Value&gt;) {
    <b>let</b> data = &<b>mut</b> map.data;
    <b>let</b> len = <a href="_length">vector::length</a>(data);
    <b>let</b> i = 0;
    <b>while</b> (i &lt; len) {
        <b>let</b> element = <a href="_borrow">vector::borrow</a>(data, i);
        <b>if</b> (&element.key == &key) {
            <a href="_push_back">vector::push_back</a>(data, <a href="simple_map.md#0x2_simple_map_Element">Element</a> { key, value});
            <a href="_swap">vector::swap</a>(data, i, len);
            <b>let</b> <a href="simple_map.md#0x2_simple_map_Element">Element</a> { key, value } = <a href="_pop_back">vector::pop_back</a>(data);
            <b>return</b> (std::option::some(key), std::option::some(value))
        };
        i = i + 1;
    };
    <a href="_push_back">vector::push_back</a>(&<b>mut</b> map.data, <a href="simple_map.md#0x2_simple_map_Element">Element</a> { key, value });
    (std::option::none(), std::option::none())
}
</code></pre>



</details>

<a name="0x2_simple_map_keys"></a>

## Function `keys`

Return all keys in the map. This requires keys to be copyable.


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_keys">keys</a>&lt;Key: <b>copy</b>, Value&gt;(map: &<a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;Key, Value&gt;): <a href="">vector</a>&lt;Key&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_keys">keys</a>&lt;Key: <b>copy</b>, Value&gt;(map: &<a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt;): <a href="">vector</a>&lt;Key&gt; {
    <b>let</b> i = 0;
    <b>let</b> keys: <a href="">vector</a>&lt;Key&gt; = <a href="_empty">vector::empty</a>();
    <b>let</b> len = <a href="_length">vector::length</a>(&map.data);
    <b>while</b> (i &lt; len) {
        <b>let</b> e = <a href="_borrow">vector::borrow</a>(&map.data, i);
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> keys, e.key);
        i = i + 1;
    };
    keys
}
</code></pre>



</details>

<a name="0x2_simple_map_values"></a>

## Function `values`

Return all values in the map. This requires values to be copyable.


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_values">values</a>&lt;Key, Value: <b>copy</b>&gt;(map: &<a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;Key, Value&gt;): <a href="">vector</a>&lt;Value&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_values">values</a>&lt;Key, Value: <b>copy</b>&gt;(map: &<a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt;): <a href="">vector</a>&lt;Value&gt; {
    <b>let</b> i = 0;
    <b>let</b> values: <a href="">vector</a>&lt;Value&gt; = <a href="_empty">vector::empty</a>();
    <b>let</b> len = <a href="_length">vector::length</a>(&map.data);
    <b>while</b> (i &lt; len) {
        <b>let</b> e = <a href="_borrow">vector::borrow</a>(&map.data, i);
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> values, e.value);
        i = i + 1;
    };
    values
}
</code></pre>



</details>

<a name="0x2_simple_map_to_vec_pair"></a>

## Function `to_vec_pair`

Transform the map into two vectors with the keys and values respectively
Primarily used to destroy a map


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_to_vec_pair">to_vec_pair</a>&lt;Key: store, Value: store&gt;(map: <a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;Key, Value&gt;): (<a href="">vector</a>&lt;Key&gt;, <a href="">vector</a>&lt;Value&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_to_vec_pair">to_vec_pair</a>&lt;Key: store, Value: store&gt;(
    map: <a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt;): (<a href="">vector</a>&lt;Key&gt;, <a href="">vector</a>&lt;Value&gt;) {
    <b>let</b> keys: <a href="">vector</a>&lt;Key&gt; = <a href="_empty">vector::empty</a>();
    <b>let</b> values: <a href="">vector</a>&lt;Value&gt; = <a href="_empty">vector::empty</a>();
    <b>let</b> <a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a> { data } = map;
    <b>let</b> i = 0;
    <b>let</b> len = <a href="_length">vector::length</a>(&data);
    <b>while</b> (i &lt; len) {
        <b>let</b> e = <a href="_pop_back">vector::pop_back</a>(&<b>mut</b> data);
        <b>let</b> <a href="simple_map.md#0x2_simple_map_Element">Element</a> { key, value } = e; <a href="_push_back">vector::push_back</a>(&<b>mut</b> keys, key); <a href="_push_back">vector::push_back</a>(&<b>mut</b> values, value);
        i = i + 1;
    };
    <a href="_destroy_empty">vector::destroy_empty</a>(data);
    (keys, values)
}
</code></pre>



</details>

<a name="0x2_simple_map_remove"></a>

## Function `remove`



<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_remove">remove</a>&lt;Key: store, Value: store&gt;(map: &<b>mut</b> <a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;Key, Value&gt;, key: &Key): (Key, Value)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="simple_map.md#0x2_simple_map_remove">remove</a>&lt;Key: store, Value: store&gt;(
    map: &<b>mut</b> <a href="simple_map.md#0x2_simple_map_SimpleMap">SimpleMap</a>&lt;Key, Value&gt;,
    key: &Key,
): (Key, Value) {
    <b>let</b> maybe_idx = <a href="simple_map.md#0x2_simple_map_find">find</a>(map, key);
    <b>assert</b>!(<a href="_is_some">option::is_some</a>(&maybe_idx), <a href="_invalid_argument">error::invalid_argument</a>(<a href="simple_map.md#0x2_simple_map_EKEY_NOT_FOUND">EKEY_NOT_FOUND</a>));
    <b>let</b> placement = <a href="_extract">option::extract</a>(&<b>mut</b> maybe_idx);
    <b>let</b> <a href="simple_map.md#0x2_simple_map_Element">Element</a> { key, value } = <a href="_swap_remove">vector::swap_remove</a>(&<b>mut</b> map.data, placement);
    (key, value)
}
</code></pre>



</details>
