
<a name="0x2_multibase_key"></a>

# Module `0x2::multibase_key`

Defines the <code><a href="multibase_key.md#0x2_multibase_key">multibase_key</a></code> module, providing key-type aware encoding and decoding
operations for cryptographic keys with multicodec prefixes.


<a name="@Overview_0"></a>

### Overview


This module builds on top of <code><a href="multibase_codec.md#0x2_multibase_codec">multibase_codec</a></code> to provide specialized encoding/decoding
for cryptographic keys. It handles:

1. Key type enumeration (Ed25519, Secp256k1, Secp256r1, Rs256)
2. Multicodec prefixes for different key types
3. Key length validation
4. Encoding/decoding with type information


<a name="@Key_Types_and_Multicodec_Prefixes_1"></a>

### Key Types and Multicodec Prefixes


* Ed25519: KEY_TYPE_ED25519 = 1, multicodec prefix = 0xed01
* Secp256k1: KEY_TYPE_SECP256K1 = 2, multicodec prefix = 0xe701
* Secp256r1 (ECDSA P-256): KEY_TYPE_ECDSAR1 = 3, multicodec prefix = 0x1200

The encoding process adds the appropriate multicodec prefix to the raw key bytes
before applying base58btc encoding.


    -  [Overview](#@Overview_0)
    -  [Key Types and Multicodec Prefixes](#@Key_Types_and_Multicodec_Prefixes_1)
-  [Struct `KeyInfo`](#0x2_multibase_key_KeyInfo)
-  [Constants](#@Constants_2)
-  [Function `key_info_type`](#0x2_multibase_key_key_info_type)
-  [Function `key_info_bytes`](#0x2_multibase_key_key_info_bytes)
-  [Function `key_type_ed25519`](#0x2_multibase_key_key_type_ed25519)
-  [Function `key_type_secp256k1`](#0x2_multibase_key_key_type_secp256k1)
-  [Function `key_type_ecdsar1`](#0x2_multibase_key_key_type_ecdsar1)
-  [Function `key_type_rs256`](#0x2_multibase_key_key_type_rs256)
-  [Function `multicodec_prefix_for_type`](#0x2_multibase_key_multicodec_prefix_for_type)
-  [Function `encode_with_type`](#0x2_multibase_key_encode_with_type)
-  [Function `decode_with_type`](#0x2_multibase_key_decode_with_type)
-  [Function `encode_ed25519_key`](#0x2_multibase_key_encode_ed25519_key)
-  [Function `encode_secp256k1_key`](#0x2_multibase_key_encode_secp256k1_key)
-  [Function `encode_ecdsar1_key`](#0x2_multibase_key_encode_ecdsar1_key)
-  [Function `decode_ed25519_key`](#0x2_multibase_key_decode_ed25519_key)
-  [Function `decode_secp256k1_key`](#0x2_multibase_key_decode_secp256k1_key)
-  [Function `decode_secp256r1_key`](#0x2_multibase_key_decode_secp256r1_key)
-  [Function `decode_with_type_option`](#0x2_multibase_key_decode_with_type_option)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="multibase_codec.md#0x2_multibase_codec">0x2::multibase_codec</a>;
</code></pre>



<a name="0x2_multibase_key_KeyInfo"></a>

## Struct `KeyInfo`

A struct to hold the key type and raw key bytes
Used as a workaround for Move's lack of support for tuple types in Option


<pre><code><b>struct</b> <a href="multibase_key.md#0x2_multibase_key_KeyInfo">KeyInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_2"></a>

## Constants


<a name="0x2_multibase_key_ETestAssertionFailed"></a>



<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_ETestAssertionFailed">ETestAssertionFailed</a>: u64 = 100;
</code></pre>



<a name="0x2_multibase_key_ECDSAR1_COMPRESSED_PUBLIC_KEY_LENGTH"></a>

The length of Secp256r1 compressed public keys in bytes


<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_ECDSAR1_COMPRESSED_PUBLIC_KEY_LENGTH">ECDSAR1_COMPRESSED_PUBLIC_KEY_LENGTH</a>: u64 = 33;
</code></pre>



<a name="0x2_multibase_key_ED25519_PUBLIC_KEY_LENGTH"></a>

The length of Ed25519 public keys in bytes


<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_ED25519_PUBLIC_KEY_LENGTH">ED25519_PUBLIC_KEY_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x2_multibase_key_ErrorInvalidDidKeyIdentifier"></a>

Error when the did:key identifier is invalid


<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_ErrorInvalidDidKeyIdentifier">ErrorInvalidDidKeyIdentifier</a>: u64 = 5;
</code></pre>



<a name="0x2_multibase_key_ErrorInvalidEd25519KeyLength"></a>

Error when the Ed25519 key length is invalid


<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_ErrorInvalidEd25519KeyLength">ErrorInvalidEd25519KeyLength</a>: u64 = 1;
</code></pre>



<a name="0x2_multibase_key_ErrorInvalidPublicKeyMultibaseFormat"></a>

Error when the format of the publicKeyMultibase string is invalid or cannot be parsed


<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_ErrorInvalidPublicKeyMultibaseFormat">ErrorInvalidPublicKeyMultibaseFormat</a>: u64 = 7;
</code></pre>



<a name="0x2_multibase_key_ErrorInvalidRs256KeyLength"></a>

Error when the Rs256 key length is invalid


<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_ErrorInvalidRs256KeyLength">ErrorInvalidRs256KeyLength</a>: u64 = 4;
</code></pre>



<a name="0x2_multibase_key_ErrorInvalidSecp256k1KeyLength"></a>

Error when the Secp256k1 key length is invalid


<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_ErrorInvalidSecp256k1KeyLength">ErrorInvalidSecp256k1KeyLength</a>: u64 = 2;
</code></pre>



<a name="0x2_multibase_key_ErrorInvalidSecp256r1KeyLength"></a>

Error when the Secp256r1 key length is invalid


<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_ErrorInvalidSecp256r1KeyLength">ErrorInvalidSecp256r1KeyLength</a>: u64 = 3;
</code></pre>



<a name="0x2_multibase_key_ErrorUnsupportedKeyType"></a>

Error when an unsupported key type is used


<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_ErrorUnsupportedKeyType">ErrorUnsupportedKeyType</a>: u64 = 6;
</code></pre>



<a name="0x2_multibase_key_KEY_TYPE_ECDSAR1"></a>



<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_KEY_TYPE_ECDSAR1">KEY_TYPE_ECDSAR1</a>: u8 = 3;
</code></pre>



<a name="0x2_multibase_key_KEY_TYPE_ED25519"></a>



<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_KEY_TYPE_ED25519">KEY_TYPE_ED25519</a>: u8 = 1;
</code></pre>



<a name="0x2_multibase_key_KEY_TYPE_RS256"></a>



<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_KEY_TYPE_RS256">KEY_TYPE_RS256</a>: u8 = 4;
</code></pre>



<a name="0x2_multibase_key_KEY_TYPE_SECP256K1"></a>



<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_KEY_TYPE_SECP256K1">KEY_TYPE_SECP256K1</a>: u8 = 2;
</code></pre>



<a name="0x2_multibase_key_MULTICODEC_ECDSA_R1_PREFIX"></a>



<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_MULTICODEC_ECDSA_R1_PREFIX">MULTICODEC_ECDSA_R1_PREFIX</a>: <a href="">vector</a>&lt;u8&gt; = [18, 0];
</code></pre>



<a name="0x2_multibase_key_MULTICODEC_ED25519_PREFIX"></a>



<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_MULTICODEC_ED25519_PREFIX">MULTICODEC_ED25519_PREFIX</a>: <a href="">vector</a>&lt;u8&gt; = [237, 1];
</code></pre>



<a name="0x2_multibase_key_MULTICODEC_SECP256K1_PREFIX"></a>



<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_MULTICODEC_SECP256K1_PREFIX">MULTICODEC_SECP256K1_PREFIX</a>: <a href="">vector</a>&lt;u8&gt; = [231, 1];
</code></pre>



<a name="0x2_multibase_key_RS256_PUBLIC_KEY_MODULUS_MINIMUM_LENGTH"></a>

The minimum length of RSASSA-PKCS1-v1_5 public key modulus in bits


<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_RS256_PUBLIC_KEY_MODULUS_MINIMUM_LENGTH">RS256_PUBLIC_KEY_MODULUS_MINIMUM_LENGTH</a>: u64 = 2048;
</code></pre>



<a name="0x2_multibase_key_SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH"></a>

The length of Secp256k1 compressed public keys in bytes


<pre><code><b>const</b> <a href="multibase_key.md#0x2_multibase_key_SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH">SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH</a>: u64 = 33;
</code></pre>



<a name="0x2_multibase_key_key_info_type"></a>

## Function `key_info_type`

Get the key type from a KeyInfo struct


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_key_info_type">key_info_type</a>(key_info: &<a href="multibase_key.md#0x2_multibase_key_KeyInfo">multibase_key::KeyInfo</a>): u8
</code></pre>



<a name="0x2_multibase_key_key_info_bytes"></a>

## Function `key_info_bytes`

Get the key bytes from a KeyInfo struct


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_key_info_bytes">key_info_bytes</a>(key_info: &<a href="multibase_key.md#0x2_multibase_key_KeyInfo">multibase_key::KeyInfo</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_multibase_key_key_type_ed25519"></a>

## Function `key_type_ed25519`

Returns the key type constant for Ed25519


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_key_type_ed25519">key_type_ed25519</a>(): u8
</code></pre>



<a name="0x2_multibase_key_key_type_secp256k1"></a>

## Function `key_type_secp256k1`

Returns the key type constant for Secp256k1


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_key_type_secp256k1">key_type_secp256k1</a>(): u8
</code></pre>



<a name="0x2_multibase_key_key_type_ecdsar1"></a>

## Function `key_type_ecdsar1`

Returns the key type constant for Secp256r1 (ECDSA P-256)


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_key_type_ecdsar1">key_type_ecdsar1</a>(): u8
</code></pre>



<a name="0x2_multibase_key_key_type_rs256"></a>

## Function `key_type_rs256`

Returns the key type constant for Rs256 (RSASSA-PKCS1-v1_5)


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_key_type_rs256">key_type_rs256</a>(): u8
</code></pre>



<a name="0x2_multibase_key_multicodec_prefix_for_type"></a>

## Function `multicodec_prefix_for_type`

Get the multicodec prefix for a given key type

@param key_type - The key type (1=Ed25519, 2=Secp256k1, 3=Secp256r1, 4=Rs256)
@return - The multicodec prefix bytes


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_multicodec_prefix_for_type">multicodec_prefix_for_type</a>(key_type: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_multibase_key_encode_with_type"></a>

## Function `encode_with_type`

Encodes a public key with multicodec prefix and multibase encoding

@param pubkey - The raw public key bytes
@param key_type - The key type (1=Ed25519, 2=Secp256k1, 3=Secp256r1)
@return - A multibase encoded string with appropriate prefix


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_encode_with_type">encode_with_type</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;, key_type: u8): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_key_decode_with_type"></a>

## Function `decode_with_type`

Decodes a multibase-encoded key string with multicodec prefix

@param encoded_str - The multibase encoded key string
@return - A tuple of (key_type, raw_key_bytes), or abort if invalid


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_decode_with_type">decode_with_type</a>(encoded_str: &<a href="_String">string::String</a>): (u8, <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x2_multibase_key_encode_ed25519_key"></a>

## Function `encode_ed25519_key`

Encodes an Ed25519 public key using base58btc with multibase prefix

@param pubkey - The raw Ed25519 public key bytes
@return - A multibase encoded string with 'z' prefix


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_encode_ed25519_key">encode_ed25519_key</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_key_encode_secp256k1_key"></a>

## Function `encode_secp256k1_key`

Encodes a Secp256k1 compressed public key using base58btc with multibase prefix

@param pubkey - The raw Secp256k1 compressed public key bytes (33 bytes)
@return - A multibase encoded string with 'z' prefix


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_encode_secp256k1_key">encode_secp256k1_key</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_key_encode_ecdsar1_key"></a>

## Function `encode_ecdsar1_key`

Encodes a Secp256r1 compressed public key using base58btc with multibase prefix

@param pubkey - The raw Secp256r1 compressed public key bytes (33 bytes)
@return - A multibase encoded string with 'z' prefix


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_encode_ecdsar1_key">encode_ecdsar1_key</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_key_decode_ed25519_key"></a>

## Function `decode_ed25519_key`

Decodes a multibase-encoded Ed25519 public key

@param pk_mb_str - The multibase encoded Ed25519 public key string
@return - Option containing the decoded public key bytes, or none if decoding fails


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_decode_ed25519_key">decode_ed25519_key</a>(pk_mb_str: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_multibase_key_decode_secp256k1_key"></a>

## Function `decode_secp256k1_key`

Decodes a multibase-encoded Secp256k1 compressed public key

@param pk_mb_str - The multibase encoded Secp256k1 public key string
@return - Option containing the decoded public key bytes, or none if decoding fails


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_decode_secp256k1_key">decode_secp256k1_key</a>(pk_mb_str: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_multibase_key_decode_secp256r1_key"></a>

## Function `decode_secp256r1_key`

Decodes a Secp256r1 public key from a multibase encoded string

@param pk_mb_str - The multibase encoded string
@return - Option containing the raw Secp256r1 public key bytes, or none if decoding fails


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_decode_secp256r1_key">decode_secp256r1_key</a>(pk_mb_str: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_multibase_key_decode_with_type_option"></a>

## Function `decode_with_type_option`

Helper function to decode a multibase-encoded key string with multicodec prefix,
returning an Option instead of aborting on failure

@param encoded_str - The multibase encoded key string
@return - Option containing a KeyInfo struct with key_type and key_bytes, or none if invalid


<pre><code><b>public</b> <b>fun</b> <a href="multibase_key.md#0x2_multibase_key_decode_with_type_option">decode_with_type_option</a>(encoded_str: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="multibase_key.md#0x2_multibase_key_KeyInfo">multibase_key::KeyInfo</a>&gt;
</code></pre>
