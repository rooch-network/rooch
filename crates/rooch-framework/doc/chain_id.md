
<a name="0x3_chain_id"></a>

# Module `0x3::chain_id`



-  [Resource `ChainID`](#0x3_chain_id_ChainID)
-  [Function `genesis_init`](#0x3_chain_id_genesis_init)
-  [Function `chain_id`](#0x3_chain_id_chain_id)


<pre><code><b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
</code></pre>



<a name="0x3_chain_id_ChainID"></a>

## Resource `ChainID`



<pre><code><b>struct</b> <a href="chain_id.md#0x3_chain_id_ChainID">ChainID</a> <b>has</b> <b>copy</b>, drop, store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_chain_id_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="chain_id.md#0x3_chain_id_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, genesis_account: &<a href="">signer</a>, <a href="chain_id.md#0x3_chain_id">chain_id</a>: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="chain_id.md#0x3_chain_id_genesis_init">genesis_init</a>(ctx: &<b>mut</b> StorageContext, genesis_account: &<a href="">signer</a>, <a href="chain_id.md#0x3_chain_id">chain_id</a>: u64){
    <b>let</b> <a href="chain_id.md#0x3_chain_id">chain_id</a> = <a href="chain_id.md#0x3_chain_id_ChainID">ChainID</a>{
        id: <a href="chain_id.md#0x3_chain_id">chain_id</a>
    };
    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, genesis_account, <a href="chain_id.md#0x3_chain_id">chain_id</a>);
}
</code></pre>



</details>

<a name="0x3_chain_id_chain_id"></a>

## Function `chain_id`



<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id">chain_id</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="chain_id.md#0x3_chain_id">chain_id</a>(ctx: &StorageContext) : u64 {
    <b>let</b> <a href="chain_id.md#0x3_chain_id">chain_id</a> = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="chain_id.md#0x3_chain_id_ChainID">ChainID</a>&gt;(ctx, @rooch_framework);
    <a href="chain_id.md#0x3_chain_id">chain_id</a>.id
}
</code></pre>



</details>
