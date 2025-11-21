
<a name="0x3_genesis"></a>

# Module `0x3::genesis`



-  [Struct `GenesisContext`](#0x3_genesis_GenesisContext)
-  [Constants](#@Constants_0)
-  [Function `init_v23`](#0x3_genesis_init_v23)
-  [Function `init_v25`](#0x3_genesis_init_v25)
-  [Function `genesis_init_rgas_locked_unit`](#0x3_genesis_genesis_init_rgas_locked_unit)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::core_addresses</a>;
<b>use</b> <a href="">0x2::module_store</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="account.md#0x3_account">0x3::account</a>;
<b>use</b> <a href="account_coin_store.md#0x3_account_coin_store">0x3::account_coin_store</a>;
<b>use</b> <a href="address_mapping.md#0x3_address_mapping">0x3::address_mapping</a>;
<b>use</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry">0x3::auth_validator_registry</a>;
<b>use</b> <a href="bitcoin_address.md#0x3_bitcoin_address">0x3::bitcoin_address</a>;
<b>use</b> <a href="builtin_validators.md#0x3_builtin_validators">0x3::builtin_validators</a>;
<b>use</b> <a href="chain_id.md#0x3_chain_id">0x3::chain_id</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="did.md#0x3_did">0x3::did</a>;
<b>use</b> <a href="gas_coin.md#0x3_gas_coin">0x3::gas_coin</a>;
<b>use</b> <a href="onchain_config.md#0x3_onchain_config">0x3::onchain_config</a>;
<b>use</b> <a href="payment_channel.md#0x3_payment_channel">0x3::payment_channel</a>;
<b>use</b> <a href="transaction_fee.md#0x3_transaction_fee">0x3::transaction_fee</a>;
</code></pre>



<a name="0x3_genesis_GenesisContext"></a>

## Struct `GenesisContext`

GenesisContext is a genesis init parameters in the TxContext.


<pre><code><b>struct</b> <a href="genesis.md#0x3_genesis_GenesisContext">GenesisContext</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_genesis_ErrorGenesisInit"></a>



<pre><code><b>const</b> <a href="genesis.md#0x3_genesis_ErrorGenesisInit">ErrorGenesisInit</a>: u64 = 1;
</code></pre>



<a name="0x3_genesis_GENESIS_INIT_GAS_AMOUNT"></a>



<pre><code><b>const</b> <a href="genesis.md#0x3_genesis_GENESIS_INIT_GAS_AMOUNT">GENESIS_INIT_GAS_AMOUNT</a>: <a href="">u256</a> = 50000000000000000;
</code></pre>



<a name="0x3_genesis_GENESIS_INIT_RGAS_LOCKED_UNIT"></a>



<pre><code><b>const</b> <a href="genesis.md#0x3_genesis_GENESIS_INIT_RGAS_LOCKED_UNIT">GENESIS_INIT_RGAS_LOCKED_UNIT</a>: <a href="">u256</a> = 10000000000;
</code></pre>



<a name="0x3_genesis_init_v23"></a>

## Function `init_v23`



<pre><code><b>public</b> entry <b>fun</b> <a href="genesis.md#0x3_genesis_init_v23">init_v23</a>()
</code></pre>



<a name="0x3_genesis_init_v25"></a>

## Function `init_v25`



<pre><code><b>public</b> entry <b>fun</b> <a href="genesis.md#0x3_genesis_init_v25">init_v25</a>(sender: &<a href="">signer</a>)
</code></pre>



<a name="0x3_genesis_genesis_init_rgas_locked_unit"></a>

## Function `genesis_init_rgas_locked_unit`



<pre><code><b>public</b> <b>fun</b> <a href="genesis.md#0x3_genesis_genesis_init_rgas_locked_unit">genesis_init_rgas_locked_unit</a>(): <a href="">u256</a>
</code></pre>
