
<a name="0x3_coin"></a>

# Module `0x3::coin`

This module provides the foundation for typesafe Coins.


-  [Struct `Coin`](#0x3_coin_Coin)
-  [Resource `CoinInfo`](#0x3_coin_CoinInfo)
-  [Struct `MintEvent`](#0x3_coin_MintEvent)
-  [Struct `BurnEvent`](#0x3_coin_BurnEvent)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_coin_genesis_init)
-  [Function `coin_address`](#0x3_coin_coin_address)
-  [Function `check_coin_info_registered`](#0x3_coin_check_coin_info_registered)
-  [Function `is_registered`](#0x3_coin_is_registered)
-  [Function `coin_info_id`](#0x3_coin_coin_info_id)
-  [Function `name`](#0x3_coin_name)
-  [Function `symbol`](#0x3_coin_symbol)
-  [Function `decimals`](#0x3_coin_decimals)
-  [Function `supply`](#0x3_coin_supply)
-  [Function `is_same_coin`](#0x3_coin_is_same_coin)
-  [Function `destroy_zero`](#0x3_coin_destroy_zero)
-  [Function `extract`](#0x3_coin_extract)
-  [Function `extract_all`](#0x3_coin_extract_all)
-  [Function `merge`](#0x3_coin_merge)
-  [Function `value`](#0x3_coin_value)
-  [Function `zero`](#0x3_coin_zero)
-  [Function `borrow_coin_info`](#0x3_coin_borrow_coin_info)
-  [Function `borrow_mut_coin_info_extend`](#0x3_coin_borrow_mut_coin_info_extend)
-  [Function `register_extend`](#0x3_coin_register_extend)
-  [Function `mint_extend`](#0x3_coin_mint_extend)
-  [Function `burn_extend`](#0x3_coin_burn_extend)
-  [Function `unpack`](#0x3_coin_unpack)
-  [Function `pack`](#0x3_coin_pack)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::type_info</a>;
</code></pre>



<a name="0x3_coin_Coin"></a>

## Struct `Coin`

Core data structures
Main structure representing a coin.
Note the <code>CoinType</code> must have <code>key</code> ability.
if the <code>CoinType</code> has <code>store</code> ability, the <code><a href="coin.md#0x3_coin_Coin">Coin</a></code> is a public coin, the user can operate it directly by coin module's function.
Otherwise, the <code><a href="coin.md#0x3_coin_Coin">Coin</a></code> is a private coin, the user can only operate it by <code>CoinType</code> module's function.
The Coin has no ability, it is a hot potato type, only can handle by Coin module.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType: key&gt;
</code></pre>



<a name="0x3_coin_CoinInfo"></a>

## Resource `CoinInfo`

Information about a specific coin type. Stored in the global Object storage.
CoinInfo<CoinType> is a singleton object, the <code>coin_type</code> is the unique key.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_CoinInfo">CoinInfo</a>&lt;CoinType: key&gt; <b>has</b> key
</code></pre>



<a name="0x3_coin_MintEvent"></a>

## Struct `MintEvent`

Event emitted when coin minted.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_MintEvent">MintEvent</a> <b>has</b> drop, store
</code></pre>



<a name="0x3_coin_BurnEvent"></a>

## Struct `BurnEvent`

Event emitted when coin burned.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_BurnEvent">BurnEvent</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_coin_MAX_U64"></a>

Maximum possible aggregatable coin value.


<pre><code><b>const</b> <a href="coin.md#0x3_coin_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x3_coin_MAX_U128"></a>

Maximum possible coin supply.


<pre><code><b>const</b> <a href="coin.md#0x3_coin_MAX_U128">MAX_U128</a>: u128 = 340282366920938463463374607431768211455;
</code></pre>



<a name="0x3_coin_MAX_U256"></a>



<pre><code><b>const</b> <a href="coin.md#0x3_coin_MAX_U256">MAX_U256</a>: u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935;
</code></pre>



<a name="0x3_coin_ErrorCoinInfoAlreadyRegistered"></a>

<code>CoinType</code> is already registered as a coin


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorCoinInfoAlreadyRegistered">ErrorCoinInfoAlreadyRegistered</a>: u64 = 2;
</code></pre>



<a name="0x3_coin_ErrorCoinInfoNotRegistered"></a>

<code>CoinType</code> is not registered as a coin


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorCoinInfoNotRegistered">ErrorCoinInfoNotRegistered</a>: u64 = 1;
</code></pre>



<a name="0x3_coin_ErrorCoinInfosNotFound"></a>

Global CoinInfos should exist


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorCoinInfosNotFound">ErrorCoinInfosNotFound</a>: u64 = 8;
</code></pre>



<a name="0x3_coin_ErrorCoinNameTooLong"></a>

Name of the coin is too long


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorCoinNameTooLong">ErrorCoinNameTooLong</a>: u64 = 6;
</code></pre>



<a name="0x3_coin_ErrorCoinSymbolTooLong"></a>

Symbol of the coin is too long


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorCoinSymbolTooLong">ErrorCoinSymbolTooLong</a>: u64 = 7;
</code></pre>



<a name="0x3_coin_ErrorDestroyOfNonZeroCoin"></a>

Cannot destroy non-zero coins


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorDestroyOfNonZeroCoin">ErrorDestroyOfNonZeroCoin</a>: u64 = 4;
</code></pre>



<a name="0x3_coin_ErrorInSufficientBalance"></a>

Not enough coins to extract


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorInSufficientBalance">ErrorInSufficientBalance</a>: u64 = 3;
</code></pre>



<a name="0x3_coin_ErrorZeroCoinAmount"></a>

Coin amount cannot be zero


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorZeroCoinAmount">ErrorZeroCoinAmount</a>: u64 = 5;
</code></pre>



<a name="0x3_coin_MAX_COIN_NAME_LENGTH"></a>



<pre><code><b>const</b> <a href="coin.md#0x3_coin_MAX_COIN_NAME_LENGTH">MAX_COIN_NAME_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x3_coin_MAX_COIN_SYMBOL_LENGTH"></a>



<pre><code><b>const</b> <a href="coin.md#0x3_coin_MAX_COIN_SYMBOL_LENGTH">MAX_COIN_SYMBOL_LENGTH</a>: u64 = 10;
</code></pre>



<a name="0x3_coin_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_genesis_init">genesis_init</a>(_ctx: &<b>mut</b> <a href="_Context">context::Context</a>, _genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x3_coin_coin_address"></a>

## Function `coin_address`

A helper function that returns the address of CoinType.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_coin_address">coin_address</a>&lt;CoinType: key&gt;(): <b>address</b>
</code></pre>



<a name="0x3_coin_check_coin_info_registered"></a>

## Function `check_coin_info_registered`

A helper function that check the <code>CoinType</code> is registered, if not, abort.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_check_coin_info_registered">check_coin_info_registered</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>)
</code></pre>



<a name="0x3_coin_is_registered"></a>

## Function `is_registered`

Returns <code><b>true</b></code> if the type <code>CoinType</code> is an registered coin.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_registered">is_registered</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>): bool
</code></pre>



<a name="0x3_coin_coin_info_id"></a>

## Function `coin_info_id`

Return the ObjectID of Object<CoinInfo<CoinType>>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_coin_info_id">coin_info_id</a>&lt;CoinType: key&gt;(): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x3_coin_name"></a>

## Function `name`

Returns the name of the coin.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_name">name</a>&lt;CoinType: key&gt;(coin_info: &<a href="coin.md#0x3_coin_CoinInfo">coin::CoinInfo</a>&lt;CoinType&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_coin_symbol"></a>

## Function `symbol`

Returns the symbol of the coin, usually a shorter version of the name.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_symbol">symbol</a>&lt;CoinType: key&gt;(coin_info: &<a href="coin.md#0x3_coin_CoinInfo">coin::CoinInfo</a>&lt;CoinType&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_coin_decimals"></a>

## Function `decimals`

Returns the number of decimals used to get its user representation.
For example, if <code>decimals</code> equals <code>2</code>, a balance of <code>505</code> coins should
be displayed to a user as <code>5.05</code> (<code>505 / 10 ** 2</code>).


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_decimals">decimals</a>&lt;CoinType: key&gt;(coin_info: &<a href="coin.md#0x3_coin_CoinInfo">coin::CoinInfo</a>&lt;CoinType&gt;): u8
</code></pre>



<a name="0x3_coin_supply"></a>

## Function `supply`

Returns the amount of coin in existence.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_supply">supply</a>&lt;CoinType: key&gt;(coin_info: &<a href="coin.md#0x3_coin_CoinInfo">coin::CoinInfo</a>&lt;CoinType&gt;): u256
</code></pre>



<a name="0x3_coin_is_same_coin"></a>

## Function `is_same_coin`

Return true if the type <code>CoinType1</code> is same with <code>CoinType2</code>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_same_coin">is_same_coin</a>&lt;CoinType1, CoinType2&gt;(): bool
</code></pre>



<a name="0x3_coin_destroy_zero"></a>

## Function `destroy_zero`

Destroys a zero-value coin. Calls will fail if the <code>value</code> in the passed-in <code><a href="coin.md#0x3_coin">coin</a></code> is non-zero
so it is impossible to "burn" any non-zero amount of <code><a href="coin.md#0x3_coin_Coin">Coin</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_destroy_zero">destroy_zero</a>&lt;CoinType: key&gt;(zero_coin: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<a name="0x3_coin_extract"></a>

## Function `extract`

Extracts <code>amount</code> from the passed-in <code><a href="coin.md#0x3_coin">coin</a></code>, where the original coin is modified in place.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_extract">extract</a>&lt;CoinType: key&gt;(<a href="coin.md#0x3_coin">coin</a>: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<a name="0x3_coin_extract_all"></a>

## Function `extract_all`

Extracts the entire amount from the passed-in <code><a href="coin.md#0x3_coin">coin</a></code>, where the original coin is modified in place.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_extract_all">extract_all</a>&lt;CoinType: key&gt;(<a href="coin.md#0x3_coin">coin</a>: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<a name="0x3_coin_merge"></a>

## Function `merge`

"Merges" the two given coins.  The coin passed in as <code>dst_coin</code> will have a value equal
to the sum of the two coins (<code>dst_coin</code> and <code>source_coin</code>).


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_merge">merge</a>&lt;CoinType: key&gt;(dst_coin: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;, source_coin: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<a name="0x3_coin_value"></a>

## Function `value`

Returns the <code>value</code> passed in <code><a href="coin.md#0x3_coin">coin</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_value">value</a>&lt;CoinType: key&gt;(<a href="coin.md#0x3_coin">coin</a>: &<a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;): u256
</code></pre>



<a name="0x3_coin_zero"></a>

## Function `zero`

Create a new <code><a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;</code> with a value of <code>0</code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_zero">zero</a>&lt;CoinType: key&gt;(): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<a name="0x3_coin_borrow_coin_info"></a>

## Function `borrow_coin_info`

Borrow the CoinInfo<CoinType>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_borrow_coin_info">borrow_coin_info</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>): &<a href="coin.md#0x3_coin_CoinInfo">coin::CoinInfo</a>&lt;CoinType&gt;
</code></pre>



<a name="0x3_coin_borrow_mut_coin_info_extend"></a>

## Function `borrow_mut_coin_info_extend`

Borrow the mutable CoinInfo<CoinType>
This function is protected by <code>private_generics</code>, so it can only be called by the <code>CoinType</code> module.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_borrow_mut_coin_info_extend">borrow_mut_coin_info_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>): &<b>mut</b> <a href="coin.md#0x3_coin_CoinInfo">coin::CoinInfo</a>&lt;CoinType&gt;
</code></pre>



<a name="0x3_coin_register_extend"></a>

## Function `register_extend`

Creates a new Coin with given <code>CoinType</code>
This function is protected by <code>private_generics</code>, so it can only be called by the <code>CoinType</code> module.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_register_extend">register_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, name: <a href="_String">string::String</a>, symbol: <a href="_String">string::String</a>, decimals: u8): &<b>mut</b> <a href="coin.md#0x3_coin_CoinInfo">coin::CoinInfo</a>&lt;CoinType&gt;
</code></pre>



<a name="0x3_coin_mint_extend"></a>

## Function `mint_extend`

Mint new <code><a href="coin.md#0x3_coin_Coin">Coin</a></code>, this function is only called by the <code>CoinType</code> module, for the developer to extend custom mint logic


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_mint_extend">mint_extend</a>&lt;CoinType: key&gt;(coin_info: &<b>mut</b> <a href="coin.md#0x3_coin_CoinInfo">coin::CoinInfo</a>&lt;CoinType&gt;, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<a name="0x3_coin_burn_extend"></a>

## Function `burn_extend`

Burn <code><a href="coin.md#0x3_coin">coin</a></code>
This function is only called by the <code>CoinType</code> module, for the developer to extend custom burn logic


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_burn_extend">burn_extend</a>&lt;CoinType: key&gt;(coin_info: &<b>mut</b> <a href="coin.md#0x3_coin_CoinInfo">coin::CoinInfo</a>&lt;CoinType&gt;, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<a name="0x3_coin_unpack"></a>

## Function `unpack`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_unpack">unpack</a>&lt;CoinType: key&gt;(<a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;): u256
</code></pre>



<a name="0x3_coin_pack"></a>

## Function `pack`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_pack">pack</a>&lt;CoinType: key&gt;(value: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>
