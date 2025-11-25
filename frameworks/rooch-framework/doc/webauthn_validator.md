
<a name="0x3_webauthn_validator"></a>

# Module `0x3::webauthn_validator`

WebAuthn validator implementation (DEPRECATED)

This validator has been deprecated. Please use did_validator (ID=4) with WebAuthnV0 envelope instead.
Migration: Use Authenticator.did(txHash, signer, vmFragment, SigningEnvelope.WebAuthnV0)


-  [Struct `WebauthnValidator`](#0x3_webauthn_validator_WebauthnValidator)
-  [Struct `WebauthnAuthPayload`](#0x3_webauthn_validator_WebauthnAuthPayload)
-  [Struct `ClientData`](#0x3_webauthn_validator_ClientData)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0x3_webauthn_validator_auth_validator_id)
-  [Function `validate`](#0x3_webauthn_validator_validate)
-  [Function `unwrap_webauthn_auth_payload`](#0x3_webauthn_validator_unwrap_webauthn_auth_payload)
-  [Function `unwrap_client_data`](#0x3_webauthn_validator_unwrap_client_data)


<pre><code><b>use</b> <a href="">0x1::string</a>;
</code></pre>



<a name="0x3_webauthn_validator_WebauthnValidator"></a>

## Struct `WebauthnValidator`



<pre><code><b>struct</b> <a href="webauthn_validator.md#0x3_webauthn_validator_WebauthnValidator">WebauthnValidator</a> <b>has</b> drop, store
</code></pre>



<a name="0x3_webauthn_validator_WebauthnAuthPayload"></a>

## Struct `WebauthnAuthPayload`

BCS-serialised payload sent by the browser / SDK (DEPRECATED)
This struct is kept for compatibility but should not be used.


<pre><code>#[data_struct]
<b>struct</b> <a href="webauthn_validator.md#0x3_webauthn_validator_WebauthnAuthPayload">WebauthnAuthPayload</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_webauthn_validator_ClientData"></a>

## Struct `ClientData`

Client data struct (DEPRECATED)
This struct is kept for compatibility but should not be used.


<pre><code>#[data_struct]
<b>struct</b> <a href="webauthn_validator.md#0x3_webauthn_validator_ClientData">ClientData</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_webauthn_validator_ErrorValidatorDeprecated"></a>

Error code indicating this validator has been deprecated


<pre><code><b>const</b> <a href="webauthn_validator.md#0x3_webauthn_validator_ErrorValidatorDeprecated">ErrorValidatorDeprecated</a>: u64 = 2001;
</code></pre>



<a name="0x3_webauthn_validator_WEBAUTHN_AUTH_VALIDATOR_ID"></a>

Identifier reserved for the WebAuthn validator. Must stay in sync with
<code><a href="builtin_validators.md#0x3_builtin_validators">builtin_validators</a>.<b>move</b></code>.


<pre><code><b>const</b> <a href="webauthn_validator.md#0x3_webauthn_validator_WEBAUTHN_AUTH_VALIDATOR_ID">WEBAUTHN_AUTH_VALIDATOR_ID</a>: u64 = 3;
</code></pre>



<a name="0x3_webauthn_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="webauthn_validator.md#0x3_webauthn_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0x3_webauthn_validator_validate"></a>

## Function `validate`

Validate the incoming authenticator payload (DEPRECATED)
This function always aborts with ErrorValidatorDeprecated.
Please migrate to did_validator (ID=4) with WebAuthnV0 envelope.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="webauthn_validator.md#0x3_webauthn_validator_validate">validate</a>(_authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_webauthn_validator_unwrap_webauthn_auth_payload"></a>

## Function `unwrap_webauthn_auth_payload`

Unwrap WebAuthn auth payload (DEPRECATED)
This function is kept for compatibility but should not be used.


<pre><code><b>public</b> <b>fun</b> <a href="webauthn_validator.md#0x3_webauthn_validator_unwrap_webauthn_auth_payload">unwrap_webauthn_auth_payload</a>(payload: <a href="webauthn_validator.md#0x3_webauthn_validator_WebauthnAuthPayload">webauthn_validator::WebauthnAuthPayload</a>): (u8, <a href="">vector</a>&lt;u8&gt;, <a href="">vector</a>&lt;u8&gt;, <a href="">vector</a>&lt;u8&gt;, <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_webauthn_validator_unwrap_client_data"></a>

## Function `unwrap_client_data`

Unwrap client data (DEPRECATED)
This function is kept for compatibility but should not be used.


<pre><code><b>public</b> <b>fun</b> <a href="webauthn_validator.md#0x3_webauthn_validator_unwrap_client_data">unwrap_client_data</a>(client_data: <a href="webauthn_validator.md#0x3_webauthn_validator_ClientData">webauthn_validator::ClientData</a>): (<a href="_String">string::String</a>, <a href="_String">string::String</a>, <a href="_String">string::String</a>)
</code></pre>
