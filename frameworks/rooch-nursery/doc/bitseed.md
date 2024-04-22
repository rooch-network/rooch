
<a name="0xa_bitseed"></a>

# Module `0xa::bitseed`



-  [Resource `Bitseed`](#0x4_bitseed_Bitseed)
-  [Struct `DeployOp`](#0x4_bitseed_DeployOp)
-  [Struct `MintOp`](#0x4_bitseed_MintOp)
-  [Struct `BitseedCoinInfo`](#0x4_bitseed_BitseedCoinInfo)
-  [Resource `BitseedStore`](#0x4_bitseed_BitseedStore)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_bitseed_genesis_init)
-  [Function `bitseed_deploy_key`](#0x4_bitseed_bitseed_deploy_key)
-  [Function `bitseed_mint_key`](#0x4_bitseed_bitseed_mint_key)
-  [Function `is_bitseed`](#0x4_bitseed_is_bitseed)
-  [Function `is_bitseed_deploy`](#0x4_bitseed_is_bitseed_deploy)
-  [Function `is_bitseed_mint`](#0x4_bitseed_is_bitseed_mint)
-  [Function `inscription_to_bitseed_deploy`](#0x4_bitseed_inscription_to_bitseed_deploy)
-  [Function `inscription_to_bitseed_mint`](#0x4_bitseed_inscription_to_bitseed_mint)
-  [Function `inscribe_generate`](#0x4_bitseed_inscribe_generate)
-  [Function `inscribe_verify`](#0x4_bitseed_inscribe_verify)
-  [Function `mint_op_is_valid`](#0x4_bitseed_mint_op_is_valid)
-  [Function `mint_op_attributes`](#0x4_bitseed_mint_op_attributes)
-  [Function `deploy_op_generator`](#0x4_bitseed_deploy_op_generator)
-  [Function `deploy_op_args`](#0x4_bitseed_deploy_op_args)
-  [Function `process`](#0x4_bitseed_process)
-  [Function `process_inscription`](#0x4_bitseed_process_inscription)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::string_utils</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::wasm</a>;
<b>use</b> <a href="ord.md#0x4_ord">0x4::ord</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
<b>use</b> <a href="utxo.md#0x4_utxo">0x4::utxo</a>;
</code></pre>



<a name="0x4_bitseed_Bitseed"></a>

## Resource `Bitseed`



<pre><code><b>struct</b> <a href="bitseed.md#0x4_bitseed_Bitseed">Bitseed</a> <b>has</b> key
</code></pre>



<a name="0xa_bitseed_DeployOp"></a>

## Struct `DeployOp`



<pre><code><b>struct</b> <a href="bitseed.md#0xa_bitseed_DeployOp">DeployOp</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_bitseed_MintOp"></a>

## Struct `MintOp`



<pre><code><b>struct</b> <a href="bitseed.md#0xa_bitseed_MintOp">MintOp</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_bitseed_BitseedCoinInfo"></a>

## Struct `BitseedCoinInfo`



<pre><code><b>struct</b> <a href="bitseed.md#0x4_bitseed_BitseedCoinInfo">BitseedCoinInfo</a> <b>has</b> <b>copy</b>, store
</code></pre>



<a name="0x4_bitseed_BitseedStore"></a>

## Resource `BitseedStore`



<pre><code><b>struct</b> <a href="bitseed.md#0x4_bitseed_BitseedStore">BitseedStore</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_bitseed_BIT_SEED_DEPLOY"></a>



<pre><code><b>const</b> <a href="bitseed.md#0xa_bitseed_BIT_SEED_DEPLOY">BIT_SEED_DEPLOY</a>: <a href="">vector</a>&lt;u8&gt; = [98, 105, 116, 115, 101, 101, 100, 95, 100, 101, 112, 108, 111, 121];
</code></pre>



<a name="0xa_bitseed_BIT_SEED_MINT"></a>



<pre><code><b>const</b> <a href="bitseed.md#0xa_bitseed_BIT_SEED_MINT">BIT_SEED_MINT</a>: <a href="">vector</a>&lt;u8&gt; = [98, 105, 116, 115, 101, 101, 100, 95, 109, 105, 110, 116];
</code></pre>



<a name="0xa_bitseed_bitseed_deploy_key"></a>

## Function `bitseed_deploy_key`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_bitseed_deploy_key">bitseed_deploy_key</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_bitseed_bitseed_mint_key"></a>

## Function `bitseed_mint_key`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_bitseed_mint_key">bitseed_mint_key</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_bitseed_is_bitseed"></a>

## Function `is_bitseed`



<<<<<<< HEAD:frameworks/rooch-nursery/doc/bitseed.md
<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_is_bitseed">is_bitseed</a>(json_map: &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;): bool
=======
<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0x4_bitseed_is_bitseed">is_bitseed</a>(inscription: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>): bool
>>>>>>> 07e1fdeb... feat: rebase from main 4:frameworks/bitcoin-move/doc/bitseed.md
</code></pre>



<a name="0xa_bitseed_is_bitseed_deploy"></a>

## Function `is_bitseed_deploy`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_is_bitseed_deploy">is_bitseed_deploy</a>(json_map: &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;): bool
</code></pre>



<a name="0xa_bitseed_is_bitseed_mint"></a>

## Function `is_bitseed_mint`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_is_bitseed_mint">is_bitseed_mint</a>(json_map: &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;): bool
</code></pre>



<a name="0xa_bitseed_inscription_to_bitseed_deploy"></a>

## Function `inscription_to_bitseed_deploy`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_inscription_to_bitseed_deploy">inscription_to_bitseed_deploy</a>(from: <b>address</b>, <b>to</b>: <b>address</b>, json_map: &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;): <a href="bitseed.md#0xa_bitseed_DeployOp">bitseed::DeployOp</a>
</code></pre>



<a name="0xa_bitseed_inscription_to_bitseed_mint"></a>

## Function `inscription_to_bitseed_mint`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_inscription_to_bitseed_mint">inscription_to_bitseed_mint</a>(from: <b>address</b>, <b>to</b>: <b>address</b>, json_map: &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;): <a href="bitseed.md#0xa_bitseed_MintOp">bitseed::MintOp</a>
</code></pre>



<a name="0xa_bitseed_inscribe_generate"></a>

## Function `inscribe_generate`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_inscribe_generate">inscribe_generate</a>(wasm_bytes: <a href="">vector</a>&lt;u8&gt;, deploy_args: <a href="">vector</a>&lt;u8&gt;, seed: <a href="">vector</a>&lt;u8&gt;, user_input: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_bitseed_inscribe_verify"></a>

## Function `inscribe_verify`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_inscribe_verify">inscribe_verify</a>(wasm_bytes: <a href="">vector</a>&lt;u8&gt;, deploy_args: <a href="">vector</a>&lt;u8&gt;, seed: <a href="">vector</a>&lt;u8&gt;, user_input: <a href="">vector</a>&lt;u8&gt;, attributes_output: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<a name="0xa_bitseed_mint_op_is_valid"></a>

## Function `mint_op_is_valid`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_mint_op_is_valid">mint_op_is_valid</a>(mint_op: &<a href="bitseed.md#0xa_bitseed_MintOp">bitseed::MintOp</a>): u8
</code></pre>



<a name="0xa_bitseed_mint_op_attributes"></a>

## Function `mint_op_attributes`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_mint_op_attributes">mint_op_attributes</a>(mint_op: &<a href="bitseed.md#0xa_bitseed_MintOp">bitseed::MintOp</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_bitseed_deploy_op_generator"></a>

## Function `deploy_op_generator`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_deploy_op_generator">deploy_op_generator</a>(deploy_op: &<a href="bitseed.md#0xa_bitseed_DeployOp">bitseed::DeployOp</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0xa_bitseed_deploy_op_args"></a>

## Function `deploy_op_args`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_deploy_op_args">deploy_op_args</a>(deploy_op: &<a href="bitseed.md#0xa_bitseed_DeployOp">bitseed::DeployOp</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_bitseed_process"></a>

## Function `process`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0x4_bitseed_process">process</a>(tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>)
</code></pre>



<a name="0x4_bitseed_process_inscription"></a>

## Function `process_inscription`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0x4_bitseed_process_inscription">process_inscription</a>(tx: &<a href="types.md#0x4_types_Transaction">types::Transaction</a>, inscription: &<a href="ord.md#0x4_ord_Inscription">ord::Inscription</a>)
</code></pre>
