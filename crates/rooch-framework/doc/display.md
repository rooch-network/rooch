
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
<b>use</b> <a href="">0x2::simple_map</a>;
</code></pre>



<a name="0x3_display_Display"></a>

## Resource `Display`



<pre><code><b>struct</b> <a href="display.md#0x3_display_Display">Display</a> <b>has</b> <b>copy</b>, drop, store, key
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



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_new">new</a>(): <a href="display.md#0x3_display_Display">display::Display</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_new">new</a> (): <a href="display.md#0x3_display_Display">Display</a> {
    <a href="display.md#0x3_display_Display">Display</a> {
        sample_map: <a href="_create">simple_map::create</a>()
    }
}
</code></pre>



</details>

<a name="0x3_display_set"></a>

## Function `set`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_set">set</a>(self: &<b>mut</b> <a href="display.md#0x3_display_Display">display::Display</a>, key: <a href="_String">string::String</a>, value: <a href="_String">string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_set">set</a> (self: &<b>mut</b> <a href="display.md#0x3_display_Display">Display</a>, key: String, value: String) {
    <a href="_add">simple_map::add</a>(&<b>mut</b> self.sample_map, key, value);
}
</code></pre>



</details>

<a name="0x3_display_borrow"></a>

## Function `borrow`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_borrow">borrow</a>(self: &<a href="display.md#0x3_display_Display">display::Display</a>, key: &<a href="_String">string::String</a>): &<a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_borrow">borrow</a> (self: &<a href="display.md#0x3_display_Display">Display</a>, key: &String): &String {
    <a href="_borrow">simple_map::borrow</a>(&self.sample_map, key)
}
</code></pre>



</details>

<a name="0x3_display_borrow_mut"></a>

## Function `borrow_mut`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_borrow_mut">borrow_mut</a>(self: &<b>mut</b> <a href="display.md#0x3_display_Display">display::Display</a>, key: &<a href="_String">string::String</a>): &<b>mut</b> <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_borrow_mut">borrow_mut</a> (self: &<b>mut</b> <a href="display.md#0x3_display_Display">Display</a>, key: &String): &<b>mut</b> String {
    <a href="_borrow_mut">simple_map::borrow_mut</a>(&<b>mut</b> self.sample_map, key)
}
</code></pre>



</details>

<a name="0x3_display_remove"></a>

## Function `remove`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_remove">remove</a>(self: &<b>mut</b> <a href="display.md#0x3_display_Display">display::Display</a>, key: &<a href="_String">string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_remove">remove</a> (self: &<b>mut</b> <a href="display.md#0x3_display_Display">Display</a>, key: &String) {
    <a href="_remove">simple_map::remove</a>(&<b>mut</b> self.sample_map, key);
}
</code></pre>



</details>

<a name="0x3_display_keys"></a>

## Function `keys`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_keys">keys</a>(self: &<a href="display.md#0x3_display_Display">display::Display</a>): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_keys">keys</a> (self: & <a href="display.md#0x3_display_Display">Display</a>): <a href="">vector</a>&lt;String&gt; {
    <a href="_keys">simple_map::keys</a>(& self.sample_map)
}
</code></pre>



</details>

<a name="0x3_display_values"></a>

## Function `values`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_values">values</a>(self: &<a href="display.md#0x3_display_Display">display::Display</a>): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_values">values</a> (self: & <a href="display.md#0x3_display_Display">Display</a>): <a href="">vector</a>&lt;String&gt; {
    <a href="_values">simple_map::values</a>(& self.sample_map)
}
</code></pre>



</details>

<a name="0x3_display_contains_key"></a>

## Function `contains_key`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_contains_key">contains_key</a>(self: &<a href="display.md#0x3_display_Display">display::Display</a>, key: &<a href="_String">string::String</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x3_display_contains_key">contains_key</a> (self: & <a href="display.md#0x3_display_Display">Display</a>, key: &String): bool {
    <a href="_contains_key">simple_map::contains_key</a>(& self.sample_map, key)
}
</code></pre>



</details>
