
<a name="0x3_auth_payload"></a>

# Module `0x3::auth_payload`



-  [Struct `AuthPayload`](#0x3_auth_payload_AuthPayload)
-  [Constants](#@Constants_0)
-  [Function `from_bytes`](#0x3_auth_payload_from_bytes)
-  [Function `encode_full_message`](#0x3_auth_payload_encode_full_message)
-  [Function `signature`](#0x3_auth_payload_signature)
-  [Function `message_prefix`](#0x3_auth_payload_message_prefix)
-  [Function `message_info`](#0x3_auth_payload_message_info)
-  [Function `public_key`](#0x3_auth_payload_public_key)
-  [Function `from_address`](#0x3_auth_payload_from_address)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::hex</a>;
</code></pre>



<a name="0x3_auth_payload_AuthPayload"></a>

## Struct `AuthPayload`



<pre><code>#[data_struct]
<b>struct</b> <a href="auth_payload.md#0x3_auth_payload_AuthPayload">AuthPayload</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_auth_payload_ErrorInvalidSignature"></a>



<pre><code><b>const</b> <a href="auth_payload.md#0x3_auth_payload_ErrorInvalidSignature">ErrorInvalidSignature</a>: u64 = 1;
</code></pre>



<a name="0x3_auth_payload_MessgaeInfoPrefix"></a>



<pre><code><b>const</b> <a href="auth_payload.md#0x3_auth_payload_MessgaeInfoPrefix">MessgaeInfoPrefix</a>: <a href="">vector</a>&lt;u8&gt; = [82, 111, 111, 99, 104, 32, 84, 114, 97, 110, 115, 97, 99, 116, 105, 111, 110, 58, 10];
</code></pre>



<a name="0x3_auth_payload_from_bytes"></a>

## Function `from_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_from_bytes">from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>
</code></pre>



<a name="0x3_auth_payload_encode_full_message"></a>

## Function `encode_full_message`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_encode_full_message">encode_full_message</a>(self: &<a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>, tx_hash: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_signature"></a>

## Function `signature`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_signature">signature</a>(payload: <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_message_prefix"></a>

## Function `message_prefix`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_message_prefix">message_prefix</a>(payload: <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_message_info"></a>

## Function `message_info`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_message_info">message_info</a>(payload: <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_public_key"></a>

## Function `public_key`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_public_key">public_key</a>(payload: <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_from_address"></a>

## Function `from_address`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_from_address">from_address</a>(payload: <a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="_String">string::String</a>
</code></pre>
