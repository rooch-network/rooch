
<a name="0x3_bitcoin_address"></a>

# Module `0x3::bitcoin_address`



-  [Struct `BTCAddress`](#0x3_bitcoin_address_BTCAddress)
-  [Constants](#@Constants_0)
-  [Function `from_script`](#0x3_bitcoin_address_from_script)
-  [Function `from_bytes`](#0x3_bitcoin_address_from_bytes)
-  [Function `is_p2pkh`](#0x3_bitcoin_address_is_p2pkh)
-  [Function `is_p2sh`](#0x3_bitcoin_address_is_p2sh)
-  [Function `is_witness_program`](#0x3_bitcoin_address_is_witness_program)
-  [Function `as_bytes`](#0x3_bitcoin_address_as_bytes)
-  [Function `into_bytes`](#0x3_bitcoin_address_into_bytes)
-  [Function `to_bech32`](#0x3_bitcoin_address_to_bech32)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf">0x3::bitcoin_script_buf</a>;
</code></pre>



<a name="0x3_bitcoin_address_BTCAddress"></a>

## Struct `BTCAddress`

BTCAddress is a struct that represents a Bitcoin address.
We just keep the raw bytes of the address and do care about the network.


<pre><code><b>struct</b> <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_bitcoin_address_ErrorAddressBytesLen"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_ErrorAddressBytesLen">ErrorAddressBytesLen</a>: u64 = 1;
</code></pre>



<a name="0x3_bitcoin_address_P2PKH_ADDR_BYTE_LEN"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2PKH_ADDR_BYTE_LEN">P2PKH_ADDR_BYTE_LEN</a>: u64 = 21;
</code></pre>



<a name="0x3_bitcoin_address_P2PKH_ADDR_DECIMAL_PREFIX_MAIN"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2PKH_ADDR_DECIMAL_PREFIX_MAIN">P2PKH_ADDR_DECIMAL_PREFIX_MAIN</a>: u8 = 0;
</code></pre>



<a name="0x3_bitcoin_address_P2PKH_ADDR_DECIMAL_PREFIX_TEST"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2PKH_ADDR_DECIMAL_PREFIX_TEST">P2PKH_ADDR_DECIMAL_PREFIX_TEST</a>: u8 = 111;
</code></pre>



<a name="0x3_bitcoin_address_P2SH_ADDR_BYTE_LEN"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2SH_ADDR_BYTE_LEN">P2SH_ADDR_BYTE_LEN</a>: u64 = 21;
</code></pre>



<a name="0x3_bitcoin_address_P2SH_ADDR_DECIMAL_PREFIX_MAIN"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2SH_ADDR_DECIMAL_PREFIX_MAIN">P2SH_ADDR_DECIMAL_PREFIX_MAIN</a>: u8 = 5;
</code></pre>



<a name="0x3_bitcoin_address_P2SH_ADDR_DECIMAL_PREFIX_TEST"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2SH_ADDR_DECIMAL_PREFIX_TEST">P2SH_ADDR_DECIMAL_PREFIX_TEST</a>: u8 = 196;
</code></pre>



<a name="0x3_bitcoin_address_PUBKEY_HASH_LEN"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_PUBKEY_HASH_LEN">PUBKEY_HASH_LEN</a>: u64 = 20;
</code></pre>



<a name="0x3_bitcoin_address_SCRIPT_HASH_LEN"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_SCRIPT_HASH_LEN">SCRIPT_HASH_LEN</a>: u64 = 20;
</code></pre>



<a name="0x3_bitcoin_address_from_script"></a>

## Function `from_script`

from_script returns a BTCAddress from a ScriptBuf.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_from_script">from_script</a>(s: &<a href="bitcoin_script_buf.md#0x3_bitcoin_script_buf_ScriptBuf">bitcoin_script_buf::ScriptBuf</a>): <a href="_Option">option::Option</a>&lt;<a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>&gt;
</code></pre>



<a name="0x3_bitcoin_address_from_bytes"></a>

## Function `from_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_from_bytes">from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>
</code></pre>



<a name="0x3_bitcoin_address_is_p2pkh"></a>

## Function `is_p2pkh`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_is_p2pkh">is_p2pkh</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>): bool
</code></pre>



<a name="0x3_bitcoin_address_is_p2sh"></a>

## Function `is_p2sh`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_is_p2sh">is_p2sh</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>): bool
</code></pre>



<a name="0x3_bitcoin_address_is_witness_program"></a>

## Function `is_witness_program`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_is_witness_program">is_witness_program</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>): bool
</code></pre>



<a name="0x3_bitcoin_address_as_bytes"></a>

## Function `as_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_as_bytes">as_bytes</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_address_into_bytes"></a>

## Function `into_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_into_bytes">into_bytes</a>(addr: <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_address_to_bech32"></a>

## Function `to_bech32`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_to_bech32">to_bech32</a>(_addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>): <a href="_String">string::String</a>
</code></pre>
