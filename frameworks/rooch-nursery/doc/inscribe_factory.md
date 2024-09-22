
<a name="0xa_inscribe_factory"></a>

# Module `0xa::inscribe_factory`

Bitseed inscribe inscription factory


-  [Struct `MergeState`](#0xa_inscribe_factory_MergeState)
-  [Resource `BitseedEventStore`](#0xa_inscribe_factory_BitseedEventStore)
-  [Struct `InscribeGenerateArgs`](#0xa_inscribe_factory_InscribeGenerateArgs)
-  [Struct `InscribeGenerateOutput`](#0xa_inscribe_factory_InscribeGenerateOutput)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0xa_inscribe_factory_genesis_init)
-  [Function `bitseed_deploy_key`](#0xa_inscribe_factory_bitseed_deploy_key)
-  [Function `bitseed_mint_key`](#0xa_inscribe_factory_bitseed_mint_key)
-  [Function `inscribe_verify`](#0xa_inscribe_factory_inscribe_verify)
-  [Function `process_bitseed_event`](#0xa_inscribe_factory_process_bitseed_event)


<pre><code><b>use</b> <a href="">0x1::bcs</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::address</a>;
<b>use</b> <a href="">0x2::cbor</a>;
<b>use</b> <a href="">0x2::event_queue</a>;
<b>use</b> <a href="">0x2::hash</a>;
<b>use</b> <a href="">0x2::hex</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::result</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::string_utils</a>;
<b>use</b> <a href="">0x4::bitcoin</a>;
<b>use</b> <a href="">0x4::ord</a>;
<b>use</b> <a href="">0x4::types</a>;
<b>use</b> <a href="bitseed.md#0xa_bitseed">0xa::bitseed</a>;
<b>use</b> <a href="tick_info.md#0xa_tick_info">0xa::tick_info</a>;
<b>use</b> <a href="wasm.md#0xa_wasm">0xa::wasm</a>;
</code></pre>



<a name="0xa_inscribe_factory_MergeState"></a>

## Struct `MergeState`



<pre><code><b>struct</b> <a href="inscribe_factory.md#0xa_inscribe_factory_MergeState">MergeState</a> <b>has</b> store
</code></pre>



<a name="0xa_inscribe_factory_BitseedEventStore"></a>

## Resource `BitseedEventStore`



<pre><code><b>struct</b> <a href="inscribe_factory.md#0xa_inscribe_factory_BitseedEventStore">BitseedEventStore</a> <b>has</b> key
</code></pre>



<a name="0xa_inscribe_factory_InscribeGenerateArgs"></a>

## Struct `InscribeGenerateArgs`



<pre><code>#[data_struct]
<b>struct</b> <a href="inscribe_factory.md#0xa_inscribe_factory_InscribeGenerateArgs">InscribeGenerateArgs</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_inscribe_factory_InscribeGenerateOutput"></a>

## Struct `InscribeGenerateOutput`



<pre><code><b>struct</b> <a href="inscribe_factory.md#0xa_inscribe_factory_InscribeGenerateOutput">InscribeGenerateOutput</a> <b>has</b> store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_inscribe_factory_BIT_SEED_DEPLOY"></a>



<pre><code><b>const</b> <a href="inscribe_factory.md#0xa_inscribe_factory_BIT_SEED_DEPLOY">BIT_SEED_DEPLOY</a>: <a href="">vector</a>&lt;u8&gt; = [98, 105, 116, 115, 101, 101, 100, 95, 100, 101, 112, 108, 111, 121];
</code></pre>



<a name="0xa_inscribe_factory_BIT_SEED_GENERATOR_TICK"></a>



<pre><code><b>const</b> <a href="inscribe_factory.md#0xa_inscribe_factory_BIT_SEED_GENERATOR_TICK">BIT_SEED_GENERATOR_TICK</a>: <a href="">vector</a>&lt;u8&gt; = [103, 101, 110, 101, 114, 97, 116, 111, 114];
</code></pre>



<a name="0xa_inscribe_factory_BIT_SEED_MINT"></a>



<pre><code><b>const</b> <a href="inscribe_factory.md#0xa_inscribe_factory_BIT_SEED_MINT">BIT_SEED_MINT</a>: <a href="">vector</a>&lt;u8&gt; = [98, 105, 116, 115, 101, 101, 100, 95, 109, 105, 110, 116];
</code></pre>



<a name="0xa_inscribe_factory_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="inscribe_factory.md#0xa_inscribe_factory_genesis_init">genesis_init</a>()
</code></pre>



<a name="0xa_inscribe_factory_bitseed_deploy_key"></a>

## Function `bitseed_deploy_key`



<pre><code><b>public</b> <b>fun</b> <a href="inscribe_factory.md#0xa_inscribe_factory_bitseed_deploy_key">bitseed_deploy_key</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_inscribe_factory_bitseed_mint_key"></a>

## Function `bitseed_mint_key`



<pre><code><b>public</b> <b>fun</b> <a href="inscribe_factory.md#0xa_inscribe_factory_bitseed_mint_key">bitseed_mint_key</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_inscribe_factory_inscribe_verify"></a>

## Function `inscribe_verify`



<pre><code><b>public</b> <b>fun</b> <a href="inscribe_factory.md#0xa_inscribe_factory_inscribe_verify">inscribe_verify</a>(wasm_bytes: <a href="">vector</a>&lt;u8&gt;, deploy_args: <a href="">vector</a>&lt;u8&gt;, seed: <a href="">vector</a>&lt;u8&gt;, user_input: <a href="_String">string::String</a>, metadata: &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="">vector</a>&lt;u8&gt;&gt;, content_type: <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, body: <a href="">vector</a>&lt;u8&gt;): (bool, <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;)
</code></pre>



<a name="0xa_inscribe_factory_process_bitseed_event"></a>

## Function `process_bitseed_event`



<pre><code><b>public</b> entry <b>fun</b> <a href="inscribe_factory.md#0xa_inscribe_factory_process_bitseed_event">process_bitseed_event</a>(batch_size: u64)
</code></pre>
