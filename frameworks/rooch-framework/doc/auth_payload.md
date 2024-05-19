
<a name="0x3_auth_payload"></a>

# Module `0x3::auth_payload`



-  [Struct `AuthPayload`](#0x3_auth_payload_AuthPayload)
-  [Function `from_bytes`](#0x3_auth_payload_from_bytes)
-  [Function `sign`](#0x3_auth_payload_sign)
-  [Function `sign_info_prefix`](#0x3_auth_payload_sign_info_prefix)
-  [Function `sign_info`](#0x3_auth_payload_sign_info)
-  [Function `public_key`](#0x3_auth_payload_public_key)
-  [Function `from_address`](#0x3_auth_payload_from_address)


<pre><code><b>use</b> <a href="">0x2::bcs</a>;
</code></pre>



<a name="0x3_auth_payload_AuthPayload"></a>

## Struct `AuthPayload`



<pre><code>#[data_struct]
<b>struct</b> <a href="auth_payload.md#0x3_auth_payload_AuthPayload">AuthPayload</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_auth_payload_from_bytes"></a>

## Function `from_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_from_bytes">from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>
</code></pre>



<a name="0x3_auth_payload_sign"></a>

## Function `sign`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_sign">sign</a>(payload: <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_sign_info_prefix"></a>

## Function `sign_info_prefix`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_sign_info_prefix">sign_info_prefix</a>(payload: <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_sign_info"></a>

## Function `sign_info`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_sign_info">sign_info</a>(payload: <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_public_key"></a>

## Function `public_key`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_public_key">public_key</a>(payload: <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_from_address"></a>

## Function `from_address`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_from_address">from_address</a>(payload: <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>
