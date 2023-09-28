
<a name="0x3_genesis"></a>

# Module `0x3::genesis`



-  [Struct `GenesisContext`](#0x3_genesis_GenesisContext)
-  [Constants](#@Constants_0)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="account.md#0x3_account">0x3::account</a>;
<b>use</b> <a href="address_mapping.md#0x3_address_mapping">0x3::address_mapping</a>;
<b>use</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry">0x3::auth_validator_registry</a>;
<b>use</b> <a href="builtin_validators.md#0x3_builtin_validators">0x3::builtin_validators</a>;
<b>use</b> <a href="chain_id.md#0x3_chain_id">0x3::chain_id</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client">0x3::ethereum_light_client</a>;
<b>use</b> <a href="gas_coin.md#0x3_gas_coin">0x3::gas_coin</a>;
<b>use</b> <a href="timestamp.md#0x3_timestamp">0x3::timestamp</a>;
<b>use</b> <a href="transaction_fee.md#0x3_transaction_fee">0x3::transaction_fee</a>;
</code></pre>



<a name="0x3_genesis_GenesisContext"></a>

## Struct `GenesisContext`

GenesisContext is a genesis init parameters in the TxContext.


<pre><code><b>struct</b> <a href="genesis.md#0x3_genesis_GenesisContext">GenesisContext</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="chain_id.md#0x3_chain_id">chain_id</a>: u64</code>
</dt>
<dd>

</dd>
<dt>
<code><a href="timestamp.md#0x3_timestamp">timestamp</a>: u64</code>
</dt>
<dd>
 genesis timestamp in microseconds
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_genesis_ErrorGenesisInit"></a>



<pre><code><b>const</b> <a href="genesis.md#0x3_genesis_ErrorGenesisInit">ErrorGenesisInit</a>: u64 = 1;
</code></pre>
