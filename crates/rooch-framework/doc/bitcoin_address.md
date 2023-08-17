
<a name="0x3_bitcoin_address"></a>

# Module `0x3::bitcoin_address`



-  [Struct `BTCAddress`](#0x3_bitcoin_address_BTCAddress)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x3_bitcoin_address_new)
-  [Function `as_bytes`](#0x3_bitcoin_address_as_bytes)
-  [Function `into_bytes`](#0x3_bitcoin_address_into_bytes)
-  [Function `create_p2pkh_address`](#0x3_bitcoin_address_create_p2pkh_address)
-  [Function `create_p2sh_address`](#0x3_bitcoin_address_create_p2sh_address)
-  [Function `create_bech32_address`](#0x3_bitcoin_address_create_bech32_address)


<pre><code><b>use</b> <a href="encoding.md#0x3_encoding">0x3::encoding</a>;
</code></pre>



<a name="0x3_bitcoin_address_BTCAddress"></a>

## Struct `BTCAddress`



<pre><code><b>struct</b> <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>bytes: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_bitcoin_address_BECH32_ADDR_LENGTH"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_BECH32_ADDR_LENGTH">BECH32_ADDR_LENGTH</a>: u64 = 42;
</code></pre>



<a name="0x3_bitcoin_address_P2PKH_ADDR_LENGTH"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2PKH_ADDR_LENGTH">P2PKH_ADDR_LENGTH</a>: u64 = 34;
</code></pre>



<a name="0x3_bitcoin_address_P2SH_ADDR_LENGTH"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2SH_ADDR_LENGTH">P2SH_ADDR_LENGTH</a>: u64 = 34;
</code></pre>



<a name="0x3_bitcoin_address_P2TR_ADDR_LENGTH"></a>



<pre><code><b>const</b> <a href="bitcoin_address.md#0x3_bitcoin_address_P2TR_ADDR_LENGTH">P2TR_ADDR_LENGTH</a>: u64 = 62;
</code></pre>



<a name="0x3_bitcoin_address_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_new">new</a>(pub_key: <a href="">vector</a>&lt;u8&gt;, decimal_prefix: u8, version: u8): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_new">new</a>(pub_key: <a href="">vector</a>&lt;u8&gt;, decimal_prefix: u8, version: u8): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a> {
    <b>let</b> <a href="bitcoin_address.md#0x3_bitcoin_address">bitcoin_address</a> = <b>if</b> (decimal_prefix == 0) { // P2PKH <b>address</b>
        <a href="bitcoin_address.md#0x3_bitcoin_address_create_p2pkh_address">create_p2pkh_address</a>(pub_key)
    } <b>else</b> <b>if</b> (decimal_prefix == 5) { // P2SH <b>address</b>
        <a href="bitcoin_address.md#0x3_bitcoin_address_create_p2sh_address">create_p2sh_address</a>(pub_key)
    } <b>else</b> { // Segwit Bech32 or Taproot Bech32m <b>address</b>
        <a href="bitcoin_address.md#0x3_bitcoin_address_create_bech32_address">create_bech32_address</a>(pub_key, version)
    };

    <a href="bitcoin_address.md#0x3_bitcoin_address">bitcoin_address</a>
}
</code></pre>



</details>

<a name="0x3_bitcoin_address_as_bytes"></a>

## Function `as_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_as_bytes">as_bytes</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_as_bytes">as_bytes</a>(addr: &<a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a>): &<a href="">vector</a>&lt;u8&gt; {
    &addr.bytes
}
</code></pre>



</details>

<a name="0x3_bitcoin_address_into_bytes"></a>

## Function `into_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_into_bytes">into_bytes</a>(addr: <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_into_bytes">into_bytes</a>(addr: <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a>): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a> { bytes } = addr;
    bytes
}
</code></pre>



</details>

<a name="0x3_bitcoin_address_create_p2pkh_address"></a>

## Function `create_p2pkh_address`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_create_p2pkh_address">create_p2pkh_address</a>(pub_key: <a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_create_p2pkh_address">create_p2pkh_address</a>(pub_key: <a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a> {
    <b>let</b> address_bytes = <a href="encoding.md#0x3_encoding_p2pkh">encoding::p2pkh</a>(&pub_key);

    <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a> {
        bytes: address_bytes
    }
}
</code></pre>



</details>

<a name="0x3_bitcoin_address_create_p2sh_address"></a>

## Function `create_p2sh_address`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_create_p2sh_address">create_p2sh_address</a>(pub_key: <a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_create_p2sh_address">create_p2sh_address</a>(pub_key: <a href="">vector</a>&lt;u8&gt;): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a> {
    <b>let</b> address_bytes = <a href="encoding.md#0x3_encoding_p2sh">encoding::p2sh</a>(&pub_key);

    <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a> {
        bytes: address_bytes
    }
}
</code></pre>



</details>

<a name="0x3_bitcoin_address_create_bech32_address"></a>

## Function `create_bech32_address`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_create_bech32_address">create_bech32_address</a>(pub_key: <a href="">vector</a>&lt;u8&gt;, version: u8): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_address.md#0x3_bitcoin_address_create_bech32_address">create_bech32_address</a>(pub_key: <a href="">vector</a>&lt;u8&gt;, version: u8): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a> {
    <b>let</b> address_bytes = <a href="encoding.md#0x3_encoding_bech32">encoding::bech32</a>(&pub_key, version);

    <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">BTCAddress</a> {
        bytes: address_bytes
    }
}
</code></pre>



</details>
