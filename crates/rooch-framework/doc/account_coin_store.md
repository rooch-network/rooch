
<a name="0x3_account_coin_store"></a>

# Module `0x3::account_coin_store`



-  [Resource `AutoAcceptCoins`](#0x3_account_coin_store_AutoAcceptCoins)
-  [Resource `CoinStores`](#0x3_account_coin_store_CoinStores)
-  [Struct `AcceptCoinEvent`](#0x3_account_coin_store_AcceptCoinEvent)
-  [Struct `DepositEvent`](#0x3_account_coin_store_DepositEvent)
-  [Struct `WithdrawEvent`](#0x3_account_coin_store_WithdrawEvent)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_account_coin_store_genesis_init)
-  [Function `init_account_coin_stores`](#0x3_account_coin_store_init_account_coin_stores)
-  [Function `balance`](#0x3_account_coin_store_balance)
-  [Function `coin_store_id`](#0x3_account_coin_store_coin_store_id)
-  [Function `coin_stores_handle`](#0x3_account_coin_store_coin_stores_handle)
-  [Function `is_accept_coin`](#0x3_account_coin_store_is_accept_coin)
-  [Function `can_auto_accept_coin`](#0x3_account_coin_store_can_auto_accept_coin)
-  [Function `do_accept_coin`](#0x3_account_coin_store_do_accept_coin)
-  [Function `set_auto_accept_coin`](#0x3_account_coin_store_set_auto_accept_coin)
-  [Function `withdraw`](#0x3_account_coin_store_withdraw)
-  [Function `deposit`](#0x3_account_coin_store_deposit)
-  [Function `transfer`](#0x3_account_coin_store_transfer)
-  [Function `exist_account_coin_store`](#0x3_account_coin_store_exist_account_coin_store)
-  [Function `is_account_coin_store_frozen`](#0x3_account_coin_store_is_account_coin_store_frozen)
-  [Function `withdraw_extend`](#0x3_account_coin_store_withdraw_extend)
-  [Function `deposit_extend`](#0x3_account_coin_store_deposit_extend)
-  [Function `transfer_extend`](#0x3_account_coin_store_transfer_extend)
-  [Function `accept_coin_entry`](#0x3_account_coin_store_accept_coin_entry)
-  [Function `enable_auto_accept_coin_entry`](#0x3_account_coin_store_enable_auto_accept_coin_entry)
-  [Function `disable_auto_accept_coin_entry`](#0x3_account_coin_store_disable_auto_accept_coin_entry)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::object_ref</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="coin_store.md#0x3_coin_store">0x3::coin_store</a>;
</code></pre>



<a name="0x3_account_coin_store_AutoAcceptCoins"></a>

## Resource `AutoAcceptCoins`

A resource that holds the AutoAcceptCoin config for all accounts.
The main scenario is that the user can actively turn off the AutoAcceptCoin setting to avoid automatically receiving Coin


<pre><code><b>struct</b> <a href="account_coin_store.md#0x3_account_coin_store_AutoAcceptCoins">AutoAcceptCoins</a> <b>has</b> key
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

<a name="0x3_account_coin_store_CoinStores"></a>

## Resource `CoinStores`

A resource that holds all the ObjectRef<CoinStore> for account.


<pre><code><b>struct</b> <a href="account_coin_store.md#0x3_account_coin_store_CoinStores">CoinStores</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_stores: <a href="_Table">table::Table</a>&lt;<a href="_String">string::String</a>, <a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="coin_store.md#0x3_coin_store_CoinStore">coin_store::CoinStore</a>&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_account_coin_store_AcceptCoinEvent"></a>

## Struct `AcceptCoinEvent`

Event for auto accept coin set


<pre><code><b>struct</b> <a href="account_coin_store.md#0x3_account_coin_store_AcceptCoinEvent">AcceptCoinEvent</a> <b>has</b> drop, store
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

<a name="0x3_account_coin_store_DepositEvent"></a>

## Struct `DepositEvent`

Event emitted when some amount of a coin is deposited into an account.


<pre><code><b>struct</b> <a href="account_coin_store.md#0x3_account_coin_store_DepositEvent">DepositEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_store_id: <a href="_ObjectID">object::ObjectID</a></code>
</dt>
<dd>
 The id of the coin store that was deposited to
</dd>
<dt>
<code>coin_type: <a href="_String">string::String</a></code>
</dt>
<dd>
 The type of the coin that was sent
</dd>
<dt>
<code>amount: u256</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_account_coin_store_WithdrawEvent"></a>

## Struct `WithdrawEvent`

Event emitted when some amount of a coin is withdrawn from an account.


<pre><code><b>struct</b> <a href="account_coin_store.md#0x3_account_coin_store_WithdrawEvent">WithdrawEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_store_id: <a href="_ObjectID">object::ObjectID</a></code>
</dt>
<dd>
 The id of the coin store that was withdrawn from
</dd>
<dt>
<code>coin_type: <a href="_String">string::String</a></code>
</dt>
<dd>
 The type of the coin that was sent
</dd>
<dt>
<code>amount: u256</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_account_coin_store_ErrorAccountNotAcceptCoin"></a>

Account hasn't accept <code>CoinType</code>


<pre><code><b>const</b> <a href="account_coin_store.md#0x3_account_coin_store_ErrorAccountNotAcceptCoin">ErrorAccountNotAcceptCoin</a>: u64 = 1;
</code></pre>



<a name="0x3_account_coin_store_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, genesis_account: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_genesis_init">genesis_init</a>(ctx: &<b>mut</b> Context, genesis_account: &<a href="">signer</a>) {
    <b>let</b> auto_accepted_coins = <a href="account_coin_store.md#0x3_account_coin_store_AutoAcceptCoins">AutoAcceptCoins</a> {
        auto_accept_coins: <a href="_new">table::new</a>&lt;<b>address</b>, bool&gt;(ctx),
    };
    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, genesis_account, auto_accepted_coins);
}
</code></pre>



</details>

<a name="0x3_account_coin_store_init_account_coin_stores"></a>

## Function `init_account_coin_stores`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_init_account_coin_stores">init_account_coin_stores</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_init_account_coin_stores">init_account_coin_stores</a>(ctx: &<b>mut</b> Context, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>){
    <b>let</b> coin_stores = <a href="account_coin_store.md#0x3_account_coin_store_CoinStores">CoinStores</a> {
        coin_stores: <a href="_new">table::new</a>&lt;<a href="_String">string::String</a>, ObjectRef&lt;CoinStore&gt;&gt;(ctx),
    };
    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, <a href="account.md#0x3_account">account</a>, coin_stores);
}
</code></pre>



</details>

<a name="0x3_account_coin_store_balance"></a>

## Function `balance`

Returns the balance of <code>addr</code> for provided <code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_balance">balance</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_balance">balance</a>&lt;CoinType: key&gt;(ctx: &Context, addr: <b>address</b>): u256 {
    <b>let</b> coin_store_id_option = <a href="account_coin_store.md#0x3_account_coin_store_coin_store_id">coin_store_id</a>&lt;CoinType&gt;(ctx, addr);
    <b>if</b> (<a href="_is_some">option::is_some</a>(&coin_store_id_option)) {
        <b>let</b> coin_store_id = <a href="_extract">option::extract</a>(&<b>mut</b> coin_store_id_option);
        <a href="coin_store.md#0x3_coin_store_get_balance_with_id">coin_store::get_balance_with_id</a>(ctx, coin_store_id)
    } <b>else</b> {
        0u256
    }
}
</code></pre>



</details>

<a name="0x3_account_coin_store_coin_store_id"></a>

## Function `coin_store_id`

Return the account CoinStore object id for addr


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_coin_store_id">coin_store_id</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_coin_store_id">coin_store_id</a>&lt;CoinType: key&gt;(ctx: &Context, addr: <b>address</b>): Option&lt;ObjectID&gt; {
    <b>if</b> (<a href="account_coin_store.md#0x3_account_coin_store_exist_account_coin_store">exist_account_coin_store</a>&lt;CoinType&gt;(ctx, addr)) {
        <b>let</b> coin_stores = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="account_coin_store.md#0x3_account_coin_store_CoinStores">CoinStores</a>&gt;(ctx, addr);
        <b>let</b> coin_type = <a href="_type_name">type_info::type_name</a>&lt;CoinType&gt;();
        <b>let</b> coin_store_ref = <a href="_borrow">table::borrow</a>(&coin_stores.coin_stores, coin_type);
        <a href="_some">option::some</a>(<a href="_id">object_ref::id</a>(coin_store_ref))
    } <b>else</b> {
        <a href="_none">option::none</a>&lt;ObjectID&gt;()
    }
}
</code></pre>



</details>

<a name="0x3_account_coin_store_coin_stores_handle"></a>

## Function `coin_stores_handle`

Return CoinStores table handle for addr


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_coin_stores_handle">coin_stores_handle</a>(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_coin_stores_handle">coin_stores_handle</a>(ctx: &Context, addr: <b>address</b>): Option&lt;ObjectID&gt; {
    <b>if</b> (<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account_coin_store.md#0x3_account_coin_store_CoinStores">CoinStores</a>&gt;(ctx, addr))
    {
        <b>let</b> coin_stores = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="account_coin_store.md#0x3_account_coin_store_CoinStores">CoinStores</a>&gt;(ctx, addr);
        <a href="_some">option::some</a>(*<a href="_handle">table::handle</a>(&coin_stores.coin_stores))
    } <b>else</b> {
        <a href="_none">option::none</a>&lt;ObjectID&gt;()
    }
}
</code></pre>



</details>

<a name="0x3_account_coin_store_is_accept_coin"></a>

## Function `is_accept_coin`

Return whether the account at <code>addr</code> accept <code>Coin</code> type coins


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_is_accept_coin">is_accept_coin</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_is_accept_coin">is_accept_coin</a>&lt;CoinType: key&gt;(ctx: &Context, addr: <b>address</b>): bool {
    <b>if</b> (<a href="account_coin_store.md#0x3_account_coin_store_can_auto_accept_coin">can_auto_accept_coin</a>(ctx, addr)) {
        <b>true</b>
    } <b>else</b> {
        <a href="account_coin_store.md#0x3_account_coin_store_exist_account_coin_store">exist_account_coin_store</a>&lt;CoinType&gt;(ctx, addr)
    }
}
</code></pre>



</details>

<a name="0x3_account_coin_store_can_auto_accept_coin"></a>

## Function `can_auto_accept_coin`

Check whether the address can auto accept coin.
Default is true if absent


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_can_auto_accept_coin">can_auto_accept_coin</a>(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_can_auto_accept_coin">can_auto_accept_coin</a>(ctx: &Context, addr: <b>address</b>): bool {
    <b>let</b> auto_accept_coins = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="account_coin_store.md#0x3_account_coin_store_AutoAcceptCoins">AutoAcceptCoins</a>&gt;(ctx, @rooch_framework);
    <b>if</b> (<a href="_contains">table::contains</a>&lt;<b>address</b>, bool&gt;(&auto_accept_coins.auto_accept_coins, addr)) {
        <b>return</b> *<a href="_borrow">table::borrow</a>&lt;<b>address</b>, bool&gt;(&auto_accept_coins.auto_accept_coins, addr)
    };
    <b>true</b>
}
</code></pre>



</details>

<a name="0x3_account_coin_store_do_accept_coin"></a>

## Function `do_accept_coin`

Add a balance of <code>Coin</code> type to the sending account.
If user turns off AutoAcceptCoin, call this method to receive the corresponding Coin


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_do_accept_coin">do_accept_coin</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_do_accept_coin">do_accept_coin</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> Context, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    <a href="account_coin_store.md#0x3_account_coin_store_ensure_coin_store_bypass_auto_accept_flag">ensure_coin_store_bypass_auto_accept_flag</a>&lt;CoinType&gt;(ctx, addr);
}
</code></pre>



</details>

<a name="0x3_account_coin_store_set_auto_accept_coin"></a>

## Function `set_auto_accept_coin`

Configure whether auto-accept coins.


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_set_auto_accept_coin">set_auto_accept_coin</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, enable: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_set_auto_accept_coin">set_auto_accept_coin</a>(ctx: &<b>mut</b> Context, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, enable: bool)  {
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    <b>let</b> auto_accept_coins = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="account_coin_store.md#0x3_account_coin_store_AutoAcceptCoins">AutoAcceptCoins</a>&gt;(ctx, @rooch_framework);
    <a href="_upsert">table::upsert</a>&lt;<b>address</b>, bool&gt;(&<b>mut</b> auto_accept_coins.auto_accept_coins, addr, enable);

    <a href="_emit">event::emit</a>&lt;<a href="account_coin_store.md#0x3_account_coin_store_AcceptCoinEvent">AcceptCoinEvent</a>&gt;(ctx,
        <a href="account_coin_store.md#0x3_account_coin_store_AcceptCoinEvent">AcceptCoinEvent</a> {
            enable,
        },
    );
}
</code></pre>



</details>

<a name="0x3_account_coin_store_withdraw"></a>

## Function `withdraw`

Withdraw specifed <code>amount</code> of coin <code>CoinType</code> from the signing account.
This public entry function requires the <code>CoinType</code> to have <code>key</code> and <code>store</code> abilities.


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_withdraw">withdraw</a>&lt;CoinType: store, key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_withdraw">withdraw</a>&lt;CoinType: key + store&gt;(
    ctx: &<b>mut</b> Context,
    <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>,
    amount: u256,
): Coin&lt;CoinType&gt; {
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    <a href="account_coin_store.md#0x3_account_coin_store_withdraw_internal">withdraw_internal</a>&lt;CoinType&gt;(ctx, addr, amount)
}
</code></pre>



</details>

<a name="0x3_account_coin_store_deposit"></a>

## Function `deposit`

Deposit the coin into the recipient's account and emit an event.
This public entry function requires the <code>CoinType</code> to have <code>key</code> and <code>store</code> abilities.


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_deposit">deposit</a>&lt;CoinType: store, key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_deposit">deposit</a>&lt;CoinType: key + store&gt;(ctx: &<b>mut</b> Context, addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: Coin&lt;CoinType&gt;) {
    <a href="account_coin_store.md#0x3_account_coin_store_deposit_internal">deposit_internal</a>(ctx, addr, <a href="coin.md#0x3_coin">coin</a>);
}
</code></pre>



</details>

<a name="0x3_account_coin_store_transfer"></a>

## Function `transfer`

Transfer <code>amount</code> of coins <code>CoinType</code> from <code>from</code> to <code><b>to</b></code>.
Any account and module can call this function to transfer coins, the <code>CoinType</code> must have <code>key</code> and <code>store</code> abilities.


<pre><code><b>public</b> <b>fun</b> <a href="transfer.md#0x3_transfer">transfer</a>&lt;CoinType: store, key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, from: &<a href="">signer</a>, <b>to</b>: <b>address</b>, amount: u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transfer.md#0x3_transfer">transfer</a>&lt;CoinType: key + store&gt;(
    ctx: &<b>mut</b> Context,
    from: &<a href="">signer</a>,
    <b>to</b>: <b>address</b>,
    amount: u256,
) {
    <b>let</b> from_addr = <a href="_address_of">signer::address_of</a>(from);
    <a href="account_coin_store.md#0x3_account_coin_store_transfer_internal">transfer_internal</a>&lt;CoinType&gt;(ctx, from_addr, <b>to</b>, amount);
}
</code></pre>



</details>

<a name="0x3_account_coin_store_exist_account_coin_store"></a>

## Function `exist_account_coin_store`



<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_exist_account_coin_store">exist_account_coin_store</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_exist_account_coin_store">exist_account_coin_store</a>&lt;CoinType: key&gt;(ctx: &Context, addr: <b>address</b>): bool {
    <b>if</b> (<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account_coin_store.md#0x3_account_coin_store_CoinStores">CoinStores</a>&gt;(ctx, addr)) {
        <b>let</b> coin_stores = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="account_coin_store.md#0x3_account_coin_store_CoinStores">CoinStores</a>&gt;(ctx, addr);
        <b>let</b> coin_type = <a href="_type_name">type_info::type_name</a>&lt;CoinType&gt;();
        <a href="_contains">table::contains</a>(&coin_stores.coin_stores, coin_type)
    } <b>else</b> {
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x3_account_coin_store_is_account_coin_store_frozen"></a>

## Function `is_account_coin_store_frozen`



<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_is_account_coin_store_frozen">is_account_coin_store_frozen</a>&lt;CoinType: key&gt;(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_is_account_coin_store_frozen">is_account_coin_store_frozen</a>&lt;CoinType: key&gt;(ctx: &Context, addr: <b>address</b>): bool {
    <b>if</b> (<a href="account_coin_store.md#0x3_account_coin_store_exist_account_coin_store">exist_account_coin_store</a>&lt;CoinType&gt;(ctx, addr)) {
        <b>let</b> <a href="coin_store.md#0x3_coin_store">coin_store</a> = <a href="account_coin_store.md#0x3_account_coin_store_borrow_account_coin_store">borrow_account_coin_store</a>&lt;CoinType&gt;(ctx, addr);
        <a href="coin_store.md#0x3_coin_store_is_frozen">coin_store::is_frozen</a>(<a href="coin_store.md#0x3_coin_store">coin_store</a>)
    } <b>else</b> {
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x3_account_coin_store_withdraw_extend"></a>

## Function `withdraw_extend`

Withdraw specifed <code>amount</code> of coin <code>CoinType</code> from any addr, this function does not check the Coin <code>frozen</code> attribute
This function is only called by the <code>CoinType</code> module, for the developer to extend custom withdraw logic


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_withdraw_extend">withdraw_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, addr: <b>address</b>, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_withdraw_extend">withdraw_extend</a>&lt;CoinType: key&gt;(
    ctx: &<b>mut</b> Context,
    addr: <b>address</b>,
    amount: u256,
): Coin&lt;CoinType&gt; {
    <a href="account_coin_store.md#0x3_account_coin_store_withdraw_internal">withdraw_internal</a>&lt;CoinType&gt;(ctx, addr, amount)
}
</code></pre>



</details>

<a name="0x3_account_coin_store_deposit_extend"></a>

## Function `deposit_extend`

Deposit the coin into the recipient's account and emit an event.
This function is only called by the <code>CoinType</code> module, for the developer to extend custom deposit logic


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_deposit_extend">deposit_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_deposit_extend">deposit_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> Context, addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: Coin&lt;CoinType&gt;) {
    <a href="account_coin_store.md#0x3_account_coin_store_deposit_internal">deposit_internal</a>(ctx, addr, <a href="coin.md#0x3_coin">coin</a>);
}
</code></pre>



</details>

<a name="0x3_account_coin_store_transfer_extend"></a>

## Function `transfer_extend`

Transfer <code>amount</code> of coins <code>CoinType</code> from <code>from</code> to <code><b>to</b></code>.
This function is only called by the <code>CoinType</code> module, for the developer to extend custom transfer logic


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_transfer_extend">transfer_extend</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, from: <b>address</b>, <b>to</b>: <b>address</b>, amount: u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_transfer_extend">transfer_extend</a>&lt;CoinType: key&gt;(
    ctx: &<b>mut</b> Context,
    from: <b>address</b>,
    <b>to</b>: <b>address</b>,
    amount: u256,
) {
    <a href="account_coin_store.md#0x3_account_coin_store_transfer_internal">transfer_internal</a>&lt;CoinType&gt;(ctx, from, <b>to</b>, amount);
}
</code></pre>



</details>

<a name="0x3_account_coin_store_accept_coin_entry"></a>

## Function `accept_coin_entry`

Creating a resource that stores balance of <code>CoinType</code> on user's account.
Required if user wants to start accepting deposits of <code>CoinType</code> in his account.


<pre><code><b>public</b> entry <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_accept_coin_entry">accept_coin_entry</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_accept_coin_entry">accept_coin_entry</a>&lt;CoinType: key&gt;(ctx: &<b>mut</b> Context, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="account_coin_store.md#0x3_account_coin_store_do_accept_coin">do_accept_coin</a>&lt;CoinType&gt;(ctx, <a href="account.md#0x3_account">account</a>)
}
</code></pre>



</details>

<a name="0x3_account_coin_store_enable_auto_accept_coin_entry"></a>

## Function `enable_auto_accept_coin_entry`

Enable account's auto-accept-coin feature.
The script function is reenterable.


<pre><code><b>public</b> entry <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_enable_auto_accept_coin_entry">enable_auto_accept_coin_entry</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_enable_auto_accept_coin_entry">enable_auto_accept_coin_entry</a>(ctx: &<b>mut</b> Context, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="account_coin_store.md#0x3_account_coin_store_set_auto_accept_coin">set_auto_accept_coin</a>(ctx, <a href="account.md#0x3_account">account</a>, <b>true</b>)
}
</code></pre>



</details>

<a name="0x3_account_coin_store_disable_auto_accept_coin_entry"></a>

## Function `disable_auto_accept_coin_entry`

Disable account's auto-accept-coin feature.
The script function is reenterable.


<pre><code><b>public</b> entry <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_disable_auto_accept_coin_entry">disable_auto_accept_coin_entry</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account_coin_store.md#0x3_account_coin_store_disable_auto_accept_coin_entry">disable_auto_accept_coin_entry</a>(ctx: &<b>mut</b> Context, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="account_coin_store.md#0x3_account_coin_store_set_auto_accept_coin">set_auto_accept_coin</a>(ctx, <a href="account.md#0x3_account">account</a>, <b>false</b>);
}
</code></pre>



</details>
