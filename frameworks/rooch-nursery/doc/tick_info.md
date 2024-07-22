
<a name="0xa_tick_info"></a>

# Module `0xa::tick_info`



-  [Resource `TickInfoStore`](#0xa_tick_info_TickInfoStore)
-  [Resource `TickInfo`](#0xa_tick_info_TickInfo)
-  [Constants](#@Constants_0)
-  [Function `borrow_tick_info`](#0xa_tick_info_borrow_tick_info)
-  [Function `deploy_tick`](#0xa_tick_info_deploy_tick)
-  [Function `mint`](#0xa_tick_info_mint)
-  [Function `mint_on_bitcoin`](#0xa_tick_info_mint_on_bitcoin)
-  [Function `metaprotocol`](#0xa_tick_info_metaprotocol)
-  [Function `tick`](#0xa_tick_info_tick)
-  [Function `generator`](#0xa_tick_info_generator)
-  [Function `factory`](#0xa_tick_info_factory)
-  [Function `max`](#0xa_tick_info_max)
-  [Function `deploy_args`](#0xa_tick_info_deploy_args)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::result</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="">0x4::ord</a>;
<b>use</b> <a href="bitseed_on_l2.md#0xa_bitseed_on_l2">0xa::bitseed_on_l2</a>;
</code></pre>



<a name="0xa_tick_info_TickInfoStore"></a>

## Resource `TickInfoStore`

Store the tick -> TickInfo ObjectID mapping in Object<TickInfoStore> dynamic fields.


<pre><code><b>struct</b> <a href="tick_info.md#0xa_tick_info_TickInfoStore">TickInfoStore</a> <b>has</b> key
</code></pre>



<a name="0xa_tick_info_TickInfo"></a>

## Resource `TickInfo`



<pre><code><b>struct</b> <a href="tick_info.md#0xa_tick_info_TickInfo">TickInfo</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_tick_info_ErrorInvalidMintFactory"></a>



<pre><code><b>const</b> <a href="tick_info.md#0xa_tick_info_ErrorInvalidMintFactory">ErrorInvalidMintFactory</a>: u64 = 4;
</code></pre>



<a name="0xa_tick_info_ErrorMaxSupplyReached"></a>



<pre><code><b>const</b> <a href="tick_info.md#0xa_tick_info_ErrorMaxSupplyReached">ErrorMaxSupplyReached</a>: u64 = 5;
</code></pre>



<a name="0xa_tick_info_ErrorMetaprotocolNotFound"></a>



<pre><code><b>const</b> <a href="tick_info.md#0xa_tick_info_ErrorMetaprotocolNotFound">ErrorMetaprotocolNotFound</a>: u64 = 1;
</code></pre>



<a name="0xa_tick_info_ErrorNoMintFactory"></a>



<pre><code><b>const</b> <a href="tick_info.md#0xa_tick_info_ErrorNoMintFactory">ErrorNoMintFactory</a>: u64 = 3;
</code></pre>



<a name="0xa_tick_info_ErrorTickNotFound"></a>



<pre><code><b>const</b> <a href="tick_info.md#0xa_tick_info_ErrorTickNotFound">ErrorTickNotFound</a>: u64 = 2;
</code></pre>



<a name="0xa_tick_info_borrow_tick_info"></a>

## Function `borrow_tick_info`



<pre><code><b>public</b> <b>fun</b> <a href="tick_info.md#0xa_tick_info_borrow_tick_info">borrow_tick_info</a>(metaprotocol: <a href="_String">string::String</a>, tick: <a href="_String">string::String</a>): &<a href="tick_info.md#0xa_tick_info_TickInfo">tick_info::TickInfo</a>
</code></pre>



<a name="0xa_tick_info_deploy_tick"></a>

## Function `deploy_tick`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tick_info.md#0xa_tick_info_deploy_tick">deploy_tick</a>(metaprotocol: <a href="_String">string::String</a>, tick: <a href="_String">string::String</a>, generator: <a href="_Option">option::Option</a>&lt;<a href="_InscriptionID">ord::InscriptionID</a>&gt;, factory: <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, max: u64, repeat: u64, deploy_args: <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0xa_tick_info_mint"></a>

## Function `mint`



<pre><code>#[private_generics(#[F])]
<b>public</b> <b>fun</b> <a href="tick_info.md#0xa_tick_info_mint">mint</a>&lt;F&gt;(metaprotocol: <a href="_String">string::String</a>, tick: <a href="_String">string::String</a>, amount: u64): <a href="_Object">object::Object</a>&lt;<a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">bitseed_on_l2::Bitseed</a>&gt;
</code></pre>



<a name="0xa_tick_info_mint_on_bitcoin"></a>

## Function `mint_on_bitcoin`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tick_info.md#0xa_tick_info_mint_on_bitcoin">mint_on_bitcoin</a>(metaprotocol: <a href="_String">string::String</a>, tick: <a href="_String">string::String</a>, amount: u64): <a href="_Result">result::Result</a>&lt;<a href="_Object">object::Object</a>&lt;<a href="bitseed_on_l2.md#0xa_bitseed_on_l2_Bitseed">bitseed_on_l2::Bitseed</a>&gt;&gt;
</code></pre>



<a name="0xa_tick_info_metaprotocol"></a>

## Function `metaprotocol`



<pre><code><b>public</b> <b>fun</b> <a href="tick_info.md#0xa_tick_info_metaprotocol">metaprotocol</a>(<a href="tick_info.md#0xa_tick_info">tick_info</a>: &<a href="tick_info.md#0xa_tick_info_TickInfo">tick_info::TickInfo</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0xa_tick_info_tick"></a>

## Function `tick`



<pre><code><b>public</b> <b>fun</b> <a href="tick_info.md#0xa_tick_info_tick">tick</a>(<a href="tick_info.md#0xa_tick_info">tick_info</a>: &<a href="tick_info.md#0xa_tick_info_TickInfo">tick_info::TickInfo</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0xa_tick_info_generator"></a>

## Function `generator`



<pre><code><b>public</b> <b>fun</b> <a href="tick_info.md#0xa_tick_info_generator">generator</a>(<a href="tick_info.md#0xa_tick_info">tick_info</a>: &<a href="tick_info.md#0xa_tick_info_TickInfo">tick_info::TickInfo</a>): <a href="_Option">option::Option</a>&lt;<a href="_InscriptionID">ord::InscriptionID</a>&gt;
</code></pre>



<a name="0xa_tick_info_factory"></a>

## Function `factory`



<pre><code><b>public</b> <b>fun</b> <a href="tick_info.md#0xa_tick_info_factory">factory</a>(<a href="tick_info.md#0xa_tick_info">tick_info</a>: &<a href="tick_info.md#0xa_tick_info_TickInfo">tick_info::TickInfo</a>): <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0xa_tick_info_max"></a>

## Function `max`



<pre><code><b>public</b> <b>fun</b> <a href="tick_info.md#0xa_tick_info_max">max</a>(<a href="tick_info.md#0xa_tick_info">tick_info</a>: &<a href="tick_info.md#0xa_tick_info_TickInfo">tick_info::TickInfo</a>): u64
</code></pre>



<a name="0xa_tick_info_deploy_args"></a>

## Function `deploy_args`



<pre><code><b>public</b> <b>fun</b> <a href="tick_info.md#0xa_tick_info_deploy_args">deploy_args</a>(<a href="tick_info.md#0xa_tick_info">tick_info</a>: &<a href="tick_info.md#0xa_tick_info_TickInfo">tick_info::TickInfo</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>
