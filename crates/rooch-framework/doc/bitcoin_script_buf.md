
<a name="0x3_bitcoin_script_buf"></a>

# Module `0x3::bitcoin_script_buf`



-  [Struct `ScriptBuf`](#0x3_bitcoin_script_buf_ScriptBuf)
-  [Function `new`](#0x3_bitcoin_script_buf_new)
-  [Function `bytes`](#0x3_bitcoin_script_buf_bytes)
-  [Function `is_p2sh`](#0x3_bitcoin_script_buf_is_p2sh)
-  [Function `p2sh_script_hash`](#0x3_bitcoin_script_buf_p2sh_script_hash)
-  [Function `is_p2pkh`](#0x3_bitcoin_script_buf_is_p2pkh)
-  [Function `p2pkh_pubkey_hash`](#0x3_bitcoin_script_buf_p2pkh_pubkey_hash)
-  [Function `is_witness_program`](#0x3_bitcoin_script_buf_is_witness_program)
-  [Function `witness_program`](#0x3_bitcoin_script_buf_witness_program)


<pre><code><b>use</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode">0x3::bitcoin_opcode</a>;
</code></pre>



<a name="0x3_bitcoin_script_buf_ScriptBuf"></a>

## Struct `ScriptBuf`



<pre><code><b>struct</b> <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">ScriptBuf</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_bitcoin_script_buf_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_new">new</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">bitcoin_script_buf::ScriptBuf</a>
</code></pre>



<a name="0x3_bitcoin_script_buf_bytes"></a>

## Function `bytes`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_bytes">bytes</a>(self: &<a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">bitcoin_script_buf::ScriptBuf</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_script_buf_is_p2sh"></a>

## Function `is_p2sh`

Checks if the given script is a P2SH script.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_is_p2sh">is_p2sh</a>(self: &<a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">bitcoin_script_buf::ScriptBuf</a>): bool
</code></pre>



<a name="0x3_bitcoin_script_buf_p2sh_script_hash"></a>

## Function `p2sh_script_hash`

Get the script hash from a P2SH script.
This function does not check if the script is a P2SH script, the caller must do that.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_p2sh_script_hash">p2sh_script_hash</a>(self: &<a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">bitcoin_script_buf::ScriptBuf</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_script_buf_is_p2pkh"></a>

## Function `is_p2pkh`

Checks if the given script is a P2PKH script.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_is_p2pkh">is_p2pkh</a>(self: &<a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">bitcoin_script_buf::ScriptBuf</a>): bool
</code></pre>



<a name="0x3_bitcoin_script_buf_p2pkh_pubkey_hash"></a>

## Function `p2pkh_pubkey_hash`

Get the public key hash from a P2PKH script.
This function does not check if the script is a P2PKH script, the caller must do that.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_p2pkh_pubkey_hash">p2pkh_pubkey_hash</a>(self: &<a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">bitcoin_script_buf::ScriptBuf</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_script_buf_is_witness_program"></a>

## Function `is_witness_program`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_is_witness_program">is_witness_program</a>(self: &<a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">bitcoin_script_buf::ScriptBuf</a>): bool
</code></pre>



<a name="0x3_bitcoin_script_buf_witness_program"></a>

## Function `witness_program`

Get the witness program from a witness program script.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_witness_program">witness_program</a>(self: &<a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">bitcoin_script_buf::ScriptBuf</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>
