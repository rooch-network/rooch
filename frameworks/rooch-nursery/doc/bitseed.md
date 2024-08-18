
<a name="0xa_bitseed"></a>

# Module `0xa::bitseed`



-  [Resource `Bitseed`](#0xa_bitseed_Bitseed)
-  [Constants](#@Constants_0)
-  [Function `metaprotocol`](#0xa_bitseed_metaprotocol)
-  [Function `new`](#0xa_bitseed_new)
-  [Function `is_same_type`](#0xa_bitseed_is_same_type)
-  [Function `is_mergeable`](#0xa_bitseed_is_mergeable)
-  [Function `merge`](#0xa_bitseed_merge)
-  [Function `is_splitable`](#0xa_bitseed_is_splitable)
-  [Function `split`](#0xa_bitseed_split)
-  [Function `seal_metaprotocol_validity`](#0xa_bitseed_seal_metaprotocol_validity)
-  [Function `add_metaprotocol_attachment`](#0xa_bitseed_add_metaprotocol_attachment)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::copyable_any</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x4::ord</a>;
</code></pre>



<a name="0xa_bitseed_Bitseed"></a>

## Resource `Bitseed`

Bitseed is a SFT asset type.


<pre><code><b>struct</b> <a href="bitseed.md#0xa_bitseed_Bitseed">Bitseed</a> <b>has</b> store, key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_bitseed_BIT_SEED_DEPLOY"></a>



<pre><code><b>const</b> <a href="bitseed.md#0xa_bitseed_BIT_SEED_DEPLOY">BIT_SEED_DEPLOY</a>: <a href="">vector</a>&lt;u8&gt; = [98, 105, 116, 115, 101, 101, 100, 95, 100, 101, 112, 108, 111, 121];
</code></pre>



<a name="0xa_bitseed_BIT_SEED_GENERATOR_TICK"></a>



<pre><code><b>const</b> <a href="bitseed.md#0xa_bitseed_BIT_SEED_GENERATOR_TICK">BIT_SEED_GENERATOR_TICK</a>: <a href="">vector</a>&lt;u8&gt; = [103, 101, 110, 101, 114, 97, 116, 111, 114];
</code></pre>



<a name="0xa_bitseed_BIT_SEED_MINT"></a>



<pre><code><b>const</b> <a href="bitseed.md#0xa_bitseed_BIT_SEED_MINT">BIT_SEED_MINT</a>: <a href="">vector</a>&lt;u8&gt; = [98, 105, 116, 115, 101, 101, 100, 95, 109, 105, 110, 116];
</code></pre>



<a name="0xa_bitseed_ErrorBitseedNotMergeable"></a>



<pre><code><b>const</b> <a href="bitseed.md#0xa_bitseed_ErrorBitseedNotMergeable">ErrorBitseedNotMergeable</a>: u64 = 1;
</code></pre>



<a name="0xa_bitseed_ErrorBitseedNotSplittable"></a>



<pre><code><b>const</b> <a href="bitseed.md#0xa_bitseed_ErrorBitseedNotSplittable">ErrorBitseedNotSplittable</a>: u64 = 2;
</code></pre>



<a name="0xa_bitseed_ErrorInvalidAmount"></a>



<pre><code><b>const</b> <a href="bitseed.md#0xa_bitseed_ErrorInvalidAmount">ErrorInvalidAmount</a>: u64 = 3;
</code></pre>



<a name="0xa_bitseed_METAPROTOCOL"></a>



<pre><code><b>const</b> <a href="bitseed.md#0xa_bitseed_METAPROTOCOL">METAPROTOCOL</a>: <a href="">vector</a>&lt;u8&gt; = [98, 105, 116, 115, 101, 101, 100];
</code></pre>



<a name="0xa_bitseed_metaprotocol"></a>

## Function `metaprotocol`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_metaprotocol">metaprotocol</a>(): <a href="_String">string::String</a>
</code></pre>



<a name="0xa_bitseed_new"></a>

## Function `new`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bitseed.md#0xa_bitseed_new">new</a>(metaprotocol: <a href="_String">string::String</a>, tick: <a href="_String">string::String</a>, bid: <b>address</b>, amount: u64, content_type: <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, body: <a href="">vector</a>&lt;u8&gt;): <a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_Bitseed">bitseed::Bitseed</a>&gt;
</code></pre>



<a name="0xa_bitseed_is_same_type"></a>

## Function `is_same_type`

Check if the two bitseeds are the same type.


<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_is_same_type">is_same_type</a>(bitseed1_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_Bitseed">bitseed::Bitseed</a>&gt;, bitseed2_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_Bitseed">bitseed::Bitseed</a>&gt;): bool
</code></pre>



<a name="0xa_bitseed_is_mergeable"></a>

## Function `is_mergeable`

Check if the two bitseeds are mergeable.


<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_is_mergeable">is_mergeable</a>(bitseed1_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_Bitseed">bitseed::Bitseed</a>&gt;, bitseed2_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_Bitseed">bitseed::Bitseed</a>&gt;): bool
</code></pre>



<a name="0xa_bitseed_merge"></a>

## Function `merge`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_merge">merge</a>(bitseed1_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_Bitseed">bitseed::Bitseed</a>&gt;, bitseed2_obj: <a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_Bitseed">bitseed::Bitseed</a>&gt;)
</code></pre>



<a name="0xa_bitseed_is_splitable"></a>

## Function `is_splitable`

Check if the bitseed is splittable.


<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_is_splitable">is_splitable</a>(bitseed_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_Bitseed">bitseed::Bitseed</a>&gt;): bool
</code></pre>



<a name="0xa_bitseed_split"></a>

## Function `split`

Split the bitseed and return the new bitseed.


<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_split">split</a>(bitseed_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_Bitseed">bitseed::Bitseed</a>&gt;, amount: u64): <a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_Bitseed">bitseed::Bitseed</a>&gt;
</code></pre>



<a name="0xa_bitseed_seal_metaprotocol_validity"></a>

## Function `seal_metaprotocol_validity`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bitseed.md#0xa_bitseed_seal_metaprotocol_validity">seal_metaprotocol_validity</a>(inscription_id: <a href="_InscriptionID">ord::InscriptionID</a>, is_valid: bool, invalid_reason: <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;)
</code></pre>



<a name="0xa_bitseed_add_metaprotocol_attachment"></a>

## Function `add_metaprotocol_attachment`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bitseed.md#0xa_bitseed_add_metaprotocol_attachment">add_metaprotocol_attachment</a>(inscription_id: <a href="_InscriptionID">ord::InscriptionID</a>, attachment: <a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_Bitseed">bitseed::Bitseed</a>&gt;)
</code></pre>
