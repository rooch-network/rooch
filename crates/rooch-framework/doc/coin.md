
<a name="0x3_coin"></a>

# Module `0x3::coin`

This module provides the foundation for typesafe Coins.


-  [Struct `Coin`](#0x3_coin_Coin)
-  [Struct `CoinInfo`](#0x3_coin_CoinInfo)
-  [Resource `CoinInfos`](#0x3_coin_CoinInfos)
-  [Struct `MintEvent`](#0x3_coin_MintEvent)
-  [Struct `BurnEvent`](#0x3_coin_BurnEvent)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_coin_genesis_init)
-  [Function `check_coin_info_registered`](#0x3_coin_check_coin_info_registered)
-  [Function `is_registered`](#0x3_coin_is_registered)
-  [Function `name`](#0x3_coin_name)
-  [Function `symbol`](#0x3_coin_symbol)
-  [Function `decimals`](#0x3_coin_decimals)
-  [Function `supply`](#0x3_coin_supply)
-  [Function `is_same_coin`](#0x3_coin_is_same_coin)
-  [Function `coin_infos_handle`](#0x3_coin_coin_infos_handle)
-  [Function `destroy_zero`](#0x3_coin_destroy_zero)
-  [Function `extract`](#0x3_coin_extract)
-  [Function `extract_all`](#0x3_coin_extract_all)
-  [Function `merge`](#0x3_coin_merge)
-  [Function `value`](#0x3_coin_value)
-  [Function `zero`](#0x3_coin_zero)
-  [Function `register_extend`](#0x3_coin_register_extend)
-  [Function `mint_extend`](#0x3_coin_mint_extend)
-  [Function `burn_extend`](#0x3_coin_burn_extend)
-  [Function `unpack`](#0x3_coin_unpack)
-  [Function `pack`](#0x3_coin_pack)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object_id</a>;
<b>use</b> <a href="">0x2::table</a>;
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



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>value: u256</code>
</dt>
<dd>
 Amount of coin this address has.
 Following the ERC20 standard, both asset balance and supply are expressed in u256
</dd>
</dl>


</details>

<a name="0x3_coin_CoinInfo"></a>

## Struct `CoinInfo`

Information about a specific coin type. Stored in the global CoinInfos table.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_CoinInfo">CoinInfo</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_type: <a href="_String">string::String</a></code>
</dt>
<dd>
 Type of the coin: <code>address::my_module::XCoin</code>, same as <code>moveos_std::type_info::type_name&lt;CoinType&gt;()</code>.
 The name and symbol can repeat across different coin types, but the coin type must be unique.
</dd>
<dt>
<code>name: <a href="_String">string::String</a></code>
</dt>
<dd>
 Name of the coin.
</dd>
<dt>
<code>symbol: <a href="_String">string::String</a></code>
</dt>
<dd>
 Symbol of the coin, usually a shorter version of the name.
 For example, Singapore Dollar is SGD.
</dd>
<dt>
<code>decimals: u8</code>
</dt>
<dd>
 Number of decimals used to get its user representation.
 For example, if <code>decimals</code> equals <code>2</code>, a balance of <code>505</code> coins should
 be displayed to a user as <code>5.05</code> (<code>505 / 10 ** 2</code>).
</dd>
<dt>
<code>supply: u256</code>
</dt>
<dd>
 The total value for the coin represented by coin type. Mutable.
</dd>
</dl>


</details>

<a name="0x3_coin_CoinInfos"></a>

## Resource `CoinInfos`

A resource that holds the CoinInfo for all accounts.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_infos: <a href="_Table">table::Table</a>&lt;<a href="_String">string::String</a>, <a href="coin.md#0x3_coin_CoinInfo">coin::CoinInfo</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_coin_MintEvent"></a>

## Struct `MintEvent`

Event emitted when coin minted.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_MintEvent">MintEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_type: <a href="_String">string::String</a></code>
</dt>
<dd>
 The type of coin that was minted
</dd>
<dt>
<code>amount: u256</code>
</dt>
<dd>
 coins added to the system
</dd>
</dl>


</details>

<a name="0x3_coin_BurnEvent"></a>

## Struct `BurnEvent`

Event emitted when coin burned.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_BurnEvent">BurnEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_type: <a href="_String">string::String</a></code>
</dt>
<dd>
 The type of coin that was burned
</dd>
<dt>
<code>amount: u256</code>
</dt>
<dd>
 coins removed from the system
</dd>
</dl>


</details>

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


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorCoinInfoAlreadyRegistered">ErrorCoinInfoAlreadyRegistered</a>: u64 = 1;
</code></pre>



<a name="0x3_coin_ErrorCoinInfoNotRegistered"></a>

<code>CoinType</code> is not registered as a coin


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorCoinInfoNotRegistered">ErrorCoinInfoNotRegistered</a>: u64 = 0;
</code></pre>



<a name="0x3_coin_ErrorCoinInfosNotFound"></a>

Global CoinInfos should exist


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorCoinInfosNotFound">ErrorCoinInfosNotFound</a>: u64 = 9;
</code></pre>



<a name="0x3_coin_ErrorCoinNameTooLong"></a>

Name of the coin is too long


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorCoinNameTooLong">ErrorCoinNameTooLong</a>: u64 = 5;
</code></pre>



<a name="0x3_coin_ErrorCoinSymbolTooLong"></a>

Symbol of the coin is too long


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorCoinSymbolTooLong">ErrorCoinSymbolTooLong</a>: u64 = 6;
</code></pre>



<a name="0x3_coin_ErrorDestroyOfNonZeroCoin"></a>

Cannot destroy non-zero coins


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorDestroyOfNonZeroCoin">ErrorDestroyOfNonZeroCoin</a>: u64 = 3;
</code></pre>



<a name="0x3_coin_ErrorInSufficientBalance"></a>

Not enough coins to extract


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorInSufficientBalance">ErrorInSufficientBalance</a>: u64 = 2;
</code></pre>



<a name="0x3_coin_ErrorZeroCoinAmount"></a>

Coin amount cannot be zero


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorZeroCoinAmount">ErrorZeroCoinAmount</a>: u64 = 4;
</code></pre>



<a name="0x3_coin_MAX_COIN_NAME_LENGTH"></a>



<pre><code><b>const</b> <a href="coin.md#0x3_coin_MAX_COIN_NAME_LENGTH">MAX_COIN_NAME_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x3_coin_MAX_COIN_SYMBOL_LENGTH"></a>



<pre><code><b>const</b> <a href="coin.md#0x3_coin_MAX_COIN_SYMBOL_LENGTH">MAX_COIN_SYMBOL_LENGTH</a>: u64 = 10;
</code></pre>



<a name="0x3_coin_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, genesis_account: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_genesis_init">genesis_init</a>(ctx: &<b>mut</b> Context, genesis_account: &<a href="">signer</a>) {
    <b>let</b> coin_infos = <a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a> {
        coin_infos: <a href="_new">table::new</a>(ctx),
    };
    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, genesis_account, coin_infos);
}
</code></pre>



</details>

<a name="0x3_coin_check_coin_info_registered"></a>

## Function `check_coin_info_registered`

A helper function that check the <code>CoinType</code> is registered, if not, abort.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_check_coin_info_registered">check_coin_info_registered</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_check_coin_info_registered">check_coin_info_registered</a>&lt;CoinType: key&gt;(ctx: &Context){
    <b>assert</b>!(<a href="coin.md#0x3_coin_is_registered">is_registered</a>&lt;CoinType&gt;(ctx), <a href="_not_found">error::not_found</a>(<a href="coin.md#0x3_coin_ErrorCoinInfoNotRegistered">ErrorCoinInfoNotRegistered</a>));
}
</code></pre>



</details>

<a name="0x3_coin_is_registered"></a>

## Function `is_registered`

Returns <code><b>true</b></code> if the type <code>CoinType</code> is an registered coin.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_registered">is_registered</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_registered">is_registered</a>&lt;CoinType: key&gt;(ctx: &Context): bool {
    <b>if</b> (<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a>&gt;(ctx, @rooch_framework)) {
        <b>let</b> coin_infos = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a>&gt;(ctx, @rooch_framework);
        <b>let</b> coin_type = <a href="_type_name">type_info::type_name</a>&lt;CoinType&gt;();
        <a href="_contains">table::contains</a>(&coin_infos.coin_infos, coin_type)
    } <b>else</b> {
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x3_coin_name"></a>

## Function `name`

Returns the name of the coin.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_name">name</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_name">name</a>&lt;CoinType: key&gt;(ctx: &Context): <a href="_String">string::String</a> {
    <a href="coin.md#0x3_coin_borrow_coin_info">borrow_coin_info</a>&lt;CoinType&gt;(ctx).name
}
</code></pre>



</details>

<a name="0x3_coin_symbol"></a>

## Function `symbol`

Returns the symbol of the coin, usually a shorter version of the name.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_symbol">symbol</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_symbol">symbol</a>&lt;CoinType: key&gt;(ctx: &Context): <a href="_String">string::String</a> {
    <a href="coin.md#0x3_coin_borrow_coin_info">borrow_coin_info</a>&lt;CoinType&gt;(ctx).symbol
}
</code></pre>



</details>

<a name="0x3_coin_decimals"></a>

## Function `decimals`

Returns the number of decimals used to get its user representation.
For example, if <code>decimals</code> equals <code>2</code>, a balance of <code>505</code> coins should
be displayed to a user as <code>5.05</code> (<code>505 / 10 ** 2</code>).


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_decimals">decimals</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_decimals">decimals</a>&lt;CoinType: key&gt;(ctx: &Context): u8 {
    <a href="coin.md#0x3_coin_borrow_coin_info">borrow_coin_info</a>&lt;CoinType&gt;(ctx).decimals
}
</code></pre>



</details>

<a name="0x3_coin_supply"></a>

## Function `supply`

Returns the amount of coin in existence.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_supply">supply</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_supply">supply</a>&lt;CoinType: key&gt;(ctx: &Context): u256 {
    <a href="coin.md#0x3_coin_borrow_coin_info">borrow_coin_info</a>&lt;CoinType&gt;(ctx).supply
}
</code></pre>



</details>

<a name="0x3_coin_is_same_coin"></a>

## Function `is_same_coin`

Return true if the type <code>CoinType1</code> is same with <code>CoinType2</code>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_same_coin">is_same_coin</a>&lt;CoinType1, CoinType2&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_same_coin">is_same_coin</a>&lt;CoinType1, CoinType2&gt;(): bool {
    <b>return</b> type_of&lt;CoinType1&gt;() == type_of&lt;CoinType2&gt;()
}
</code></pre>



</details>

<a name="0x3_coin_coin_infos_handle"></a>

## Function `coin_infos_handle`

Return CoinInfos table handle


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_coin_infos_handle">coin_infos_handle</a>(ctx: &<a href="_Context">context::Context</a>): <a href="_ObjectID">object_id::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_coin_infos_handle">coin_infos_handle</a>(ctx: &Context): ObjectID {
    // <a href="coin.md#0x3_coin">coin</a> info ensured via the Genesis transaction, so it should always exist
    <b>assert</b>!(<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a>&gt;(ctx, @rooch_framework), <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorCoinInfosNotFound">ErrorCoinInfosNotFound</a>));
    <b>let</b> coin_infos = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a>&gt;(ctx, @rooch_framework);
    *<a href="_handle">table::handle</a>(&coin_infos.coin_infos)
}
</code></pre>



</details>

<a name="0x3_coin_destroy_zero"></a>

## Function `destroy_zero`

Destroys a zero-value coin. Calls will fail if the <code>value</code> in the passed-in <code><a href="coin.md#0x3_coin">coin</a></code> is non-zero
so it is impossible to "burn" any non-zero amount of <code><a href="coin.md#0x3_coin_Coin">Coin</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_destroy_zero">destroy_zero</a>&lt;CoinType: key&gt;(zero_coin: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_destroy_zero">destroy_zero</a>&lt;CoinType: key&gt;(zero_coin: <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="coin.md#0x3_coin_Coin">Coin</a> { value } = zero_coin;
    <b>assert</b>!(value == 0, <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorDestroyOfNonZeroCoin">ErrorDestroyOfNonZeroCoin</a>))
}
</code></pre>



</details>

<a name="0x3_coin_extract"></a>

## Function `extract`

Extracts <code>amount</code> from the passed-in <code><a href="coin.md#0x3_coin">coin</a></code>, where the original coin is modified in place.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_extract">extract</a>&lt;CoinType: key&gt;(<a href="coin.md#0x3_coin">coin</a>: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_extract">extract</a>&lt;CoinType: key&gt;(<a href="coin.md#0x3_coin">coin</a>: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;, amount: u256): <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
    <b>assert</b>!(<a href="coin.md#0x3_coin">coin</a>.value &gt;= amount, <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorInSufficientBalance">ErrorInSufficientBalance</a>));
    <a href="coin.md#0x3_coin">coin</a>.value = <a href="coin.md#0x3_coin">coin</a>.value - amount;
    <a href="coin.md#0x3_coin_Coin">Coin</a> { value: amount }
}
</code></pre>



</details>

<a name="0x3_coin_extract_all"></a>

## Function `extract_all`

Extracts the entire amount from the passed-in <code><a href="coin.md#0x3_coin">coin</a></code>, where the original coin is modified in place.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_extract_all">extract_all</a>&lt;CoinType: key&gt;(<a href="coin.md#0x3_coin">coin</a>: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_extract_all">extract_all</a>&lt;CoinType: key&gt;(<a href="coin.md#0x3_coin">coin</a>: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;): <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
    <b>let</b> total_value = <a href="coin.md#0x3_coin">coin</a>.value;
    <a href="coin.md#0x3_coin">coin</a>.value = 0;
    <a href="coin.md#0x3_coin_Coin">Coin</a> { value: total_value }
}
</code></pre>



</details>

<a name="0x3_coin_merge"></a>

## Function `merge`

"Merges" the two given coins.  The coin passed in as <code>dst_coin</code> will have a value equal
to the sum of the two coins (<code>dst_coin</code> and <code>source_coin</code>).


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_merge">merge</a>&lt;CoinType: key&gt;(dst_coin: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;, source_coin: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_merge">merge</a>&lt;CoinType: key&gt;(dst_coin: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;, source_coin: <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="coin.md#0x3_coin_Coin">Coin</a> { value } = source_coin;
    dst_coin.value = dst_coin.value + value;
}
</code></pre>



</details>

<a name="0x3_coin_value"></a>

## Function `value`

Returns the <code>value</code> passed in <code><a href="coin.md#0x3_coin">coin</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_value">value</a>&lt;CoinType: key&gt;(<a href="coin.md#0x3_coin">coin</a>: &<a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_value">value</a>&lt;CoinType: key&gt;(<a href="coin.md#0x3_coin">coin</a>: &<a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;): u256 {
    <a href="coin.md#0x3_coin">coin</a>.value
}
</code></pre>



</details>

<a name="0x3_coin_zero"></a>

## Function `zero`

Create a new <code><a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;</code> with a value of <code>0</code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_zero">zero</a>&lt;CoinType: key&gt;(): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_zero">zero</a>&lt;CoinType: key&gt;(): <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
    <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
        value: 0
    }
}
</code></pre>



</details>

<a name="0x3_coin_register_extend"></a>

## Function `register_extend`

Creates a new Coin with given <code>CoinType</code>
This function is protected by <code>private_generics</code>, so it can only be called by the <code>CoinType</code> module.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_register_extend">register_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, name: <a href="_String">string::String</a>, symbol: <a href="_String">string::String</a>, decimals: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_register_extend">register_extend</a>&lt;CoinType: key&gt;(
    ctx: &<b>mut</b> Context,
    name: <a href="_String">string::String</a>,
    symbol: <a href="_String">string::String</a>,
    decimals: u8,
){

    <b>let</b> coin_infos = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a>&gt;(ctx, @rooch_framework);
    <b>let</b> coin_type = <a href="_type_name">type_info::type_name</a>&lt;CoinType&gt;();

    <b>assert</b>!(
        !<a href="_contains">table::contains</a>(&coin_infos.coin_infos, coin_type),
        <a href="_already_exists">error::already_exists</a>(<a href="coin.md#0x3_coin_ErrorCoinInfoAlreadyRegistered">ErrorCoinInfoAlreadyRegistered</a>),
    );

    <b>assert</b>!(<a href="_length">string::length</a>(&name) &lt;= <a href="coin.md#0x3_coin_MAX_COIN_NAME_LENGTH">MAX_COIN_NAME_LENGTH</a>, <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorCoinNameTooLong">ErrorCoinNameTooLong</a>));
    <b>assert</b>!(<a href="_length">string::length</a>(&symbol) &lt;= <a href="coin.md#0x3_coin_MAX_COIN_SYMBOL_LENGTH">MAX_COIN_SYMBOL_LENGTH</a>, <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorCoinSymbolTooLong">ErrorCoinSymbolTooLong</a>));

    <b>let</b> coin_info = <a href="coin.md#0x3_coin_CoinInfo">CoinInfo</a> {
        coin_type,
        name,
        symbol,
        decimals,
        supply: 0u256,
    };
    <a href="_add">table::add</a>(&<b>mut</b> coin_infos.coin_infos, coin_type, coin_info);
}
</code></pre>



</details>

<a name="0x3_coin_mint_extend"></a>

## Function `mint_extend`

Mint new <code><a href="coin.md#0x3_coin_Coin">Coin</a></code>, this function is only called by the <code>CoinType</code> module, for the developer to extend custom mint logic


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_mint_extend">mint_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_mint_extend">mint_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> Context,amount: u256) : <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
    <a href="coin.md#0x3_coin_mint_internal">mint_internal</a>&lt;CoinType&gt;(ctx, amount)
}
</code></pre>



</details>

<a name="0x3_coin_burn_extend"></a>

## Function `burn_extend`

Burn <code><a href="coin.md#0x3_coin">coin</a></code>
This function is only called by the <code>CoinType</code> module, for the developer to extend custom burn logic


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_burn_extend">burn_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_burn_extend">burn_extend</a>&lt;CoinType: key&gt;(
    ctx: &<b>mut</b> Context,
    <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;,
) {
    <a href="coin.md#0x3_coin_burn_internal">burn_internal</a>(ctx, <a href="coin.md#0x3_coin">coin</a>)
}
</code></pre>



</details>

<a name="0x3_coin_unpack"></a>

## Function `unpack`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_unpack">unpack</a>&lt;CoinType: key&gt;(<a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_unpack">unpack</a>&lt;CoinType: key&gt;(<a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;) : u256 {
    <b>let</b> <a href="coin.md#0x3_coin_Coin">Coin</a> { value } = <a href="coin.md#0x3_coin">coin</a>;
    value
}
</code></pre>



</details>

<a name="0x3_coin_pack"></a>

## Function `pack`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_pack">pack</a>&lt;CoinType: key&gt;(value: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_pack">pack</a>&lt;CoinType: key&gt;(value: u256) : <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
    <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
        value
    }
}
</code></pre>



</details>
