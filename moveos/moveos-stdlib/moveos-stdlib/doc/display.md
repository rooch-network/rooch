
<a name="0x2_display"></a>

# Module `0x2::display`



-  [Resource `Display`](#0x2_display_Display)
-  [Function `resource_display`](#0x2_display_resource_display)
-  [Function `object_display`](#0x2_display_object_display)
-  [Function `set_value`](#0x2_display_set_value)
-  [Function `borrow_value`](#0x2_display_borrow_value)
-  [Function `borrow_mut_value`](#0x2_display_borrow_mut_value)
-  [Function `remove_value`](#0x2_display_remove_value)
-  [Function `keys`](#0x2_display_keys)
-  [Function `values`](#0x2_display_values)
-  [Function `contains_key`](#0x2_display_contains_key)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="context.md#0x2_context">0x2::context</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="simple_map.md#0x2_simple_map">0x2::simple_map</a>;
</code></pre>



<a name="0x2_display_Display"></a>

## Resource `Display`

Display<T> is a singleton object
It is used to define the display of the <code>T</code>
The Display Object is permanent, can not be deleted after created.


<pre><code><b>struct</b> <a href="display.md#0x2_display_Display">Display</a>&lt;T&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>sample_map: <a href="simple_map.md#0x2_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x2_display_resource_display"></a>

## Function `resource_display`

Create or borrow_mut Display object for resource <code>T</code>
Only the module of <code>T</code> can call this function.


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_resource_display">resource_display</a>&lt;T: key&gt;(ctx: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_resource_display">resource_display</a>&lt;T: key&gt;(ctx: &<b>mut</b> Context): &<b>mut</b> Object&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;T&gt;&gt; {
    <b>let</b> obj = <a href="context.md#0x2_context_new_singleton">context::new_singleton</a>(ctx, <a href="display.md#0x2_display_Display">Display</a>&lt;T&gt; {
        sample_map: <a href="simple_map.md#0x2_simple_map_create">simple_map::create</a>()
    });
    <a href="object.md#0x2_object_to_permanent">object::to_permanent</a>(obj);
    <a href="context.md#0x2_context_borrow_mut_singleton">context::borrow_mut_singleton</a>&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;T&gt;&gt;(ctx)
}
</code></pre>



</details>

<a name="0x2_display_object_display"></a>

## Function `object_display`

Create or borrow_mut Display object for <code>Object&lt;T&gt;</code>
Only the module of <code>T</code> can call this function.


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_object_display">object_display</a>&lt;T: key&gt;(ctx: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_object_display">object_display</a>&lt;T: key&gt;(ctx: &<b>mut</b> Context): &<b>mut</b> Object&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;Object&lt;T&gt;&gt;&gt; {
    <b>if</b> (<a href="context.md#0x2_context_exist_singleton">context::exist_singleton</a>&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;Object&lt;T&gt;&gt;&gt;(ctx)) {
        <a href="context.md#0x2_context_borrow_mut_singleton">context::borrow_mut_singleton</a>&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;Object&lt;T&gt;&gt;&gt;(ctx)
    }<b>else</b>{
        <b>let</b> obj = <a href="context.md#0x2_context_new_singleton">context::new_singleton</a>(ctx, <a href="display.md#0x2_display_Display">Display</a>&lt;Object&lt;T&gt;&gt; {
            sample_map: <a href="simple_map.md#0x2_simple_map_create">simple_map::create</a>()
        });
        <a href="object.md#0x2_object_to_permanent">object::to_permanent</a>(obj);
        <a href="context.md#0x2_context_borrow_mut_singleton">context::borrow_mut_singleton</a>&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;Object&lt;T&gt;&gt;&gt;(ctx)
    }
}
</code></pre>



</details>

<a name="0x2_display_set_value"></a>

## Function `set_value`

Set the key-value pair for the display object
If the key already exists, the value will be updated, otherwise a new key-value pair will be created.


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_set_value">set_value</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;, key: <a href="_String">string::String</a>, value: <a href="_String">string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_set_value">set_value</a>&lt;T&gt;(self: &<b>mut</b> Object&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;T&gt;&gt;, key: String, value: String) {
    <b>let</b> display_ref = <a href="object.md#0x2_object_borrow_mut">object::borrow_mut</a>(self);
    <a href="simple_map.md#0x2_simple_map_upsert">simple_map::upsert</a>(&<b>mut</b> display_ref.sample_map, key, value);
}
</code></pre>



</details>

<a name="0x2_display_borrow_value"></a>

## Function `borrow_value`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_borrow_value">borrow_value</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;, key: &<a href="_String">string::String</a>): &<a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_borrow_value">borrow_value</a>&lt;T&gt;(self: & Object&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;T&gt;&gt; , key: &String): &String {
    <b>let</b> display_ref = <a href="object.md#0x2_object_borrow">object::borrow</a>(self);
    <a href="simple_map.md#0x2_simple_map_borrow">simple_map::borrow</a>(&display_ref.sample_map, key)
}
</code></pre>



</details>

<a name="0x2_display_borrow_mut_value"></a>

## Function `borrow_mut_value`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_borrow_mut_value">borrow_mut_value</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;, key: &<a href="_String">string::String</a>): &<b>mut</b> <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_borrow_mut_value">borrow_mut_value</a>&lt;T&gt;(self: &<b>mut</b> Object&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;T&gt;&gt;, key: &String): &<b>mut</b> String {
    <b>let</b> display_ref = <a href="object.md#0x2_object_borrow_mut">object::borrow_mut</a>(self);
    <a href="simple_map.md#0x2_simple_map_borrow_mut">simple_map::borrow_mut</a>(&<b>mut</b> display_ref.sample_map, key)
}
</code></pre>



</details>

<a name="0x2_display_remove_value"></a>

## Function `remove_value`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_remove_value">remove_value</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;, key: &<a href="_String">string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_remove_value">remove_value</a>&lt;T&gt;(self: &<b>mut</b> Object&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;T&gt;&gt;, key: &String) {
    <b>let</b> display_ref = <a href="object.md#0x2_object_borrow_mut">object::borrow_mut</a>(self);
    <a href="simple_map.md#0x2_simple_map_remove">simple_map::remove</a>(&<b>mut</b> display_ref.sample_map, key);
}
</code></pre>



</details>

<a name="0x2_display_keys"></a>

## Function `keys`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_keys">keys</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_keys">keys</a>&lt;T&gt;(self: & Object&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;T&gt;&gt;): <a href="">vector</a>&lt;String&gt; {
    <b>let</b> display_ref = <a href="object.md#0x2_object_borrow">object::borrow</a>(self);
    <a href="simple_map.md#0x2_simple_map_keys">simple_map::keys</a>(& display_ref.sample_map)
}
</code></pre>



</details>

<a name="0x2_display_values"></a>

## Function `values`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_values">values</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_values">values</a>&lt;T&gt;(self: & Object&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;T&gt;&gt;): <a href="">vector</a>&lt;String&gt; {
    <b>let</b> display_ref = <a href="object.md#0x2_object_borrow">object::borrow</a>(self);
    <a href="simple_map.md#0x2_simple_map_values">simple_map::values</a>(& display_ref.sample_map)
}
</code></pre>



</details>

<a name="0x2_display_contains_key"></a>

## Function `contains_key`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_contains_key">contains_key</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;, key: &<a href="_String">string::String</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_contains_key">contains_key</a>&lt;T&gt;(self: & Object&lt;<a href="display.md#0x2_display_Display">Display</a>&lt;T&gt;&gt;, key: &String): bool {
    <b>let</b> display_ref = <a href="object.md#0x2_object_borrow">object::borrow</a>(self);
    <a href="simple_map.md#0x2_simple_map_contains_key">simple_map::contains_key</a>(& display_ref.sample_map, key)
}
</code></pre>



</details>
