
<a name="0x2_big_vector"></a>

# Module `0x2::big_vector`



-  [Struct `BigVector`](#0x2_big_vector_BigVector)
-  [Constants](#@Constants_0)
-  [Function `empty`](#0x2_big_vector_empty)
-  [Function `singleton`](#0x2_big_vector_singleton)
-  [Function `destroy_empty`](#0x2_big_vector_destroy_empty)
-  [Function `destroy`](#0x2_big_vector_destroy)
-  [Function `borrow`](#0x2_big_vector_borrow)
-  [Function `borrow_mut`](#0x2_big_vector_borrow_mut)
-  [Function `append`](#0x2_big_vector_append)
-  [Function `push_back`](#0x2_big_vector_push_back)
-  [Function `pop_back`](#0x2_big_vector_pop_back)
-  [Function `remove`](#0x2_big_vector_remove)
-  [Function `swap_remove`](#0x2_big_vector_swap_remove)
-  [Function `swap`](#0x2_big_vector_swap)
-  [Function `reverse`](#0x2_big_vector_reverse)
-  [Function `index_of`](#0x2_big_vector_index_of)
-  [Function `contains`](#0x2_big_vector_contains)
-  [Function `to_vector`](#0x2_big_vector_to_vector)
-  [Function `length`](#0x2_big_vector_length)
-  [Function `is_empty`](#0x2_big_vector_is_empty)


<pre><code><b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="table.md#0x2_table">0x2::table</a>;
</code></pre>



<a name="0x2_big_vector_BigVector"></a>

## Struct `BigVector`

A scalable vector implementation based on tables where elements are grouped into buckets.
Each bucket has a capacity of <code>bucket_size</code> elements.


<pre><code><b>struct</b> <a href="big_vector.md#0x2_big_vector_BigVector">BigVector</a>&lt;T: store&gt; <b>has</b> store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_big_vector_ErrorBucketSizeIllegal"></a>

bucket_size cannot be 0


<pre><code><b>const</b> <a href="big_vector.md#0x2_big_vector_ErrorBucketSizeIllegal">ErrorBucketSizeIllegal</a>: u64 = 4;
</code></pre>



<a name="0x2_big_vector_ErrorIndexOutOfBound"></a>

Vector index is out of bounds


<pre><code><b>const</b> <a href="big_vector.md#0x2_big_vector_ErrorIndexOutOfBound">ErrorIndexOutOfBound</a>: u64 = 1;
</code></pre>



<a name="0x2_big_vector_ErrorVectorEmpty"></a>

Cannot pop back from an empty vector


<pre><code><b>const</b> <a href="big_vector.md#0x2_big_vector_ErrorVectorEmpty">ErrorVectorEmpty</a>: u64 = 3;
</code></pre>



<a name="0x2_big_vector_ErrorVectorNotEmpty"></a>

Cannot destroy a non-empty vector


<pre><code><b>const</b> <a href="big_vector.md#0x2_big_vector_ErrorVectorNotEmpty">ErrorVectorNotEmpty</a>: u64 = 2;
</code></pre>



<a name="0x2_big_vector_empty"></a>

## Function `empty`

Regular Vector API
Create an empty vector.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_empty">empty</a>&lt;T: store&gt;(bucket_size: u64): <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;
</code></pre>



<a name="0x2_big_vector_singleton"></a>

## Function `singleton`

Create a vector of length 1 containing the passed in element.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_singleton">singleton</a>&lt;T: store&gt;(element: T, bucket_size: u64): <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;
</code></pre>



<a name="0x2_big_vector_destroy_empty"></a>

## Function `destroy_empty`

Destroy the vector <code>v</code>.
Aborts if <code>v</code> is not empty.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_destroy_empty">destroy_empty</a>&lt;T: store&gt;(v: <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;)
</code></pre>



<a name="0x2_big_vector_destroy"></a>

## Function `destroy`

Destroy the vector <code>v</code> if T has <code>drop</code>


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_destroy">destroy</a>&lt;T: drop, store&gt;(v: <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;)
</code></pre>



<a name="0x2_big_vector_borrow"></a>

## Function `borrow`

Acquire an immutable reference to the <code>i</code>th element of the vector <code>v</code>.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_borrow">borrow</a>&lt;T: store&gt;(v: &<a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;, i: u64): &T
</code></pre>



<a name="0x2_big_vector_borrow_mut"></a>

## Function `borrow_mut`

Return a mutable reference to the <code>i</code>th element in the vector <code>v</code>.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_borrow_mut">borrow_mut</a>&lt;T: store&gt;(v: &<b>mut</b> <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;, i: u64): &<b>mut</b> T
</code></pre>



<a name="0x2_big_vector_append"></a>

## Function `append`

Empty and destroy the other vector, and push each of the elements in the other vector onto the lhs vector in the
same order as they occurred in other.
Disclaimer: This function is costly. Use it at your own discretion.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_append">append</a>&lt;T: store&gt;(lhs: &<b>mut</b> <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;, other: <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;)
</code></pre>



<a name="0x2_big_vector_push_back"></a>

## Function `push_back`

Add element <code>val</code> to the end of the vector <code>v</code>. It grows the buckets when the current buckets are full.
This operation will cost more gas when it adds new bucket.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_push_back">push_back</a>&lt;T: store&gt;(v: &<b>mut</b> <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;, val: T)
</code></pre>



<a name="0x2_big_vector_pop_back"></a>

## Function `pop_back`

Pop an element from the end of vector <code>v</code>. It doesn't shrink the buckets even if they're empty.
Call <code>shrink_to_fit</code> explicity to deallocate empty buckets.
Aborts if <code>v</code> is empty.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_pop_back">pop_back</a>&lt;T: store&gt;(v: &<b>mut</b> <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;): T
</code></pre>



<a name="0x2_big_vector_remove"></a>

## Function `remove`

Remove the element at index i in the vector v and return the owned value that was previously stored at i in v.
All elements occurring at indices greater than i will be shifted down by 1. Will abort if i is out of bounds.
Disclaimer: This function is costly. Use it at your own discretion.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_remove">remove</a>&lt;T: store&gt;(v: &<b>mut</b> <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;, i: u64): T
</code></pre>



<a name="0x2_big_vector_swap_remove"></a>

## Function `swap_remove`

Swap the <code>i</code>th element of the vector <code>v</code> with the last element and then pop the vector.
This is O(1), but does not preserve ordering of elements in the vector.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_swap_remove">swap_remove</a>&lt;T: store&gt;(v: &<b>mut</b> <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;, i: u64): T
</code></pre>



<a name="0x2_big_vector_swap"></a>

## Function `swap`

Swap the elements at the i'th and j'th indices in the vector v. Will abort if either of i or j are out of bounds
for v.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_swap">swap</a>&lt;T: store&gt;(v: &<b>mut</b> <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;, i: u64, j: u64)
</code></pre>



<a name="0x2_big_vector_reverse"></a>

## Function `reverse`

Reverse the order of the elements in the vector v in-place.
Disclaimer: This function is costly. Use it at your own discretion.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_reverse">reverse</a>&lt;T: store&gt;(v: &<b>mut</b> <a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;)
</code></pre>



<a name="0x2_big_vector_index_of"></a>

## Function `index_of`

Return the index of the first occurrence of an element in v that is equal to e. Returns (true, index) if such an
element was found, and (false, 0) otherwise.
Disclaimer: This function is costly. Use it at your own discretion.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_index_of">index_of</a>&lt;T: store&gt;(v: &<a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;, val: &T): (bool, u64)
</code></pre>



<a name="0x2_big_vector_contains"></a>

## Function `contains`

Return if an element equal to e exists in the vector v.
Disclaimer: This function is costly. Use it at your own discretion.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_contains">contains</a>&lt;T: store&gt;(v: &<a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;, val: &T): bool
</code></pre>



<a name="0x2_big_vector_to_vector"></a>

## Function `to_vector`

Convert a big vector to a native vector, which is supposed to be called mostly by view functions to get an
atomic view of the whole vector.
Disclaimer: This function may be costly as the big vector may be huge in size. Use it at your own discretion.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_to_vector">to_vector</a>&lt;T: <b>copy</b>, store&gt;(v: &<a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;): <a href="">vector</a>&lt;T&gt;
</code></pre>



<a name="0x2_big_vector_length"></a>

## Function `length`

Return the length of the vector.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_length">length</a>&lt;T: store&gt;(v: &<a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;): u64
</code></pre>



<a name="0x2_big_vector_is_empty"></a>

## Function `is_empty`

Return <code><b>true</b></code> if the vector <code>v</code> has no elements and <code><b>false</b></code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="big_vector.md#0x2_big_vector_is_empty">is_empty</a>&lt;T: store&gt;(v: &<a href="big_vector.md#0x2_big_vector_BigVector">big_vector::BigVector</a>&lt;T&gt;): bool
</code></pre>
