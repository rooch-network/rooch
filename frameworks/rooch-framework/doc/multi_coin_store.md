
<a name="0x3_multi_coin_store"></a>

# Module `0x3::multi_coin_store`



-  [Struct `Balance`](#0x3_multi_coin_store_Balance)
-  [Resource `CoinStoreField`](#0x3_multi_coin_store_CoinStoreField)
-  [Resource `MultiCoinStore`](#0x3_multi_coin_store_MultiCoinStore)
-  [Struct `CreateEvent`](#0x3_multi_coin_store_CreateEvent)
-  [Struct `DepositEvent`](#0x3_multi_coin_store_DepositEvent)
-  [Struct `WithdrawEvent`](#0x3_multi_coin_store_WithdrawEvent)
-  [Struct `FreezeEvent`](#0x3_multi_coin_store_FreezeEvent)
-  [Struct `RemoveEvent`](#0x3_multi_coin_store_RemoveEvent)
-  [Constants](#@Constants_0)
-  [Function `exist_coin_store_field`](#0x3_multi_coin_store_exist_coin_store_field)
-  [Function `remove_coin_store_field`](#0x3_multi_coin_store_remove_coin_store_field)
-  [Function `balance`](#0x3_multi_coin_store_balance)
-  [Function `is_frozen`](#0x3_multi_coin_store_is_frozen)
-  [Function `withdraw`](#0x3_multi_coin_store_withdraw)
-  [Function `deposit`](#0x3_multi_coin_store_deposit)
-  [Function `transfer`](#0x3_multi_coin_store_transfer)
-  [Function `ensure_coin_type_has_key_and_store_ability`](#0x3_multi_coin_store_ensure_coin_type_has_key_and_store_ability)
-  [Function `freeze_coin_store`](#0x3_multi_coin_store_freeze_coin_store)
-  [Function `create_multi_coin_store`](#0x3_multi_coin_store_create_multi_coin_store)
-  [Function `borrow_mut_coin_store_internal`](#0x3_multi_coin_store_borrow_mut_coin_store_internal)
-  [Function `create_coin_store_field_if_not_exist`](#0x3_multi_coin_store_create_coin_store_field_if_not_exist)
-  [Function `withdraw_internal`](#0x3_multi_coin_store_withdraw_internal)
-  [Function `deposit_internal`](#0x3_multi_coin_store_deposit_internal)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::ability</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
</code></pre>



<a name="0x3_multi_coin_store_Balance"></a>

## Struct `Balance`

The Balance resource that stores the balance of a specific coin type.


<pre><code><b>struct</b> <a href="multi_coin_store.md#0x3_multi_coin_store_Balance">Balance</a> <b>has</b> store
</code></pre>



<a name="0x3_multi_coin_store_CoinStoreField"></a>

## Resource `CoinStoreField`

A holder of a specific coin types.
The non-generic coin store field that holds coins by coin_type


<pre><code><b>struct</b> <a href="multi_coin_store.md#0x3_multi_coin_store_CoinStoreField">CoinStoreField</a> <b>has</b> store, key
</code></pre>



<a name="0x3_multi_coin_store_MultiCoinStore"></a>

## Resource `MultiCoinStore`

The non-generic coin store that holds all coins for every account


<pre><code><b>struct</b> <a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">MultiCoinStore</a> <b>has</b> key
</code></pre>



<a name="0x3_multi_coin_store_CreateEvent"></a>

## Struct `CreateEvent`

Event emitted when a coin store is created.


<pre><code><b>struct</b> <a href="multi_coin_store.md#0x3_multi_coin_store_CreateEvent">CreateEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_multi_coin_store_DepositEvent"></a>

## Struct `DepositEvent`

Event emitted when some amount of a coin is deposited into a coin store.


<pre><code><b>struct</b> <a href="multi_coin_store.md#0x3_multi_coin_store_DepositEvent">DepositEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_multi_coin_store_WithdrawEvent"></a>

## Struct `WithdrawEvent`

Event emitted when some amount of a coin is withdrawn from a coin store.


<pre><code><b>struct</b> <a href="multi_coin_store.md#0x3_multi_coin_store_WithdrawEvent">WithdrawEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_multi_coin_store_FreezeEvent"></a>

## Struct `FreezeEvent`

Event emitted when a coin store is frozen or unfrozen.


<pre><code><b>struct</b> <a href="multi_coin_store.md#0x3_multi_coin_store_FreezeEvent">FreezeEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_multi_coin_store_RemoveEvent"></a>

## Struct `RemoveEvent`

Event emitted when a coin store is removed.


<pre><code><b>struct</b> <a href="multi_coin_store.md#0x3_multi_coin_store_RemoveEvent">RemoveEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_multi_coin_store_ErrorInsufficientBalance"></a>

Not enough balance to withdraw from CoinStore


<pre><code><b>const</b> <a href="multi_coin_store.md#0x3_multi_coin_store_ErrorInsufficientBalance">ErrorInsufficientBalance</a>: u64 = 4;
</code></pre>



<a name="0x3_multi_coin_store_ErrorCoinStoreIsFrozen"></a>

CoinStore is frozen. Coins cannot be deposited or withdrawn


<pre><code><b>const</b> <a href="multi_coin_store.md#0x3_multi_coin_store_ErrorCoinStoreIsFrozen">ErrorCoinStoreIsFrozen</a>: u64 = 2;
</code></pre>



<a name="0x3_multi_coin_store_ErrorCoinStoreNotFound"></a>

The CoinStore is not found in the global object store


<pre><code><b>const</b> <a href="multi_coin_store.md#0x3_multi_coin_store_ErrorCoinStoreNotFound">ErrorCoinStoreNotFound</a>: u64 = 1;
</code></pre>



<a name="0x3_multi_coin_store_ErrorCoinStoreTransferNotSupported"></a>

Transfer is not supported for CoinStore


<pre><code><b>const</b> <a href="multi_coin_store.md#0x3_multi_coin_store_ErrorCoinStoreTransferNotSupported">ErrorCoinStoreTransferNotSupported</a>: u64 = 5;
</code></pre>



<a name="0x3_multi_coin_store_ErrorCoinTypeAndStoreMismatch"></a>

The CoinType parameter and CoinType in CoinStore do not match


<pre><code><b>const</b> <a href="multi_coin_store.md#0x3_multi_coin_store_ErrorCoinTypeAndStoreMismatch">ErrorCoinTypeAndStoreMismatch</a>: u64 = 3;
</code></pre>



<a name="0x3_multi_coin_store_ErrorCoinTypeShouldHaveKeyAndStoreAbility"></a>

Coin type should have key and store ability


<pre><code><b>const</b> <a href="multi_coin_store.md#0x3_multi_coin_store_ErrorCoinTypeShouldHaveKeyAndStoreAbility">ErrorCoinTypeShouldHaveKeyAndStoreAbility</a>: u64 = 6;
</code></pre>



<a name="0x3_multi_coin_store_exist_coin_store_field"></a>

## Function `exist_coin_store_field`



<pre><code><b>public</b> <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_exist_coin_store_field">exist_coin_store_field</a>(coin_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">multi_coin_store::MultiCoinStore</a>&gt;, coin_type: <a href="_String">string::String</a>): bool
</code></pre>



<a name="0x3_multi_coin_store_remove_coin_store_field"></a>

## Function `remove_coin_store_field`



<pre><code><b>public</b> <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_remove_coin_store_field">remove_coin_store_field</a>(coin_store_object: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">multi_coin_store::MultiCoinStore</a>&gt;, coin_type: <a href="_String">string::String</a>): <a href="coin.md#0x3_coin_GenericCoin">coin::GenericCoin</a>
</code></pre>



<a name="0x3_multi_coin_store_balance"></a>

## Function `balance`



<pre><code><b>public</b> <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_balance">balance</a>(coin_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">multi_coin_store::MultiCoinStore</a>&gt;, coin_type: <a href="_String">string::String</a>): <a href="">u256</a>
</code></pre>



<a name="0x3_multi_coin_store_is_frozen"></a>

## Function `is_frozen`



<pre><code><b>public</b> <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_is_frozen">is_frozen</a>(coin_store_obj: &<a href="_Object">object::Object</a>&lt;<a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">multi_coin_store::MultiCoinStore</a>&gt;, coin_type: <a href="_String">string::String</a>): bool
</code></pre>



<a name="0x3_multi_coin_store_withdraw"></a>

## Function `withdraw`



<pre><code><b>public</b> <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_withdraw">withdraw</a>(coin_store_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">multi_coin_store::MultiCoinStore</a>&gt;, coin_type: <a href="_String">string::String</a>, amount: <a href="">u256</a>): <a href="coin.md#0x3_coin_GenericCoin">coin::GenericCoin</a>
</code></pre>



<a name="0x3_multi_coin_store_deposit"></a>

## Function `deposit`



<pre><code><b>public</b> <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_deposit">deposit</a>(coin_store_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">multi_coin_store::MultiCoinStore</a>&gt;, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_GenericCoin">coin::GenericCoin</a>)
</code></pre>



<a name="0x3_multi_coin_store_transfer"></a>

## Function `transfer`



<pre><code><b>public</b> <b>fun</b> <a href="transfer.md#0x3_transfer">transfer</a>(_coin_store_obj: <a href="_Object">object::Object</a>&lt;<a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">multi_coin_store::MultiCoinStore</a>&gt;, _owner: <b>address</b>)
</code></pre>



<a name="0x3_multi_coin_store_ensure_coin_type_has_key_and_store_ability"></a>

## Function `ensure_coin_type_has_key_and_store_ability`



<pre><code><b>public</b> <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_ensure_coin_type_has_key_and_store_ability">ensure_coin_type_has_key_and_store_ability</a>(coin_type: <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_multi_coin_store_freeze_coin_store"></a>

## Function `freeze_coin_store`

Freeze or Unfreeze a CoinStore to prevent withdraw and desposit


<pre><code><b>public</b> <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_freeze_coin_store">freeze_coin_store</a>(coin_store_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">multi_coin_store::MultiCoinStore</a>&gt;, coin_type: <a href="_String">string::String</a>, frozen: bool)
</code></pre>



<a name="0x3_multi_coin_store_create_multi_coin_store"></a>

## Function `create_multi_coin_store`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_create_multi_coin_store">create_multi_coin_store</a>(<a href="">account</a>: <b>address</b>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x3_multi_coin_store_borrow_mut_coin_store_internal"></a>

## Function `borrow_mut_coin_store_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_borrow_mut_coin_store_internal">borrow_mut_coin_store_internal</a>(object_id: <a href="_ObjectID">object::ObjectID</a>): &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">multi_coin_store::MultiCoinStore</a>&gt;
</code></pre>



<a name="0x3_multi_coin_store_create_coin_store_field_if_not_exist"></a>

## Function `create_coin_store_field_if_not_exist`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_create_coin_store_field_if_not_exist">create_coin_store_field_if_not_exist</a>(coin_store_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">multi_coin_store::MultiCoinStore</a>&gt;, coin_type: <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_multi_coin_store_withdraw_internal"></a>

## Function `withdraw_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_withdraw_internal">withdraw_internal</a>(coin_store_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">multi_coin_store::MultiCoinStore</a>&gt;, coin_type: <a href="_String">string::String</a>, amount: <a href="">u256</a>): <a href="coin.md#0x3_coin_GenericCoin">coin::GenericCoin</a>
</code></pre>



<a name="0x3_multi_coin_store_deposit_internal"></a>

## Function `deposit_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="multi_coin_store.md#0x3_multi_coin_store_deposit_internal">deposit_internal</a>(coin_store_obj: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="multi_coin_store.md#0x3_multi_coin_store_MultiCoinStore">multi_coin_store::MultiCoinStore</a>&gt;, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_GenericCoin">coin::GenericCoin</a>)
</code></pre>
