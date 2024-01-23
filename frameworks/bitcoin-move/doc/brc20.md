
<a name="0x4_brc20"></a>

# Module `0x4::brc20`



-  [Struct `BRC20CoinInfo`](#0x4_brc20_BRC20CoinInfo)
-  [Struct `BRC20Balance`](#0x4_brc20_BRC20Balance)
-  [Resource `BRC20Store`](#0x4_brc20_BRC20Store)
-  [Struct `Op`](#0x4_brc20_Op)
-  [Struct `DeployOp`](#0x4_brc20_DeployOp)
-  [Struct `MintOp`](#0x4_brc20_MintOp)
-  [Struct `TransferOp`](#0x4_brc20_TransferOp)
-  [Function `genesis_init`](#0x4_brc20_genesis_init)
-  [Function `new_op`](#0x4_brc20_new_op)
-  [Function `clone_op`](#0x4_brc20_clone_op)
-  [Function `drop_op`](#0x4_brc20_drop_op)
-  [Function `is_brc20`](#0x4_brc20_is_brc20)
-  [Function `process_utxo_op`](#0x4_brc20_process_utxo_op)
-  [Function `process_inscribe_op`](#0x4_brc20_process_inscribe_op)
-  [Function `get_tick_info`](#0x4_brc20_get_tick_info)
-  [Function `get_balance`](#0x4_brc20_get_balance)


<pre><code><b>use</b> <a href="">0x1::debug</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::object_id</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::string_utils</a>;
<b>use</b> <a href="">0x2::table</a>;
</code></pre>



<a name="0x4_brc20_BRC20CoinInfo"></a>

## Struct `BRC20CoinInfo`



<pre><code><b>struct</b> <a href="brc20.md#0x4_brc20_BRC20CoinInfo">BRC20CoinInfo</a> <b>has</b> <b>copy</b>, store
</code></pre>



<a name="0x4_brc20_BRC20Balance"></a>

## Struct `BRC20Balance`



<pre><code><b>struct</b> <a href="brc20.md#0x4_brc20_BRC20Balance">BRC20Balance</a> <b>has</b> store
</code></pre>



<a name="0x4_brc20_BRC20Store"></a>

## Resource `BRC20Store`



<pre><code><b>struct</b> <a href="brc20.md#0x4_brc20_BRC20Store">BRC20Store</a> <b>has</b> key
</code></pre>



<a name="0x4_brc20_Op"></a>

## Struct `Op`

The brc20 operation


<pre><code><b>struct</b> <a href="brc20.md#0x4_brc20_Op">Op</a> <b>has</b> store
</code></pre>



<a name="0x4_brc20_DeployOp"></a>

## Struct `DeployOp`

The brc20 deploy operation
https://domo-2.gitbook.io/brc-20-experiment/
```json
{
"p": "brc-20",
"op": "deploy",
"tick": "ordi",
"max": "21000000",
"lim": "1000"
}
```


<pre><code><b>struct</b> <a href="brc20.md#0x4_brc20_DeployOp">DeployOp</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_brc20_MintOp"></a>

## Struct `MintOp`

The brc20 mint operation
https://domo-2.gitbook.io/brc-20-experiment/
```json
{
"p": "brc-20",
"op": "mint",
"tick": "ordi",
"amt": "1000"
}
```


<pre><code><b>struct</b> <a href="brc20.md#0x4_brc20_MintOp">MintOp</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_brc20_TransferOp"></a>

## Struct `TransferOp`

The brc20 transfer operation
https://domo-2.gitbook.io/brc-20-experiment/
```json
{
"p": "brc-20",
"op": "transfer",
"tick": "ordi",
"to": "",
"amt": "100"
}


<pre><code><b>struct</b> <a href="brc20.md#0x4_brc20_TransferOp">TransferOp</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_brc20_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="brc20.md#0x4_brc20_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, _genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x4_brc20_new_op"></a>

## Function `new_op`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="brc20.md#0x4_brc20_new_op">new_op</a>(from: <b>address</b>, <b>to</b>: <b>address</b>, json_map: <a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;): <a href="brc20.md#0x4_brc20_Op">brc20::Op</a>
</code></pre>



<a name="0x4_brc20_clone_op"></a>

## Function `clone_op`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x4_brc20_clone_op">clone_op</a>(self: &<a href="brc20.md#0x4_brc20_Op">brc20::Op</a>): <a href="brc20.md#0x4_brc20_Op">brc20::Op</a>
</code></pre>



<a name="0x4_brc20_drop_op"></a>

## Function `drop_op`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x4_brc20_drop_op">drop_op</a>(op: <a href="brc20.md#0x4_brc20_Op">brc20::Op</a>)
</code></pre>



<a name="0x4_brc20_is_brc20"></a>

## Function `is_brc20`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x4_brc20_is_brc20">is_brc20</a>(json_map: &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;): bool
</code></pre>



<a name="0x4_brc20_process_utxo_op"></a>

## Function `process_utxo_op`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="brc20.md#0x4_brc20_process_utxo_op">process_utxo_op</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, op: <a href="brc20.md#0x4_brc20_Op">brc20::Op</a>): bool
</code></pre>



<a name="0x4_brc20_process_inscribe_op"></a>

## Function `process_inscribe_op`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="brc20.md#0x4_brc20_process_inscribe_op">process_inscribe_op</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, op: <a href="brc20.md#0x4_brc20_Op">brc20::Op</a>): bool
</code></pre>



<a name="0x4_brc20_get_tick_info"></a>

## Function `get_tick_info`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x4_brc20_get_tick_info">get_tick_info</a>(brc20_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="brc20.md#0x4_brc20_BRC20Store">brc20::BRC20Store</a>&gt;, tick: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="brc20.md#0x4_brc20_BRC20CoinInfo">brc20::BRC20CoinInfo</a>&gt;
</code></pre>



<a name="0x4_brc20_get_balance"></a>

## Function `get_balance`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x4_brc20_get_balance">get_balance</a>(brc20_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="brc20.md#0x4_brc20_BRC20Store">brc20::BRC20Store</a>&gt;, tick: &<a href="_String">string::String</a>, <b>address</b>: <b>address</b>): u256
</code></pre>
