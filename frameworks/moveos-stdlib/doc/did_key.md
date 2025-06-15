
<a name="0x2_did_key"></a>

# Module `0x2::did_key`

Defines the <code><a href="did_key.md#0x2_did_key">did_key</a></code> module, providing operations specific to did:key identifiers.


<a name="@Overview_0"></a>

### Overview


The did:key method is a simple and convenient way to express public keys as DIDs.
This module builds on top of <code><a href="multibase_key.md#0x2_multibase_key">multibase_key</a></code> to provide specialized functions for
generating and parsing did:key identifiers.


<a name="@Format_1"></a>

### Format


A did:key identifier has the format:
<code>did:key:&lt;multibase-encoded-<b>public</b>-key-<b>with</b>-multicodec-prefix&gt;</code>

For example:
- Ed25519: <code>did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK</code>
- Secp256k1: <code>did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme</code>

The multibase-encoded part is created by:
1. Adding the appropriate multicodec prefix to the raw public key bytes
2. Encoding the result with base58btc
3. Adding the 'z' prefix (for base58btc)


    -  [Overview](#@Overview_0)
    -  [Format](#@Format_1)
-  [Constants](#@Constants_2)
-  [Function `generate`](#0x2_did_key_generate)
-  [Function `generate_ed25519`](#0x2_did_key_generate_ed25519)
-  [Function `generate_secp256k1`](#0x2_did_key_generate_secp256k1)
-  [Function `generate_secp256r1`](#0x2_did_key_generate_secp256r1)
-  [Function `parse`](#0x2_did_key_parse)
-  [Function `parse_ed25519`](#0x2_did_key_parse_ed25519)
-  [Function `parse_secp256k1`](#0x2_did_key_parse_secp256k1)
-  [Function `parse_secp256r1`](#0x2_did_key_parse_secp256r1)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="multibase_key.md#0x2_multibase_key">0x2::multibase_key</a>;
</code></pre>



<a name="@Constants_2"></a>

## Constants


<a name="0x2_did_key_ETestAssertionFailed"></a>



<pre><code><b>const</b> <a href="did_key.md#0x2_did_key_ETestAssertionFailed">ETestAssertionFailed</a>: u64 = 100;
</code></pre>



<a name="0x2_did_key_DID_KEY_METHOD"></a>

The did:key method string


<pre><code><b>const</b> <a href="did_key.md#0x2_did_key_DID_KEY_METHOD">DID_KEY_METHOD</a>: <a href="">vector</a>&lt;u8&gt; = [100, 105, 100, 58, 107, 101, 121, 58];
</code></pre>



<a name="0x2_did_key_ErrorInvalidDidKeyFormat"></a>

Error when the did:key string format is invalid


<pre><code><b>const</b> <a href="did_key.md#0x2_did_key_ErrorInvalidDidKeyFormat">ErrorInvalidDidKeyFormat</a>: u64 = 1;
</code></pre>



<a name="0x2_did_key_generate"></a>

## Function `generate`

Generate a did:key identifier from a public key and key type

@param pubkey - The raw public key bytes
@param key_type - The key type (1=Ed25519, 2=Secp256k1, 3=Secp256r1)
@return - A did:key identifier string


<pre><code><b>public</b> <b>fun</b> <a href="did_key.md#0x2_did_key_generate">generate</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;, key_type: u8): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_did_key_generate_ed25519"></a>

## Function `generate_ed25519`

Generate a did:key identifier from an Ed25519 public key

@param pubkey - The raw Ed25519 public key bytes (32 bytes)
@return - A did:key identifier string


<pre><code><b>public</b> <b>fun</b> <a href="did_key.md#0x2_did_key_generate_ed25519">generate_ed25519</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_did_key_generate_secp256k1"></a>

## Function `generate_secp256k1`

Generate a did:key identifier from a Secp256k1 public key

@param pubkey - The raw Secp256k1 compressed public key bytes (33 bytes)
@return - A did:key identifier string


<pre><code><b>public</b> <b>fun</b> <a href="did_key.md#0x2_did_key_generate_secp256k1">generate_secp256k1</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_did_key_generate_secp256r1"></a>

## Function `generate_secp256r1`

Generate a did:key identifier from a Secp256r1 public key

@param pubkey - The raw Secp256r1 compressed public key bytes (33 bytes)
@return - A did:key identifier string


<pre><code><b>public</b> <b>fun</b> <a href="did_key.md#0x2_did_key_generate_secp256r1">generate_secp256r1</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_did_key_parse"></a>

## Function `parse`

Parse a did:key identifier to extract the key type and raw public key bytes

@param did_key_str - The did:key identifier string
@return - A tuple of (key_type, raw_key_bytes), or abort if invalid


<pre><code><b>public</b> <b>fun</b> <a href="did_key.md#0x2_did_key_parse">parse</a>(did_key_str: &<a href="_String">string::String</a>): (u8, <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x2_did_key_parse_ed25519"></a>

## Function `parse_ed25519`

Parse a did:key identifier and check if it's an Ed25519 key

@param did_key_str - The did:key identifier string
@return - Option containing the raw Ed25519 public key bytes, or none if not an Ed25519 key or invalid


<pre><code><b>public</b> <b>fun</b> <a href="did_key.md#0x2_did_key_parse_ed25519">parse_ed25519</a>(did_key_str: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_did_key_parse_secp256k1"></a>

## Function `parse_secp256k1`

Parse a did:key identifier and check if it's a Secp256k1 key

@param did_key_str - The did:key identifier string
@return - Option containing the raw Secp256k1 public key bytes, or none if not a Secp256k1 key or invalid


<pre><code><b>public</b> <b>fun</b> <a href="did_key.md#0x2_did_key_parse_secp256k1">parse_secp256k1</a>(did_key_str: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_did_key_parse_secp256r1"></a>

## Function `parse_secp256r1`

Parse a did:key identifier and check if it's a Secp256r1 key

@param did_key_str - The did:key identifier string
@return - Option containing the raw Secp256r1 public key bytes, or none if not a Secp256r1 key or invalid


<pre><code><b>public</b> <b>fun</b> <a href="did_key.md#0x2_did_key_parse_secp256r1">parse_secp256r1</a>(did_key_str: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>
