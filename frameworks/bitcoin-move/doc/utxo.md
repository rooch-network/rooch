
<a name="0x4_utxo"></a>

# Module `0x4::utxo`



-  [Struct `OutputID`](#0x4_utxo_OutputID)
-  [Resource `UTXO`](#0x4_utxo_UTXO)
-  [Struct `UTXOSeal`](#0x4_utxo_UTXOSeal)
-  [Struct `SealOut`](#0x4_utxo_SealOut)
-  [Struct `UTXORange`](#0x4_utxo_UTXORange)
-  [Struct `TempState`](#0x4_utxo_TempState)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x4_utxo_new)
-  [Function `new_id`](#0x4_utxo_new_id)
-  [Function `new_utxo_range`](#0x4_utxo_new_utxo_range)
-  [Function `value`](#0x4_utxo_value)
-  [Function `txid`](#0x4_utxo_txid)
-  [Function `vout`](#0x4_utxo_vout)
-  [Function `exists_utxo`](#0x4_utxo_exists_utxo)
-  [Function `borrow_utxo`](#0x4_utxo_borrow_utxo)
-  [Function `utxo_range_txid`](#0x4_utxo_utxo_range_txid)
-  [Function `utxo_range_vout`](#0x4_utxo_utxo_range_vout)
-  [Function `utxo_range_range`](#0x4_utxo_utxo_range_range)
-  [Function `seal`](#0x4_utxo_seal)
-  [Function `has_seal`](#0x4_utxo_has_seal)
-  [Function `get_seals`](#0x4_utxo_get_seals)
-  [Function `remove_seals`](#0x4_utxo_remove_seals)
-  [Function `add_seal`](#0x4_utxo_add_seal)
-  [Function `transfer`](#0x4_utxo_transfer)
-  [Function `take`](#0x4_utxo_take)
-  [Function `remove`](#0x4_utxo_remove)
-  [Function `new_utxo_seal`](#0x4_utxo_new_utxo_seal)
-  [Function `unpack_utxo_seal`](#0x4_utxo_unpack_utxo_seal)
-  [Function `new_seal_out`](#0x4_utxo_new_seal_out)
-  [Function `unpack_seal_out`](#0x4_utxo_unpack_seal_out)
-  [Function `add_temp_state`](#0x4_utxo_add_temp_state)
-  [Function `contains_temp_state`](#0x4_utxo_contains_temp_state)
-  [Function `borrow_temp_state`](#0x4_utxo_borrow_temp_state)
-  [Function `borrow_mut_temp_state`](#0x4_utxo_borrow_mut_temp_state)
-  [Function `remove_temp_state`](#0x4_utxo_remove_temp_state)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::bag</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_multimap</a>;
<b>use</b> <a href="">0x2::type_info</a>;
</code></pre>



<a name="0x4_utxo_OutputID"></a>

## Struct `OutputID`

The transaction output ID


<pre><code><b>struct</b> <a href="utxo.md#0x4_utxo_OutputID">OutputID</a> <b>has</b> <b>copy</b>, drop, store
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



<a name="0x4_utxo_UTXORange"></a>

## Struct `UTXORange`



<pre><code><b>struct</b> <a href="utxo.md#0x4_utxo_UTXORange">UTXORange</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x4_utxo_TempState"></a>

## Struct `TempState`



<pre><code><b>struct</b> <a href="utxo.md#0x4_utxo_TempState">TempState</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_utxo_TEMPORARY_AREA"></a>



<pre><code><b>const</b> <a href="utxo.md#0x4_utxo_TEMPORARY_AREA">TEMPORARY_AREA</a>: <a href="">vector</a>&lt;u8&gt; = [116, 101, 109, 112, 111, 114, 97, 114, 121, 95, 97, 114, 101, 97];
</code></pre>



<a name="0x4_utxo_new"></a>

## Function `new`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_new">new</a>(txid: <b>address</b>, vout: u32, value: u64): <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;
</code></pre>



<a name="0x4_utxo_new_id"></a>

## Function `new_id`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_new_id">new_id</a>(txid: <b>address</b>, vout: u32): <a href="utxo.md#0x4_utxo_OutputID">utxo::OutputID</a>
</code></pre>



<a name="0x4_utxo_new_utxo_range"></a>

## Function `new_utxo_range`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_new_utxo_range">new_utxo_range</a>(txid: <b>address</b>, vout: u32, start_offset: u64, end_offset: u64): <a href="utxo.md#0x4_utxo_UTXORange">utxo::UTXORange</a>
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



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_exists_utxo">exists_utxo</a>(txid: <b>address</b>, vout: u32): bool
</code></pre>



<a name="0x4_utxo_borrow_utxo"></a>

## Function `borrow_utxo`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_borrow_utxo">borrow_utxo</a>(txid: <b>address</b>, vout: u32): &<a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;
</code></pre>



<a name="0x4_utxo_utxo_range_txid"></a>

## Function `utxo_range_txid`

Get the UTXORange's txid


<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_utxo_range_txid">utxo_range_txid</a>(utxo_range: &<a href="utxo.md#0x4_utxo_UTXORange">utxo::UTXORange</a>): <b>address</b>
</code></pre>



<a name="0x4_utxo_utxo_range_vout"></a>

## Function `utxo_range_vout`

Get the UTXORange's vout


<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_utxo_range_vout">utxo_range_vout</a>(utxo_range: &<a href="utxo.md#0x4_utxo_UTXORange">utxo::UTXORange</a>): u32
</code></pre>



<a name="0x4_utxo_utxo_range_range"></a>

## Function `utxo_range_range`

Get the UTXORange's value range


<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_utxo_range_range">utxo_range_range</a>(utxo_range: &<a href="utxo.md#0x4_utxo_UTXORange">utxo::UTXORange</a>): (u64, u64)
</code></pre>



<a name="0x4_utxo_seal"></a>

## Function `seal`

Seal the UTXO with a protocol, the T is the protocol object


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_seal">seal</a>&lt;T&gt;(<a href="utxo.md#0x4_utxo">utxo</a>: &<b>mut</b> <a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>, seal_obj: &<a href="_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<a name="0x4_utxo_has_seal"></a>

## Function `has_seal`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_has_seal">has_seal</a>&lt;T&gt;(<a href="utxo.md#0x4_utxo">utxo</a>: &<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>): bool
</code></pre>



<a name="0x4_utxo_get_seals"></a>

## Function `get_seals`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_get_seals">get_seals</a>&lt;T&gt;(<a href="utxo.md#0x4_utxo">utxo</a>: &<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>): <a href="">vector</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<a name="0x4_utxo_remove_seals"></a>

## Function `remove_seals`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_remove_seals">remove_seals</a>&lt;T&gt;(<a href="utxo.md#0x4_utxo">utxo</a>: &<b>mut</b> <a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>): <a href="">vector</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<a name="0x4_utxo_add_seal"></a>

## Function `add_seal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_add_seal">add_seal</a>(<a href="utxo.md#0x4_utxo">utxo</a>: &<b>mut</b> <a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>, utxo_seal: <a href="utxo.md#0x4_utxo_UTXOSeal">utxo::UTXOSeal</a>)
</code></pre>



<a name="0x4_utxo_transfer"></a>

## Function `transfer`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="">transfer</a>(utxo_obj: <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;, <b>to</b>: <b>address</b>)
</code></pre>



<a name="0x4_utxo_take"></a>

## Function `take`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_take">take</a>(object_id: <a href="_ObjectID">object::ObjectID</a>): (<b>address</b>, <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;)
</code></pre>



<a name="0x4_utxo_remove"></a>

## Function `remove`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x4_utxo_remove">remove</a>(utxo_obj: <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x4_utxo_UTXO">utxo::UTXO</a>&gt;): <a href="_SimpleMultiMap">simple_multimap::SimpleMultiMap</a>&lt;<a href="_String">string::String</a>, <a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<a name="0x4_utxo_new_utxo_seal"></a>

## Function `new_utxo_seal`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_new_utxo_seal">new_utxo_seal</a>(protocol: <a href="_String">string::String</a>, object_id: <a href="_ObjectID">object::ObjectID</a>): <a href="utxo.md#0x4_utxo_UTXOSeal">utxo::UTXOSeal</a>
</code></pre>



<a name="0x4_utxo_unpack_utxo_seal"></a>

## Function `unpack_utxo_seal`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_unpack_utxo_seal">unpack_utxo_seal</a>(utxo_seal: <a href="utxo.md#0x4_utxo_UTXOSeal">utxo::UTXOSeal</a>): (<a href="_String">string::String</a>, <a href="_ObjectID">object::ObjectID</a>)
</code></pre>



<a name="0x4_utxo_new_seal_out"></a>

## Function `new_seal_out`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_new_seal_out">new_seal_out</a>(output_index: u32, object_id: <a href="_ObjectID">object::ObjectID</a>): <a href="utxo.md#0x4_utxo_SealOut">utxo::SealOut</a>
</code></pre>



<a name="0x4_utxo_unpack_seal_out"></a>

## Function `unpack_seal_out`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x4_utxo_unpack_seal_out">unpack_seal_out</a>(seal_out: <a href="utxo.md#0x4_utxo_SealOut">utxo::SealOut</a>): (u32, <a href="_ObjectID">object::ObjectID</a>)
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
