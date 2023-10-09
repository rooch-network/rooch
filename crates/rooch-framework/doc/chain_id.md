
<a name="0x3_chain_id"></a>

# Module `0x3::chain_id`



-  [Resource `ChainID`](#0x3_chain_id_ChainID)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_chain_id_genesis_init)
-  [Function `chain_id`](#0x3_chain_id_chain_id)
-  [Function `is_local`](#0x3_chain_id_is_local)
-  [Function `is_dev`](#0x3_chain_id_is_dev)
-  [Function `is_test`](#0x3_chain_id_is_test)
-  [Function `is_main`](#0x3_chain_id_is_main)


<pre><code><b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::context</a>;
</code></pre>



<a name="0x3_chain_id_ChainID"></a>

## Resource `ChainID`

The ChainID in the global storage


<pre><code><b>struct</b> <a href="chain_id.md#0x3_chain_id_ChainID">ChainID</a> <b>has</b> <b>copy</b>, drop, store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_chain_id_CHAIN_ID_DEV"></a>



<pre><code><b>const</b> <a href="chain_id.md#0x3_chain_id_CHAIN_ID_DEV">CHAIN_ID_DEV</a>: u64 = 20230103;
</code></pre>



<a name="0x3_chain_id_CHAIN_ID_LOCAL"></a>



<pre><code><b>const</b> <a href="chain_id.md#0x3_chain_id_CHAIN_ID_LOCAL">CHAIN_ID_LOCAL</a>: u64 = 20230104;
</code></pre>



<a name="0x3_chain_id_CHAIN_ID_MAIN"></a>



<pre><code><b>const</b> <a href="chain_id.md#0x3_chain_id_CHAIN_ID_MAIN">CHAIN_ID_MAIN</a>: u64 = 20230101;
</code></pre>



<a name="0x3_chain_id_CHAIN_ID_TEST"></a>



<pre><code><b>const</b> <a href="chain_id.md#0x3_chain_id_CHAIN_ID_TEST">CHAIN_ID_TEST</a>: u64 = 20230102;
</code></pre>



<a name="0x3_chain_id_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="chain_id.md#0x3_chain_id_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, genesis_account: &<a href="">signer</a>, <a href="chain_id.md#0x3_chain_id">chain_id</a>: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="chain_id.md#0x3_chain_id_genesis_init">genesis_init</a>(ctx: &<b>mut</b> Context, genesis_account: &<a href="">signer</a>, <a href="chain_id.md#0x3_chain_id">chain_id</a>: u64){
    <b>let</b> <a href="chain_id.md#0x3_chain_id">chain_id</a> = <a href="chain_id.md#0x3_chain_id_ChainID">ChainID</a>{
        id: <a href="chain_id.md#0x3_chain_id">chain_id</a>
    };
    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, genesis_account, <a href="chain_id.md#0x3_chain_id">chain_id</a>);
}
</code></pre>



</details>

<a name="0x3_chain_id_chain_id"></a>

## Function `chain_id`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id">chain_id</a>(ctx: &<a href="_Context">context::Context</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id">chain_id</a>(ctx: &Context) : u64 {
    <b>let</b> <a href="chain_id.md#0x3_chain_id">chain_id</a> = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="chain_id.md#0x3_chain_id_ChainID">ChainID</a>&gt;(ctx, @rooch_framework);
    <a href="chain_id.md#0x3_chain_id">chain_id</a>.id
}
</code></pre>



</details>

<a name="0x3_chain_id_is_local"></a>

## Function `is_local`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_is_local">is_local</a>(ctx: &<a href="_Context">context::Context</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_is_local">is_local</a>(ctx: &Context) : bool {
    <a href="chain_id.md#0x3_chain_id">chain_id</a>(ctx) == <a href="chain_id.md#0x3_chain_id_CHAIN_ID_LOCAL">CHAIN_ID_LOCAL</a>
}
</code></pre>



</details>

<a name="0x3_chain_id_is_dev"></a>

## Function `is_dev`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_is_dev">is_dev</a>(ctx: &<a href="_Context">context::Context</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_is_dev">is_dev</a>(ctx: &Context) : bool {
    <a href="chain_id.md#0x3_chain_id">chain_id</a>(ctx) == <a href="chain_id.md#0x3_chain_id_CHAIN_ID_DEV">CHAIN_ID_DEV</a>
}
</code></pre>



</details>

<a name="0x3_chain_id_is_test"></a>

## Function `is_test`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_is_test">is_test</a>(ctx: &<a href="_Context">context::Context</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_is_test">is_test</a>(ctx: &Context) : bool {
    <a href="chain_id.md#0x3_chain_id">chain_id</a>(ctx) == <a href="chain_id.md#0x3_chain_id_CHAIN_ID_TEST">CHAIN_ID_TEST</a>
}
</code></pre>



</details>

<a name="0x3_chain_id_is_main"></a>

## Function `is_main`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_is_main">is_main</a>(ctx: &<a href="_Context">context::Context</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id_is_main">is_main</a>(ctx: &Context) : bool {
    <a href="chain_id.md#0x3_chain_id">chain_id</a>(ctx) == <a href="chain_id.md#0x3_chain_id_CHAIN_ID_MAIN">CHAIN_ID_MAIN</a>
}
</code></pre>



</details>
