
<a name="0x2_sort"></a>

# Module `0x2::sort`

Utility functions for sorting vector.


-  [Function `quick_sort`](#0x2_sort_quick_sort)
-  [Function `sort`](#0x2_sort_sort)


<pre><code><b>use</b> <a href="compare.md#0x2_compare">0x2::compare</a>;
</code></pre>



<a name="0x2_sort_quick_sort"></a>

## Function `quick_sort`

Sorts a vector using quick sort algorithm.


<pre><code><b>public</b> <b>fun</b> <a href="sort.md#0x2_sort_quick_sort">quick_sort</a>&lt;T: <b>copy</b>, drop&gt;(data: <a href="">vector</a>&lt;T&gt;): <a href="">vector</a>&lt;T&gt;
</code></pre>



<a name="0x2_sort_sort"></a>

## Function `sort`

Sorts a vector, returning a new vector with the sorted elements.
The sort algorithm used is quick sort, it maybe changed in the future.


<pre><code><b>public</b> <b>fun</b> <a href="sort.md#0x2_sort">sort</a>&lt;T: <b>copy</b>, drop&gt;(data: <a href="">vector</a>&lt;T&gt;): <a href="">vector</a>&lt;T&gt;
</code></pre>
