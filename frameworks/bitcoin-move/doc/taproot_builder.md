
<a name="0x4_taproot_builder"></a>

# Module `0x4::taproot_builder`

Taproot is a module that provides Bitcoin Taproot related functions.


-  [Struct `TaprootBuilder`](#0x4_taproot_builder_TaprootBuilder)
-  [Struct `NodeInfo`](#0x4_taproot_builder_NodeInfo)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x4_taproot_builder_new)
-  [Function `add_leaf`](#0x4_taproot_builder_add_leaf)
-  [Function `finalize`](#0x4_taproot_builder_finalize)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::compare</a>;
<b>use</b> <a href="">0x2::consensus_codec</a>;
<b>use</b> <a href="">0x2::hash</a>;
<b>use</b> <a href="">0x2::result</a>;
<b>use</b> <a href="script_buf.md#0x4_script_buf">0x4::script_buf</a>;
</code></pre>



<a name="0x4_taproot_builder_TaprootBuilder"></a>

## Struct `TaprootBuilder`



<pre><code><b>struct</b> <a href="taproot_builder.md#0x4_taproot_builder_TaprootBuilder">TaprootBuilder</a> <b>has</b> drop, store
</code></pre>



<a name="0x4_taproot_builder_NodeInfo"></a>

## Struct `NodeInfo`



<pre><code><b>struct</b> <a href="taproot_builder.md#0x4_taproot_builder_NodeInfo">NodeInfo</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_taproot_builder_ErrorInvalidMerkleTreeDepth"></a>



<pre><code><b>const</b> <a href="taproot_builder.md#0x4_taproot_builder_ErrorInvalidMerkleTreeDepth">ErrorInvalidMerkleTreeDepth</a>: u64 = 1;
</code></pre>



<a name="0x4_taproot_builder_ErrorNodeNotInDfsOrder"></a>



<pre><code><b>const</b> <a href="taproot_builder.md#0x4_taproot_builder_ErrorNodeNotInDfsOrder">ErrorNodeNotInDfsOrder</a>: u64 = 2;
</code></pre>



<a name="0x4_taproot_builder_ErrorOverCompleteTree"></a>



<pre><code><b>const</b> <a href="taproot_builder.md#0x4_taproot_builder_ErrorOverCompleteTree">ErrorOverCompleteTree</a>: u64 = 3;
</code></pre>



<a name="0x4_taproot_builder_ErrorUnreachable"></a>



<pre><code><b>const</b> <a href="taproot_builder.md#0x4_taproot_builder_ErrorUnreachable">ErrorUnreachable</a>: u64 = 4;
</code></pre>



<a name="0x4_taproot_builder_TAG_TAP_BRANCH"></a>



<pre><code><b>const</b> <a href="taproot_builder.md#0x4_taproot_builder_TAG_TAP_BRANCH">TAG_TAP_BRANCH</a>: <a href="">vector</a>&lt;u8&gt; = [84, 97, 112, 66, 114, 97, 110, 99, 104];
</code></pre>



<a name="0x4_taproot_builder_TAG_TAP_LEAF"></a>



<pre><code><b>const</b> <a href="taproot_builder.md#0x4_taproot_builder_TAG_TAP_LEAF">TAG_TAP_LEAF</a>: <a href="">vector</a>&lt;u8&gt; = [84, 97, 112, 76, 101, 97, 102];
</code></pre>



<a name="0x4_taproot_builder_TAPROOT_CONTROL_MAX_NODE_COUNT"></a>



<pre><code><b>const</b> <a href="taproot_builder.md#0x4_taproot_builder_TAPROOT_CONTROL_MAX_NODE_COUNT">TAPROOT_CONTROL_MAX_NODE_COUNT</a>: u64 = 128;
</code></pre>



<a name="0x4_taproot_builder_TAPROOT_LEAF_TAPSCRIPT"></a>

Tapscript leaf version.


<pre><code><b>const</b> <a href="taproot_builder.md#0x4_taproot_builder_TAPROOT_LEAF_TAPSCRIPT">TAPROOT_LEAF_TAPSCRIPT</a>: u8 = 192;
</code></pre>



<a name="0x4_taproot_builder_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="taproot_builder.md#0x4_taproot_builder_new">new</a>(): <a href="taproot_builder.md#0x4_taproot_builder_TaprootBuilder">taproot_builder::TaprootBuilder</a>
</code></pre>



<a name="0x4_taproot_builder_add_leaf"></a>

## Function `add_leaf`



<pre><code><b>public</b> <b>fun</b> <a href="taproot_builder.md#0x4_taproot_builder_add_leaf">add_leaf</a>(builder: &<b>mut</b> <a href="taproot_builder.md#0x4_taproot_builder_TaprootBuilder">taproot_builder::TaprootBuilder</a>, depth: u8, <a href="script_buf.md#0x4_script_buf">script_buf</a>: <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>): &<b>mut</b> <a href="taproot_builder.md#0x4_taproot_builder_TaprootBuilder">taproot_builder::TaprootBuilder</a>
</code></pre>



<a name="0x4_taproot_builder_finalize"></a>

## Function `finalize`

Finalize the builder, return the state root,
We use the address to represent the hash.


<pre><code><b>public</b> <b>fun</b> <a href="taproot_builder.md#0x4_taproot_builder_finalize">finalize</a>(builder: <a href="taproot_builder.md#0x4_taproot_builder_TaprootBuilder">taproot_builder::TaprootBuilder</a>): <a href="_Result">result::Result</a>&lt;<b>address</b>, <a href="taproot_builder.md#0x4_taproot_builder_TaprootBuilder">taproot_builder::TaprootBuilder</a>&gt;
</code></pre>
