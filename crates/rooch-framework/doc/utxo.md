
<a name="0x3_utxo"></a>

# Module `0x3::utxo`



-  [Resource `UTXO`](#0x3_utxo_UTXO)
-  [Struct `UTXOSeal`](#0x3_utxo_UTXOSeal)
-  [Struct `SealOut`](#0x3_utxo_SealOut)
-  [Function `new`](#0x3_utxo_new)
-  [Function `value`](#0x3_utxo_value)
-  [Function `txid`](#0x3_utxo_txid)
-  [Function `vout`](#0x3_utxo_vout)
-  [Function `seal`](#0x3_utxo_seal)
-  [Function `has_seal`](#0x3_utxo_has_seal)
-  [Function `get_seal`](#0x3_utxo_get_seal)
-  [Function `add_seal`](#0x3_utxo_add_seal)
-  [Function `transfer`](#0x3_utxo_transfer)
-  [Function `take`](#0x3_utxo_take)
-  [Function `remove`](#0x3_utxo_remove)
-  [Function `new_utxo_seal`](#0x3_utxo_new_utxo_seal)
-  [Function `unpack_utxo_seal`](#0x3_utxo_unpack_utxo_seal)
-  [Function `new_seal_out`](#0x3_utxo_new_seal_out)
-  [Function `unpack_seal_out`](#0x3_utxo_unpack_seal_out)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::type_info</a>;
</code></pre>



<a name="0x3_utxo_UTXO"></a>

## Resource `UTXO`

The UTXO Object


<pre><code><b>struct</b> <a href="utxo.md#0x3_utxo_UTXO">UTXO</a> <b>has</b> key
</code></pre>



<a name="0x3_utxo_UTXOSeal"></a>

## Struct `UTXOSeal`



<pre><code><b>struct</b> <a href="utxo.md#0x3_utxo_UTXOSeal">UTXOSeal</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_utxo_SealOut"></a>

## Struct `SealOut`



<pre><code><b>struct</b> <a href="utxo.md#0x3_utxo_SealOut">SealOut</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_utxo_new"></a>

## Function `new`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x3_utxo_new">new</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, txid: <b>address</b>, vout: u32, value: u64): <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x3_utxo_UTXO">utxo::UTXO</a>&gt;
</code></pre>



<a name="0x3_utxo_value"></a>

## Function `value`

Get the UTXO's value


<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x3_utxo_value">value</a>(<a href="utxo.md#0x3_utxo">utxo</a>: &<a href="utxo.md#0x3_utxo_UTXO">utxo::UTXO</a>): u64
</code></pre>



<a name="0x3_utxo_txid"></a>

## Function `txid`

Get the UTXO's txid


<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x3_utxo_txid">txid</a>(<a href="utxo.md#0x3_utxo">utxo</a>: &<a href="utxo.md#0x3_utxo_UTXO">utxo::UTXO</a>): <b>address</b>
</code></pre>



<a name="0x3_utxo_vout"></a>

## Function `vout`

Get the UTXO's vout


<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x3_utxo_vout">vout</a>(<a href="utxo.md#0x3_utxo">utxo</a>: &<a href="utxo.md#0x3_utxo_UTXO">utxo::UTXO</a>): u32
</code></pre>



<a name="0x3_utxo_seal"></a>

## Function `seal`

Seal the UTXO with a protocol, the T is the protocol object


<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x3_utxo_seal">seal</a>&lt;T&gt;(<a href="utxo.md#0x3_utxo">utxo</a>: &<b>mut</b> <a href="utxo.md#0x3_utxo_UTXO">utxo::UTXO</a>, seal_obj: &<a href="_Object">object::Object</a>&lt;T&gt;)
</code></pre>



<a name="0x3_utxo_has_seal"></a>

## Function `has_seal`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x3_utxo_has_seal">has_seal</a>&lt;T&gt;(<a href="utxo.md#0x3_utxo">utxo</a>: &<a href="utxo.md#0x3_utxo_UTXO">utxo::UTXO</a>): bool
</code></pre>



<a name="0x3_utxo_get_seal"></a>

## Function `get_seal`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x3_utxo_get_seal">get_seal</a>&lt;T&gt;(<a href="utxo.md#0x3_utxo">utxo</a>: &<a href="utxo.md#0x3_utxo_UTXO">utxo::UTXO</a>): <a href="_Option">option::Option</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<a name="0x3_utxo_add_seal"></a>

## Function `add_seal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x3_utxo_add_seal">add_seal</a>(<a href="utxo.md#0x3_utxo">utxo</a>: &<b>mut</b> <a href="utxo.md#0x3_utxo_UTXO">utxo::UTXO</a>, utxo_seal: <a href="utxo.md#0x3_utxo_UTXOSeal">utxo::UTXOSeal</a>)
</code></pre>



<a name="0x3_utxo_transfer"></a>

## Function `transfer`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transfer.md#0x3_transfer">transfer</a>(utxo_obj: <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x3_utxo_UTXO">utxo::UTXO</a>&gt;, <b>to</b>: <b>address</b>)
</code></pre>



<a name="0x3_utxo_take"></a>

## Function `take`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x3_utxo_take">take</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, object_id: <a href="_ObjectID">object::ObjectID</a>): <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x3_utxo_UTXO">utxo::UTXO</a>&gt;
</code></pre>



<a name="0x3_utxo_remove"></a>

## Function `remove`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="utxo.md#0x3_utxo_remove">remove</a>(utxo_obj: <a href="_Object">object::Object</a>&lt;<a href="utxo.md#0x3_utxo_UTXO">utxo::UTXO</a>&gt;): <a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<a name="0x3_utxo_new_utxo_seal"></a>

## Function `new_utxo_seal`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x3_utxo_new_utxo_seal">new_utxo_seal</a>(protocol: <a href="_String">string::String</a>, object_id: <a href="_ObjectID">object::ObjectID</a>): <a href="utxo.md#0x3_utxo_UTXOSeal">utxo::UTXOSeal</a>
</code></pre>



<a name="0x3_utxo_unpack_utxo_seal"></a>

## Function `unpack_utxo_seal`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x3_utxo_unpack_utxo_seal">unpack_utxo_seal</a>(utxo_seal: <a href="utxo.md#0x3_utxo_UTXOSeal">utxo::UTXOSeal</a>): (<a href="_String">string::String</a>, <a href="_ObjectID">object::ObjectID</a>)
</code></pre>



<a name="0x3_utxo_new_seal_out"></a>

## Function `new_seal_out`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x3_utxo_new_seal_out">new_seal_out</a>(output_index: u64, object_id: <a href="_ObjectID">object::ObjectID</a>): <a href="utxo.md#0x3_utxo_SealOut">utxo::SealOut</a>
</code></pre>



<a name="0x3_utxo_unpack_seal_out"></a>

## Function `unpack_seal_out`



<pre><code><b>public</b> <b>fun</b> <a href="utxo.md#0x3_utxo_unpack_seal_out">unpack_seal_out</a>(seal_out: <a href="utxo.md#0x3_utxo_SealOut">utxo::SealOut</a>): (u64, <a href="_ObjectID">object::ObjectID</a>)
</code></pre>
