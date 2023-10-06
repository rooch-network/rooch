
<a name="0x3_ethereum_light_client"></a>

# Module `0x3::ethereum_light_client`



-  [Struct `BlockHeader`](#0x3_ethereum_light_client_BlockHeader)
-  [Resource `BlockStore`](#0x3_ethereum_light_client_BlockStore)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_ethereum_light_client_genesis_init)
-  [Function `submit_new_block`](#0x3_ethereum_light_client_submit_new_block)
-  [Function `get_block`](#0x3_ethereum_light_client_get_block)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="ethereum_address.md#0x3_ethereum_address">0x3::ethereum_address</a>;
<b>use</b> <a href="timestamp.md#0x3_timestamp">0x3::timestamp</a>;
</code></pre>



<a name="0x3_ethereum_light_client_BlockHeader"></a>

## Struct `BlockHeader`



<pre><code><b>struct</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_BlockHeader">BlockHeader</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="">hash</a>: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>
 Hash of the block
</dd>
<dt>
<code>parent_hash: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>
 Hash of the parent
</dd>
<dt>
<code>uncles_hash: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>
 Hash of the uncles
</dd>
<dt>
<code>author: <a href="ethereum_address.md#0x3_ethereum_address_ETHAddress">ethereum_address::ETHAddress</a></code>
</dt>
<dd>
 Miner/author's address.
</dd>
<dt>
<code>state_root: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>
 State root hash
</dd>
<dt>
<code>transactions_root: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>
 Transactions root hash
</dd>
<dt>
<code>receipts_root: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>
 Transactions receipts root hash
</dd>
<dt>
<code>logs_bloom: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>
 Logs bloom
</dd>
<dt>
<code>difficulty: u256</code>
</dt>
<dd>
 Difficulty
</dd>
<dt>
<code>number: u64</code>
</dt>
<dd>
 Block number.
</dd>
<dt>
<code>gas_limit: u256</code>
</dt>
<dd>
 Gas Limit
</dd>
<dt>
<code>gas_used: u256</code>
</dt>
<dd>
 Gas Used
</dd>
<dt>
<code><a href="timestamp.md#0x3_timestamp">timestamp</a>: u256</code>
</dt>
<dd>
 Timestamp
</dd>
<dt>
<code>extra_data: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>
 Extra data
</dd>
</dl>


</details>

<a name="0x3_ethereum_light_client_BlockStore"></a>

## Resource `BlockStore`



<pre><code><b>struct</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_BlockStore">BlockStore</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>blocks: <a href="_Table">table::Table</a>&lt;u64, <a href="ethereum_light_client.md#0x3_ethereum_light_client_BlockHeader">ethereum_light_client::BlockHeader</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_ethereum_light_client_ErrorBlockNotFound"></a>



<pre><code><b>const</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_ErrorBlockNotFound">ErrorBlockNotFound</a>: u64 = 1;
</code></pre>



<a name="0x3_ethereum_light_client_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, genesis_account: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_genesis_init">genesis_init</a>(ctx: &<b>mut</b> StorageContext, genesis_account: &<a href="">signer</a>){
    <b>let</b> block_store = <a href="ethereum_light_client.md#0x3_ethereum_light_client_BlockStore">BlockStore</a>{
        blocks: <a href="_new">table::new</a>(ctx),
    };
    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, genesis_account, block_store);
}
</code></pre>



</details>

<a name="0x3_ethereum_light_client_submit_new_block"></a>

## Function `submit_new_block`

The relay server submit a new Ethereum block to the light client.


<pre><code><b>public</b> entry <b>fun</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_submit_new_block">submit_new_block</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, block_header_bytes: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_submit_new_block">submit_new_block</a>(ctx: &<b>mut</b> StorageContext, block_header_bytes: <a href="">vector</a>&lt;u8&gt;){
    <a href="ethereum_light_client.md#0x3_ethereum_light_client_process_block">process_block</a>(ctx, block_header_bytes);
}
</code></pre>



</details>

<a name="0x3_ethereum_light_client_get_block"></a>

## Function `get_block`

Get block via block_number


<pre><code><b>public</b> <b>fun</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_get_block">get_block</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, block_number: u64): &<a href="ethereum_light_client.md#0x3_ethereum_light_client_BlockHeader">ethereum_light_client::BlockHeader</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ethereum_light_client.md#0x3_ethereum_light_client_get_block">get_block</a>(ctx: &StorageContext, block_number: u64): &<a href="ethereum_light_client.md#0x3_ethereum_light_client_BlockHeader">BlockHeader</a>{
    <b>let</b> block_store = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="ethereum_light_client.md#0x3_ethereum_light_client_BlockStore">BlockStore</a>&gt;(ctx, @rooch_framework);
    <b>assert</b>!(<a href="_contains">table::contains</a>(&block_store.blocks, block_number), <a href="_invalid_argument">error::invalid_argument</a>(<a href="ethereum_light_client.md#0x3_ethereum_light_client_ErrorBlockNotFound">ErrorBlockNotFound</a>));
    <a href="_borrow">table::borrow</a>(&block_store.blocks, block_number)
}
</code></pre>



</details>
