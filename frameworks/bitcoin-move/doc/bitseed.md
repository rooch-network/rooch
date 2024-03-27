
<a name="0x4_bitseed"></a>

# Module `0x4::bitseed`



-  [Struct `DeployOp`](#0x4_bitseed_DeployOp)
-  [Struct `MintOp`](#0x4_bitseed_MintOp)
-  [Function `is_bitseed`](#0x4_bitseed_is_bitseed)
-  [Function `is_bitseed_deploy`](#0x4_bitseed_is_bitseed_deploy)
-  [Function `is_bitseed_mint`](#0x4_bitseed_is_bitseed_mint)
-  [Function `inscription_to_bitseed_deploy`](#0x4_bitseed_inscription_to_bitseed_deploy)
-  [Function `inscription_to_bitseed_mint`](#0x4_bitseed_inscription_to_bitseed_mint)
-  [Function `inscribe_generate`](#0x4_bitseed_inscribe_generate)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::string_utils</a>;
<b>use</b> <a href="">0x2::wasm</a>;
</code></pre>



<a name="0x4_bitseed_DeployOp"></a>

## Struct `DeployOp`



<pre><code><b>struct</b> <a href="bitseed.md#0x4_bitseed_DeployOp">DeployOp</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_bitseed_MintOp"></a>

## Struct `MintOp`



<pre><code><b>struct</b> <a href="bitseed.md#0x4_bitseed_MintOp">MintOp</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_bitseed_is_bitseed"></a>

## Function `is_bitseed`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0x4_bitseed_is_bitseed">is_bitseed</a>(json_map: &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;): bool
</code></pre>



<a name="0x4_bitseed_is_bitseed_deploy"></a>

## Function `is_bitseed_deploy`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0x4_bitseed_is_bitseed_deploy">is_bitseed_deploy</a>(json_map: &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;): bool
</code></pre>



<a name="0x4_bitseed_is_bitseed_mint"></a>

## Function `is_bitseed_mint`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0x4_bitseed_is_bitseed_mint">is_bitseed_mint</a>(json_map: &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;): bool
</code></pre>



<a name="0x4_bitseed_inscription_to_bitseed_deploy"></a>

## Function `inscription_to_bitseed_deploy`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0x4_bitseed_inscription_to_bitseed_deploy">inscription_to_bitseed_deploy</a>(from: <b>address</b>, <b>to</b>: <b>address</b>, json_map: &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;): <a href="bitseed.md#0x4_bitseed_DeployOp">bitseed::DeployOp</a>
</code></pre>



<a name="0x4_bitseed_inscription_to_bitseed_mint"></a>

## Function `inscription_to_bitseed_mint`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0x4_bitseed_inscription_to_bitseed_mint">inscription_to_bitseed_mint</a>(from: <b>address</b>, <b>to</b>: <b>address</b>, json_map: &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;): <a href="bitseed.md#0x4_bitseed_MintOp">bitseed::MintOp</a>
</code></pre>



<a name="0x4_bitseed_inscribe_generate"></a>

## Function `inscribe_generate`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0x4_bitseed_inscribe_generate">inscribe_generate</a>(wasm_bytes: <a href="">vector</a>&lt;u8&gt;, deploy_args: <a href="">vector</a>&lt;u8&gt;, seed: <a href="">vector</a>&lt;u8&gt;, user_input: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>
