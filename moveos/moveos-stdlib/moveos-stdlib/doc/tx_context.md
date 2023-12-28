
<a name="0x2_tx_context"></a>

# Module `0x2::tx_context`



-  [Struct `GasPaymentAccount`](#0x2_tx_context_GasPaymentAccount)
-  [Struct `TxContext`](#0x2_tx_context_TxContext)
-  [Struct `ModuleUpgradeFlag`](#0x2_tx_context_ModuleUpgradeFlag)
-  [Constants](#@Constants_0)
-  [Function `sender`](#0x2_tx_context_sender)
-  [Function `sequence_number`](#0x2_tx_context_sequence_number)
-  [Function `max_gas_amount`](#0x2_tx_context_max_gas_amount)
-  [Function `fresh_address`](#0x2_tx_context_fresh_address)
-  [Function `derive_id`](#0x2_tx_context_derive_id)
-  [Function `tx_hash`](#0x2_tx_context_tx_hash)
-  [Function `add`](#0x2_tx_context_add)
-  [Function `get`](#0x2_tx_context_get)
-  [Function `contains`](#0x2_tx_context_contains)
-  [Function `tx_meta`](#0x2_tx_context_tx_meta)
-  [Function `tx_gas_payment_account`](#0x2_tx_context_tx_gas_payment_account)
-  [Function `tx_result`](#0x2_tx_context_tx_result)
-  [Function `set_module_upgrade_flag`](#0x2_tx_context_set_module_upgrade_flag)
-  [Function `drop`](#0x2_tx_context_drop)


<pre><code><b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="copyable_any.md#0x2_copyable_any">0x2::copyable_any</a>;
<b>use</b> <a href="simple_map.md#0x2_simple_map">0x2::simple_map</a>;
<b>use</b> <a href="tx_meta.md#0x2_tx_meta">0x2::tx_meta</a>;
<b>use</b> <a href="tx_result.md#0x2_tx_result">0x2::tx_result</a>;
<b>use</b> <a href="type_info.md#0x2_type_info">0x2::type_info</a>;
</code></pre>



<a name="0x2_tx_context_GasPaymentAccount"></a>

## Struct `GasPaymentAccount`

An account address for paying gas during the transaction validation stage.


<pre><code><b>struct</b> <a href="tx_context.md#0x2_tx_context_GasPaymentAccount">GasPaymentAccount</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x2_tx_context_TxContext"></a>

## Struct `TxContext`

Information about the transaction currently being executed.
This cannot be constructed by a transaction--it is a privileged object created by
the VM, stored in a <code>Context</code> and passed in to the entrypoint of the transaction as <code>&<b>mut</b> Context</code>.


<pre><code><b>struct</b> <a href="tx_context.md#0x2_tx_context_TxContext">TxContext</a>
</code></pre>



<a name="0x2_tx_context_ModuleUpgradeFlag"></a>

## Struct `ModuleUpgradeFlag`



<pre><code><b>struct</b> <a href="tx_context.md#0x2_tx_context_ModuleUpgradeFlag">ModuleUpgradeFlag</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_tx_context_ErrorInvalidContext"></a>



<pre><code><b>const</b> <a href="tx_context.md#0x2_tx_context_ErrorInvalidContext">ErrorInvalidContext</a>: u64 = 1;
</code></pre>



<a name="0x2_tx_context_sender"></a>

## Function `sender`

Return the address of the user that signed the current transaction


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_sender">sender</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <b>address</b>
</code></pre>



<a name="0x2_tx_context_sequence_number"></a>

## Function `sequence_number`

Return the sequence number of the current transaction


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_sequence_number">sequence_number</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): u64
</code></pre>



<a name="0x2_tx_context_max_gas_amount"></a>

## Function `max_gas_amount`

Return the max gas to be used


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_max_gas_amount">max_gas_amount</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): u64
</code></pre>



<a name="0x2_tx_context_fresh_address"></a>

## Function `fresh_address`

Generate a new unique address,


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_fresh_address">fresh_address</a>(ctx: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <b>address</b>
</code></pre>



<a name="0x2_tx_context_derive_id"></a>

## Function `derive_id`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_derive_id">derive_id</a>(<a href="">hash</a>: <a href="">vector</a>&lt;u8&gt;, index: u64): <b>address</b>
</code></pre>



<a name="0x2_tx_context_tx_hash"></a>

## Function `tx_hash`

Return the hash of the current transaction


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_tx_hash">tx_hash</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_tx_context_add"></a>

## Function `add`

Add a value to the context map


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_add">add</a>&lt;T: <b>copy</b>, drop, store&gt;(self: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>, value: T)
</code></pre>



<a name="0x2_tx_context_get"></a>

## Function `get`

Get a value from the context map


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_get">get</a>&lt;T: <b>copy</b>, drop, store&gt;(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="_Option">option::Option</a>&lt;T&gt;
</code></pre>



<a name="0x2_tx_context_contains"></a>

## Function `contains`

Check if the key is in the context map


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_contains">contains</a>&lt;T: <b>copy</b>, drop, store&gt;(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): bool
</code></pre>



<a name="0x2_tx_context_tx_meta"></a>

## Function `tx_meta`

Get the transaction meta data
The TxMeta is writed by the VM before the transaction execution.
The meta data is only available when executing or validating a transaction, otherwise abort(eg. readonly function call).


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_meta.md#0x2_tx_meta">tx_meta</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="tx_meta.md#0x2_tx_meta_TxMeta">tx_meta::TxMeta</a>
</code></pre>



<a name="0x2_tx_context_tx_gas_payment_account"></a>

## Function `tx_gas_payment_account`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_tx_gas_payment_account">tx_gas_payment_account</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <b>address</b>
</code></pre>



<a name="0x2_tx_context_tx_result"></a>

## Function `tx_result`

The result is only available in the <code>post_execute</code> function.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_result.md#0x2_tx_result">tx_result</a>(self: &<a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="tx_result.md#0x2_tx_result_TxResult">tx_result::TxResult</a>
</code></pre>



<a name="0x2_tx_context_set_module_upgrade_flag"></a>

## Function `set_module_upgrade_flag`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="tx_context.md#0x2_tx_context_set_module_upgrade_flag">set_module_upgrade_flag</a>(self: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>, is_upgrade: bool)
</code></pre>



<a name="0x2_tx_context_drop"></a>

## Function `drop`



<pre><code><b>public</b> <b>fun</b> <a href="tx_context.md#0x2_tx_context_drop">drop</a>(self: <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>)
</code></pre>
