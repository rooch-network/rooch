
<a name="0xa_bitseed"></a>

# Module `0xa::bitseed`



-  [Resource `Bitseed`](#0xa_bitseed_Bitseed)
-  [Struct `BitseedCoinInfo`](#0xa_bitseed_BitseedCoinInfo)
-  [Resource `BitseedStore`](#0xa_bitseed_BitseedStore)
-  [Struct `InscribeGenerateArgs`](#0xa_bitseed_InscribeGenerateArgs)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0xa_bitseed_genesis_init)
-  [Function `bitseed_deploy_key`](#0xa_bitseed_bitseed_deploy_key)
-  [Function `bitseed_mint_key`](#0xa_bitseed_bitseed_mint_key)
-  [Function `get_coin_info`](#0xa_bitseed_get_coin_info)
-  [Function `coin_info_tick`](#0xa_bitseed_coin_info_tick)
-  [Function `coin_info_generator`](#0xa_bitseed_coin_info_generator)
-  [Function `coin_info_max`](#0xa_bitseed_coin_info_max)
-  [Function `coin_info_repeat`](#0xa_bitseed_coin_info_repeat)
-  [Function `coin_info_has_user_input`](#0xa_bitseed_coin_info_has_user_input)
-  [Function `coin_info_deploy_args_option`](#0xa_bitseed_coin_info_deploy_args_option)
-  [Function `coin_info_deploy_args`](#0xa_bitseed_coin_info_deploy_args)
-  [Function `coin_info_supply`](#0xa_bitseed_coin_info_supply)
-  [Function `inscribe_verify`](#0xa_bitseed_inscribe_verify)
-  [Function `process_inscription`](#0xa_bitseed_process_inscription)
-  [Function `view_validity`](#0xa_bitseed_view_validity)


<pre><code><b>use</b> <a href="">0x1::bcs</a>;
<b>use</b> <a href="">0x1::debug</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::address</a>;
<b>use</b> <a href="">0x2::cbor</a>;
<b>use</b> <a href="">0x2::hash</a>;
<b>use</b> <a href="">0x2::hex</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::string_utils</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::wasm</a>;
<b>use</b> <a href="">0x4::bitcoin</a>;
<b>use</b> <a href="">0x4::ord</a>;
<b>use</b> <a href="">0x4::types</a>;
</code></pre>



<a name="0xa_bitseed_Bitseed"></a>

## Resource `Bitseed`



<pre><code><b>struct</b> <a href="bitseed.md#0xa_bitseed_Bitseed">Bitseed</a> <b>has</b> key
</code></pre>



<a name="0xa_bitseed_BitseedCoinInfo"></a>

## Struct `BitseedCoinInfo`



<pre><code><b>struct</b> <a href="bitseed.md#0xa_bitseed_BitseedCoinInfo">BitseedCoinInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_bitseed_BitseedStore"></a>

## Resource `BitseedStore`



<pre><code><b>struct</b> <a href="bitseed.md#0xa_bitseed_BitseedStore">BitseedStore</a> <b>has</b> key
</code></pre>



<a name="0xa_bitseed_InscribeGenerateArgs"></a>

## Struct `InscribeGenerateArgs`



<pre><code>#[data_struct]
<b>struct</b> <a href="bitseed.md#0xa_bitseed_InscribeGenerateArgs">InscribeGenerateArgs</a> <b>has</b> <b>copy</b>, drop, store
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



<a name="0xa_bitseed_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bitseed.md#0xa_bitseed_genesis_init">genesis_init</a>(_genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0xa_bitseed_bitseed_deploy_key"></a>

## Function `bitseed_deploy_key`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_bitseed_deploy_key">bitseed_deploy_key</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_bitseed_bitseed_mint_key"></a>

## Function `bitseed_mint_key`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_bitseed_mint_key">bitseed_mint_key</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_bitseed_get_coin_info"></a>

## Function `get_coin_info`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_get_coin_info">get_coin_info</a>(bitseed_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_BitseedStore">bitseed::BitseedStore</a>&gt;, tick: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="bitseed.md#0xa_bitseed_BitseedCoinInfo">bitseed::BitseedCoinInfo</a>&gt;
</code></pre>



<a name="0xa_bitseed_coin_info_tick"></a>

## Function `coin_info_tick`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_coin_info_tick">coin_info_tick</a>(self: &<a href="bitseed.md#0xa_bitseed_BitseedCoinInfo">bitseed::BitseedCoinInfo</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0xa_bitseed_coin_info_generator"></a>

## Function `coin_info_generator`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_coin_info_generator">coin_info_generator</a>(self: &<a href="bitseed.md#0xa_bitseed_BitseedCoinInfo">bitseed::BitseedCoinInfo</a>): <a href="_Option">option::Option</a>&lt;<a href="_InscriptionID">ord::InscriptionID</a>&gt;
</code></pre>



<a name="0xa_bitseed_coin_info_max"></a>

## Function `coin_info_max`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_coin_info_max">coin_info_max</a>(self: &<a href="bitseed.md#0xa_bitseed_BitseedCoinInfo">bitseed::BitseedCoinInfo</a>): u64
</code></pre>



<a name="0xa_bitseed_coin_info_repeat"></a>

## Function `coin_info_repeat`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_coin_info_repeat">coin_info_repeat</a>(self: &<a href="bitseed.md#0xa_bitseed_BitseedCoinInfo">bitseed::BitseedCoinInfo</a>): u64
</code></pre>



<a name="0xa_bitseed_coin_info_has_user_input"></a>

## Function `coin_info_has_user_input`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_coin_info_has_user_input">coin_info_has_user_input</a>(self: &<a href="bitseed.md#0xa_bitseed_BitseedCoinInfo">bitseed::BitseedCoinInfo</a>): bool
</code></pre>



<a name="0xa_bitseed_coin_info_deploy_args_option"></a>

## Function `coin_info_deploy_args_option`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_coin_info_deploy_args_option">coin_info_deploy_args_option</a>(self: &<a href="bitseed.md#0xa_bitseed_BitseedCoinInfo">bitseed::BitseedCoinInfo</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0xa_bitseed_coin_info_deploy_args"></a>

## Function `coin_info_deploy_args`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_coin_info_deploy_args">coin_info_deploy_args</a>(self: &<a href="bitseed.md#0xa_bitseed_BitseedCoinInfo">bitseed::BitseedCoinInfo</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_bitseed_coin_info_supply"></a>

## Function `coin_info_supply`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_coin_info_supply">coin_info_supply</a>(self: &<a href="bitseed.md#0xa_bitseed_BitseedCoinInfo">bitseed::BitseedCoinInfo</a>): u64
</code></pre>



<a name="0xa_bitseed_inscribe_verify"></a>

## Function `inscribe_verify`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_inscribe_verify">inscribe_verify</a>(wasm_bytes: <a href="">vector</a>&lt;u8&gt;, deploy_args: <a href="">vector</a>&lt;u8&gt;, seed: <a href="">vector</a>&lt;u8&gt;, user_input: <a href="">vector</a>&lt;u8&gt;, attributes_output: <a href="">vector</a>&lt;u8&gt;): (bool, <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;)
</code></pre>



<a name="0xa_bitseed_process_inscription"></a>

## Function `process_inscription`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_process_inscription">process_inscription</a>(inscription: &<a href="_Inscription">ord::Inscription</a>)
</code></pre>



<a name="0xa_bitseed_view_validity"></a>

## Function `view_validity`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0xa_bitseed_view_validity">view_validity</a>(inscription_id_str: <a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="_MetaprotocolValidity">ord::MetaprotocolValidity</a>&gt;
</code></pre>
