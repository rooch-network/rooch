
<a name="0xa_mint_get_factory"></a>

# Module `0xa::mint_get_factory`



-  [Struct `MintGetFactory`](#0xa_mint_get_factory_MintGetFactory)
-  [Struct `DeployArgs`](#0xa_mint_get_factory_DeployArgs)
-  [Constants](#@Constants_0)
-  [Function `mint`](#0xa_mint_get_factory_mint)
-  [Function `do_mint`](#0xa_mint_get_factory_do_mint)
-  [Function `factory_type`](#0xa_mint_get_factory_factory_type)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="bitseed.md#0xa_bitseed">0xa::bitseed</a>;
<b>use</b> <a href="tick_info.md#0xa_tick_info">0xa::tick_info</a>;
</code></pre>



<a name="0xa_mint_get_factory_MintGetFactory"></a>

## Struct `MintGetFactory`



<pre><code><b>struct</b> <a href="mint_get_factory.md#0xa_mint_get_factory_MintGetFactory">MintGetFactory</a> <b>has</b> store
</code></pre>



<a name="0xa_mint_get_factory_DeployArgs"></a>

## Struct `DeployArgs`



<pre><code>#[data_struct]
<b>struct</b> <a href="mint_get_factory.md#0xa_mint_get_factory_DeployArgs">DeployArgs</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_mint_get_factory_DEFAULT_AMOUNT_PER_MINT"></a>



<pre><code><b>const</b> <a href="mint_get_factory.md#0xa_mint_get_factory_DEFAULT_AMOUNT_PER_MINT">DEFAULT_AMOUNT_PER_MINT</a>: u64 = 10000;
</code></pre>



<a name="0xa_mint_get_factory_ErrorInvalidInitLockedArgs"></a>



<pre><code><b>const</b> <a href="mint_get_factory.md#0xa_mint_get_factory_ErrorInvalidInitLockedArgs">ErrorInvalidInitLockedArgs</a>: u64 = 1;
</code></pre>



<a name="0xa_mint_get_factory_ErrorInvalidMintFunction"></a>



<pre><code><b>const</b> <a href="mint_get_factory.md#0xa_mint_get_factory_ErrorInvalidMintFunction">ErrorInvalidMintFunction</a>: u64 = 2;
</code></pre>



<a name="0xa_mint_get_factory_mint"></a>

## Function `mint`



<pre><code><b>public</b> entry <b>fun</b> <a href="mint_get_factory.md#0xa_mint_get_factory_mint">mint</a>(metaprotocol: <a href="_String">string::String</a>, tick: <a href="_String">string::String</a>)
</code></pre>



<a name="0xa_mint_get_factory_do_mint"></a>

## Function `do_mint`



<pre><code><b>public</b> <b>fun</b> <a href="mint_get_factory.md#0xa_mint_get_factory_do_mint">do_mint</a>(metaprotocol: <a href="_String">string::String</a>, tick: <a href="_String">string::String</a>): <a href="_Object">object::Object</a>&lt;<a href="bitseed.md#0xa_bitseed_Bitseed">bitseed::Bitseed</a>&gt;
</code></pre>



<a name="0xa_mint_get_factory_factory_type"></a>

## Function `factory_type`



<pre><code><b>public</b> <b>fun</b> <a href="mint_get_factory.md#0xa_mint_get_factory_factory_type">factory_type</a>(): <a href="_String">string::String</a>
</code></pre>
