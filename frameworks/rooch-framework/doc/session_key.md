
<a name="0x3_session_key"></a>

# Module `0x3::session_key`



-  [Struct `SessionScope`](#0x3_session_key_SessionScope)
-  [Struct `SessionKey`](#0x3_session_key_SessionKey)
-  [Resource `SessionKeys`](#0x3_session_key_SessionKeys)
-  [Constants](#@Constants_0)
-  [Function `max_inactive_interval`](#0x3_session_key_max_inactive_interval)
-  [Function `signature_scheme_ed25519`](#0x3_session_key_signature_scheme_ed25519)
-  [Function `signature_scheme_secp256k1`](#0x3_session_key_signature_scheme_secp256k1)
-  [Function `signature_scheme_ecdsar1`](#0x3_session_key_signature_scheme_ecdsar1)
-  [Function `signing_envelope_raw_tx_hash`](#0x3_session_key_signing_envelope_raw_tx_hash)
-  [Function `signing_envelope_bitcoin_message_v0`](#0x3_session_key_signing_envelope_bitcoin_message_v0)
-  [Function `signing_envelope_webauthn_v0`](#0x3_session_key_signing_envelope_webauthn_v0)
-  [Function `new_session_scope`](#0x3_session_key_new_session_scope)
-  [Function `is_expired`](#0x3_session_key_is_expired)
-  [Function `is_expired_session_key`](#0x3_session_key_is_expired_session_key)
-  [Function `has_session_key`](#0x3_session_key_has_session_key)
-  [Function `exists_session_key`](#0x3_session_key_exists_session_key)
-  [Function `get_session_key`](#0x3_session_key_get_session_key)
-  [Function `create_session_key`](#0x3_session_key_create_session_key)
-  [Function `create_session_key_internal`](#0x3_session_key_create_session_key_internal)
-  [Function `create_session_key_entry`](#0x3_session_key_create_session_key_entry)
-  [Function `create_session_key_with_multi_scope_entry`](#0x3_session_key_create_session_key_with_multi_scope_entry)
-  [Function `parse_scope_string`](#0x3_session_key_parse_scope_string)
-  [Function `create_session_key_with_scope_strings_entry`](#0x3_session_key_create_session_key_with_scope_strings_entry)
-  [Function `in_session_scope`](#0x3_session_key_in_session_scope)
-  [Function `active_session_key`](#0x3_session_key_active_session_key)
-  [Function `contains_session_key`](#0x3_session_key_contains_session_key)
-  [Function `remove_session_key`](#0x3_session_key_remove_session_key)
-  [Function `remove_session_key_entry`](#0x3_session_key_remove_session_key_entry)
-  [Function `get_session_keys_handle`](#0x3_session_key_get_session_keys_handle)
-  [Function `ed25519_public_key_to_authentication_key`](#0x3_session_key_ed25519_public_key_to_authentication_key)
-  [Function `secp256k1_public_key_to_authentication_key`](#0x3_session_key_secp256k1_public_key_to_authentication_key)
-  [Function `secp256r1_public_key_to_authentication_key`](#0x3_session_key_secp256r1_public_key_to_authentication_key)
-  [Function `hex_lowercase`](#0x3_session_key_hex_lowercase)
-  [Function `bitcoin_message_digest`](#0x3_session_key_bitcoin_message_digest)
-  [Function `build_canonical_template`](#0x3_session_key_build_canonical_template)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::address</a>;
<b>use</b> <a href="">0x2::hash</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::string_utils</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::timestamp</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x2::tx_meta</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
</code></pre>



<a name="0x3_session_key_SessionScope"></a>

## Struct `SessionScope`

The session's scope


<pre><code><b>struct</b> <a href="session_key.md#0x3_session_key_SessionScope">SessionScope</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_session_key_SessionKey"></a>

## Struct `SessionKey`



<pre><code><b>struct</b> <a href="session_key.md#0x3_session_key_SessionKey">SessionKey</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_session_key_SessionKeys"></a>

## Resource `SessionKeys`



<pre><code><b>struct</b> <a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_session_key_ErrorInvalidMaxInactiveInterval"></a>

The max inactive interval is invalid


<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ErrorInvalidMaxInactiveInterval">ErrorInvalidMaxInactiveInterval</a>: u64 = 5;
</code></pre>



<a name="0x3_session_key_ErrorSessionKeyAlreadyExists"></a>

The session key already exists


<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ErrorSessionKeyAlreadyExists">ErrorSessionKeyAlreadyExists</a>: u64 = 2;
</code></pre>



<a name="0x3_session_key_ErrorSessionKeyCreatePermissionDenied"></a>

Create session key in this context is not allowed


<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ErrorSessionKeyCreatePermissionDenied">ErrorSessionKeyCreatePermissionDenied</a>: u64 = 1;
</code></pre>



<a name="0x3_session_key_ErrorSessionKeyIsInvalid"></a>

The session key is invalid


<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ErrorSessionKeyIsInvalid">ErrorSessionKeyIsInvalid</a>: u64 = 3;
</code></pre>



<a name="0x3_session_key_ErrorSessionScopePartLengthNotMatch"></a>

The lengths of the parts of the session's scope do not match.


<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ErrorSessionScopePartLengthNotMatch">ErrorSessionScopePartLengthNotMatch</a>: u64 = 4;
</code></pre>



<a name="0x3_session_key_MAX_INACTIVE_INTERVAL"></a>



<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_MAX_INACTIVE_INTERVAL">MAX_INACTIVE_INTERVAL</a>: u64 = 31536000;
</code></pre>



<a name="0x3_session_key_SIGNATURE_SCHEME_ECDSAR1"></a>



<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_SIGNATURE_SCHEME_ECDSAR1">SIGNATURE_SCHEME_ECDSAR1</a>: u8 = 2;
</code></pre>



<a name="0x3_session_key_SIGNATURE_SCHEME_ED25519"></a>



<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_SIGNATURE_SCHEME_ED25519">SIGNATURE_SCHEME_ED25519</a>: u8 = 0;
</code></pre>



<a name="0x3_session_key_SIGNATURE_SCHEME_SECP256K1"></a>



<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_SIGNATURE_SCHEME_SECP256K1">SIGNATURE_SCHEME_SECP256K1</a>: u8 = 1;
</code></pre>



<a name="0x3_session_key_SIGNING_ENVELOPE_BITCOIN_MESSAGE_V0"></a>



<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_SIGNING_ENVELOPE_BITCOIN_MESSAGE_V0">SIGNING_ENVELOPE_BITCOIN_MESSAGE_V0</a>: u8 = 1;
</code></pre>



<a name="0x3_session_key_SIGNING_ENVELOPE_RAW_TX_HASH"></a>



<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_SIGNING_ENVELOPE_RAW_TX_HASH">SIGNING_ENVELOPE_RAW_TX_HASH</a>: u8 = 0;
</code></pre>



<a name="0x3_session_key_SIGNING_ENVELOPE_WEBAUTHN_V0"></a>



<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_SIGNING_ENVELOPE_WEBAUTHN_V0">SIGNING_ENVELOPE_WEBAUTHN_V0</a>: u8 = 2;
</code></pre>



<a name="0x3_session_key_max_inactive_interval"></a>

## Function `max_inactive_interval`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_max_inactive_interval">max_inactive_interval</a>(): u64
</code></pre>



<a name="0x3_session_key_signature_scheme_ed25519"></a>

## Function `signature_scheme_ed25519`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_signature_scheme_ed25519">signature_scheme_ed25519</a>(): u8
</code></pre>



<a name="0x3_session_key_signature_scheme_secp256k1"></a>

## Function `signature_scheme_secp256k1`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_signature_scheme_secp256k1">signature_scheme_secp256k1</a>(): u8
</code></pre>



<a name="0x3_session_key_signature_scheme_ecdsar1"></a>

## Function `signature_scheme_ecdsar1`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_signature_scheme_ecdsar1">signature_scheme_ecdsar1</a>(): u8
</code></pre>



<a name="0x3_session_key_signing_envelope_raw_tx_hash"></a>

## Function `signing_envelope_raw_tx_hash`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_signing_envelope_raw_tx_hash">signing_envelope_raw_tx_hash</a>(): u8
</code></pre>



<a name="0x3_session_key_signing_envelope_bitcoin_message_v0"></a>

## Function `signing_envelope_bitcoin_message_v0`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_signing_envelope_bitcoin_message_v0">signing_envelope_bitcoin_message_v0</a>(): u8
</code></pre>



<a name="0x3_session_key_signing_envelope_webauthn_v0"></a>

## Function `signing_envelope_webauthn_v0`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_signing_envelope_webauthn_v0">signing_envelope_webauthn_v0</a>(): u8
</code></pre>



<a name="0x3_session_key_new_session_scope"></a>

## Function `new_session_scope`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_new_session_scope">new_session_scope</a>(module_address: <b>address</b>, module_name: <a href="_String">string::String</a>, function_name: <a href="_String">string::String</a>): <a href="session_key.md#0x3_session_key_SessionScope">session_key::SessionScope</a>
</code></pre>



<a name="0x3_session_key_is_expired"></a>

## Function `is_expired`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_is_expired">is_expired</a>(<a href="session_key.md#0x3_session_key">session_key</a>: &<a href="session_key.md#0x3_session_key_SessionKey">session_key::SessionKey</a>): bool
</code></pre>



<a name="0x3_session_key_is_expired_session_key"></a>

## Function `is_expired_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_is_expired_session_key">is_expired_session_key</a>(account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<a name="0x3_session_key_has_session_key"></a>

## Function `has_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_has_session_key">has_session_key</a>(account_address: <b>address</b>): bool
</code></pre>



<a name="0x3_session_key_exists_session_key"></a>

## Function `exists_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_exists_session_key">exists_session_key</a>(account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<a name="0x3_session_key_get_session_key"></a>

## Function `get_session_key`

Get the session key of the account_address by the authentication key


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_get_session_key">get_session_key</a>(account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;<a href="session_key.md#0x3_session_key_SessionKey">session_key::SessionKey</a>&gt;
</code></pre>



<a name="0x3_session_key_create_session_key"></a>

## Function `create_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key">create_session_key</a>(sender: &<a href="">signer</a>, app_name: <a href="_String">string::String</a>, app_url: <a href="_String">string::String</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scopes: <a href="">vector</a>&lt;<a href="session_key.md#0x3_session_key_SessionScope">session_key::SessionScope</a>&gt;, max_inactive_interval: u64)
</code></pre>



<a name="0x3_session_key_create_session_key_internal"></a>

## Function `create_session_key_internal`

Create session key internal, it is used to create session key for DID document
It is allowed to create session key by the other session key


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key_internal">create_session_key_internal</a>(sender: &<a href="">signer</a>, app_name: <a href="_String">string::String</a>, app_url: <a href="_String">string::String</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scopes: <a href="">vector</a>&lt;<a href="session_key.md#0x3_session_key_SessionScope">session_key::SessionScope</a>&gt;, max_inactive_interval: u64)
</code></pre>



<a name="0x3_session_key_create_session_key_entry"></a>

## Function `create_session_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key_entry">create_session_key_entry</a>(sender: &<a href="">signer</a>, app_name: <a href="_String">string::String</a>, app_url: <a href="_String">string::String</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scope_module_address: <b>address</b>, scope_module_name: <a href="_String">string::String</a>, scope_function_name: <a href="_String">string::String</a>, max_inactive_interval: u64)
</code></pre>



<a name="0x3_session_key_create_session_key_with_multi_scope_entry"></a>

## Function `create_session_key_with_multi_scope_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key_with_multi_scope_entry">create_session_key_with_multi_scope_entry</a>(sender: &<a href="">signer</a>, app_name: <a href="_String">string::String</a>, app_url: <a href="_String">string::String</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scope_module_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;, scope_module_names: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, scope_function_names: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, max_inactive_interval: u64)
</code></pre>



<a name="0x3_session_key_parse_scope_string"></a>

## Function `parse_scope_string`

Parse a scope string in the format "address::module::function"
Example: "0x1::counter::increment" or "0x2::*::*"


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_parse_scope_string">parse_scope_string</a>(scope_str: <a href="_String">string::String</a>): <a href="session_key.md#0x3_session_key_SessionScope">session_key::SessionScope</a>
</code></pre>



<a name="0x3_session_key_create_session_key_with_scope_strings_entry"></a>

## Function `create_session_key_with_scope_strings_entry`

Create session key with scope strings entry function
This is a more convenient version that allows passing scope strings directly
Format: "address::module::function", e.g., "0x1::counter::increment" or "0x2::*::*"


<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key_with_scope_strings_entry">create_session_key_with_scope_strings_entry</a>(sender: &<a href="">signer</a>, app_name: <a href="_String">string::String</a>, app_url: <a href="_String">string::String</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scope_strings: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, max_inactive_interval: u64)
</code></pre>



<a name="0x3_session_key_in_session_scope"></a>

## Function `in_session_scope`

Check the current tx is in the session scope or not


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_in_session_scope">in_session_scope</a>(<a href="session_key.md#0x3_session_key">session_key</a>: &<a href="session_key.md#0x3_session_key_SessionKey">session_key::SessionKey</a>): bool
</code></pre>



<a name="0x3_session_key_active_session_key"></a>

## Function `active_session_key`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_active_session_key">active_session_key</a>(authentication_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_session_key_contains_session_key"></a>

## Function `contains_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_contains_session_key">contains_session_key</a>(sender_addr: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<a name="0x3_session_key_remove_session_key"></a>

## Function `remove_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_remove_session_key">remove_session_key</a>(sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_session_key_remove_session_key_entry"></a>

## Function `remove_session_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_remove_session_key_entry">remove_session_key_entry</a>(sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_session_key_get_session_keys_handle"></a>

## Function `get_session_keys_handle`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_get_session_keys_handle">get_session_keys_handle</a>(account_address: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<a name="0x3_session_key_ed25519_public_key_to_authentication_key"></a>

## Function `ed25519_public_key_to_authentication_key`

Derives the authentication key for an Ed25519 public key.
This is consistent with how session_validator derives it.


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_ed25519_public_key_to_authentication_key">ed25519_public_key_to_authentication_key</a>(public_key: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_session_key_secp256k1_public_key_to_authentication_key"></a>

## Function `secp256k1_public_key_to_authentication_key`

Derives the authentication key for a Secp256k1 public key.
This follows the same pattern as Ed25519 but with a different scheme identifier.


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_secp256k1_public_key_to_authentication_key">secp256k1_public_key_to_authentication_key</a>(public_key: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_session_key_secp256r1_public_key_to_authentication_key"></a>

## Function `secp256r1_public_key_to_authentication_key`

Derives the authentication key for a Secp256r1 public key.
This follows the same pattern as Ed25519 but with a different scheme identifier.


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_secp256r1_public_key_to_authentication_key">secp256r1_public_key_to_authentication_key</a>(public_key: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_session_key_hex_lowercase"></a>

## Function `hex_lowercase`

Convert a 32-byte hash to lowercase hex string (64 ASCII characters)


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_hex_lowercase">hex_lowercase</a>(<a href="">hash</a>: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_session_key_bitcoin_message_digest"></a>

## Function `bitcoin_message_digest`

Compute Bitcoin message digest: SHA256(SHA256("Bitcoin Signed Message:\n" + VarInt(len) + message))


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_bitcoin_message_digest">bitcoin_message_digest</a>(message: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x3_session_key_build_canonical_template"></a>

## Function `build_canonical_template`

Build canonical template for BitcoinMessageV0 envelope
Uses the same format as auth_payload.move: "Rooch Transaction:\n" + hex(tx_hash)


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_build_canonical_template">build_canonical_template</a>(tx_hash: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>
