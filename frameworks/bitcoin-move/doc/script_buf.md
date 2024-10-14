
<a name="0x4_script_buf"></a>

# Module `0x4::script_buf`



-  [Struct `ScriptBuf`](#0x4_script_buf_ScriptBuf)
-  [Constants](#@Constants_0)
-  [Function `empty`](#0x4_script_buf_empty)
-  [Function `new`](#0x4_script_buf_new)
-  [Function `single`](#0x4_script_buf_single)
-  [Function `new_p2pkh`](#0x4_script_buf_new_p2pkh)
-  [Function `new_p2sh`](#0x4_script_buf_new_p2sh)
-  [Function `script_pubkey`](#0x4_script_buf_script_pubkey)
-  [Function `match_script_pubkey`](#0x4_script_buf_match_script_pubkey)
-  [Function `is_empty`](#0x4_script_buf_is_empty)
-  [Function `bytes`](#0x4_script_buf_bytes)
-  [Function `into_bytes`](#0x4_script_buf_into_bytes)
-  [Function `is_p2sh`](#0x4_script_buf_is_p2sh)
-  [Function `p2sh_script_hash`](#0x4_script_buf_p2sh_script_hash)
-  [Function `is_p2pkh`](#0x4_script_buf_is_p2pkh)
-  [Function `p2pkh_pubkey_hash`](#0x4_script_buf_p2pkh_pubkey_hash)
-  [Function `is_witness_program`](#0x4_script_buf_is_witness_program)
-  [Function `witness_program`](#0x4_script_buf_witness_program)
-  [Function `is_op_return`](#0x4_script_buf_is_op_return)
-  [Function `push_opcode`](#0x4_script_buf_push_opcode)
-  [Function `push_data`](#0x4_script_buf_push_data)
-  [Function `push_int`](#0x4_script_buf_push_int)
-  [Function `push_key`](#0x4_script_buf_push_key)
-  [Function `push_x_only_key`](#0x4_script_buf_push_x_only_key)


<pre><code><b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x3::bitcoin_address</a>;
<b>use</b> <a href="opcode.md#0x4_opcode">0x4::opcode</a>;
</code></pre>



<a name="0x4_script_buf_ScriptBuf"></a>

## Struct `ScriptBuf`



<pre><code>#[data_struct]
<b>struct</b> <a href="script_buf.md#0x4_script_buf_ScriptBuf">ScriptBuf</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_script_buf_BITCOIN_PUBKEY_HASH_SIZE"></a>



<pre><code><b>const</b> <a href="script_buf.md#0x4_script_buf_BITCOIN_PUBKEY_HASH_SIZE">BITCOIN_PUBKEY_HASH_SIZE</a>: u64 = 20;
</code></pre>



<a name="0x4_script_buf_BITCOIN_PUBKEY_SIZE"></a>



<pre><code><b>const</b> <a href="script_buf.md#0x4_script_buf_BITCOIN_PUBKEY_SIZE">BITCOIN_PUBKEY_SIZE</a>: u64 = 33;
</code></pre>



<a name="0x4_script_buf_BITCOIN_SCRIPT_HASH_SIZE"></a>



<pre><code><b>const</b> <a href="script_buf.md#0x4_script_buf_BITCOIN_SCRIPT_HASH_SIZE">BITCOIN_SCRIPT_HASH_SIZE</a>: u64 = 20;
</code></pre>



<a name="0x4_script_buf_BITCOIN_X_ONLY_PUBKEY_SIZE"></a>



<pre><code><b>const</b> <a href="script_buf.md#0x4_script_buf_BITCOIN_X_ONLY_PUBKEY_SIZE">BITCOIN_X_ONLY_PUBKEY_SIZE</a>: u64 = 32;
</code></pre>



<a name="0x4_script_buf_ErrorInvalidKeySize"></a>



<pre><code><b>const</b> <a href="script_buf.md#0x4_script_buf_ErrorInvalidKeySize">ErrorInvalidKeySize</a>: u64 = 1;
</code></pre>



<a name="0x4_script_buf_ErrorInvalidPubkeyHash"></a>



<pre><code><b>const</b> <a href="script_buf.md#0x4_script_buf_ErrorInvalidPubkeyHash">ErrorInvalidPubkeyHash</a>: u64 = 3;
</code></pre>



<a name="0x4_script_buf_ErrorInvalidScriptHash"></a>



<pre><code><b>const</b> <a href="script_buf.md#0x4_script_buf_ErrorInvalidScriptHash">ErrorInvalidScriptHash</a>: u64 = 4;
</code></pre>



<a name="0x4_script_buf_ErrorNumberOverflow"></a>



<pre><code><b>const</b> <a href="script_buf.md#0x4_script_buf_ErrorNumberOverflow">ErrorNumberOverflow</a>: u64 = 2;
</code></pre>



<a name="0x4_script_buf_I64_MAX"></a>



<pre><code><b>const</b> <a href="script_buf.md#0x4_script_buf_I64_MAX">I64_MAX</a>: u64 = 9223372036854775807;
</code></pre>



<a name="0x4_script_buf_empty"></a>

## Function `empty`



<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_empty">empty</a>(): <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>
</code></pre>



<a name="0x4_script_buf_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_new">new</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>
</code></pre>



<a name="0x4_script_buf_single"></a>

## Function `single`



<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_single">single</a>(<a href="opcode.md#0x4_opcode">opcode</a>: u8): <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>
</code></pre>



<a name="0x4_script_buf_new_p2pkh"></a>

## Function `new_p2pkh`



<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_new_p2pkh">new_p2pkh</a>(pubkey_hash: <a href="">vector</a>&lt;u8&gt;): <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>
</code></pre>



<a name="0x4_script_buf_new_p2sh"></a>

## Function `new_p2sh`



<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_new_p2sh">new_p2sh</a>(script_hash: <a href="">vector</a>&lt;u8&gt;): <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>
</code></pre>



<a name="0x4_script_buf_script_pubkey"></a>

## Function `script_pubkey`

Generates a script pubkey spending to this address.


<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_script_pubkey">script_pubkey</a>(addr: &<a href="_BitcoinAddress">bitcoin_address::BitcoinAddress</a>): <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>
</code></pre>



<a name="0x4_script_buf_match_script_pubkey"></a>

## Function `match_script_pubkey`

Returns true if the address creates a particular script


<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_match_script_pubkey">match_script_pubkey</a>(addr: &<a href="_BitcoinAddress">bitcoin_address::BitcoinAddress</a>, sb: &<a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>): bool
</code></pre>



<a name="0x4_script_buf_is_empty"></a>

## Function `is_empty`



<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_is_empty">is_empty</a>(self: &<a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>): bool
</code></pre>



<a name="0x4_script_buf_bytes"></a>

## Function `bytes`



<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_bytes">bytes</a>(self: &<a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_script_buf_into_bytes"></a>

## Function `into_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_into_bytes">into_bytes</a>(self: <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_script_buf_is_p2sh"></a>

## Function `is_p2sh`

Checks if the given script is a P2SH script.


<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_is_p2sh">is_p2sh</a>(self: &<a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>): bool
</code></pre>



<a name="0x4_script_buf_p2sh_script_hash"></a>

## Function `p2sh_script_hash`

Get the script hash from a P2SH script.
This function does not check if the script is a P2SH script, the caller must do that.


<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_p2sh_script_hash">p2sh_script_hash</a>(self: &<a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_script_buf_is_p2pkh"></a>

## Function `is_p2pkh`

Checks if the given script is a P2PKH script.


<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_is_p2pkh">is_p2pkh</a>(self: &<a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>): bool
</code></pre>



<a name="0x4_script_buf_p2pkh_pubkey_hash"></a>

## Function `p2pkh_pubkey_hash`

Get the public key hash from a P2PKH script.
This function does not check if the script is a P2PKH script, the caller must do that.


<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_p2pkh_pubkey_hash">p2pkh_pubkey_hash</a>(self: &<a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_script_buf_is_witness_program"></a>

## Function `is_witness_program`



<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_is_witness_program">is_witness_program</a>(self: &<a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>): bool
</code></pre>



<a name="0x4_script_buf_witness_program"></a>

## Function `witness_program`

Get the witness program from a witness program script.


<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_witness_program">witness_program</a>(self: &<a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_script_buf_is_op_return"></a>

## Function `is_op_return`

Checks if the given script is an OP_RETURN script.


<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_is_op_return">is_op_return</a>(self: &<a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>): bool
</code></pre>



<a name="0x4_script_buf_push_opcode"></a>

## Function `push_opcode`



<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_push_opcode">push_opcode</a>(self: &<b>mut</b> <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>, <a href="opcode.md#0x4_opcode">opcode</a>: u8)
</code></pre>



<a name="0x4_script_buf_push_data"></a>

## Function `push_data`



<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_push_data">push_data</a>(self: &<b>mut</b> <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>, data: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x4_script_buf_push_int"></a>

## Function `push_int`

Adds instructions to push an integer onto the stack.

Integers are encoded as little-endian signed-magnitude numbers, but there are dedicated
opcodes to push some small integers.
Because there no i64 type in Move, we use u64 to represent the integer.
The value over the I64_MAX will abort, we can support negative value in the future.


<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_push_int">push_int</a>(self: &<b>mut</b> <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>, n: u64)
</code></pre>



<a name="0x4_script_buf_push_key"></a>

## Function `push_key`

Push a Bitcoin public key to the script


<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_push_key">push_key</a>(self: &<b>mut</b> <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>, key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x4_script_buf_push_x_only_key"></a>

## Function `push_x_only_key`

Push a Bitcoin x-only public key to the script


<pre><code><b>public</b> <b>fun</b> <a href="script_buf.md#0x4_script_buf_push_x_only_key">push_x_only_key</a>(self: &<b>mut</b> <a href="script_buf.md#0x4_script_buf_ScriptBuf">script_buf::ScriptBuf</a>, key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>
