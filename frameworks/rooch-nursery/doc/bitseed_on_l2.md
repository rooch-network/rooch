
<a name="0xa_bitseed_on_l2"></a>

# Module `0xa::bitseed_on_l2`

Bitseed on L2, in the future, this module will merge into the bitseed module.


-  [Resource `Bitseed`](#0xa_bitseed_on_l2_Bitseed)
-  [Constants](#@Constants_0)
-  [Function `new`](#0xa_bitseed_on_l2_new)
-  [Function `is_same_type`](#0xa_bitseed_on_l2_is_same_type)
-  [Function `is_mergeable`](#0xa_bitseed_on_l2_is_mergeable)
-  [Function `merge`](#0xa_bitseed_on_l2_merge)
-  [Function `is_splitable`](#0xa_bitseed_on_l2_is_splitable)
-  [Function `split`](#0xa_bitseed_on_l2_split)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::object</a>;
</code></pre>



<a name="0xa_bitseed_on_l2_Bitseed"></a>

## Resource `Bitseed`

Bitseed is a SFT asset type.


<pre><code><b>struct</b> <a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">Bitseed</a> <b>has</b> store, key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_bitseed_on_l2_ErrorBitseedNotMergeable"></a>



<pre><code><b>const</b> <a href="bitseed_on_l2.md#0xa_bitseed_on_l2_ErrorBitseedNotMergeable">ErrorBitseedNotMergeable</a>: u64 = 1;
</code></pre>



<a name="0xa_bitseed_on_l2_ErrorBitseedNotSplittable"></a>



<pre><code><b>const</b> <a href="bitseed_on_l2.md#0xa_bitseed_on_l2_ErrorBitseedNotSplittable">ErrorBitseedNotSplittable</a>: u64 = 2;
</code></pre>



<a name="0xa_bitseed_on_l2_ErrorInvalidAmount"></a>



<pre><code><b>const</b> <a href="bitseed_on_l2.md#0xa_bitseed_on_l2_ErrorInvalidAmount">ErrorInvalidAmount</a>: u64 = 3;
</code></pre>



<a name="0xa_bitseed_on_l2_new"></a>

## Function `new`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bitseed_on_l2.md#0xa_bitseed_on_l2_new">new</a>(metaprotocol: <a href="_String">string::String</a>, tick: <a href="_String">string::String</a>, bid: <b>address</b>, amount: u64, content_type: <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, body: <a href="">vector</a>&lt;u8&gt;): <a href="_Object">object::Object</a>&lt;<a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">bitseed_on_l2::Bitseed</a>&gt;
</code></pre>



<a name="0xa_bitseed_on_l2_is_same_type"></a>

## Function `is_same_type`

Check if the two bitseeds are the same type.


<pre><code><b>public</b> <b>fun</b> <a href="bitseed_on_l2.md#0xa_bitseed_on_l2_is_same_type">is_same_type</a>(bitseed1_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">bitseed_on_l2::Bitseed</a>&gt;, bitseed2_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">bitseed_on_l2::Bitseed</a>&gt;): bool
</code></pre>



<a name="0xa_bitseed_on_l2_is_mergeable"></a>

## Function `is_mergeable`

Check if the two bitseeds are mergeable.


<pre><code><b>public</b> <b>fun</b> <a href="bitseed_on_l2.md#0xa_bitseed_on_l2_is_mergeable">is_mergeable</a>(bitseed1_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">bitseed_on_l2::Bitseed</a>&gt;, bitseed2_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">bitseed_on_l2::Bitseed</a>&gt;): bool
</code></pre>



<a name="0xa_bitseed_on_l2_merge"></a>

## Function `merge`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed_on_l2.md#0xa_bitseed_on_l2_merge">merge</a>(bitseed1_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">bitseed_on_l2::Bitseed</a>&gt;, bitseed2_obj: <a href="_Object">object::Object</a>&lt;<a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">bitseed_on_l2::Bitseed</a>&gt;)
</code></pre>



<a name="0xa_bitseed_on_l2_is_splitable"></a>

## Function `is_splitable`

Check if the bitseed is splittable.


<pre><code><b>public</b> <b>fun</b> <a href="bitseed_on_l2.md#0xa_bitseed_on_l2_is_splitable">is_splitable</a>(bitseed_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">bitseed_on_l2::Bitseed</a>&gt;): bool
</code></pre>



<a name="0xa_bitseed_on_l2_split"></a>

## Function `split`

Split the bitseed and return the new bitseed.


<pre><code><b>public</b> <b>fun</b> <a href="bitseed_on_l2.md#0xa_bitseed_on_l2_split">split</a>(bitseed_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">bitseed_on_l2::Bitseed</a>&gt;, amount: u64): <a href="_Object">object::Object</a>&lt;<a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">bitseed_on_l2::Bitseed</a>&gt;
</code></pre>
