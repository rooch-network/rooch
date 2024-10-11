
<a name="0x4_utxo"></a>

# Module `0x4::utxo`



-  [Resource `UTXO`](#0x4_utxo_UTXO)
-  [Struct `UTXOSeal`](#0x4_utxo_UTXOSeal)
-  [Struct `SealOut`](#0x4_utxo_SealOut)
-  [Struct `SpendUTXOEvent`](#0x4_utxo_SpendUTXOEvent)
-  [Struct `ReceiveUTXOEvent`](#0x4_utxo_ReceiveUTXOEvent)
-  [Struct `TempStateDropEvent`](#0x4_utxo_TempStateDropEvent)
-  [Resource `BitcoinUTXOStore`](#0x4_utxo_BitcoinUTXOStore)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x4_utxo_genesis_init)
-  [Function `borrow_utxo_store`](#0x4_utxo_borrow_utxo_store)
-  [Function `borrow_mut_utxo_store`](#0x4_utxo_borrow_mut_utxo_store)
-  [Function `new`](#0x4_utxo_new)
-  [Function `mock_utxo`](#0x4_utxo_mock_utxo)
-  [Function `derive_utxo_id`](#0x4_utxo_derive_utxo_id)
-  [Function `value`](#0x4_utxo_value)
-  [Function `txid`](#0x4_utxo_txid)
-  [Function `vout`](#0x4_utxo_vout)
-  [Function `exists_utxo`](#0x4_utxo_exists_utxo)
-  [Function `borrow_utxo`](#0x4_utxo_borrow_utxo)
-  [Function `borrow_mut_utxo`](#0x4_utxo_borrow_mut_utxo)
-  [Function `has_seal`](#0x4_utxo_has_seal)
-  [Function `has_seal_internal`](#0x4_utxo_has_seal_internal)
-  [Function `get_seals`](#0x4_utxo_get_seals)
-  [Function `remove_seals_internal`](#0x4_utxo_remove_seals_internal)
-  [Function `add_seal_internal`](#0x4_utxo_add_seal_internal)
-  [Function `transfer`](#0x4_utxo_transfer)
-  [Function `take`](#0x4_utxo_take)
-  [Function `remove`](#0x4_utxo_remove)
-  [Function `drop`](#0x4_utxo_drop)
-  [Function `new_utxo_seal`](#0x4_utxo_new_utxo_seal)
-  [Function `unpack_utxo_seal`](#0x4_utxo_unpack_utxo_seal)
-  [Function `new_seal_out`](#0x4_utxo_new_seal_out)
-  [Function `unpack_seal_out`](#0x4_utxo_unpack_seal_out)
-  [Function `add_temp_state`](#0x4_utxo_add_temp_state)
-  [Function `contains_temp_state`](#0x4_utxo_contains_temp_state)
-  [Function `borrow_temp_state`](#0x4_utxo_borrow_temp_state)
-  [Function `borrow_mut_temp_state`](#0x4_utxo_borrow_mut_temp_state)
-  [Function `remove_temp_state`](#0x4_utxo_remove_temp_state)
-  [Function `check_utxo_input`](#0x4_utxo_check_utxo_input)
-  [Function `unpack_spend_utxo_event`](#0x4_utxo_unpack_spend_utxo_event)
-  [Function `unpack_receive_utxo_event`](#0x4_utxo_unpack_receive_utxo_event)
-  [Function `unpack_temp_state_drop_event`](#0x4_utxo_unpack_temp_state_drop_event)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::address</a>;
<b>use</b> <a href="">0x2::event_queue</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_multimap</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="">0x3::chain_id</a>;
<b>use</b> <a href="temp_state.md#0x4_temp_state">0x4::temp_state</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
</code></pre>



<a name="0x4_utxo_UTXO"></a>

## Resource `UTXO`

The UTXO Object


<pre><code><b>struct</b> <a href="utxo.md#0x4_utxo_UTXO">UTXO</a> <b>has</b> key
</code></pre>



<a name="0x4_utxo_UTXOSeal"></a>

## Struct `UTXOSeal`



<pre><code><b>struct</b> <a href="utxo.md#0x4_utxo_UTXOSeal">UTXOSeal</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_utxo_SealOut"></a>

## Struct `SealOut`



<pre><code><b>struct</b> <a href="utxo.md#0x4_utxo_SealOut">SealOut</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_utxo_SpendUTXOEvent"></a>

## Struct `SpendUTXOEvent`

Event emitted when a UTXO is spent
In the Bitcoin UTXO model, there's no inherent concept of sender and receiver.
However, for simplifying payment scenarios, we define sender and receiver as follows:
- Sender: The address of the first input UTXO that can be identified
- Receiver: The address of each output UTXO that can be identified


<pre><code><b>struct</b> <a href="utxo.md#0x4_utxo_SpendUTXOEvent">SpendUTXOEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_utxo_ReceiveUTXOEvent"></a>

## Struct `ReceiveUTXOEvent`

Event emitted when a UTXO is received
In the Bitcoin UTXO model, there's no inherent concept of sender and receiver.
However, for simplifying payment scenarios, we define sender and receiver as follows:
- Sender: The address of the first input UTXO that can be identified
- Receiver: The address of each output UTXO that can be identified


<pre><code><b>struct</b> <a href="utxo.md#0x4_utxo_ReceiveUTXOEvent">ReceiveUTXOEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_utxo_TempStateDropEvent"></a>

## Struct `TempStateDropEvent`

Event emitted when the temporary state of a UTXO is dropped
The temporary state is dropped when the UTXO is spent
The event is onchain event, and the event_queue name is type_name of the temporary state


<pre><code><b>struct</b> <a href="utxo.md#0x4_utxo_TempStateDropEvent">TempStateDropEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_utxo_BitcoinUTXOStore"></a>

## Resource `BitcoinUTXOStore`



<pre><code><b>struct</b> <a href="utxo.md#0x4_utxo_BitcoinUTXOStore">BitcoinUTXOStore</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_utxo_ErrorDeprecatedFunction"></a>



<pre><code><b>const</b> <a href="utxo.md#0x4_utxo_ErrorDeprecatedFunction">ErrorDeprecatedFunction</a>: u64 = 1;
</code></pre>



<a name="0x4_utxo_TEMPORARY_AREA"></a>



<pre><code><b>const</b> <a href="utxo.md#0x4_utxo_TEMPORARY_AREA">TEMPORARY_AREA</a>: <a href="">vector</a>&lt;u8&gt; = [116, 101, 109, 112, 111, 114, 97, 114, 121, 95, 97, 114, 101, 97];
</code></pre>



<a name="0x4_utxo_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_genesis_init">genesis_init</a>()
</code></pre>



<a name="0x4_utxo_borrow_utxo_store"></a>

## Function `borrow_utxo_store`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_borrow_utxo_store">borrow_utxo_store</a>(): &<a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_BitcoinUTXOStore">utxo::BitcoinUTXOStore</a>&gt;
</code></pre>



<a name="0x4_utxo_borrow_mut_utxo_store"></a>

## Function `borrow_mut_utxo_store`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_borrow_mut_utxo_store">borrow_mut_utxo_store</a>(): &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_BitcoinUTXOStore">utxo::BitcoinUTXOStore</a>&gt;
</code></pre>



<a name="0x4_utxo_new"></a>

## Function `new`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_new">new</a>(txid: <b>address</b>, vout: u32, value: u64): <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;
</code></pre>



<a name="0x4_utxo_mock_utxo"></a>

## Function `mock_utxo`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_mock_utxo">mock_utxo</a>(outpoint: <a href="types.md#0x4_types_OutPoint">types::OutPoint</a>, value: u64): <a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>
</code></pre>



<a name="0x4_utxo_derive_utxo_id"></a>

## Function `derive_utxo_id`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_derive_utxo_id">derive_utxo_id</a>(outpoint: <a href="types.md#0x4_types_OutPoint">types::OutPoint</a>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x4_utxo_value"></a>

## Function `value`

Get the UTXO's value


<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_value">value</a>(<a href="utxo.md#0x4_utxo">utxo</a>: &<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>): u64
</code></pre>



<a name="0x4_utxo_txid"></a>

## Function `txid`

Get the UTXO's txid


<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_txid">txid</a>(<a href="utxo.md#0x4_utxo">utxo</a>: &<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>): <b>address</b>
</code></pre>



<a name="0x4_utxo_vout"></a>

## Function `vout`

Get the UTXO's vout


<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_vout">vout</a>(<a href="utxo.md#0x4_utxo">utxo</a>: &<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>): u32
</code></pre>



<a name="0x4_utxo_exists_utxo"></a>

## Function `exists_utxo`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_exists_utxo">exists_utxo</a>(outpoint: <a href="types.md#0x4_types_OutPoint">types::OutPoint</a>): bool
</code></pre>



<a name="0x4_utxo_borrow_utxo"></a>

## Function `borrow_utxo`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_borrow_utxo">borrow_utxo</a>(outpoint: <a href="types.md#0x4_types_OutPoint">types::OutPoint</a>): &<a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;
</code></pre>



<a name="0x4_utxo_borrow_mut_utxo"></a>

## Function `borrow_mut_utxo`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_borrow_mut_utxo">borrow_mut_utxo</a>(outpoint: <a href="types.md#0x4_types_OutPoint">types::OutPoint</a>): &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;
</code></pre>



<a name="0x4_utxo_has_seal"></a>

## Function `has_seal`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_has_seal">has_seal</a>&lt;T&gt;(<a href="utxo.md#0x4_utxo">utxo</a>: &<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>): bool
</code></pre>



<a name="0x4_utxo_has_seal_internal"></a>

## Function `has_seal_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_has_seal_internal">has_seal_internal</a>(<a href="utxo.md#0x4_utxo">utxo</a>: &<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>, protocol: &<a href="_String">string::String</a>): bool
</code></pre>



<a name="0x4_utxo_get_seals"></a>

## Function `get_seals`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_get_seals">get_seals</a>&lt;T&gt;(<a href="utxo.md#0x4_utxo">utxo</a>: &<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>): <a href="">vector</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<a name="0x4_utxo_remove_seals_internal"></a>

## Function `remove_seals_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_remove_seals_internal">remove_seals_internal</a>&lt;T&gt;(<a href="utxo.md#0x4_utxo">utxo</a>: &<b>mut</b> <a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>): <a href="">vector</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<a name="0x4_utxo_add_seal_internal"></a>

## Function `add_seal_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_add_seal_internal">add_seal_internal</a>(<a href="utxo.md#0x4_utxo">utxo</a>: &<b>mut</b> <a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>, utxo_seal: <a href="utxo.md#0x4_utxo_UTXOSeal">utxo::UTXOSeal</a>)
</code></pre>



<a name="0x4_utxo_transfer"></a>

## Function `transfer`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="">transfer</a>(utxo_obj: <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;, sender: <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;, receiver: <b>address</b>)
</code></pre>



<a name="0x4_utxo_take"></a>

## Function `take`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_take">take</a>(object_id: <a href="_ObjectID">object::ObjectID</a>): <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;
</code></pre>



<a name="0x4_utxo_remove"></a>

## Function `remove`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_remove">remove</a>(utxo_obj: <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;): <a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>
</code></pre>



<a name="0x4_utxo_drop"></a>

## Function `drop`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_drop">drop</a>(<a href="utxo.md#0x4_utxo">utxo</a>: <a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>)
</code></pre>



<a name="0x4_utxo_new_utxo_seal"></a>

## Function `new_utxo_seal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_new_utxo_seal">new_utxo_seal</a>(protocol: <a href="_String">string::String</a>, seal_object_id: <a href="_ObjectID">object::ObjectID</a>): <a href="utxo.md#0x4_utxo_UTXOSeal">utxo::UTXOSeal</a>
</code></pre>



<a name="0x4_utxo_unpack_utxo_seal"></a>

## Function `unpack_utxo_seal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_unpack_utxo_seal">unpack_utxo_seal</a>(utxo_seal: <a href="utxo.md#0x4_utxo_UTXOSeal">utxo::UTXOSeal</a>): (<a href="_String">string::String</a>, <a href="_ObjectID">object::ObjectID</a>)
</code></pre>



<a name="0x4_utxo_new_seal_out"></a>

## Function `new_seal_out`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_new_seal_out">new_seal_out</a>(vout: u32, seal: <a href="utxo.md#0x4_utxo_UTXOSeal">utxo::UTXOSeal</a>): <a href="utxo.md#0x4_utxo_SealOut">utxo::SealOut</a>
</code></pre>



<a name="0x4_utxo_unpack_seal_out"></a>

## Function `unpack_seal_out`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_unpack_seal_out">unpack_seal_out</a>(seal_out: <a href="utxo.md#0x4_utxo_SealOut">utxo::SealOut</a>): (u32, <a href="utxo.md#0x4_utxo_UTXOSeal">utxo::UTXOSeal</a>)
</code></pre>



<a name="0x4_utxo_add_temp_state"></a>

## Function `add_temp_state`



<pre><code>#[private_generics(#[S])]
<b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_add_temp_state">add_temp_state</a>&lt;S: drop, store&gt;(<a href="utxo.md#0x4_utxo">utxo</a>: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;, state: S)
</code></pre>



<a name="0x4_utxo_contains_temp_state"></a>

## Function `contains_temp_state`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_contains_temp_state">contains_temp_state</a>&lt;S: drop, store&gt;(<a href="utxo.md#0x4_utxo">utxo</a>: &<a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;): bool
</code></pre>



<a name="0x4_utxo_borrow_temp_state"></a>

## Function `borrow_temp_state`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_borrow_temp_state">borrow_temp_state</a>&lt;S: drop, store&gt;(<a href="utxo.md#0x4_utxo">utxo</a>: &<a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;): &S
</code></pre>



<a name="0x4_utxo_borrow_mut_temp_state"></a>

## Function `borrow_mut_temp_state`



<pre><code>#[private_generics(#[S])]
<b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_borrow_mut_temp_state">borrow_mut_temp_state</a>&lt;S: drop, store&gt;(<a href="utxo.md#0x4_utxo">utxo</a>: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;): &<b>mut</b> S
</code></pre>



<a name="0x4_utxo_remove_temp_state"></a>

## Function `remove_temp_state`



<pre><code>#[private_generics(#[S])]
<b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_remove_temp_state">remove_temp_state</a>&lt;S: drop, store&gt;(<a href="utxo.md#0x4_utxo">utxo</a>: &<b>mut</b> <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;): S
</code></pre>



<a name="0x4_utxo_check_utxo_input"></a>

## Function `check_utxo_input`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_check_utxo_input">check_utxo_input</a>(): bool
</code></pre>



<a name="0x4_utxo_unpack_spend_utxo_event"></a>

## Function `unpack_spend_utxo_event`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_unpack_spend_utxo_event">unpack_spend_utxo_event</a>(<a href="">event</a>: <a href="utxo.md#0x4_utxo_SpendUTXOEvent">utxo::SpendUTXOEvent</a>): (<b>address</b>, <b>address</b>, <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;, u64)
</code></pre>



<a name="0x4_utxo_unpack_receive_utxo_event"></a>

## Function `unpack_receive_utxo_event`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_unpack_receive_utxo_event">unpack_receive_utxo_event</a>(<a href="">event</a>: <a href="utxo.md#0x4_utxo_ReceiveUTXOEvent">utxo::ReceiveUTXOEvent</a>): (<b>address</b>, <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;, <b>address</b>, u64)
</code></pre>



<a name="0x4_utxo_unpack_temp_state_drop_event"></a>

## Function `unpack_temp_state_drop_event`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_unpack_temp_state_drop_event">unpack_temp_state_drop_event</a>(<a href="">event</a>: <a href="utxo.md#0x4_utxo_TempStateDropEvent">utxo::TempStateDropEvent</a>): (<a href="_ObjectID">object::ObjectID</a>, <a href="types.md#0x4_types_OutPoint">types::OutPoint</a>, u64)
</code></pre>
