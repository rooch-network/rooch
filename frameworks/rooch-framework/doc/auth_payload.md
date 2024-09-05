
<a name="0x3_auth_payload"></a>

# Module `0x3::auth_payload`



-  [Struct `AuthPayload`](#0x3_auth_payload_AuthPayload)
-  [Struct `MultisignAuthPayload`](#0x3_auth_payload_MultisignAuthPayload)
-  [Struct `SignData`](#0x3_auth_payload_SignData)
-  [Constants](#@Constants_0)
-  [Function `new_sign_data`](#0x3_auth_payload_new_sign_data)
-  [Function `from_bytes`](#0x3_auth_payload_from_bytes)
-  [Function `encode_full_message`](#0x3_auth_payload_encode_full_message)
-  [Function `signature`](#0x3_auth_payload_signature)
-  [Function `message_prefix`](#0x3_auth_payload_message_prefix)
-  [Function `message_info`](#0x3_auth_payload_message_info)
-  [Function `public_key`](#0x3_auth_payload_public_key)
-  [Function `from_address`](#0x3_auth_payload_from_address)
-  [Function `multisign_from_bytes`](#0x3_auth_payload_multisign_from_bytes)
-  [Function `multisign_signatures`](#0x3_auth_payload_multisign_signatures)
-  [Function `multisign_message_prefix`](#0x3_auth_payload_multisign_message_prefix)
-  [Function `multisign_message_info`](#0x3_auth_payload_multisign_message_info)
-  [Function `multisign_public_keys`](#0x3_auth_payload_multisign_public_keys)
-  [Function `multisign_encode_full_message`](#0x3_auth_payload_multisign_encode_full_message)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::consensus_codec</a>;
<b>use</b> <a href="">0x2::hex</a>;
</code></pre>



<a name="0x3_auth_payload_AuthPayload"></a>

## Struct `AuthPayload`



<pre><code>#[data_struct]
<b>struct</b> <a href="auth_payload.md#0x3_auth_payload_AuthPayload">AuthPayload</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_auth_payload_MultisignAuthPayload"></a>

## Struct `MultisignAuthPayload`



<pre><code>#[data_struct]
<b>struct</b> <a href="auth_payload.md#0x3_auth_payload_MultisignAuthPayload">MultisignAuthPayload</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_auth_payload_SignData"></a>

## Struct `SignData`



<pre><code>#[data_struct]
<b>struct</b> <a href="auth_payload.md#0x3_auth_payload_SignData">SignData</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_auth_payload_ErrorInvalidSignature"></a>



<pre><code><b>const</b> <a href="auth_payload.md#0x3_auth_payload_ErrorInvalidSignature">ErrorInvalidSignature</a>: u64 = 1;
</code></pre>



<a name="0x3_auth_payload_MessageInfoPrefix"></a>



<pre><code><b>const</b> <a href="auth_payload.md#0x3_auth_payload_MessageInfoPrefix">MessageInfoPrefix</a>: <a href="">vector</a>&lt;u8&gt; = [82, 111, 111, 99, 104, 32, 84, 114, 97, 110, 115, 97, 99, 116, 105, 111, 110, 58, 10];
</code></pre>



<a name="0x3_auth_payload_MessagePrefix"></a>



<pre><code><b>const</b> <a href="auth_payload.md#0x3_auth_payload_MessagePrefix">MessagePrefix</a>: <a href="">vector</a>&lt;u8&gt; = [66, 105, 116, 99, 111, 105, 110, 32, 83, 105, 103, 110, 101, 100, 32, 77, 101, 115, 115, 97, 103, 101, 58, 10];
</code></pre>



<a name="0x3_auth_payload_new_sign_data"></a>

## Function `new_sign_data`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_new_sign_data">new_sign_data</a>(message_prefix: <a href="">vector</a>&lt;u8&gt;, message_info: <a href="">vector</a>&lt;u8&gt;): <a href="auth_payload.md#0x3_auth_payload_SignData">auth_payload::SignData</a>
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



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_signature">signature</a>(payload: &<a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_message_prefix"></a>

## Function `message_prefix`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_message_prefix">message_prefix</a>(payload: &<a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_message_info"></a>

## Function `message_info`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_message_info">message_info</a>(payload: &<a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_public_key"></a>

## Function `public_key`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_public_key">public_key</a>(payload: &<a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_from_address"></a>

## Function `from_address`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_from_address">from_address</a>(payload: &<a href="auth_payload.md#0x3_auth_payload_AuthPayload">auth_payload::AuthPayload</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_auth_payload_multisign_from_bytes"></a>

## Function `multisign_from_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_multisign_from_bytes">multisign_from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="auth_payload.md#0x3_auth_payload_MultisignAuthPayload">auth_payload::MultisignAuthPayload</a>
</code></pre>



<a name="0x3_auth_payload_multisign_signatures"></a>

## Function `multisign_signatures`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_multisign_signatures">multisign_signatures</a>(payload: &<a href="auth_payload.md#0x3_auth_payload_MultisignAuthPayload">auth_payload::MultisignAuthPayload</a>): &<a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_auth_payload_multisign_message_prefix"></a>

## Function `multisign_message_prefix`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_multisign_message_prefix">multisign_message_prefix</a>(payload: &<a href="auth_payload.md#0x3_auth_payload_MultisignAuthPayload">auth_payload::MultisignAuthPayload</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_multisign_message_info"></a>

## Function `multisign_message_info`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_multisign_message_info">multisign_message_info</a>(payload: &<a href="auth_payload.md#0x3_auth_payload_MultisignAuthPayload">auth_payload::MultisignAuthPayload</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_auth_payload_multisign_public_keys"></a>

## Function `multisign_public_keys`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_multisign_public_keys">multisign_public_keys</a>(payload: &<a href="auth_payload.md#0x3_auth_payload_MultisignAuthPayload">auth_payload::MultisignAuthPayload</a>): &<a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_auth_payload_multisign_encode_full_message"></a>

## Function `multisign_encode_full_message`



<pre><code><b>public</b> <b>fun</b> <a href="auth_payload.md#0x3_auth_payload_multisign_encode_full_message">multisign_encode_full_message</a>(self: &<a href="auth_payload.md#0x3_auth_payload_MultisignAuthPayload">auth_payload::MultisignAuthPayload</a>, tx_hash: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>
