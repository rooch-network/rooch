
<a name="0x3_bitcoin_address"></a>

# Module `0x3::bitcoin_address`



-  [Struct `BTCAddress`](#0x3_bitcoin_address_BTCAddress)
-  [Constants](#@Constants_0)
-  [Function `new_legacy`](#0x3_bitcoin_address_new_legacy)
-  [Function `new_bech32`](#0x3_bitcoin_address_new_bech32)
-  [Function `from_bytes`](#0x3_bitcoin_address_from_bytes)
-  [Function `as_bytes`](#0x3_bitcoin_address_as_bytes)
-  [Function `into_bytes`](#0x3_bitcoin_address_into_bytes)
-  [Function `create_p2pkh_address`](#0x3_bitcoin_address_create_p2pkh_address)
-  [Function `create_p2sh_address`](#0x3_bitcoin_address_create_p2sh_address)
-  [Function `create_bech32_address`](#0x3_bitcoin_address_create_bech32_address)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1">0x3::ecdsa_k1</a>;
<b>use</b> <a href="encoding.md#0x3_encoding">0x3::encoding</a>;
</code></pre>



<a name="0x3_bitcoin_address_BTCAddress"></a>

## Struct `BTCAddress`



<pre><code><b>struct</b> <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_bitcoin_address_ErrorInvalidScriptVersion"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_ErrorInvalidScriptVersion">ErrorInvalidScriptVersion</a>: u64 = 2;
</code></pre>



<a name="0x3_bitcoin_address_BECH32_ADDR_LENGTH"></a>

Bech32 addresses including P2WPKH and P2WSH are 42 characters


<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_BECH32_ADDR_LENGTH">BECH32_ADDR_LENGTH</a>: u64 = 42;
</code></pre>



<a name="0x3_bitcoin_address_ErrorInvalidCompressedPublicKeyLength"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_ErrorInvalidCompressedPublicKeyLength">ErrorInvalidCompressedPublicKeyLength</a>: u64 = 3;
</code></pre>



<a name="0x3_bitcoin_address_ErrorInvalidDecimalPrefix"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_ErrorInvalidDecimalPrefix">ErrorInvalidDecimalPrefix</a>: u64 = 1;
</code></pre>



<a name="0x3_bitcoin_address_ErrorInvalidHashedPublicKeyLength"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_ErrorInvalidHashedPublicKeyLength">ErrorInvalidHashedPublicKeyLength</a>: u64 = 4;
</code></pre>



<a name="0x3_bitcoin_address_ErrorInvalidSchnorrPublicKeyLength"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_ErrorInvalidSchnorrPublicKeyLength">ErrorInvalidSchnorrPublicKeyLength</a>: u64 = 5;
</code></pre>



<a name="0x3_bitcoin_address_P2PKH_ADDR_DECIMAL_PREFIX"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2PKH_ADDR_DECIMAL_PREFIX">P2PKH_ADDR_DECIMAL_PREFIX</a>: u8 = 0;
</code></pre>



<a name="0x3_bitcoin_address_P2PKH_ADDR_LENGTH"></a>

P2PKH addresses are 34 characters


<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2PKH_ADDR_LENGTH">P2PKH_ADDR_LENGTH</a>: u64 = 34;
</code></pre>



<a name="0x3_bitcoin_address_P2SH_ADDR_DECIMAL_PREFIX"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2SH_ADDR_DECIMAL_PREFIX">P2SH_ADDR_DECIMAL_PREFIX</a>: u8 = 5;
</code></pre>



<a name="0x3_bitcoin_address_P2SH_ADDR_LENGTH"></a>

P2SH addresses are 34 characters


<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2SH_ADDR_LENGTH">P2SH_ADDR_LENGTH</a>: u64 = 34;
</code></pre>



<a name="0x3_bitcoin_address_P2TR_ADDR_LENGTH"></a>

P2TR addresses with Bech32m encoding are 62 characters


<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2TR_ADDR_LENGTH">P2TR_ADDR_LENGTH</a>: u64 = 62;
</code></pre>



<a name="0x3_bitcoin_address_new_legacy"></a>

## Function `new_legacy`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_new_legacy">new_legacy</a>(pub_key: &<a href="">vector</a>&lt;u8&gt;, decimal_prefix: u8): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>
</code></pre>



<a name="0x3_bitcoin_address_new_bech32"></a>

## Function `new_bech32`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_new_bech32">new_bech32</a>(pub_key: &<a href="">vector</a>&lt;u8&gt;, version: u8): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>
</code></pre>



<a name="0x3_bitcoin_address_from_bytes"></a>

## Function `from_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_from_bytes">from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>
</code></pre>



<a name="0x3_bitcoin_address_as_bytes"></a>

## Function `as_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_as_bytes">as_bytes</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_address_into_bytes"></a>

## Function `into_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_into_bytes">into_bytes</a>(addr: <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_bitcoin_address_create_p2pkh_address"></a>

## Function `create_p2pkh_address`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_create_p2pkh_address">create_p2pkh_address</a>(pub_key: &<a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>
</code></pre>



<a name="0x3_bitcoin_address_create_p2sh_address"></a>

## Function `create_p2sh_address`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_create_p2sh_address">create_p2sh_address</a>(pub_key: &<a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>
</code></pre>



<a name="0x3_bitcoin_address_create_bech32_address"></a>

## Function `create_bech32_address`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_create_bech32_address">create_bech32_address</a>(pub_key: &<a href="">vector</a>&lt;u8&gt;, version: u8): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>
</code></pre>
