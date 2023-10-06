
<a name="0x3_coin"></a>

# Module `0x3::coin`

This module provides the foundation for typesafe Coins.


-  [Struct `Coin`](#0x3_coin_Coin)
-  [Resource `CoinStore`](#0x3_coin_CoinStore)
-  [Resource `CoinInfo`](#0x3_coin_CoinInfo)
-  [Resource `CoinInfos`](#0x3_coin_CoinInfos)
-  [Resource `AutoAcceptCoins`](#0x3_coin_AutoAcceptCoins)
-  [Resource `CoinStores`](#0x3_coin_CoinStores)
-  [Struct `DepositEvent`](#0x3_coin_DepositEvent)
-  [Struct `WithdrawEvent`](#0x3_coin_WithdrawEvent)
-  [Struct `AcceptCoinEvent`](#0x3_coin_AcceptCoinEvent)
-  [Struct `MintEvent`](#0x3_coin_MintEvent)
-  [Struct `BurnEvent`](#0x3_coin_BurnEvent)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_coin_genesis_init)
-  [Function `init_account_coin_store`](#0x3_coin_init_account_coin_store)
-  [Function `balance`](#0x3_coin_balance)
-  [Function `is_registered`](#0x3_coin_is_registered)
-  [Function `name`](#0x3_coin_name)
-  [Function `symbol`](#0x3_coin_symbol)
-  [Function `decimals`](#0x3_coin_decimals)
-  [Function `supply`](#0x3_coin_supply)
-  [Function `is_same_coin`](#0x3_coin_is_same_coin)
-  [Function `coin_store_handle`](#0x3_coin_coin_store_handle)
-  [Function `coin_info_handle`](#0x3_coin_coin_info_handle)
-  [Function `is_account_accept_coin`](#0x3_coin_is_account_accept_coin)
-  [Function `can_auto_accept_coin`](#0x3_coin_can_auto_accept_coin)
-  [Function `do_accept_coin`](#0x3_coin_do_accept_coin)
-  [Function `set_auto_accept_coin`](#0x3_coin_set_auto_accept_coin)
-  [Function `withdraw`](#0x3_coin_withdraw)
-  [Function `deposit`](#0x3_coin_deposit)
-  [Function `transfer`](#0x3_coin_transfer)
-  [Function `destroy_zero`](#0x3_coin_destroy_zero)
-  [Function `extract`](#0x3_coin_extract)
-  [Function `extract_all`](#0x3_coin_extract_all)
-  [Function `merge`](#0x3_coin_merge)
-  [Function `value`](#0x3_coin_value)
-  [Function `zero`](#0x3_coin_zero)
-  [Function `exist_coin_store`](#0x3_coin_exist_coin_store)
-  [Function `is_coin_store_frozen`](#0x3_coin_is_coin_store_frozen)
-  [Function `register_extend`](#0x3_coin_register_extend)
-  [Function `mint_extend`](#0x3_coin_mint_extend)
-  [Function `withdraw_extend`](#0x3_coin_withdraw_extend)
-  [Function `deposit_extend`](#0x3_coin_deposit_extend)
-  [Function `transfer_extend`](#0x3_coin_transfer_extend)
-  [Function `burn_extend`](#0x3_coin_burn_extend)
-  [Function `freeze_coin_store_extend`](#0x3_coin_freeze_coin_store_extend)
-  [Function `unfreeze_coin_store_extend`](#0x3_coin_unfreeze_coin_store_extend)
-  [Function `accept_coin_entry`](#0x3_coin_accept_coin_entry)
-  [Function `enable_auto_accept_coin_entry`](#0x3_coin_enable_auto_accept_coin_entry)
-  [Function `disable_auto_accept_coin_entry`](#0x3_coin_disable_auto_accept_coin_entry)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object_id</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="">0x2::type_table</a>;
</code></pre>



<a name="0x3_coin_Coin"></a>

## Struct `Coin`

Core data structures
Main structure representing a coin/coin in an account's custody.
Note the <code>CoinType</code> must have <code>key</code> ability.
if the <code>CoinType</code> has <code>store</code> ability, the <code><a href="coin.md#0x3_coin_Coin">Coin</a></code> is a public coin, the user can operate it directly by coin module's function.
Otherwise, the <code><a href="coin.md#0x3_coin_Coin">Coin</a></code> is a private coin, the user can only operate it by <code>CoinType</code> module's function.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType: key&gt; <b>has</b> store
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



<pre><code><b>struct</b> <a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType: key&gt; <b>has</b> key
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


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_CoinInfo">CoinInfo</a>&lt;CoinType: key&gt; <b>has</b> key
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

<a name="0x3_coin_AutoAcceptCoins"></a>

## Resource `AutoAcceptCoins`

A resource that holds the AutoAcceptCoin config for all accounts.
The main scenario is that the user can actively turn off the AutoAcceptCoin setting to avoid automatically receiving Coin


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_AutoAcceptCoins">AutoAcceptCoins</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>auto_accept_coins: <a href="_Table">table::Table</a>&lt;<b>address</b>, bool&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_coin_CoinStores"></a>

## Resource `CoinStores`

A resource that holds all the CoinStore for account.
Default Deposit Coin no longer depends on accept coin


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_CoinStores">CoinStores</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_stores: <a href="_TypeTable">type_table::TypeTable</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_coin_DepositEvent"></a>

## Struct `DepositEvent`

Event emitted when some amount of a coin is deposited into an account.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_DepositEvent">DepositEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_type_info: <a href="_TypeInfo">type_info::TypeInfo</a></code>
</dt>
<dd>
 The type info for the coin that was sent
</dd>
<dt>
<code>amount: u256</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_coin_WithdrawEvent"></a>

## Struct `WithdrawEvent`

Event emitted when some amount of a coin is withdrawn from an account.


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_WithdrawEvent">WithdrawEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_type_info: <a href="_TypeInfo">type_info::TypeInfo</a></code>
</dt>
<dd>
 The type info for the coin that was sent
</dd>
<dt>
<code>amount: u256</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_coin_AcceptCoinEvent"></a>

## Struct `AcceptCoinEvent`

Event for auto accept coin set


<pre><code><b>struct</b> <a href="coin.md#0x3_coin_AcceptCoinEvent">AcceptCoinEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>enable: bool</code>
</dt>
<dd>
 auto accept coin config
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



<a name="0x3_coin_ErrorAccountNotAcceptCoin"></a>

Account hasn't accept <code>CoinType</code>


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorAccountNotAcceptCoin">ErrorAccountNotAcceptCoin</a>: u64 = 8;
</code></pre>



<a name="0x3_coin_ErrorAccountWithCoinFrozen"></a>

CoinStore is frozen. Coins cannot be deposited or withdrawn


<pre><code><b>const</b> <a href="coin.md#0x3_coin_ErrorAccountWithCoinFrozen">ErrorAccountWithCoinFrozen</a>: u64 = 7;
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

Not enough coins to complete transaction


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



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, genesis_account: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_genesis_init">genesis_init</a>(ctx: &<b>mut</b> StorageContext, genesis_account: &<a href="">signer</a>) {
    <b>let</b> coin_infos = <a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a> {
        coin_infos: <a href="_new">type_table::new</a>(ctx),
    };
    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, genesis_account, coin_infos);

    <b>let</b> auto_accepted_coins = <a href="coin.md#0x3_coin_AutoAcceptCoins">AutoAcceptCoins</a> {
        auto_accept_coins: <a href="_new">table::new</a>&lt;<b>address</b>, bool&gt;(ctx),
    };
    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, genesis_account, auto_accepted_coins);
}
</code></pre>



</details>

<a name="0x3_coin_init_account_coin_store"></a>

## Function `init_account_coin_store`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_init_account_coin_store">init_account_coin_store</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="coin.md#0x3_coin_init_account_coin_store">init_account_coin_store</a>(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>){
    <b>let</b> coin_stores = <a href="coin.md#0x3_coin_CoinStores">CoinStores</a> {
        coin_stores: <a href="_new">type_table::new</a>(ctx),
    };
    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, <a href="account.md#0x3_account">account</a>, coin_stores);
}
</code></pre>



</details>

<a name="0x3_coin_balance"></a>

## Function `balance`

Returns the balance of <code>addr</code> for provided <code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_balance">balance</a>&lt;CoinType: key&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_balance">balance</a>&lt;CoinType: key&gt;(ctx: &StorageContext, addr: <b>address</b>): u256 {
    <b>if</b> (<a href="coin.md#0x3_coin_exist_coin_store">exist_coin_store</a>&lt;CoinType&gt;(ctx, addr)) {
        <a href="coin.md#0x3_coin_borrow_coin_store">borrow_coin_store</a>&lt;CoinType&gt;(ctx, addr).<a href="coin.md#0x3_coin">coin</a>.value
    } <b>else</b> {
        0u256
    }
}
</code></pre>



</details>

<a name="0x3_coin_is_registered"></a>

## Function `is_registered`

Returns <code><b>true</b></code> if the type <code>CoinType</code> is an registered coin.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_registered">is_registered</a>&lt;CoinType: key&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_registered">is_registered</a>&lt;CoinType: key&gt;(ctx: &StorageContext): bool {
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


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_name">name</a>&lt;CoinType: key&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_name">name</a>&lt;CoinType: key&gt;(ctx: &StorageContext): <a href="_String">string::String</a> {
    <a href="coin.md#0x3_coin_borrow_coin_info">borrow_coin_info</a>&lt;CoinType&gt;(ctx).name
}
</code></pre>



</details>

<a name="0x3_coin_symbol"></a>

## Function `symbol`

Returns the symbol of the coin, usually a shorter version of the name.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_symbol">symbol</a>&lt;CoinType: key&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_symbol">symbol</a>&lt;CoinType: key&gt;(ctx: &StorageContext): <a href="_String">string::String</a> {
    <a href="coin.md#0x3_coin_borrow_coin_info">borrow_coin_info</a>&lt;CoinType&gt;(ctx).symbol
}
</code></pre>



</details>

<a name="0x3_coin_decimals"></a>

## Function `decimals`

Returns the number of decimals used to get its user representation.
For example, if <code>decimals</code> equals <code>2</code>, a balance of <code>505</code> coins should
be displayed to a user as <code>5.05</code> (<code>505 / 10 ** 2</code>).


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_decimals">decimals</a>&lt;CoinType: key&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_decimals">decimals</a>&lt;CoinType: key&gt;(ctx: &StorageContext): u8 {
    <a href="coin.md#0x3_coin_borrow_coin_info">borrow_coin_info</a>&lt;CoinType&gt;(ctx).decimals
}
</code></pre>



</details>

<a name="0x3_coin_supply"></a>

## Function `supply`

Returns the amount of coin in existence.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_supply">supply</a>&lt;CoinType: key&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_supply">supply</a>&lt;CoinType: key&gt;(ctx: &StorageContext): u256 {
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

<a name="0x3_coin_coin_store_handle"></a>

## Function `coin_store_handle`

Return coin store handle for addr


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_coin_store_handle">coin_store_handle</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="_ObjectID">object_id::ObjectID</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_coin_store_handle">coin_store_handle</a>(ctx: &StorageContext, addr: <b>address</b>): Option&lt;ObjectID&gt; {
    <b>if</b> (<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin.md#0x3_coin_CoinStores">CoinStores</a>&gt;(ctx, addr))
    {
        <b>let</b> coin_stores = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin.md#0x3_coin_CoinStores">CoinStores</a>&gt;(ctx, addr);
        <a href="_some">option::some</a>(*<a href="_handle">type_table::handle</a>(&coin_stores.coin_stores))
    } <b>else</b> {
        <a href="_none">option::none</a>&lt;ObjectID&gt;()
    }
}
</code></pre>



</details>

<a name="0x3_coin_coin_info_handle"></a>

## Function `coin_info_handle`

Return coin info handle


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_coin_info_handle">coin_info_handle</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): <a href="_ObjectID">object_id::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_coin_info_handle">coin_info_handle</a>(ctx: &StorageContext): ObjectID {
    // <a href="coin.md#0x3_coin">coin</a> info ensured via the Genesis transaction, so it should always exist
    <b>assert</b>!(<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a>&gt;(ctx, @rooch_framework), <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorCoinInfosNotFound">ErrorCoinInfosNotFound</a>));
    <b>let</b> coin_infos = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a>&gt;(ctx, @rooch_framework);
    *<a href="_handle">type_table::handle</a>(&coin_infos.coin_infos)
}
</code></pre>



</details>

<a name="0x3_coin_is_account_accept_coin"></a>

## Function `is_account_accept_coin`

Return whether the account at <code>addr</code> accept <code><a href="coin.md#0x3_coin_Coin">Coin</a></code> type coins


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_account_accept_coin">is_account_accept_coin</a>&lt;CoinType: key&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_account_accept_coin">is_account_accept_coin</a>&lt;CoinType: key&gt;(ctx: &StorageContext, addr: <b>address</b>): bool {
    <b>if</b> (<a href="coin.md#0x3_coin_can_auto_accept_coin">can_auto_accept_coin</a>(ctx, addr)) {
        <b>true</b>
    } <b>else</b> {
        <a href="coin.md#0x3_coin_exist_coin_store">exist_coin_store</a>&lt;CoinType&gt;(ctx, addr)
    }
}
</code></pre>



</details>

<a name="0x3_coin_can_auto_accept_coin"></a>

## Function `can_auto_accept_coin`

Check whether the address can auto accept coin.
Default is true if absent


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_can_auto_accept_coin">can_auto_accept_coin</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_can_auto_accept_coin">can_auto_accept_coin</a>(ctx: &StorageContext, addr: <b>address</b>): bool {
    <b>if</b> (<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin.md#0x3_coin_AutoAcceptCoins">AutoAcceptCoins</a>&gt;(ctx, @rooch_framework)) {
        <b>let</b> auto_accept_coins = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin.md#0x3_coin_AutoAcceptCoins">AutoAcceptCoins</a>&gt;(ctx, @rooch_framework);
        <b>if</b> (<a href="_contains">table::contains</a>&lt;<b>address</b>, bool&gt;(&auto_accept_coins.auto_accept_coins, addr)) {
            <b>return</b> *<a href="_borrow">table::borrow</a>&lt;<b>address</b>, bool&gt;(&auto_accept_coins.auto_accept_coins, addr)
        }
    };
    <b>true</b>
}
</code></pre>



</details>

<a name="0x3_coin_do_accept_coin"></a>

## Function `do_accept_coin`

Add a balance of <code><a href="coin.md#0x3_coin_Coin">Coin</a></code> type to the sending account.
If user turns off AutoAcceptCoin, call this method to receive the corresponding Coin


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_do_accept_coin">do_accept_coin</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_do_accept_coin">do_accept_coin</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    <a href="coin.md#0x3_coin_ensure_coin_store_pass_auto_accept_flag">ensure_coin_store_pass_auto_accept_flag</a>&lt;CoinType&gt;(ctx, addr);
}
</code></pre>



</details>

<a name="0x3_coin_set_auto_accept_coin"></a>

## Function `set_auto_accept_coin`

Configure whether auto-accept coins.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_set_auto_accept_coin">set_auto_accept_coin</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, enable: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_set_auto_accept_coin">set_auto_accept_coin</a>(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, enable: bool)  {
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    <b>let</b> auto_accept_coins = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="coin.md#0x3_coin_AutoAcceptCoins">AutoAcceptCoins</a>&gt;(ctx, @rooch_framework);
    <a href="_upsert">table::upsert</a>&lt;<b>address</b>, bool&gt;(&<b>mut</b> auto_accept_coins.auto_accept_coins, addr, enable);

    <a href="_emit">event::emit</a>&lt;<a href="coin.md#0x3_coin_AcceptCoinEvent">AcceptCoinEvent</a>&gt;(ctx,
        <a href="coin.md#0x3_coin_AcceptCoinEvent">AcceptCoinEvent</a> {
            enable,
        },
    );
}
</code></pre>



</details>

<a name="0x3_coin_withdraw"></a>

## Function `withdraw`

Withdraw specifed <code>amount</code> of coin <code>CoinType</code> from the signing account.
This public entry function requires the <code>CoinType</code> to have <code>key</code> and <code>store</code> abilities.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_withdraw">withdraw</a>&lt;CoinType: store, key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_withdraw">withdraw</a>&lt;CoinType: key + store&gt;(
    ctx: &<b>mut</b> StorageContext,
    <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>,
    amount: u256,
): <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    // the <a href="coin.md#0x3_coin">coin</a> `frozen` only affect user withdraw, does not affect `withdraw_extend`.
    <a href="coin.md#0x3_coin_check_coin_store_frozen">check_coin_store_frozen</a>&lt;CoinType&gt;(ctx, addr);
    <a href="coin.md#0x3_coin_withdraw_internal">withdraw_internal</a>&lt;CoinType&gt;(ctx, addr, amount)
}
</code></pre>



</details>

<a name="0x3_coin_deposit"></a>

## Function `deposit`

Deposit the coin into the recipient's account and emit an event.
This public entry function requires the <code>CoinType</code> to have <code>key</code> and <code>store</code> abilities.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_deposit">deposit</a>&lt;CoinType: store, key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_deposit">deposit</a>&lt;CoinType: key + store&gt;(ctx: &<b>mut</b> StorageContext, addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;) {
    <a href="coin.md#0x3_coin_check_coin_store_frozen">check_coin_store_frozen</a>&lt;CoinType&gt;(ctx, addr);
    <a href="coin.md#0x3_coin_deposit_internal">deposit_internal</a>(ctx, addr, <a href="coin.md#0x3_coin">coin</a>);
}
</code></pre>



</details>

<a name="0x3_coin_transfer"></a>

## Function `transfer`

Transfer <code>amount</code> of coins <code>CoinType</code> from <code>from</code> to <code><b>to</b></code>.
Any account and module can call this function to transfer coins, the <code>CoinType</code> must have <code>key</code> and <code>store</code> abilities.


<pre><code><b>public</b> <b>fun</b> <a href="transfer.md#0x3_transfer">transfer</a>&lt;CoinType: store, key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, from: &<a href="">signer</a>, <b>to</b>: <b>address</b>, amount: u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transfer.md#0x3_transfer">transfer</a>&lt;CoinType: key + store&gt;(
    ctx: &<b>mut</b> StorageContext,
    from: &<a href="">signer</a>,
    <b>to</b>: <b>address</b>,
    amount: u256,
) {
    <b>let</b> from_addr = <a href="_address_of">signer::address_of</a>(from);
    <a href="coin.md#0x3_coin_check_coin_store_frozen">check_coin_store_frozen</a>&lt;CoinType&gt;(ctx, from_addr);
    <a href="coin.md#0x3_coin_check_coin_store_frozen">check_coin_store_frozen</a>&lt;CoinType&gt;(ctx, <b>to</b>);
    <a href="coin.md#0x3_coin_transfer_internal">transfer_internal</a>&lt;CoinType&gt;(ctx, from_addr, <b>to</b>, amount);
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

<a name="0x3_coin_exist_coin_store"></a>

## Function `exist_coin_store`



<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_exist_coin_store">exist_coin_store</a>&lt;CoinType: key&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_exist_coin_store">exist_coin_store</a>&lt;CoinType: key&gt;(ctx: &StorageContext, addr: <b>address</b>): bool {
    <b>if</b> (<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="coin.md#0x3_coin_CoinStores">CoinStores</a>&gt;(ctx, addr)) {
        <b>let</b> coin_stores = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="coin.md#0x3_coin_CoinStores">CoinStores</a>&gt;(ctx, addr);
        <a href="_contains">type_table::contains</a>&lt;<a href="coin.md#0x3_coin_CoinStore">CoinStore</a>&lt;CoinType&gt;&gt;(&coin_stores.coin_stores)
    } <b>else</b> {
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x3_coin_is_coin_store_frozen"></a>

## Function `is_coin_store_frozen`



<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_coin_store_frozen">is_coin_store_frozen</a>&lt;CoinType: key&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_is_coin_store_frozen">is_coin_store_frozen</a>&lt;CoinType: key&gt;(ctx: &StorageContext, addr: <b>address</b>): bool {
    <b>if</b> (<a href="coin.md#0x3_coin_exist_coin_store">exist_coin_store</a>&lt;CoinType&gt;(ctx, addr)) {
        <a href="coin.md#0x3_coin_borrow_coin_store">borrow_coin_store</a>&lt;CoinType&gt;(ctx, addr).frozen
    } <b>else</b> {
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x3_coin_register_extend"></a>

## Function `register_extend`

Creates a new Coin with given <code>CoinType</code>
The given signer also becomes the account hosting the information about the coin
(name, supply, etc.).
This function is protected by <code>private_generics</code>, so it can only be called by the <code>CoinType</code> module.


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_register_extend">register_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, name: <a href="_String">string::String</a>, symbol: <a href="_String">string::String</a>, decimals: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_register_extend">register_extend</a>&lt;CoinType: key&gt;(
    ctx: &<b>mut</b> StorageContext,
    name: <a href="_String">string::String</a>,
    symbol: <a href="_String">string::String</a>,
    decimals: u8,
){

    <b>let</b> coin_infos = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="coin.md#0x3_coin_CoinInfos">CoinInfos</a>&gt;(ctx, @rooch_framework);

    <b>assert</b>!(
        !<a href="_contains">type_table::contains</a>&lt;<a href="coin.md#0x3_coin_CoinInfo">CoinInfo</a>&lt;CoinType&gt;&gt;(&coin_infos.coin_infos),
        <a href="_already_exists">error::already_exists</a>(<a href="coin.md#0x3_coin_ErrorCoinInfoAlreadyRegistered">ErrorCoinInfoAlreadyRegistered</a>),
    );

    <b>assert</b>!(<a href="_length">string::length</a>(&name) &lt;= <a href="coin.md#0x3_coin_MAX_COIN_NAME_LENGTH">MAX_COIN_NAME_LENGTH</a>, <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorCoinNameTooLong">ErrorCoinNameTooLong</a>));
    <b>assert</b>!(<a href="_length">string::length</a>(&symbol) &lt;= <a href="coin.md#0x3_coin_MAX_COIN_SYMBOL_LENGTH">MAX_COIN_SYMBOL_LENGTH</a>, <a href="_invalid_argument">error::invalid_argument</a>(<a href="coin.md#0x3_coin_ErrorCoinSymbolTooLong">ErrorCoinSymbolTooLong</a>));

    <b>let</b> coin_info = <a href="coin.md#0x3_coin_CoinInfo">CoinInfo</a>&lt;CoinType&gt; {
        name,
        symbol,
        decimals,
        supply: 0u256,
    };
    <a href="_add">type_table::add</a>(&<b>mut</b> coin_infos.coin_infos, coin_info);
}
</code></pre>



</details>

<a name="0x3_coin_mint_extend"></a>

## Function `mint_extend`

Mint new <code><a href="coin.md#0x3_coin_Coin">Coin</a></code>, this function is only called by the <code>CoinType</code> module, for the developer to extend custom mint logic


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_mint_extend">mint_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_mint_extend">mint_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> StorageContext,amount: u256) : <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
    <a href="coin.md#0x3_coin_mint_internal">mint_internal</a>&lt;CoinType&gt;(ctx, amount)
}
</code></pre>



</details>

<a name="0x3_coin_withdraw_extend"></a>

## Function `withdraw_extend`

Withdraw specifed <code>amount</code> of coin <code>CoinType</code> from any addr, this function does not check the Coin <code>frozen</code> attribute
This function is only called by the <code>CoinType</code> module, for the developer to extend custom withdraw logic


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_withdraw_extend">withdraw_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_withdraw_extend">withdraw_extend</a>&lt;CoinType: key&gt;(
    ctx: &<b>mut</b> StorageContext,
    addr: <b>address</b>,
    amount: u256,
): <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt; {
    <a href="coin.md#0x3_coin_withdraw_internal">withdraw_internal</a>&lt;CoinType&gt;(ctx, addr, amount)
}
</code></pre>



</details>

<a name="0x3_coin_deposit_extend"></a>

## Function `deposit_extend`

Deposit the coin into the recipient's account and emit an event.
This function is only called by the <code>CoinType</code> module, for the developer to extend custom deposit logic


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_deposit_extend">deposit_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_deposit_extend">deposit_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> StorageContext, addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;) {
    <a href="coin.md#0x3_coin_deposit_internal">deposit_internal</a>(ctx, addr, <a href="coin.md#0x3_coin">coin</a>);
}
</code></pre>



</details>

<a name="0x3_coin_transfer_extend"></a>

## Function `transfer_extend`

Transfer <code>amount</code> of coins <code>CoinType</code> from <code>from</code> to <code><b>to</b></code>.
This function is only called by the <code>CoinType</code> module, for the developer to extend custom transfer logic


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_transfer_extend">transfer_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, from: <b>address</b>, <b>to</b>: <b>address</b>, amount: u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_transfer_extend">transfer_extend</a>&lt;CoinType: key&gt;(
    ctx: &<b>mut</b> StorageContext,
    from: <b>address</b>,
    <b>to</b>: <b>address</b>,
    amount: u256,
) {
    <a href="coin.md#0x3_coin_transfer_internal">transfer_internal</a>&lt;CoinType&gt;(ctx, from, <b>to</b>, amount);
}
</code></pre>



</details>

<a name="0x3_coin_burn_extend"></a>

## Function `burn_extend`

Burn <code><a href="coin.md#0x3_coin">coin</a></code>
This function is only called by the <code>CoinType</code> module, for the developer to extend custom burn logic


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_burn_extend">burn_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_burn_extend">burn_extend</a>&lt;CoinType: key&gt;(
    ctx: &<b>mut</b> StorageContext,
    <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">Coin</a>&lt;CoinType&gt;,
) {
    <a href="coin.md#0x3_coin_burn_internal">burn_internal</a>(ctx, <a href="coin.md#0x3_coin">coin</a>)
}
</code></pre>



</details>

<a name="0x3_coin_freeze_coin_store_extend"></a>

## Function `freeze_coin_store_extend`

Freeze a CoinStore to prevent transfers


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_freeze_coin_store_extend">freeze_coin_store_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_freeze_coin_store_extend">freeze_coin_store_extend</a>&lt;CoinType: key&gt;(
    ctx: &<b>mut</b> StorageContext,
    addr: <b>address</b>,
) {
    <a href="coin.md#0x3_coin_ensure_coin_store">ensure_coin_store</a>&lt;CoinType&gt;(ctx, addr);
    <b>let</b> coin_store = <a href="coin.md#0x3_coin_borrow_mut_coin_store">borrow_mut_coin_store</a>&lt;CoinType&gt;(ctx, addr);
    coin_store.frozen = <b>true</b>;
}
</code></pre>



</details>

<a name="0x3_coin_unfreeze_coin_store_extend"></a>

## Function `unfreeze_coin_store_extend`

Unfreeze a CoinStore to allow transfers


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_unfreeze_coin_store_extend">unfreeze_coin_store_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x3_coin_unfreeze_coin_store_extend">unfreeze_coin_store_extend</a>&lt;CoinType: key&gt;(
    ctx: &<b>mut</b> StorageContext,
    addr: <b>address</b>,
) {
    <a href="coin.md#0x3_coin_ensure_coin_store">ensure_coin_store</a>&lt;CoinType&gt;(ctx, addr);
    <b>let</b> coin_store = <a href="coin.md#0x3_coin_borrow_mut_coin_store">borrow_mut_coin_store</a>&lt;CoinType&gt;(ctx, addr);
    coin_store.frozen = <b>false</b>;
}
</code></pre>



</details>

<a name="0x3_coin_accept_coin_entry"></a>

## Function `accept_coin_entry`

Creating a resource that stores balance of <code>CoinType</code> on user's account.
Required if user wants to start accepting deposits of <code>CoinType</code> in his account.


<pre><code><b>public</b> entry <b>fun</b> <a href="coin.md#0x3_coin_accept_coin_entry">accept_coin_entry</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin.md#0x3_coin_accept_coin_entry">accept_coin_entry</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="coin.md#0x3_coin_do_accept_coin">do_accept_coin</a>&lt;CoinType&gt;(ctx, <a href="account.md#0x3_account">account</a>)
}
</code></pre>



</details>

<a name="0x3_coin_enable_auto_accept_coin_entry"></a>

## Function `enable_auto_accept_coin_entry`

Enable account's auto-accept-coin feature.
The script function is reenterable.


<pre><code><b>public</b> entry <b>fun</b> <a href="coin.md#0x3_coin_enable_auto_accept_coin_entry">enable_auto_accept_coin_entry</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin.md#0x3_coin_enable_auto_accept_coin_entry">enable_auto_accept_coin_entry</a>(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="coin.md#0x3_coin_set_auto_accept_coin">set_auto_accept_coin</a>(ctx, <a href="account.md#0x3_account">account</a>, <b>true</b>)
}
</code></pre>



</details>

<a name="0x3_coin_disable_auto_accept_coin_entry"></a>

## Function `disable_auto_accept_coin_entry`

Disable account's auto-accept-coin feature.
The script function is reenterable.


<pre><code><b>public</b> entry <b>fun</b> <a href="coin.md#0x3_coin_disable_auto_accept_coin_entry">disable_auto_accept_coin_entry</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="coin.md#0x3_coin_disable_auto_accept_coin_entry">disable_auto_accept_coin_entry</a>(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="coin.md#0x3_coin_set_auto_accept_coin">set_auto_accept_coin</a>(ctx, <a href="account.md#0x3_account">account</a>, <b>false</b>);
}
</code></pre>



</details>
