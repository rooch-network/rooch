
<a name="0x3_did_validator"></a>

# Module `0x3::did_validator`

This module implements the DID auth validator.
It enables direct authentication using DID Document verification methods
without requiring intermediate session key creation.


-  [Struct `DIDValidator`](#0x3_did_validator_DIDValidator)
-  [Struct `DIDAuthPayload`](#0x3_did_validator_DIDAuthPayload)
-  [Struct `WebauthnEnvelopeData`](#0x3_did_validator_WebauthnEnvelopeData)
-  [Struct `ClientData`](#0x3_did_validator_ClientData)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0x3_did_validator_auth_validator_id)
-  [Function `validate`](#0x3_did_validator_validate)


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
<b>use</b> <a href="ed25519.md#0x3_ed25519">0x3::ed25519</a>;
<b>use</b> <a href="session_key.md#0x3_session_key">0x3::session_key</a>;
</code></pre>



<a name="0x3_did_validator_DIDValidator"></a>

## Struct `DIDValidator`



<pre><code><b>struct</b> <a href="did_validator.md#0x3_did_validator_DIDValidator">DIDValidator</a> <b>has</b> drop, store
</code></pre>



<a name="0x3_did_validator_DIDAuthPayload"></a>

## Struct `DIDAuthPayload`



<pre><code>#[data_struct]
<b>struct</b> <a href="did_validator.md#0x3_did_validator_DIDAuthPayload">DIDAuthPayload</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_validator_WebauthnEnvelopeData"></a>

## Struct `WebauthnEnvelopeData`

WebAuthn envelope data (only WebAuthn-specific fields)


<pre><code>#[data_struct]
<b>struct</b> <a href="did_validator.md#0x3_did_validator_WebauthnEnvelopeData">WebauthnEnvelopeData</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_validator_ClientData"></a>

## Struct `ClientData`



<pre><code>#[data_struct]
<b>struct</b> <a href="did_validator.md#0x3_did_validator_ClientData">ClientData</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_did_validator_DID_VALIDATOR_ID"></a>

DID auth validator ID


<pre><code><b>const</b> <a href="did_validator.md#0x3_did_validator_DID_VALIDATOR_ID">DID_VALIDATOR_ID</a>: u64 = 4;
</code></pre>



<a name="0x3_did_validator_ENVELOPE_BITCOIN_MESSAGE_V0"></a>



<pre><code><b>const</b> <a href="did_validator.md#0x3_did_validator_ENVELOPE_BITCOIN_MESSAGE_V0">ENVELOPE_BITCOIN_MESSAGE_V0</a>: u8 = 1;
</code></pre>



<a name="0x3_did_validator_ENVELOPE_RAW_TX_HASH"></a>

Envelope types (same as session validator)


<pre><code><b>const</b> <a href="did_validator.md#0x3_did_validator_ENVELOPE_RAW_TX_HASH">ENVELOPE_RAW_TX_HASH</a>: u8 = 0;
</code></pre>



<a name="0x3_did_validator_ENVELOPE_WEBAUTHN_V0"></a>



<pre><code><b>const</b> <a href="did_validator.md#0x3_did_validator_ENVELOPE_WEBAUTHN_V0">ENVELOPE_WEBAUTHN_V0</a>: u8 = 2;
</code></pre>



<a name="0x3_did_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="did_validator.md#0x3_did_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0x3_did_validator_validate"></a>

## Function `validate`

Main validation function


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="did_validator.md#0x3_did_validator_validate">validate</a>(authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="did.md#0x3_did_DID">did::DID</a>
</code></pre>
