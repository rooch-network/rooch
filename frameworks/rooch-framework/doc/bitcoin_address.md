
<a name="0x3_bitcoin_address"></a>

# Module `0x3::bitcoin_address`



-  [Struct `BitcoinAddress`](#0x3_bitcoin_address_BitcoinAddress)
-  [Constants](#@Constants_0)
-  [Function `new_p2pkh`](#0x3_bitcoin_address_new_p2pkh)
-  [Function `new_p2sh`](#0x3_bitcoin_address_new_p2sh)
-  [Function `new_witness_program`](#0x3_bitcoin_address_new_witness_program)
-  [Function `new`](#0x3_bitcoin_address_new)
-  [Function `is_p2pkh`](#0x3_bitcoin_address_is_p2pkh)
-  [Function `is_p2sh`](#0x3_bitcoin_address_is_p2sh)
-  [Function `is_witness_program`](#0x3_bitcoin_address_is_witness_program)
-  [Function `is_empty`](#0x3_bitcoin_address_is_empty)
-  [Function `as_bytes`](#0x3_bitcoin_address_as_bytes)
-  [Function `into_bytes`](#0x3_bitcoin_address_into_bytes)
<<<<<<< HEAD
-  [Function `from_string`](#0x3_bitcoin_address_from_string)
-  [Function `verify_with_public_key`](#0x3_bitcoin_address_verify_with_public_key)
-  [Function `to_rooch_address`](#0x3_bitcoin_address_to_rooch_address)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::hash</a>;
=======
-  [Function `new`](#0x3_bitcoin_address_new)
-  [Function `verify_with_pk`](#0x3_bitcoin_address_verify_with_pk)


<pre><code><b>use</b> <a href="">0x1::vector</a>;
>>>>>>> a1277c33 (clean some todo)
</code></pre>



<a name="0x3_bitcoin_address_BitcoinAddress"></a>

## Struct `BitcoinAddress`

BitcoinAddress is a struct that represents a Bitcoin address.
We just keep the raw bytes of the address and do care about the network.


<pre><code>#[data_struct]
<b>struct</b> <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">BitcoinAddress</a> <b>has</b> <b>copy</b>, drop, store
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



<a name="0x3_bitcoin_address_new_p2pkh"></a>

## Function `new_p2pkh`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_new_p2pkh">new_p2pkh</a>(pubkey_hash: <a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>
</code></pre>



<a name="0x3_bitcoin_address_new_p2sh"></a>

## Function `new_p2sh`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_new_p2sh">new_p2sh</a>(script_hash: <a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>
</code></pre>



<a name="0x3_bitcoin_address_new_witness_program"></a>

## Function `new_witness_program`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_new_witness_program">new_witness_program</a>(program: <a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>
</code></pre>



<a name="0x3_bitcoin_address_new"></a>

## Function `new`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_new">new</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>
</code></pre>



<a name="0x3_bitcoin_address_is_p2pkh"></a>

## Function `is_p2pkh`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_is_p2pkh">is_p2pkh</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>): bool
</code></pre>



<a name="0x3_bitcoin_address_is_p2sh"></a>

## Function `is_p2sh`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_is_p2sh">is_p2sh</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>): bool
</code></pre>



<a name="0x3_bitcoin_address_is_witness_program"></a>

## Function `is_witness_program`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_is_witness_program">is_witness_program</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>): bool
</code></pre>



<a name="0x3_bitcoin_address_is_empty"></a>

## Function `is_empty`

Empty address is a special address that is used to if we parse address failed from script.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_is_empty">is_empty</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>): bool
</code></pre>



<a name="0x3_bitcoin_address_as_bytes"></a>

## Function `as_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_as_bytes">as_bytes</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_address_into_bytes"></a>

## Function `into_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_into_bytes">into_bytes</a>(addr: <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<<<<<<< HEAD
<a name="0x3_bitcoin_address_from_string"></a>

## Function `from_string`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_from_string">from_string</a>(addr: &<a href="_String">string::String</a>): <a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>
</code></pre>



<a name="0x3_bitcoin_address_verify_with_public_key"></a>
=======
<a name="0x3_bitcoin_address_new"></a>
>>>>>>> a1277c33 (clean some todo)

## Function `verify_with_public_key`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_verify_with_public_key">verify_with_public_key</a>(addr: &<a href="_String">string::String</a>, pk: &<a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<a name="0x3_bitcoin_address_to_rooch_address"></a>

## Function `to_rooch_address`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_to_rooch_address">to_rooch_address</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BitcoinAddress">bitcoin_address::BitcoinAddress</a>): <b>address</b>
</code></pre>
