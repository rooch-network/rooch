
<a name="0x2_bag"></a>

# Module `0x2::bag`

A bag is a heterogeneous map-like collection. The collection is similar to <code>moveos_std::table</code> in that
its keys and values are not stored within the <code><a href="bag.md#0x2_bag_Bag">Bag</a></code> value, but instead are stored using Sui's
object system. The <code><a href="bag.md#0x2_bag_Bag">Bag</a></code> struct acts only as a handle into the object system to retrieve those
keys and values.
Note that this means that <code><a href="bag.md#0x2_bag_Bag">Bag</a></code> values with exactly the same key-value mapping will not be
equal, with <code>==</code>, at runtime. For example
```
let bag1 = bag::new();
let bag2 = bag::new();
bag::add(&mut bag1, 0, false);
bag::add(&mut bag1, 1, true);
bag::add(&mut bag2, 0, false);
bag::add(&mut bag2, 1, true);
// bag1 does not equal bag2, despite having the same entries
assert!(&bag1 != &bag2, 0);
```


-  [Resource `BagInner`](#0x2_bag_BagInner)
-  [Struct `Bag`](#0x2_bag_Bag)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x2_bag_new)
-  [Function `new_dropable`](#0x2_bag_new_dropable)
-  [Function `add`](#0x2_bag_add)
-  [Function `add_dropable`](#0x2_bag_add_dropable)
-  [Function `borrow`](#0x2_bag_borrow)
-  [Function `borrow_mut`](#0x2_bag_borrow_mut)
-  [Function `remove`](#0x2_bag_remove)
-  [Function `contains`](#0x2_bag_contains)
-  [Function `contains_with_type`](#0x2_bag_contains_with_type)
-  [Function `length`](#0x2_bag_length)
-  [Function `is_empty`](#0x2_bag_is_empty)
-  [Function `destroy_empty`](#0x2_bag_destroy_empty)
-  [Function `drop`](#0x2_bag_drop)


<pre><code><b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
</code></pre>



<a name="0x2_bag_BagInner"></a>

## Resource `BagInner`



<pre><code><b>struct</b> <a href="bag.md#0x2_bag_BagInner">BagInner</a> <b>has</b> key
</code></pre>



<a name="0x2_bag_Bag"></a>

## Struct `Bag`



<pre><code><b>struct</b> <a href="bag.md#0x2_bag_Bag">Bag</a> <b>has</b> store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_bag_ErrorBagIsDropable"></a>

If add a non-dropable value to a dropable bag, will abort with this code


<pre><code><b>const</b> <a href="bag.md#0x2_bag_ErrorBagIsDropable">ErrorBagIsDropable</a>: u64 = 1;
</code></pre>



<a name="0x2_bag_ErrorBagIsNotDropable"></a>

If drop a non-dropable bag, will abort with this code


<pre><code><b>const</b> <a href="bag.md#0x2_bag_ErrorBagIsNotDropable">ErrorBagIsNotDropable</a>: u64 = 2;
</code></pre>



<a name="0x2_bag_new"></a>

## Function `new`

Creates a new, empty bag


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_new">new</a>(): <a href="bag.md#0x2_bag_Bag">bag::Bag</a>
</code></pre>



<a name="0x2_bag_new_dropable"></a>

## Function `new_dropable`

Creates a new, empty bag that can be dropped, so all its values should be dropped


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_new_dropable">new_dropable</a>(): <a href="bag.md#0x2_bag_Bag">bag::Bag</a>
</code></pre>



<a name="0x2_bag_add"></a>

## Function `add`

Adds a key-value pair to the bag <code><a href="bag.md#0x2_bag">bag</a>: &<b>mut</b> <a href="bag.md#0x2_bag_Bag">Bag</a></code>
If the bag is dropable, should call <code>add_dropable</code> instead


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_add">add</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="bag.md#0x2_bag">bag</a>: &<b>mut</b> <a href="bag.md#0x2_bag_Bag">bag::Bag</a>, k: K, v: V)
</code></pre>



<a name="0x2_bag_add_dropable"></a>

## Function `add_dropable`

Adds a key-value pair to the bag <code><a href="bag.md#0x2_bag">bag</a>: &<b>mut</b> <a href="bag.md#0x2_bag_Bag">Bag</a></code>, the <code>V</code> should be dropable


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_add_dropable">add_dropable</a>&lt;K: <b>copy</b>, drop, store, V: drop, store&gt;(<a href="bag.md#0x2_bag">bag</a>: &<b>mut</b> <a href="bag.md#0x2_bag_Bag">bag::Bag</a>, k: K, v: V)
</code></pre>



<a name="0x2_bag_borrow"></a>

## Function `borrow`

Immutable borrows the value associated with the key in the bag <code><a href="bag.md#0x2_bag">bag</a>: &<a href="bag.md#0x2_bag_Bag">Bag</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_borrow">borrow</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="bag.md#0x2_bag">bag</a>: &<a href="bag.md#0x2_bag_Bag">bag::Bag</a>, k: K): &V
</code></pre>



<a name="0x2_bag_borrow_mut"></a>

## Function `borrow_mut`

Mutably borrows the value associated with the key in the bag <code><a href="bag.md#0x2_bag">bag</a>: &<b>mut</b> <a href="bag.md#0x2_bag_Bag">Bag</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_borrow_mut">borrow_mut</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="bag.md#0x2_bag">bag</a>: &<b>mut</b> <a href="bag.md#0x2_bag_Bag">bag::Bag</a>, k: K): &<b>mut</b> V
</code></pre>



<a name="0x2_bag_remove"></a>

## Function `remove`

Mutably borrows the key-value pair in the bag <code><a href="bag.md#0x2_bag">bag</a>: &<b>mut</b> <a href="bag.md#0x2_bag_Bag">Bag</a></code> and returns the value.


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_remove">remove</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="bag.md#0x2_bag">bag</a>: &<b>mut</b> <a href="bag.md#0x2_bag_Bag">bag::Bag</a>, k: K): V
</code></pre>



<a name="0x2_bag_contains"></a>

## Function `contains`

Returns true iff there is an value associated with the key <code>k: K</code> in the bag <code><a href="bag.md#0x2_bag">bag</a>: &<a href="bag.md#0x2_bag_Bag">Bag</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_contains">contains</a>&lt;K: <b>copy</b>, drop, store&gt;(<a href="bag.md#0x2_bag">bag</a>: &<a href="bag.md#0x2_bag_Bag">bag::Bag</a>, k: K): bool
</code></pre>



<a name="0x2_bag_contains_with_type"></a>

## Function `contains_with_type`

Returns true iff there is an value associated with the key <code>k: K</code> in the bag <code><a href="bag.md#0x2_bag">bag</a>: &<a href="bag.md#0x2_bag_Bag">Bag</a></code> and the value is of type <code>V</code>


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_contains_with_type">contains_with_type</a>&lt;K: <b>copy</b>, drop, store, V: store&gt;(<a href="bag.md#0x2_bag">bag</a>: &<a href="bag.md#0x2_bag_Bag">bag::Bag</a>, k: K): bool
</code></pre>



<a name="0x2_bag_length"></a>

## Function `length`

Returns the size of the bag, the number of key-value pairs


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_length">length</a>(<a href="bag.md#0x2_bag">bag</a>: &<a href="bag.md#0x2_bag_Bag">bag::Bag</a>): u64
</code></pre>



<a name="0x2_bag_is_empty"></a>

## Function `is_empty`

Returns true iff the bag is empty (if <code>length</code> returns <code>0</code>)


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_is_empty">is_empty</a>(<a href="bag.md#0x2_bag">bag</a>: &<a href="bag.md#0x2_bag_Bag">bag::Bag</a>): bool
</code></pre>



<a name="0x2_bag_destroy_empty"></a>

## Function `destroy_empty`

Destroys an empty bag


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_destroy_empty">destroy_empty</a>(<a href="bag.md#0x2_bag">bag</a>: <a href="bag.md#0x2_bag_Bag">bag::Bag</a>)
</code></pre>



<a name="0x2_bag_drop"></a>

## Function `drop`

Drops a bag, the bag should be dropable


<pre><code><b>public</b> <b>fun</b> <a href="bag.md#0x2_bag_drop">drop</a>(<a href="bag.md#0x2_bag">bag</a>: <a href="bag.md#0x2_bag_Bag">bag::Bag</a>)
</code></pre>
