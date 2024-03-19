
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
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="simple_map.md#0x2_simple_map">0x2::simple_map</a>;
</code></pre>



<a name="0x2_display_Display"></a>

## Resource `Display`

Display<T> is used to define the display of the <code>T</code>


<pre><code><b>struct</b> <a href="display.md#0x2_display_Display">Display</a>&lt;T&gt; <b>has</b> key
</code></pre>



<a name="0x2_display_resource_display"></a>

## Function `resource_display`

Create or borrow_mut Display object for resource <code>T</code>
Only the module of <code>T</code> can call this function.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="display.md#0x2_display_resource_display">resource_display</a>&lt;T: key&gt;(): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;
</code></pre>



<a name="0x2_display_object_display"></a>

## Function `object_display`

Create or borrow_mut Display object for <code>Object&lt;T&gt;</code>
Only the module of <code>T</code> can call this function.


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="display.md#0x2_display_object_display">object_display</a>&lt;T: key&gt;(): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;<a href="object.md#0x2_object_Object">object::Object</a>&lt;T&gt;&gt;&gt;
</code></pre>



<a name="0x2_display_set_value"></a>

## Function `set_value`

Set the key-value pair for the display object
If the key already exists, the value will be updated, otherwise a new key-value pair will be created.


<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_set_value">set_value</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;, key: <a href="_String">string::String</a>, value: <a href="_String">string::String</a>)
</code></pre>



<a name="0x2_display_borrow_value"></a>

## Function `borrow_value`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_borrow_value">borrow_value</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;, key: &<a href="_String">string::String</a>): &<a href="_String">string::String</a>
</code></pre>



<a name="0x2_display_borrow_mut_value"></a>

## Function `borrow_mut_value`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_borrow_mut_value">borrow_mut_value</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;, key: &<a href="_String">string::String</a>): &<b>mut</b> <a href="_String">string::String</a>
</code></pre>



<a name="0x2_display_remove_value"></a>

## Function `remove_value`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_remove_value">remove_value</a>&lt;T&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;, key: &<a href="_String">string::String</a>)
</code></pre>



<a name="0x2_display_keys"></a>

## Function `keys`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_keys">keys</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x2_display_values"></a>

## Function `values`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_values">values</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x2_display_contains_key"></a>

## Function `contains_key`



<pre><code><b>public</b> <b>fun</b> <a href="display.md#0x2_display_contains_key">contains_key</a>&lt;T&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="display.md#0x2_display_Display">display::Display</a>&lt;T&gt;&gt;, key: &<a href="_String">string::String</a>): bool
</code></pre>
