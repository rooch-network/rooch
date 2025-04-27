
<a name="0x3_coin_migration"></a>

# Module `0x3::coin_migration`

This module provides migration functionality to transition from the generic
coin store system (using CoinType) to the non-generic multi coin store system.
It helps migrate coin stores, balances, frozen states, and accept data.


-  [Struct `AccountMigrationEvent`](#0x3_coin_migration_AccountMigrationEvent)
-  [Struct `CoinStoreMigrationEvent`](#0x3_coin_migration_CoinStoreMigrationEvent)
-  [Resource `MigrationState`](#0x3_coin_migration_MigrationState)
-  [Resource `MigrationUpdateCap`](#0x3_coin_migration_MigrationUpdateCap)
-  [Constants](#@Constants_0)
-  [Function `dispatch_cap_entry`](#0x3_coin_migration_dispatch_cap_entry)
-  [Function `ensure_has_cap`](#0x3_coin_migration_ensure_has_cap)
-  [Function `cap_address`](#0x3_coin_migration_cap_address)
-  [Function `migrate_account_entry`](#0x3_coin_migration_migrate_account_entry)
-  [Function `update_migration_state_entry`](#0x3_coin_migration_update_migration_state_entry)
-  [Function `migration_state_id`](#0x3_coin_migration_migration_state_id)
-  [Function `is_account_migrated`](#0x3_coin_migration_is_account_migrated)
-  [Function `get_migration_stats`](#0x3_coin_migration_get_migration_stats)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="account_coin_store.md#0x3_account_coin_store">0x3::account_coin_store</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="coin_store.md#0x3_coin_store">0x3::coin_store</a>;
<b>use</b> <a href="generic_coin.md#0x3_generic_coin">0x3::generic_coin</a>;
<b>use</b> <a href="multi_coin_store.md#0x3_multi_coin_store">0x3::multi_coin_store</a>;
<b>use</b> <a href="onchain_config.md#0x3_onchain_config">0x3::onchain_config</a>;
</code></pre>



<a name="0x3_coin_migration_AccountMigrationEvent"></a>

## Struct `AccountMigrationEvent`

Event emitted when an account's coin stores are migrated


<pre><code><b>struct</b> <a href="coin_migration.md#0x3_coin_migration_AccountMigrationEvent">AccountMigrationEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_coin_migration_CoinStoreMigrationEvent"></a>

## Struct `CoinStoreMigrationEvent`

Event emitted when a specific coin store is migrated for an account


<pre><code><b>struct</b> <a href="coin_migration.md#0x3_coin_migration_CoinStoreMigrationEvent">CoinStoreMigrationEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_coin_migration_MigrationState"></a>

## Resource `MigrationState`

State tracking for migration progress


<pre><code><b>struct</b> <a href="coin_migration.md#0x3_coin_migration_MigrationState">MigrationState</a> <b>has</b> store, key
</code></pre>



<a name="0x3_coin_migration_MigrationUpdateCap"></a>

## Resource `MigrationUpdateCap`

MigrationUpdateCap is the capability for manager operations, such as update migration state.


<pre><code><b>struct</b> <a href="coin_migration.md#0x3_coin_migration_MigrationUpdateCap">MigrationUpdateCap</a> <b>has</b> store, key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_coin_migration_ErrorMigrationAlreadyDone"></a>

Migration is already done for an account


<pre><code><b>const</b> <a href="coin_migration.md#0x3_coin_migration_ErrorMigrationAlreadyDone">ErrorMigrationAlreadyDone</a>: u64 = 1;
</code></pre>



<a name="0x3_coin_migration_ErrorNoCap"></a>



<pre><code><b>const</b> <a href="coin_migration.md#0x3_coin_migration_ErrorNoCap">ErrorNoCap</a>: u64 = 3;
</code></pre>



<a name="0x3_coin_migration_ErrorNothingToMigrate"></a>

Nothing to migrate for the account


<pre><code><b>const</b> <a href="coin_migration.md#0x3_coin_migration_ErrorNothingToMigrate">ErrorNothingToMigrate</a>: u64 = 2;
</code></pre>



<a name="0x3_coin_migration_dispatch_cap_entry"></a>

## Function `dispatch_cap_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="coin_migration.md#0x3_coin_migration_dispatch_cap_entry">dispatch_cap_entry</a>(<a href="">account</a>: &<a href="">signer</a>, cap_address: <b>address</b>)
</code></pre>



<a name="0x3_coin_migration_ensure_has_cap"></a>

## Function `ensure_has_cap`



<pre><code><b>public</b> <b>fun</b> <a href="coin_migration.md#0x3_coin_migration_ensure_has_cap">ensure_has_cap</a>(<a href="">account</a>: &<a href="">signer</a>)
</code></pre>



<a name="0x3_coin_migration_cap_address"></a>

## Function `cap_address`



<pre><code><b>public</b> <b>fun</b> <a href="coin_migration.md#0x3_coin_migration_cap_address">cap_address</a>(): <b>address</b>
</code></pre>



<a name="0x3_coin_migration_migrate_account_entry"></a>

## Function `migrate_account_entry`

Entry function to migrate a specific account's coin stores
The coin type must be only key to compatiable with both the public(key+store) and private(key) coins
Can be called by arbitrary user


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_migration.md#0x3_coin_migration_migrate_account_entry">migrate_account_entry</a>&lt;CoinType: key&gt;(_account: &<a href="">signer</a>, addr: <b>address</b>)
</code></pre>



<a name="0x3_coin_migration_update_migration_state_entry"></a>

## Function `update_migration_state_entry`

Entry function to update migration state for a specific account
Only called by the cap account to update migrate states


<pre><code><b>public</b> entry <b>fun</b> <a href="coin_migration.md#0x3_coin_migration_update_migration_state_entry">update_migration_state_entry</a>(<a href="">account</a>: &<a href="">signer</a>, addr: <b>address</b>)
</code></pre>



<a name="0x3_coin_migration_migration_state_id"></a>

## Function `migration_state_id`



<pre><code><b>public</b> <b>fun</b> <a href="coin_migration.md#0x3_coin_migration_migration_state_id">migration_state_id</a>(): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x3_coin_migration_is_account_migrated"></a>

## Function `is_account_migrated`

Check if an account has already been migrated


<pre><code><b>public</b> <b>fun</b> <a href="coin_migration.md#0x3_coin_migration_is_account_migrated">is_account_migrated</a>(addr: <b>address</b>): bool
</code></pre>



<a name="0x3_coin_migration_get_migration_stats"></a>

## Function `get_migration_stats`

Get migration statistics


<pre><code><b>public</b> <b>fun</b> <a href="coin_migration.md#0x3_coin_migration_get_migration_stats">get_migration_stats</a>(): u64
</code></pre>
