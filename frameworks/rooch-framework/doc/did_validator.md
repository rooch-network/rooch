
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
-  [Function `build_rooch_transaction_message`](#0x3_did_validator_build_rooch_transaction_message)
-  [Function `encode_bitcoin_message`](#0x3_did_validator_encode_bitcoin_message)
-  [Function `validate`](#0x3_did_validator_validate)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::base64</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::consensus_codec</a>;
<b>use</b> <a href="">0x2::hash</a>;
<b>use</b> <a href="">0x2::hex</a>;
<b>use</b> <a href="">0x2::json</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="did.md#0x3_did">0x3::did</a>;
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


<a name="0x3_did_validator_ErrorVerificationMethodNotFound"></a>

Verification method not found in DID document


<pre><code><b>const</b> <a href="did_validator.md#0x3_did_validator_ErrorVerificationMethodNotFound">ErrorVerificationMethodNotFound</a>: u64 = 101005;
</code></pre>



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



<a name="0x3_did_validator_ErrorDIDDocumentNotFound"></a>

DID document not found for sender address


<pre><code><b>const</b> <a href="did_validator.md#0x3_did_validator_ErrorDIDDocumentNotFound">ErrorDIDDocumentNotFound</a>: u64 = 101003;
</code></pre>



<a name="0x3_did_validator_ErrorInvalidDIDAuthPayload"></a>

Error codes for DID validator (using 101xxx range to avoid conflicts)
DID validator specific errors: 101001-101999
Invalid BCS deserialization of DID auth payload


<pre><code><b>const</b> <a href="did_validator.md#0x3_did_validator_ErrorInvalidDIDAuthPayload">ErrorInvalidDIDAuthPayload</a>: u64 = 101001;
</code></pre>



<a name="0x3_did_validator_ErrorInvalidEnvelopeMessage"></a>

Invalid message for envelope type


<pre><code><b>const</b> <a href="did_validator.md#0x3_did_validator_ErrorInvalidEnvelopeMessage">ErrorInvalidEnvelopeMessage</a>: u64 = 101006;
</code></pre>



<a name="0x3_did_validator_ErrorInvalidEnvelopeType"></a>

Invalid envelope type in DID auth payload


<pre><code><b>const</b> <a href="did_validator.md#0x3_did_validator_ErrorInvalidEnvelopeType">ErrorInvalidEnvelopeType</a>: u64 = 101002;
</code></pre>



<a name="0x3_did_validator_ErrorSignatureVerificationFailed"></a>

Signature verification failed


<pre><code><b>const</b> <a href="did_validator.md#0x3_did_validator_ErrorSignatureVerificationFailed">ErrorSignatureVerificationFailed</a>: u64 = 101007;
</code></pre>



<a name="0x3_did_validator_ErrorVerificationMethodNotAuthorized"></a>

Verification method not authorized for authentication


<pre><code><b>const</b> <a href="did_validator.md#0x3_did_validator_ErrorVerificationMethodNotAuthorized">ErrorVerificationMethodNotAuthorized</a>: u64 = 101004;
</code></pre>



<a name="0x3_did_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="did_validator.md#0x3_did_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0x3_did_validator_build_rooch_transaction_message"></a>

## Function `build_rooch_transaction_message`

Build Rooch transaction message for Bitcoin signature verification
Uses the same format as auth_payload.move: "Rooch Transaction:\n" + hex(tx_hash)
This message format is used in BitcoinMessageV0 envelope for DID authentication


<pre><code><b>public</b> <b>fun</b> <a href="did_validator.md#0x3_did_validator_build_rooch_transaction_message">build_rooch_transaction_message</a>(tx_hash: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_did_validator_encode_bitcoin_message"></a>

## Function `encode_bitcoin_message`

Encode Bitcoin message using the same format as TypeScript BitcoinSignMessage
Format: \u0018 + "Bitcoin Signed Message:\n" + varint(message_len) + message


<pre><code><b>public</b> <b>fun</b> <a href="did_validator.md#0x3_did_validator_encode_bitcoin_message">encode_bitcoin_message</a>(message: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_did_validator_validate"></a>

## Function `validate`

Main validation function


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="did_validator.md#0x3_did_validator_validate">validate</a>(authenticator_payload: <a href="">vector</a>&lt;u8&gt;): (<a href="did.md#0x3_did_DID">did::DID</a>, <a href="_String">string::String</a>)
</code></pre>
