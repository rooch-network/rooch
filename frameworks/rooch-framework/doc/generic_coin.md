
<a name="0x3_generic_coin"></a>

# Module `0x3::generic_coin`

This module provides the foundation for typesafe Generic Coins.


-  [Struct `GenericCoin`](#0x3_generic_coin_GenericCoin)
-  [Constants](#@Constants_0)
-  [Function `generic_coin_value`](#0x3_generic_coin_generic_coin_value)
-  [Function `unpack_generic_coin`](#0x3_generic_coin_unpack_generic_coin)
-  [Function `pack_generic_coin`](#0x3_generic_coin_pack_generic_coin)
-  [Function `merge_generic`](#0x3_generic_coin_merge_generic)
-  [Function `coin_type`](#0x3_generic_coin_coin_type)


<pre><code><b>use</b> <a href="">0x1::string</a>;
</code></pre>



<a name="0x3_generic_coin_GenericCoin"></a>

## Struct `GenericCoin`

Main structure representing a coin.
Note the <code>CoinType</code> must have <code>key</code> ability.
if the <code>CoinType</code> has <code>store</code> ability, the <code>Coin</code> is a public coin, the user can operate it directly by coin module's function.
Otherwise, the <code>Coin</code> is a private coin, the user can only operate it by <code>CoinType</code> module's function.
The Coin has no ability, it is a hot potato type, only can handle by Coin module.


<pre><code><b>struct</b> <a href="generic_coin.md#0x3_generic_coin_GenericCoin">GenericCoin</a>
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_generic_coin_ErrorCoinTypeNotMatch"></a>

The coin type is not match


<pre><code><b>const</b> <a href="generic_coin.md#0x3_generic_coin_ErrorCoinTypeNotMatch">ErrorCoinTypeNotMatch</a>: u64 = 1;
</code></pre>



<a name="0x3_generic_coin_generic_coin_value"></a>

## Function `generic_coin_value`



<pre><code><b>public</b> <b>fun</b> <a href="generic_coin.md#0x3_generic_coin_generic_coin_value">generic_coin_value</a>(<a href="coin.md#0x3_coin">coin</a>: &<a href="generic_coin.md#0x3_generic_coin_GenericCoin">generic_coin::GenericCoin</a>): <a href="">u256</a>
</code></pre>



<a name="0x3_generic_coin_unpack_generic_coin"></a>

## Function `unpack_generic_coin`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="generic_coin.md#0x3_generic_coin_unpack_generic_coin">unpack_generic_coin</a>(<a href="coin.md#0x3_coin">coin</a>: <a href="generic_coin.md#0x3_generic_coin_GenericCoin">generic_coin::GenericCoin</a>): (<a href="_String">string::String</a>, <a href="">u256</a>)
</code></pre>



<a name="0x3_generic_coin_pack_generic_coin"></a>

## Function `pack_generic_coin`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="generic_coin.md#0x3_generic_coin_pack_generic_coin">pack_generic_coin</a>(coin_type: <a href="_String">string::String</a>, value: <a href="">u256</a>): <a href="generic_coin.md#0x3_generic_coin_GenericCoin">generic_coin::GenericCoin</a>
</code></pre>



<a name="0x3_generic_coin_merge_generic"></a>

## Function `merge_generic`

"Merges" the two given generic coins.  The coin passed in as <code>dst_coin</code> will have a value equal
to the sum of the two generic coins (<code>dst_coin</code> and <code>source_coin</code>).


<pre><code><b>public</b> <b>fun</b> <a href="generic_coin.md#0x3_generic_coin_merge_generic">merge_generic</a>(dst_coin: &<b>mut</b> <a href="generic_coin.md#0x3_generic_coin_GenericCoin">generic_coin::GenericCoin</a>, source_coin: <a href="generic_coin.md#0x3_generic_coin_GenericCoin">generic_coin::GenericCoin</a>)
</code></pre>



<a name="0x3_generic_coin_coin_type"></a>

## Function `coin_type`

Helper function for getting the coin type name from a GenericCoin


<pre><code><b>public</b> <b>fun</b> <a href="generic_coin.md#0x3_generic_coin_coin_type">coin_type</a>(<a href="coin.md#0x3_coin">coin</a>: &<a href="generic_coin.md#0x3_generic_coin_GenericCoin">generic_coin::GenericCoin</a>): <a href="_String">string::String</a>
</code></pre>
