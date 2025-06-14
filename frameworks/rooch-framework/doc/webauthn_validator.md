
<a name="0x3_webauthn_validator"></a>

# Module `0x3::webauthn_validator`

WebAuthn validator implementation (P-256 / secp256r1)

Payload layout (see docs/dev-guide/webauthn_validator.md Section 3.1):
```
???????????????????????????????????????????????????????????????????????????
? 1 B    ? 64 B       ? 33 B      ? 4 B + *           ? *                 ?
? scheme ? signature  ? publicKey ? authenticatorData ? clientDataJSON    ?
???????????????????????????????????????????????????????????????????????????

After the fixed-length fields we encode <code>authenticatorData</code> length as a
4-byte big-endian unsigned integer so that the boundary between
<code>authenticatorData</code> and <code>clientDataJSON</code> can be determined on-chain.

The validator reconstructs the message as
authenticatorData || SHA-256(clientDataJSON)
and verifies it with the provided signature and compressed P-256 public key.

On success the corresponding session authentication key is returned.


-  [Struct `WebauthnValidator`](#0x3_webauthn_validator_WebauthnValidator)
-  [Struct `WebauthnAuthPayload`](#0x3_webauthn_validator_WebauthnAuthPayload)
-  [Struct `ClientData`](#0x3_webauthn_validator_ClientData)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0x3_webauthn_validator_auth_validator_id)
-  [Function `unwrap_webauthn_auth_payload`](#0x3_webauthn_validator_unwrap_webauthn_auth_payload)
-  [Function `unwrap_client_data`](#0x3_webauthn_validator_unwrap_client_data)
-  [Function `validate`](#0x3_webauthn_validator_validate)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::base64</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::hash</a>;
<b>use</b> <a href="">0x2::json</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="did.md#0x3_did">0x3::did</a>;
<b>use</b> <a href="ecdsa_r1.md#0x3_ecdsa_r1">0x3::ecdsa_r1</a>;
<b>use</b> <a href="session_key.md#0x3_session_key">0x3::session_key</a>;
</code></pre>



<a name="0x3_webauthn_validator_WebauthnValidator"></a>

## Struct `WebauthnValidator`



<pre><code><b>struct</b> <a href="webauthn_validator.md#0x3_webauthn_validator_WebauthnValidator">WebauthnValidator</a> <b>has</b> drop, store
</code></pre>



<a name="0x3_webauthn_validator_WebauthnAuthPayload"></a>

## Struct `WebauthnAuthPayload`

BCS-serialised payload sent by the browser / SDK. This avoids manual
offset parsing on-chain.


<pre><code>#[data_struct]
<b>struct</b> <a href="webauthn_validator.md#0x3_webauthn_validator_WebauthnAuthPayload">WebauthnAuthPayload</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_webauthn_validator_ClientData"></a>

## Struct `ClientData`



<pre><code>#[data_struct]
<b>struct</b> <a href="webauthn_validator.md#0x3_webauthn_validator_ClientData">ClientData</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_webauthn_validator_WEBAUTHN_AUTH_VALIDATOR_ID"></a>

Identifier reserved for the WebAuthn validator. Must stay in sync with
<code><a href="builtin_validators.md#0x3_builtin_validators">builtin_validators</a>.<b>move</b></code>.


<pre><code><b>const</b> <a href="webauthn_validator.md#0x3_webauthn_validator_WEBAUTHN_AUTH_VALIDATOR_ID">WEBAUTHN_AUTH_VALIDATOR_ID</a>: u64 = 3;
</code></pre>



<a name="0x3_webauthn_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="webauthn_validator.md#0x3_webauthn_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0x3_webauthn_validator_unwrap_webauthn_auth_payload"></a>

## Function `unwrap_webauthn_auth_payload`



<pre><code><b>public</b> <b>fun</b> <a href="webauthn_validator.md#0x3_webauthn_validator_unwrap_webauthn_auth_payload">unwrap_webauthn_auth_payload</a>(payload: <a href="webauthn_validator.md#0x3_webauthn_validator_WebauthnAuthPayload">webauthn_validator::WebauthnAuthPayload</a>): (u8, <a href="">vector</a>&lt;u8&gt;, <a href="">vector</a>&lt;u8&gt;, <a href="">vector</a>&lt;u8&gt;, <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_webauthn_validator_unwrap_client_data"></a>

## Function `unwrap_client_data`



<pre><code><b>public</b> <b>fun</b> <a href="webauthn_validator.md#0x3_webauthn_validator_unwrap_client_data">unwrap_client_data</a>(client_data: <a href="webauthn_validator.md#0x3_webauthn_validator_ClientData">webauthn_validator::ClientData</a>): (<a href="_String">string::String</a>, <a href="_String">string::String</a>, <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_webauthn_validator_validate"></a>

## Function `validate`

Validate the incoming authenticator payload and return the derived authentication key


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="webauthn_validator.md#0x3_webauthn_validator_validate">validate</a>(authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>
