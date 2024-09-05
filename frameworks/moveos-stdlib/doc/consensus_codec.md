
<a name="0x2_consensus_codec"></a>

# Module `0x2::consensus_codec`

This module implements the Bitcoin consensus encode/decode functions


-  [Struct `Encoder`](#0x2_consensus_codec_Encoder)
-  [Struct `Decoder`](#0x2_consensus_codec_Decoder)
-  [Constants](#@Constants_0)
-  [Function `encoder`](#0x2_consensus_codec_encoder)
-  [Function `decoder`](#0x2_consensus_codec_decoder)
-  [Function `unpack_encoder`](#0x2_consensus_codec_unpack_encoder)
-  [Function `unpack_decoder`](#0x2_consensus_codec_unpack_decoder)
-  [Function `emit_u64`](#0x2_consensus_codec_emit_u64)
-  [Function `emit_u32`](#0x2_consensus_codec_emit_u32)
-  [Function `emit_u16`](#0x2_consensus_codec_emit_u16)
-  [Function `emit_u8`](#0x2_consensus_codec_emit_u8)
-  [Function `emit_bool`](#0x2_consensus_codec_emit_bool)
-  [Function `emit_var_int`](#0x2_consensus_codec_emit_var_int)
-  [Function `emit_var_slice`](#0x2_consensus_codec_emit_var_slice)
-  [Function `peel_var_int`](#0x2_consensus_codec_peel_var_int)
-  [Function `peel_var_slice`](#0x2_consensus_codec_peel_var_slice)
-  [Function `peel_bool`](#0x2_consensus_codec_peel_bool)
-  [Function `peel_u64`](#0x2_consensus_codec_peel_u64)
-  [Function `peel_u32`](#0x2_consensus_codec_peel_u32)
-  [Function `peel_u16`](#0x2_consensus_codec_peel_u16)
-  [Function `peel_u8`](#0x2_consensus_codec_peel_u8)


<pre><code><b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x2_consensus_codec_Encoder"></a>

## Struct `Encoder`



<pre><code><b>struct</b> <a href="consensus_codec.md#0x2_consensus_codec_Encoder">Encoder</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x2_consensus_codec_Decoder"></a>

## Struct `Decoder`



<pre><code><b>struct</b> <a href="consensus_codec.md#0x2_consensus_codec_Decoder">Decoder</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_consensus_codec_ErrorInvalidLength"></a>



<pre><code><b>const</b> <a href="consensus_codec.md#0x2_consensus_codec_ErrorInvalidLength">ErrorInvalidLength</a>: u64 = 1;
</code></pre>



<a name="0x2_consensus_codec_ErrorOutOfRange"></a>



<pre><code><b>const</b> <a href="consensus_codec.md#0x2_consensus_codec_ErrorOutOfRange">ErrorOutOfRange</a>: u64 = 3;
</code></pre>



<a name="0x2_consensus_codec_ErrorNonMinimalVarInt"></a>



<pre><code><b>const</b> <a href="consensus_codec.md#0x2_consensus_codec_ErrorNonMinimalVarInt">ErrorNonMinimalVarInt</a>: u64 = 2;
</code></pre>



<a name="0x2_consensus_codec_encoder"></a>

## Function `encoder`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_encoder">encoder</a>(): <a href="consensus_codec.md#0x2_consensus_codec_Encoder">consensus_codec::Encoder</a>
</code></pre>



<a name="0x2_consensus_codec_decoder"></a>

## Function `decoder`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_decoder">decoder</a>(data: <a href="">vector</a>&lt;u8&gt;): <a href="consensus_codec.md#0x2_consensus_codec_Decoder">consensus_codec::Decoder</a>
</code></pre>



<a name="0x2_consensus_codec_unpack_encoder"></a>

## Function `unpack_encoder`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_unpack_encoder">unpack_encoder</a>(encoder: <a href="consensus_codec.md#0x2_consensus_codec_Encoder">consensus_codec::Encoder</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_consensus_codec_unpack_decoder"></a>

## Function `unpack_decoder`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_unpack_decoder">unpack_decoder</a>(decoder: <a href="consensus_codec.md#0x2_consensus_codec_Decoder">consensus_codec::Decoder</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_consensus_codec_emit_u64"></a>

## Function `emit_u64`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_emit_u64">emit_u64</a>(encoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Encoder">consensus_codec::Encoder</a>, v: u64)
</code></pre>



<a name="0x2_consensus_codec_emit_u32"></a>

## Function `emit_u32`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_emit_u32">emit_u32</a>(encoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Encoder">consensus_codec::Encoder</a>, v: u32)
</code></pre>



<a name="0x2_consensus_codec_emit_u16"></a>

## Function `emit_u16`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_emit_u16">emit_u16</a>(encoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Encoder">consensus_codec::Encoder</a>, v: u16)
</code></pre>



<a name="0x2_consensus_codec_emit_u8"></a>

## Function `emit_u8`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_emit_u8">emit_u8</a>(encoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Encoder">consensus_codec::Encoder</a>, v: u8)
</code></pre>



<a name="0x2_consensus_codec_emit_bool"></a>

## Function `emit_bool`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_emit_bool">emit_bool</a>(encoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Encoder">consensus_codec::Encoder</a>, v: bool)
</code></pre>



<a name="0x2_consensus_codec_emit_var_int"></a>

## Function `emit_var_int`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_emit_var_int">emit_var_int</a>(encoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Encoder">consensus_codec::Encoder</a>, v: u64)
</code></pre>



<a name="0x2_consensus_codec_emit_var_slice"></a>

## Function `emit_var_slice`

Emit a slice of bytes to the encoder with a varint length


<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_emit_var_slice">emit_var_slice</a>(encoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Encoder">consensus_codec::Encoder</a>, v: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x2_consensus_codec_peel_var_int"></a>

## Function `peel_var_int`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_peel_var_int">peel_var_int</a>(decoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Decoder">consensus_codec::Decoder</a>): u64
</code></pre>



<a name="0x2_consensus_codec_peel_var_slice"></a>

## Function `peel_var_slice`

Peel a slice of bytes from the decoder with a varint length


<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_peel_var_slice">peel_var_slice</a>(decoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Decoder">consensus_codec::Decoder</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_consensus_codec_peel_bool"></a>

## Function `peel_bool`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_peel_bool">peel_bool</a>(decoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Decoder">consensus_codec::Decoder</a>): bool
</code></pre>



<a name="0x2_consensus_codec_peel_u64"></a>

## Function `peel_u64`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_peel_u64">peel_u64</a>(decoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Decoder">consensus_codec::Decoder</a>): u64
</code></pre>



<a name="0x2_consensus_codec_peel_u32"></a>

## Function `peel_u32`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_peel_u32">peel_u32</a>(decoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Decoder">consensus_codec::Decoder</a>): u32
</code></pre>



<a name="0x2_consensus_codec_peel_u16"></a>

## Function `peel_u16`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_peel_u16">peel_u16</a>(decoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Decoder">consensus_codec::Decoder</a>): u16
</code></pre>



<a name="0x2_consensus_codec_peel_u8"></a>

## Function `peel_u8`



<pre><code><b>public</b> <b>fun</b> <a href="consensus_codec.md#0x2_consensus_codec_peel_u8">peel_u8</a>(decoder: &<b>mut</b> <a href="consensus_codec.md#0x2_consensus_codec_Decoder">consensus_codec::Decoder</a>): u8
</code></pre>
