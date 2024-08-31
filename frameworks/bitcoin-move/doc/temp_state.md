
<a name="0x4_temp_state"></a>

# Module `0x4::temp_state`

This module is used to store temporary states for UTXO and Inscription.


-  [Struct `TempState`](#0x4_temp_state_TempState)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x4_temp_state_new)
-  [Function `add_state`](#0x4_temp_state_add_state)
-  [Function `borrow_state`](#0x4_temp_state_borrow_state)
-  [Function `borrow_mut_state`](#0x4_temp_state_borrow_mut_state)
-  [Function `remove_state`](#0x4_temp_state_remove_state)
-  [Function `contains_state`](#0x4_temp_state_contains_state)
-  [Function `remove`](#0x4_temp_state_remove)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bag</a>;
<b>use</b> <a href="">0x2::type_info</a>;
</code></pre>



<a name="0x4_temp_state_TempState"></a>

## Struct `TempState`



<pre><code><b>struct</b> <a href="temp_state.md#0x4_temp_state_TempState">TempState</a> <b>has</b> store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_temp_state_ErrorMaxTempStateExceeded"></a>



<pre><code><b>const</b> <a href="temp_state.md#0x4_temp_state_ErrorMaxTempStateExceeded">ErrorMaxTempStateExceeded</a>: u64 = 1;
</code></pre>



<a name="0x4_temp_state_ErrorTempStateNotFound"></a>



<pre><code><b>const</b> <a href="temp_state.md#0x4_temp_state_ErrorTempStateNotFound">ErrorTempStateNotFound</a>: u64 = 2;
</code></pre>



<a name="0x4_temp_state_MAX_TEMP_STATES"></a>



<pre><code><b>const</b> <a href="temp_state.md#0x4_temp_state_MAX_TEMP_STATES">MAX_TEMP_STATES</a>: u64 = 20;
</code></pre>



<a name="0x4_temp_state_new"></a>

## Function `new`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="temp_state.md#0x4_temp_state_new">new</a>(): <a href="temp_state.md#0x4_temp_state_TempState">temp_state::TempState</a>
</code></pre>



<a name="0x4_temp_state_add_state"></a>

## Function `add_state`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="temp_state.md#0x4_temp_state_add_state">add_state</a>&lt;T: drop, store&gt;(self: &<b>mut</b> <a href="temp_state.md#0x4_temp_state_TempState">temp_state::TempState</a>, value: T)
</code></pre>



<a name="0x4_temp_state_borrow_state"></a>

## Function `borrow_state`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="temp_state.md#0x4_temp_state_borrow_state">borrow_state</a>&lt;T: drop, store&gt;(self: &<a href="temp_state.md#0x4_temp_state_TempState">temp_state::TempState</a>): &T
</code></pre>



<a name="0x4_temp_state_borrow_mut_state"></a>

## Function `borrow_mut_state`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="temp_state.md#0x4_temp_state_borrow_mut_state">borrow_mut_state</a>&lt;T: drop, store&gt;(self: &<b>mut</b> <a href="temp_state.md#0x4_temp_state_TempState">temp_state::TempState</a>): &<b>mut</b> T
</code></pre>



<a name="0x4_temp_state_remove_state"></a>

## Function `remove_state`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="temp_state.md#0x4_temp_state_remove_state">remove_state</a>&lt;T: drop, store&gt;(self: &<b>mut</b> <a href="temp_state.md#0x4_temp_state_TempState">temp_state::TempState</a>): T
</code></pre>



<a name="0x4_temp_state_contains_state"></a>

## Function `contains_state`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="temp_state.md#0x4_temp_state_contains_state">contains_state</a>&lt;T: drop, store&gt;(self: &<a href="temp_state.md#0x4_temp_state_TempState">temp_state::TempState</a>): bool
</code></pre>



<a name="0x4_temp_state_remove"></a>

## Function `remove`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="temp_state.md#0x4_temp_state_remove">remove</a>(self: <a href="temp_state.md#0x4_temp_state_TempState">temp_state::TempState</a>): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>
