
<a name="0x2_genesis"></a>

# Module `0x2::genesis`



-  [Struct `GenesisContext`](#0x2_genesis_GenesisContext)
-  [Constants](#@Constants_0)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="features.md#0x2_features">0x2::features</a>;
<b>use</b> <a href="gas_schedule.md#0x2_gas_schedule">0x2::gas_schedule</a>;
<b>use</b> <a href="module_store.md#0x2_module_store">0x2::module_store</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="signer.md#0x2_signer">0x2::signer</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_genesis_GenesisContext"></a>

## Struct `GenesisContext`

GenesisContext is a genesis init parameters in the TxContext.


<pre><code><b>struct</b> <a href="genesis.md#0x2_genesis_GenesisContext">GenesisContext</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_genesis_ErrorGenesisInit"></a>



<pre><code><b>const</b> <a href="genesis.md#0x2_genesis_ErrorGenesisInit">ErrorGenesisInit</a>: u64 = 1;
</code></pre>
