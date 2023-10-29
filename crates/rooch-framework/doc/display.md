
<a name="0x3_display"></a>

# Module `0x3::display`



-  [Resource `Display`](#0x3_display_Display)
-  [Function `new`](#0x3_display_new)
-  [Function `set`](#0x3_display_set)
-  [Function `borrow`](#0x3_display_borrow)
-  [Function `borrow_mut`](#0x3_display_borrow_mut)
-  [Function `remove`](#0x3_display_remove)
-  [Function `keys`](#0x3_display_keys)
-  [Function `values`](#0x3_display_values)
-  [Function `contains_key`](#0x3_display_contains_key)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
</code></pre>



<a name="0x3_display_Display"></a>

## Resource `Display`



<pre><code><b>struct</b> <a href="display.md#0x3_display_Display">Display</a>&lt;T&gt; <b>has</b> <b>copy</b>, drop, store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>sample_map: <a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_display_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_new">new</a>&lt;T&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="_Object">object::Object</a>&lt;<a href="display.md#0x3_display_Display">display::Display</a>&lt;T&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_new">new</a>&lt;T&gt;(ctx: &<b>mut</b> Context): Object&lt;<a href="display.md#0x3_display_Display">Display</a>&lt;T&gt;&gt; {
    <a href="_new_singleton_object">context::new_singleton_object</a>(ctx, <a href="display.md#0x3_display_Display">Display</a>&lt;T&gt; {
        sample_map: <a href="_create">simple_map::create</a>()
    })
}
</code></pre>



</details>

<a name="0x3_display_set"></a>

## Function `set`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_set">set</a>&lt;T&gt;(self: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="display.md#0x3_display_Display">display::Display</a>&lt;T&gt;&gt;, key: <a href="_String">string::String</a>, value: <a href="_String">string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_set">set</a>&lt;T&gt;(self: &<b>mut</b> Object&lt;<a href="display.md#0x3_display_Display">Display</a>&lt;T&gt;&gt;, key: String, value: String) {
    <b>let</b> display_ref = <a href="_borrow_mut">object::borrow_mut</a>(self);
    <a href="_add">simple_map::add</a>(&<b>mut</b> display_ref.sample_map, key, value);
}
</code></pre>



</details>

<a name="0x3_display_borrow"></a>

## Function `borrow`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_borrow">borrow</a>&lt;T&gt;(self: &<a href="_Object">object::Object</a>&lt;<a href="display.md#0x3_display_Display">display::Display</a>&lt;T&gt;&gt;, key: &<a href="_String">string::String</a>): &<a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_borrow">borrow</a>&lt;T&gt;(self: & Object&lt;<a href="display.md#0x3_display_Display">Display</a>&lt;T&gt;&gt; , key: &String): &String {
    <b>let</b> display_ref = <a href="_borrow">object::borrow</a>(self);
    <a href="_borrow">simple_map::borrow</a>(&display_ref.sample_map, key)
}
</code></pre>



</details>

<a name="0x3_display_borrow_mut"></a>

## Function `borrow_mut`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_borrow_mut">borrow_mut</a>&lt;T&gt;(self: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="display.md#0x3_display_Display">display::Display</a>&lt;T&gt;&gt;, key: &<a href="_String">string::String</a>): &<b>mut</b> <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_borrow_mut">borrow_mut</a>&lt;T&gt;(self: &<b>mut</b> Object&lt;<a href="display.md#0x3_display_Display">Display</a>&lt;T&gt;&gt;, key: &String): &<b>mut</b> String {
    <b>let</b> display_ref = <a href="_borrow_mut">object::borrow_mut</a>(self);
    <a href="_borrow_mut">simple_map::borrow_mut</a>(&<b>mut</b> display_ref.sample_map, key)
}
</code></pre>



</details>

<a name="0x3_display_remove"></a>

## Function `remove`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_remove">remove</a>&lt;T&gt;(self: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="display.md#0x3_display_Display">display::Display</a>&lt;T&gt;&gt;, key: &<a href="_String">string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_remove">remove</a>&lt;T&gt;(self: &<b>mut</b> Object&lt;<a href="display.md#0x3_display_Display">Display</a>&lt;T&gt;&gt;, key: &String) {
    <b>let</b> display_ref = <a href="_borrow_mut">object::borrow_mut</a>(self);
    <a href="_remove">simple_map::remove</a>(&<b>mut</b> display_ref.sample_map, key);
}
</code></pre>



</details>

<a name="0x3_display_keys"></a>

## Function `keys`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_keys">keys</a>&lt;T&gt;(self: &<a href="_Object">object::Object</a>&lt;<a href="display.md#0x3_display_Display">display::Display</a>&lt;T&gt;&gt;): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_keys">keys</a>&lt;T&gt;(self: & Object&lt;<a href="display.md#0x3_display_Display">Display</a>&lt;T&gt;&gt;): <a href="">vector</a>&lt;String&gt; {
    <b>let</b> display_ref = <a href="_borrow">object::borrow</a>(self);
    <a href="_keys">simple_map::keys</a>(& display_ref.sample_map)
}
</code></pre>



</details>

<a name="0x3_display_values"></a>

## Function `values`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_values">values</a>&lt;T&gt;(self: &<a href="_Object">object::Object</a>&lt;<a href="display.md#0x3_display_Display">display::Display</a>&lt;T&gt;&gt;): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_values">values</a>&lt;T&gt;(self: & Object&lt;<a href="display.md#0x3_display_Display">Display</a>&lt;T&gt;&gt;): <a href="">vector</a>&lt;String&gt; {
    <b>let</b> display_ref = <a href="_borrow">object::borrow</a>(self);
    <a href="_values">simple_map::values</a>(& display_ref.sample_map)
}
</code></pre>



</details>

<a name="0x3_display_contains_key"></a>

## Function `contains_key`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_contains_key">contains_key</a>&lt;T&gt;(self: &<a href="_Object">object::Object</a>&lt;<a href="display.md#0x3_display_Display">display::Display</a>&lt;T&gt;&gt;, key: &<a href="_String">string::String</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_contains_key">contains_key</a>&lt;T&gt;(self: & Object&lt;<a href="display.md#0x3_display_Display">Display</a>&lt;T&gt;&gt;, key: &String): bool {
    <b>let</b> display_ref = <a href="_borrow">object::borrow</a>(self);
    <a href="_contains_key">simple_map::contains_key</a>(& display_ref.sample_map, key)
}
</code></pre>



</details>
