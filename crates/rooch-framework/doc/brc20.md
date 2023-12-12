
<a name="0x3_brc20"></a>

# Module `0x3::brc20`



-  [Struct `BRC20CoinInfo`](#0x3_brc20_BRC20CoinInfo)
-  [Resource `BRC20Store`](#0x3_brc20_BRC20Store)
-  [Struct `Op`](#0x3_brc20_Op)
-  [Struct `DeployOp`](#0x3_brc20_DeployOp)
-  [Struct `MintOp`](#0x3_brc20_MintOp)
-  [Struct `TransferOp`](#0x3_brc20_TransferOp)
-  [Function `genesis_init`](#0x3_brc20_genesis_init)
-  [Function `is_brc20`](#0x3_brc20_is_brc20)
-  [Function `is_deploy`](#0x3_brc20_is_deploy)
-  [Function `as_deploy`](#0x3_brc20_as_deploy)
-  [Function `is_mint`](#0x3_brc20_is_mint)
-  [Function `as_mint`](#0x3_brc20_as_mint)
-  [Function `is_transfer`](#0x3_brc20_is_transfer)
-  [Function `as_transfer`](#0x3_brc20_as_transfer)
-  [Function `from_inscription`](#0x3_brc20_from_inscription)
-  [Function `from_transaction_bytes`](#0x3_brc20_from_transaction_bytes)


<pre><code><b>use</b> <a href="">0x1::debug</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::json</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::string_utils</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="bitcoin_address.md#0x3_bitcoin_address">0x3::bitcoin_address</a>;
<b>use</b> <a href="bitcoin_types.md#0x3_bitcoin_types">0x3::bitcoin_types</a>;
<b>use</b> <a href="ord.md#0x3_ord">0x3::ord</a>;
</code></pre>



<a name="0x3_brc20_BRC20CoinInfo"></a>

## Struct `BRC20CoinInfo`



<pre><code><b>struct</b> <a href="brc20.md#0x3_brc20_BRC20CoinInfo">BRC20CoinInfo</a> <b>has</b> store
</code></pre>



<a name="0x3_brc20_BRC20Store"></a>

## Resource `BRC20Store`



<pre><code><b>struct</b> <a href="brc20.md#0x3_brc20_BRC20Store">BRC20Store</a> <b>has</b> key
</code></pre>



<a name="0x3_brc20_Op"></a>

## Struct `Op`

The brc20 operation


<pre><code><b>struct</b> <a href="brc20.md#0x3_brc20_Op">Op</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_brc20_DeployOp"></a>

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


<pre><code><b>struct</b> <a href="brc20.md#0x3_brc20_DeployOp">DeployOp</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_brc20_MintOp"></a>

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


<pre><code><b>struct</b> <a href="brc20.md#0x3_brc20_MintOp">MintOp</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_brc20_TransferOp"></a>

## Struct `TransferOp`

The brc20 transfer operation
https://domo-2.gitbook.io/brc-20-experiment/
```json
{
"p": "brc-20",
"op": "transfer",
"tick": "ordi",
"amt": "100"
}


<pre><code><b>struct</b> <a href="brc20.md#0x3_brc20_TransferOp">TransferOp</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_brc20_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="brc20.md#0x3_brc20_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, _genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x3_brc20_is_brc20"></a>

## Function `is_brc20`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x3_brc20_is_brc20">is_brc20</a>(self: &<a href="brc20.md#0x3_brc20_Op">brc20::Op</a>): bool
</code></pre>



<a name="0x3_brc20_is_deploy"></a>

## Function `is_deploy`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x3_brc20_is_deploy">is_deploy</a>(self: &<a href="brc20.md#0x3_brc20_Op">brc20::Op</a>): bool
</code></pre>



<a name="0x3_brc20_as_deploy"></a>

## Function `as_deploy`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x3_brc20_as_deploy">as_deploy</a>(self: &<a href="brc20.md#0x3_brc20_Op">brc20::Op</a>): <a href="_Option">option::Option</a>&lt;<a href="brc20.md#0x3_brc20_DeployOp">brc20::DeployOp</a>&gt;
</code></pre>



<a name="0x3_brc20_is_mint"></a>

## Function `is_mint`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x3_brc20_is_mint">is_mint</a>(self: &<a href="brc20.md#0x3_brc20_Op">brc20::Op</a>): bool
</code></pre>



<a name="0x3_brc20_as_mint"></a>

## Function `as_mint`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x3_brc20_as_mint">as_mint</a>(self: &<a href="brc20.md#0x3_brc20_Op">brc20::Op</a>): <a href="_Option">option::Option</a>&lt;<a href="brc20.md#0x3_brc20_MintOp">brc20::MintOp</a>&gt;
</code></pre>



<a name="0x3_brc20_is_transfer"></a>

## Function `is_transfer`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x3_brc20_is_transfer">is_transfer</a>(self: &<a href="brc20.md#0x3_brc20_Op">brc20::Op</a>): bool
</code></pre>



<a name="0x3_brc20_as_transfer"></a>

## Function `as_transfer`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x3_brc20_as_transfer">as_transfer</a>(self: &<a href="brc20.md#0x3_brc20_Op">brc20::Op</a>): <a href="_Option">option::Option</a>&lt;<a href="brc20.md#0x3_brc20_TransferOp">brc20::TransferOp</a>&gt;
</code></pre>



<a name="0x3_brc20_from_inscription"></a>

## Function `from_inscription`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x3_brc20_from_inscription">from_inscription</a>(inscription_body: <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <a href="_Option">option::Option</a>&lt;<a href="brc20.md#0x3_brc20_Op">brc20::Op</a>&gt;
</code></pre>



<a name="0x3_brc20_from_transaction_bytes"></a>

## Function `from_transaction_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="brc20.md#0x3_brc20_from_transaction_bytes">from_transaction_bytes</a>(transaction_bytes: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;<a href="brc20.md#0x3_brc20_Op">brc20::Op</a>&gt;
</code></pre>
