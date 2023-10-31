
<a name="0x2_any"></a>

# Module `0x2::any`



-  [Struct `Any`](#0x2_any_Any)
-  [Constants](#@Constants_0)
-  [Function `pack`](#0x2_any_pack)
-  [Function `unpack`](#0x2_any_unpack)
-  [Function `type_name`](#0x2_any_type_name)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="type_info.md#0x2_type_info">0x2::type_info</a>;
</code></pre>



<a name="0x2_any_Any"></a>

## Struct `Any`

A type which can represent a value of any type. This allows for representation of 'unknown' future
values. For example, to define a resource such that it can be later be extended without breaking
changes one can do

```move
struct Resource {
field: Type,
...
extension: Option<Any>
}
```


<pre><code><b>struct</b> <a href="any.md#0x2_any_Any">Any</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_any_ErrorTypeMismatch"></a>

The type provided for <code>unpack</code> is not the same as was given for <code>pack</code>.


<pre><code><b>const</b> <a href="any.md#0x2_any_ErrorTypeMismatch">ErrorTypeMismatch</a>: u64 = 1;
</code></pre>



<a name="0x2_any_pack"></a>

## Function `pack`

Pack a value into the <code><a href="any.md#0x2_any_Any">Any</a></code> representation. Because Any can be stored and dropped, this is
also required from <code>T</code>.


<pre><code><b>public</b> <b>fun</b> <a href="any.md#0x2_any_pack">pack</a>&lt;T: drop, store&gt;(x: T): <a href="any.md#0x2_any_Any">any::Any</a>
</code></pre>



<a name="0x2_any_unpack"></a>

## Function `unpack`

Unpack a value from the <code><a href="any.md#0x2_any_Any">Any</a></code> representation. This aborts if the value has not the expected type <code>T</code>.


<pre><code><b>public</b> <b>fun</b> <a href="any.md#0x2_any_unpack">unpack</a>&lt;T&gt;(x: <a href="any.md#0x2_any_Any">any::Any</a>): T
</code></pre>



<a name="0x2_any_type_name"></a>

## Function `type_name`

Returns the type name of this Any


<pre><code><b>public</b> <b>fun</b> <a href="">type_name</a>(x: &<a href="any.md#0x2_any_Any">any::Any</a>): &<a href="_String">string::String</a>
</code></pre>
