
<a name="0x3_coin"></a>

# Module `0x3::coin`

This module provides the foundation for typesafe Coins.


-  [Struct `Coin`](#0x3_coin_Coin)
-  [Resource `CoinStore`](#0x3_coin_CoinStore)
-  [Resource `CoinInfo`](#0x3_coin_CoinInfo)
-  [Resource `CoinInfos`](#0x3_coin_CoinInfos)
-  [Struct `MintEvent`](#0x3_coin_MintEvent)
-  [Struct `BurnEvent`](#0x3_coin_BurnEvent)
-  [Resource `MintCapability`](#0x3_coin_MintCapability)
-  [Resource `FreezeCapability`](#0x3_coin_FreezeCapability)
-  [Resource `BurnCapability`](#0x3_coin_BurnCapability)
-  [Constants](#@Constants_0)
-  [Function `balance`](#0x3_coin_balance)
-  [Function `is_coin_initialized`](#0x3_coin_is_coin_initialized)
-  [Function `name`](#0x3_coin_name)
-  [Function `symbol`](#0x3_coin_symbol)
-  [Function `decimals`](#0x3_coin_decimals)
-  [Function `supply`](#0x3_coin_supply)
-  [Function `is_same_coin`](#0x3_coin_is_same_coin)
-  [Function `burn`](#0x3_coin_burn)
-  [Function `burn_from`](#0x3_coin_burn_from)
-  [Function `destroy_zero`](#0x3_coin_destroy_zero)
-  [Function `extract`](#0x3_coin_extract)
-  [Function `extract_all`](#0x3_coin_extract_all)
-  [Function `freeze_coin_store`](#0x3_coin_freeze_coin_store)
-  [Function `unfreeze_coin_store`](#0x3_coin_unfreeze_coin_store)
-  [Function `initialize`](#0x3_coin_initialize)
-  [Function `merge`](#0x3_coin_merge)
-  [Function `mint`](#0x3_coin_mint)
-  [Function `value`](#0x3_coin_value)
-  [Function `zero`](#0x3_coin_zero)
-  [Function `initialize_coin_store`](#0x3_coin_initialize_coin_store)
-  [Function `exist_coin_store`](#0x3_coin_exist_coin_store)
-  [Function `is_coin_store_frozen`](#0x3_coin_is_coin_store_frozen)
-  [Function `extract_coin`](#0x3_coin_extract_coin)
-  [Function `merge_coin`](#0x3_coin_merge_coin)
-  [Function `destroy_freeze_cap`](#0x3_coin_destroy_freeze_cap)
-  [Function `destroy_mint_cap`](#0x3_coin_destroy_mint_cap)
-  [Function `destroy_burn_cap`](#0x3_coin_destroy_burn_cap)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="">0x2::type_table</a>;
<b>use</b> <a href="core_addresses.md#0x3_core_addresses">0x3::core_addresses</a>;
</code></pre>



<a name="0x3_coin_Coin"></a>

## Struct `Coin`

Core data structures
Main structure representing a coin/coin in an account's custody.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; <b>has</b> store
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

<a name="0x3_coin_CoinStore"></a>

## Resource `CoinStore`



<pre><code><b>struct</b> <a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>frozen: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_coin_CoinInfo"></a>

## Resource `CoinInfo`

Information about a specific coin type. Stored on the creator of the coin's account.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_CoinInfo">CoinInfo</a>&lt;CoinType&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>name: <a href="_String">string::String</a></code>
</dt>
<dd>

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
 The total value for the coin represented by <code>CoinType</code>. Mutable.
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
<code>coin_infos: <a href="_TypeTable">type_table::TypeTable</a></code>
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
<code>coin_type_info: <a href="_TypeInfo">type_info::TypeInfo</a></code>
</dt>
<dd>
 full info of coin
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
<code>coin_type_info: <a href="_TypeInfo">type_info::TypeInfo</a></code>
</dt>
<dd>
 full info of coin
</dd>
<dt>
<code>amount: u256</code>
</dt>
<dd>
 coins removed from the system
</dd>
</dl>


</details>

<a name="0x3_coin_MintCapability"></a>

## Resource `MintCapability`

Capability required to mint coins.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_MintCapability">MintCapability</a>&lt;CoinType&gt; <b>has</b> <b>copy</b>, store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_coin_FreezeCapability"></a>

## Resource `FreezeCapability`

Capability required to freeze a coin store.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_FreezeCapability">FreezeCapability</a>&lt;CoinType&gt; <b>has</b> <b>copy</b>, store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_coin_BurnCapability"></a>

## Resource `BurnCapability`

Capability required to burn coins.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_BurnCapability">BurnCapability</a>&lt;CoinType&gt; <b>has</b> <b>copy</b>, store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

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



<a name="0x3_coin_ErrorCoinInfoAddressMismatch"></a>

Address of account which is used to initialize a coin <code>CoinType</code> doesn't match the deployer of module


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorCoinInfoAddressMismatch">ErrorCoinInfoAddressMismatch</a>: u64 = 1;
</code></pre>



<a name="0x3_coin_ErrorCoinInfoAlreadyPublished"></a>

<code>CoinType</code> is already initialized as a coin


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorCoinInfoAlreadyPublished">ErrorCoinInfoAlreadyPublished</a>: u64 = 2;
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

Not enough coins to complete transaction


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



<a name="0x3_coin_balance"></a>

## Function `balance`

Returns the balance of <code>addr</code> for provided <code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_balance">balance</a>&lt;CoinType&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_balance">balance</a>&lt;CoinType&gt;(ctx: &StorageContext, addr: <b>address</b>): u256 {
    <b>if</b> (<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt;&gt;(ctx, addr)) {
        <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt;&gt;(ctx, addr).<a href="coin.md#0x3_coin">coin</a>.value
    } <b>else</b> {
        0u256
    }
}
</code></pre>



</details>

<a name="0x3_coin_is_coin_initialized"></a>

## Function `is_coin_initialized`

Returns <code><b>true</b></code> if the type <code>CoinType</code> is an initialized coin.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_coin_initialized">is_coin_initialized</a>&lt;CoinType&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_coin_initialized">is_coin_initialized</a>&lt;CoinType&gt;(ctx: &StorageContext): bool {
    <b>if</b> (<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a>&gt;(ctx, @rooch_framework)) {
        <b>let</b> coin_infos = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a>&gt;(ctx, @rooch_framework);
        <a href="_contains">type_table::contains</a>&lt;<a href="coin.md#0x3_coin_CoinInfo">CoinInfo</a>&lt;CoinType&gt;&gt;(&coin_infos.coin_infos)
    } <b>else</b> {
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x3_coin_name"></a>

## Function `name`

Returns the name of the coin.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_name">name</a>&lt;CoinType&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_name">name</a>&lt;CoinType&gt;(ctx: &StorageContext): <a href="_String">string::String</a> {
    <a href="coin.md#0x3_coin_borrow_coin_info">borrow_coin_info</a>&lt;CoinType&gt;(ctx).name
}
</code></pre>



</details>

<a name="0x3_coin_symbol"></a>

## Function `symbol`

Returns the symbol of the coin, usually a shorter version of the name.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_symbol">symbol</a>&lt;CoinType&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_symbol">symbol</a>&lt;CoinType&gt;(ctx: &StorageContext): <a href="_String">string::String</a> {
    <a href="coin.md#0x3_coin_borrow_coin_info">borrow_coin_info</a>&lt;CoinType&gt;(ctx).symbol
}
</code></pre>



</details>

<a name="0x3_coin_decimals"></a>

## Function `decimals`

Returns the number of decimals used to get its user representation.
For example, if <code>decimals</code> equals <code>2</code>, a balance of <code>505</code> coins should
be displayed to a user as <code>5.05</code> (<code>505 / 10 ** 2</code>).


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_decimals">decimals</a>&lt;CoinType&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_decimals">decimals</a>&lt;CoinType&gt;(ctx: &StorageContext): u8 {
    <a href="coin.md#0x3_coin_borrow_coin_info">borrow_coin_info</a>&lt;CoinType&gt;(ctx).decimals
}
</code></pre>



</details>

<a name="0x3_coin_supply"></a>

## Function `supply`

Returns the amount of coin in existence.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_supply">supply</a>&lt;CoinType&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_supply">supply</a>&lt;CoinType&gt;(ctx: &StorageContext): u256 {
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

<a name="0x3_coin_burn"></a>

## Function `burn`

Burn <code><a href="coin.md#0x3_coin">coin</a></code> with capability.
The capability <code>_cap</code> should be passed as a reference to <code><a href="coin.md#0x3_coin_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_burn">burn</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;, _cap: &<a href="coin.md#0x3_coin_BurnCapability">coin::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_burn">burn</a>&lt;CoinType&gt;(
    ctx: &<b>mut</b> StorageContext,
    <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;,
    _cap: &<a href="coin.md#0x3_coin_BurnCapability">BurnCapability</a>&lt;CoinType&gt;,
) {
    <b>let</b> <a href="coin.md#0x3_coin_Coin">Coin</a> { value: amount } = <a href="coin.md#0x3_coin">coin</a>;

    <b>let</b> coin_type_info = <a href="_type_of">type_info::type_of</a>&lt;CoinType&gt;();
    <b>let</b> coin_info = <a href="coin.md#0x3_coin_borrow_mut_coin_info">borrow_mut_coin_info</a>&lt;CoinType&gt;(ctx);
    coin_info.supply = coin_info.supply - amount;
    <a href="_emit">event::emit</a>&lt;<a href="coin.md#0x3_coin_BurnEvent">BurnEvent</a>&gt;(ctx, <a href="coin.md#0x3_coin_BurnEvent">BurnEvent</a> {
        coin_type_info,
        amount,
    });
}
</code></pre>



</details>

<a name="0x3_coin_burn_from"></a>

## Function `burn_from`

Burn <code><a href="coin.md#0x3_coin">coin</a></code> from the specified <code><a href="account.md#0x3_account">account</a></code> with capability.
The capability <code>burn_cap</code> should be passed as a reference to <code><a href="coin.md#0x3_coin_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>.
This function shouldn't fail as it's called as part of transaction fee burning.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_burn_from">burn_from</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>, amount: u256, burn_cap: &<a href="coin.md#0x3_coin_BurnCapability">coin::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_burn_from">burn_from</a>&lt;CoinType&gt;(
    ctx: &<b>mut</b> StorageContext,
    addr: <b>address</b>,
    amount: u256,
    burn_cap: &<a href="coin.md#0x3_coin_BurnCapability">BurnCapability</a>&lt;CoinType&gt;,
) {
    <b>let</b> coin_store = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt;&gt;(ctx, addr);
    <b>let</b> coin_to_burn = <a href="coin.md#0x3_coin_extract">extract</a>(&<b>mut</b> coin_store.<a href="coin.md#0x3_coin">coin</a>, amount);
    <a href="coin.md#0x3_coin_burn">burn</a>(ctx, coin_to_burn, burn_cap);
}
</code></pre>



</details>

<a name="0x3_coin_destroy_zero"></a>

## Function `destroy_zero`

Destroys a zero-value coin. Calls will fail if the <code>value</code> in the passed-in <code><a href="coin.md#0x3_coin">coin</a></code> is non-zero
so it is impossible to "burn" any non-zero amount of <code><a href="coin.md#0x3_coin_Coin">Coin</a></code> without having
a <code><a href="coin.md#0x3_coin_BurnCapability">BurnCapability</a></code> for the specific <code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_destroy_zero">destroy_zero</a>&lt;CoinType&gt;(zero_coin: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_destroy_zero">destroy_zero</a>&lt;CoinType&gt;(zero_coin: <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="coin.md#0x3_coin_Coin">Coin</a> { value } = zero_coin;
    <b>assert</b>!(value == 0, <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorDestroyOfNonZeroCoin">ErrorDestroyOfNonZeroCoin</a>))
}
</code></pre>



</details>

<a name="0x3_coin_extract"></a>

## Function `extract`

Extracts <code>amount</code> from the passed-in <code><a href="coin.md#0x3_coin">coin</a></code>, where the original coin is modified in place.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_extract">extract</a>&lt;CoinType&gt;(<a href="coin.md#0x3_coin">coin</a>: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_extract">extract</a>&lt;CoinType&gt;(<a href="coin.md#0x3_coin">coin</a>: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;, amount: u256): <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
    <b>assert</b>!(<a href="coin.md#0x3_coin">coin</a>.value &gt;= amount, <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorInSufficientBalance">ErrorInSufficientBalance</a>));
    <a href="coin.md#0x3_coin">coin</a>.value = <a href="coin.md#0x3_coin">coin</a>.value - amount;
    <a href="coin.md#0x3_coin_Coin">Coin</a> { value: amount }
}
</code></pre>



</details>

<a name="0x3_coin_extract_all"></a>

## Function `extract_all`

Extracts the entire amount from the passed-in <code><a href="coin.md#0x3_coin">coin</a></code>, where the original coin is modified in place.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_extract_all">extract_all</a>&lt;CoinType&gt;(<a href="coin.md#0x3_coin">coin</a>: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_extract_all">extract_all</a>&lt;CoinType&gt;(<a href="coin.md#0x3_coin">coin</a>: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;): <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
    <b>let</b> total_value = <a href="coin.md#0x3_coin">coin</a>.value;
    <a href="coin.md#0x3_coin">coin</a>.value = 0;
    <a href="coin.md#0x3_coin_Coin">Coin</a> { value: total_value }
}
</code></pre>



</details>

<a name="0x3_coin_freeze_coin_store"></a>

## Function `freeze_coin_store`

Freeze a CoinStore to prevent transfers


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_freeze_coin_store">freeze_coin_store</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>, _freeze_cap: &<a href="coin.md#0x3_coin_FreezeCapability">coin::FreezeCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_freeze_coin_store">freeze_coin_store</a>&lt;CoinType&gt;(
    ctx: &<b>mut</b> StorageContext,
    addr: <b>address</b>,
    _freeze_cap: &<a href="coin.md#0x3_coin_FreezeCapability">FreezeCapability</a>&lt;CoinType&gt;,
) {
    <b>let</b> coin_store = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt;&gt;(ctx, addr);
    coin_store.frozen = <b>true</b>;
}
</code></pre>



</details>

<a name="0x3_coin_unfreeze_coin_store"></a>

## Function `unfreeze_coin_store`

Unfreeze a CoinStore to allow transfers


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_unfreeze_coin_store">unfreeze_coin_store</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>, _freeze_cap: &<a href="coin.md#0x3_coin_FreezeCapability">coin::FreezeCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_unfreeze_coin_store">unfreeze_coin_store</a>&lt;CoinType&gt;(
    ctx: &<b>mut</b> StorageContext,
    addr: <b>address</b>,
    _freeze_cap: &<a href="coin.md#0x3_coin_FreezeCapability">FreezeCapability</a>&lt;CoinType&gt;,
) {
    <b>let</b> coin_store = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt;&gt;(ctx, addr);
    coin_store.frozen = <b>false</b>;
}
</code></pre>



</details>

<a name="0x3_coin_initialize"></a>

## Function `initialize`

Creates a new Coin with given <code>CoinType</code> and returns minting/freezing/burning capabilities.
The given signer also becomes the account hosting the information about the coin
(name, supply, etc.).


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_initialize">initialize</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, name: <a href="_String">string::String</a>, symbol: <a href="_String">string::String</a>, decimals: u8): (<a href="coin.md#0x3_coin_BurnCapability">coin::BurnCapability</a>&lt;CoinType&gt;, <a href="coin.md#0x3_coin_FreezeCapability">coin::FreezeCapability</a>&lt;CoinType&gt;, <a href="coin.md#0x3_coin_MintCapability">coin::MintCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_initialize">initialize</a>&lt;CoinType&gt;(
    ctx: &<b>mut</b> StorageContext,
    <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>,
    name: <a href="_String">string::String</a>,
    symbol: <a href="_String">string::String</a>,
    decimals: u8,
): (<a href="coin.md#0x3_coin_BurnCapability">BurnCapability</a>&lt;CoinType&gt;, <a href="coin.md#0x3_coin_FreezeCapability">FreezeCapability</a>&lt;CoinType&gt;, <a href="coin.md#0x3_coin_MintCapability">MintCapability</a>&lt;CoinType&gt;) {
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    <b>assert</b>!(
        <a href="coin.md#0x3_coin_coin_address">coin_address</a>&lt;CoinType&gt;() == addr,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorCoinInfoAddressMismatch">ErrorCoinInfoAddressMismatch</a>),
    );

    <b>assert</b>!(
        !<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin.md#0x3_coin_CoinInfo">CoinInfo</a>&lt;CoinType&gt;&gt;(ctx, addr),
        <a href="_already_exists">error::already_exists</a>(<a href="coin.md#0x3_coin_ErrorCoinInfoAlreadyPublished">ErrorCoinInfoAlreadyPublished</a>),
    );

    <b>assert</b>!(<a href="_length">string::length</a>(&name) &lt;= <a href="coin.md#0x3_coin_MAX_COIN_NAME_LENGTH">MAX_COIN_NAME_LENGTH</a>, <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorCoinNameTooLong">ErrorCoinNameTooLong</a>));
    <b>assert</b>!(<a href="_length">string::length</a>(&symbol) &lt;= <a href="coin.md#0x3_coin_MAX_COIN_SYMBOL_LENGTH">MAX_COIN_SYMBOL_LENGTH</a>, <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorCoinSymbolTooLong">ErrorCoinSymbolTooLong</a>));

    <b>let</b> coin_info = <a href="coin.md#0x3_coin_CoinInfo">CoinInfo</a>&lt;CoinType&gt; {
        name,
        symbol,
        decimals,
        supply: 0u256,
    };
    // <a href="_global_move_to">account_storage::global_move_to</a>&lt;<a href="coin.md#0x3_coin_CoinInfo">CoinInfo</a>&lt;CoinType&gt;&gt;(ctx, <a href="account.md#0x3_account">account</a>, coin_info);
    <b>let</b> coin_infos = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a>&gt;(ctx, @rooch_framework);
    <a href="_add">type_table::add</a>(&<b>mut</b> coin_infos.coin_infos, coin_info);

    (<a href="coin.md#0x3_coin_BurnCapability">BurnCapability</a>&lt;CoinType&gt; {}, <a href="coin.md#0x3_coin_FreezeCapability">FreezeCapability</a>&lt;CoinType&gt; {}, <a href="coin.md#0x3_coin_MintCapability">MintCapability</a>&lt;CoinType&gt; {})
}
</code></pre>



</details>

<a name="0x3_coin_merge"></a>

## Function `merge`

"Merges" the two given coins.  The coin passed in as <code>dst_coin</code> will have a value equal
to the sum of the two coins (<code>dst_coin</code> and <code>source_coin</code>).


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_merge">merge</a>&lt;CoinType&gt;(dst_coin: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;, source_coin: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_merge">merge</a>&lt;CoinType&gt;(dst_coin: &<b>mut</b> <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;, source_coin: <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="coin.md#0x3_coin_Coin">Coin</a> { value } = source_coin;
    dst_coin.value = dst_coin.value + value;
}
</code></pre>



</details>

<a name="0x3_coin_mint"></a>

## Function `mint`

Mint new <code><a href="coin.md#0x3_coin_Coin">Coin</a></code> with capability.
The capability <code>_cap</code> should be passed as reference to <code><a href="coin.md#0x3_coin_MintCapability">MintCapability</a>&lt;CoinType&gt;</code>.
Returns minted <code><a href="coin.md#0x3_coin_Coin">Coin</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_mint">mint</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, amount: u256, _cap: &<a href="coin.md#0x3_coin_MintCapability">coin::MintCapability</a>&lt;CoinType&gt;): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_mint">mint</a>&lt;CoinType&gt;(
    ctx: &<b>mut</b> StorageContext,
    amount: u256,
    _cap: &<a href="coin.md#0x3_coin_MintCapability">MintCapability</a>&lt;CoinType&gt;,
): <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
   <b>let</b> coin_info = <a href="coin.md#0x3_coin_borrow_mut_coin_info">borrow_mut_coin_info</a>&lt;CoinType&gt;(ctx);
    coin_info.supply = coin_info.supply + amount;
    <b>let</b> coin_type_info = <a href="_type_of">type_info::type_of</a>&lt;CoinType&gt;();
    <a href="_emit">event::emit</a>&lt;<a href="coin.md#0x3_coin_MintEvent">MintEvent</a>&gt;(ctx, <a href="coin.md#0x3_coin_MintEvent">MintEvent</a> {
        coin_type_info,
        amount,
    });
    <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; { value: amount }
}
</code></pre>



</details>

<a name="0x3_coin_value"></a>

## Function `value`

Returns the <code>value</code> passed in <code><a href="coin.md#0x3_coin">coin</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_value">value</a>&lt;CoinType&gt;(<a href="coin.md#0x3_coin">coin</a>: &<a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_value">value</a>&lt;CoinType&gt;(<a href="coin.md#0x3_coin">coin</a>: &<a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;): u256 {
    <a href="coin.md#0x3_coin">coin</a>.value
}
</code></pre>



</details>

<a name="0x3_coin_zero"></a>

## Function `zero`

Create a new <code><a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;</code> with a value of <code>0</code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_zero">zero</a>&lt;CoinType&gt;(): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_zero">zero</a>&lt;CoinType&gt;(): <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
    <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
        value: 0
    }
}
</code></pre>



</details>

<a name="0x3_coin_initialize_coin_store"></a>

## Function `initialize_coin_store`



<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_initialize_coin_store">initialize_coin_store</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_initialize_coin_store">initialize_coin_store</a>&lt;CoinType&gt;(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="_global_move_to">account_storage::global_move_to</a>&lt;<a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt;&gt;(ctx, <a href="account.md#0x3_account">account</a>, <a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt; {
        <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">Coin</a> { value: 0 },
        frozen: <b>false</b>,
    });
}
</code></pre>



</details>

<a name="0x3_coin_exist_coin_store"></a>

## Function `exist_coin_store`



<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_exist_coin_store">exist_coin_store</a>&lt;CoinType&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_exist_coin_store">exist_coin_store</a>&lt;CoinType&gt;(ctx: &StorageContext, addr: <b>address</b>): bool {
    <a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt;&gt;(ctx, addr)
}
</code></pre>



</details>

<a name="0x3_coin_is_coin_store_frozen"></a>

## Function `is_coin_store_frozen`



<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_coin_store_frozen">is_coin_store_frozen</a>&lt;CoinType&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_coin_store_frozen">is_coin_store_frozen</a>&lt;CoinType&gt;(ctx: &StorageContext, addr: <b>address</b>): bool {
    <b>if</b> (<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt;&gt;(ctx, addr)) {
        <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt;&gt;(ctx, addr).frozen
    } <b>else</b> {
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x3_coin_extract_coin"></a>

## Function `extract_coin`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_extract_coin">extract_coin</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_extract_coin">extract_coin</a>&lt;CoinType&gt;(ctx: &<b>mut</b> StorageContext, addr: <b>address</b>, amount: u256): <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
    <b>let</b> coin_store = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt;&gt;(ctx, addr);
    <a href="coin.md#0x3_coin_extract">extract</a>&lt;CoinType&gt;(&<b>mut</b> coin_store.<a href="coin.md#0x3_coin">coin</a>, amount)
}
</code></pre>



</details>

<a name="0x3_coin_merge_coin"></a>

## Function `merge_coin`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_merge_coin">merge_coin</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_merge_coin">merge_coin</a>&lt;CoinType&gt;(ctx: &<b>mut</b> StorageContext, addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;) {
    <b>let</b> coin_store = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt;&gt;(ctx, addr);
    <a href="coin.md#0x3_coin_merge">merge</a>&lt;CoinType&gt;(&<b>mut</b> coin_store.<a href="coin.md#0x3_coin">coin</a>, <a href="coin.md#0x3_coin">coin</a>)
}
</code></pre>



</details>

<a name="0x3_coin_destroy_freeze_cap"></a>

## Function `destroy_freeze_cap`

Destroy a freeze capability. Freeze capability is dangerous and therefore should be destroyed if not used.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_destroy_freeze_cap">destroy_freeze_cap</a>&lt;CoinType&gt;(freeze_cap: <a href="coin.md#0x3_coin_FreezeCapability">coin::FreezeCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_destroy_freeze_cap">destroy_freeze_cap</a>&lt;CoinType&gt;(freeze_cap: <a href="coin.md#0x3_coin_FreezeCapability">FreezeCapability</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="coin.md#0x3_coin_FreezeCapability">FreezeCapability</a>&lt;CoinType&gt; {} = freeze_cap;
}
</code></pre>



</details>

<a name="0x3_coin_destroy_mint_cap"></a>

## Function `destroy_mint_cap`

Destroy a mint capability.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_destroy_mint_cap">destroy_mint_cap</a>&lt;CoinType&gt;(mint_cap: <a href="coin.md#0x3_coin_MintCapability">coin::MintCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_destroy_mint_cap">destroy_mint_cap</a>&lt;CoinType&gt;(mint_cap: <a href="coin.md#0x3_coin_MintCapability">MintCapability</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="coin.md#0x3_coin_MintCapability">MintCapability</a>&lt;CoinType&gt; {} = mint_cap;
}
</code></pre>



</details>

<a name="0x3_coin_destroy_burn_cap"></a>

## Function `destroy_burn_cap`

Destroy a burn capability.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_destroy_burn_cap">destroy_burn_cap</a>&lt;CoinType&gt;(burn_cap: <a href="coin.md#0x3_coin_BurnCapability">coin::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_destroy_burn_cap">destroy_burn_cap</a>&lt;CoinType&gt;(burn_cap: <a href="coin.md#0x3_coin_BurnCapability">BurnCapability</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="coin.md#0x3_coin_BurnCapability">BurnCapability</a>&lt;CoinType&gt; {} = burn_cap;
}
</code></pre>



</details>
