
<a name="0x2_multibase"></a>

# Module `0x2::multibase`

Defines the <code><a href="multibase.md#0x2_multibase">multibase</a></code> module, a protocol for self-identifying base encodings
for binary data expressed in text formats. This module allows disambiguation
of the encoding (e.g., base16, base58btc, base64pad) directly from the
encoded string itself by prepending a unique prefix character.


<a name="@Overview_0"></a>

### Overview


When binary data is encoded into text, various base encodings can be used.
Multibase prepends a single character code to the base-encoded data,
indicating which encoding was used. This allows data to be self-describing
as it travels beyond its original context.

The format is: <code>&lt;base-encoding-code-point&gt;&lt;base-encoded-data&gt;</code>


<a name="@Supported_Encodings_1"></a>

### Supported Encodings


This module currently supports the following encodings, along with their
respective multibase prefixes and standard names:

*   **Base16 (Hexadecimal)**:
*   Prefix: <code>'f'</code> (ASCII: 102)
*   Name: <code>"base16"</code> (alias: <code>"<a href="hex.md#0x2_hex">hex</a>"</code>)
*   Standard: RFC4648 (lowercase output)
*   **Base58 Bitcoin (base58btc)**:
*   Prefix: <code>'z'</code> (ASCII: 122)
*   Name: <code>"base58btc"</code>
*   Standard: Used in Bitcoin, common for cryptographic keys.
*   **Base64 with Padding (base64pad)**:
*   Prefix: <code>'M'</code> (ASCII: 77)
*   Name: <code>"base64pad"</code>
*   Standard: RFC4648 with padding characters (<code>=</code>).

The module is designed to be extensible for other encodings in the future.


<a name="@Error_Handling_2"></a>

### Error Handling


Functions that can fail (e.g., <code>decode</code>, <code>encode</code> with an unsupported encoding)
return an <code>Option</code> type. Specific error codes are defined for internal assertions
and can be used in tests (e.g., <code><a href="multibase.md#0x2_multibase_ErrorInvalidMultibasePrefix">ErrorInvalidMultibasePrefix</a></code>, <code><a href="multibase.md#0x2_multibase_ErrorInvalidEd25519KeyLength">ErrorInvalidEd25519KeyLength</a></code>).
Test assertions use <code><a href="multibase.md#0x2_multibase_ETestAssertionFailed">ETestAssertionFailed</a></code> plus an offset for unique error codes.

For more details on the Multibase standard, see: [https://github.com/multiformats/multibase](https://github.com/multiformats/multibase)


    -  [Overview](#@Overview_0)
    -  [Supported Encodings](#@Supported_Encodings_1)
    -  [Error Handling](#@Error_Handling_2)
-  [Constants](#@Constants_3)
-  [Function `base58btc_name`](#0x2_multibase_base58btc_name)
-  [Function `base32_name`](#0x2_multibase_base32_name)
-  [Function `base64pad_name`](#0x2_multibase_base64pad_name)
-  [Function `base16_name`](#0x2_multibase_base16_name)
-  [Function `hex_name`](#0x2_multibase_hex_name)
-  [Function `encode_base58btc`](#0x2_multibase_encode_base58btc)
-  [Function `encode_base64pad`](#0x2_multibase_encode_base64pad)
-  [Function `encode_base16`](#0x2_multibase_encode_base16)
-  [Function `encode`](#0x2_multibase_encode)
-  [Function `encode_ed25519_key`](#0x2_multibase_encode_ed25519_key)
-  [Function `encode_secp256k1_key`](#0x2_multibase_encode_secp256k1_key)
-  [Function `decode`](#0x2_multibase_decode)
-  [Function `decode_ed25519_key`](#0x2_multibase_decode_ed25519_key)
-  [Function `decode_secp256k1_key`](#0x2_multibase_decode_secp256k1_key)
-  [Function `get_prefix_for_encoding`](#0x2_multibase_get_prefix_for_encoding)
-  [Function `get_encoding_from_prefix`](#0x2_multibase_get_encoding_from_prefix)
-  [Function `extract_prefix`](#0x2_multibase_extract_prefix)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="base58.md#0x2_base58">0x2::base58</a>;
<b>use</b> <a href="base64.md#0x2_base64">0x2::base64</a>;
<b>use</b> <a href="hex.md#0x2_hex">0x2::hex</a>;
<b>use</b> <a href="string_utils.md#0x2_string_utils">0x2::string_utils</a>;
</code></pre>



<a name="@Constants_3"></a>

## Constants


<a name="0x2_multibase_BASE16_PREFIX"></a>

The prefix for base16 (hex) encoding ('f' in ASCII)


<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_BASE16_PREFIX">BASE16_PREFIX</a>: u8 = 102;
</code></pre>



<a name="0x2_multibase_BASE32_PREFIX"></a>

The prefix for base32 encoding ('b' in ASCII)


<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_BASE32_PREFIX">BASE32_PREFIX</a>: u8 = 98;
</code></pre>



<a name="0x2_multibase_BASE58BTC_PREFIX"></a>

The prefix for Ed25519 public keys in base58btc encoding ('z' in ASCII)


<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_BASE58BTC_PREFIX">BASE58BTC_PREFIX</a>: u8 = 122;
</code></pre>



<a name="0x2_multibase_BASE64PAD_PREFIX"></a>

The prefix for base64pad encoding ('M' in ASCII)


<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_BASE64PAD_PREFIX">BASE64PAD_PREFIX</a>: u8 = 77;
</code></pre>



<a name="0x2_multibase_ED25519_PUBLIC_KEY_LENGTH"></a>

The length of Ed25519 public keys in bytes


<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ED25519_PUBLIC_KEY_LENGTH">ED25519_PUBLIC_KEY_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x2_multibase_ENCODING_BASE16"></a>



<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ENCODING_BASE16">ENCODING_BASE16</a>: <a href="">vector</a>&lt;u8&gt; = [98, 97, 115, 101, 49, 54];
</code></pre>



<a name="0x2_multibase_ENCODING_BASE32"></a>



<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ENCODING_BASE32">ENCODING_BASE32</a>: <a href="">vector</a>&lt;u8&gt; = [98, 97, 115, 101, 51, 50];
</code></pre>



<a name="0x2_multibase_ENCODING_BASE58BTC"></a>



<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ENCODING_BASE58BTC">ENCODING_BASE58BTC</a>: <a href="">vector</a>&lt;u8&gt; = [98, 97, 115, 101, 53, 56, 98, 116, 99];
</code></pre>



<a name="0x2_multibase_ENCODING_BASE64PAD"></a>



<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ENCODING_BASE64PAD">ENCODING_BASE64PAD</a>: <a href="">vector</a>&lt;u8&gt; = [98, 97, 115, 101, 54, 52, 112, 97, 100];
</code></pre>



<a name="0x2_multibase_ENCODING_HEX"></a>



<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ENCODING_HEX">ENCODING_HEX</a>: <a href="">vector</a>&lt;u8&gt; = [104, 101, 120];
</code></pre>



<a name="0x2_multibase_ETestAssertionFailed"></a>



<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ETestAssertionFailed">ETestAssertionFailed</a>: u64 = 100;
</code></pre>



<a name="0x2_multibase_ErrorBase58DecodingFailed"></a>

Error when base58 decoding fails


<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ErrorBase58DecodingFailed">ErrorBase58DecodingFailed</a>: u64 = 4;
</code></pre>



<a name="0x2_multibase_ErrorEncodingFailed"></a>

Error when the encoding process fails


<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ErrorEncodingFailed">ErrorEncodingFailed</a>: u64 = 6;
</code></pre>



<a name="0x2_multibase_ErrorInvalidBase58Char"></a>

Error when an invalid base58 character is encountered


<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ErrorInvalidBase58Char">ErrorInvalidBase58Char</a>: u64 = 3;
</code></pre>



<a name="0x2_multibase_ErrorInvalidEd25519KeyLength"></a>

Error when the Ed25519 key length is invalid


<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ErrorInvalidEd25519KeyLength">ErrorInvalidEd25519KeyLength</a>: u64 = 5;
</code></pre>



<a name="0x2_multibase_ErrorInvalidMultibasePrefix"></a>

Error when an invalid multibase prefix is provided


<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ErrorInvalidMultibasePrefix">ErrorInvalidMultibasePrefix</a>: u64 = 1;
</code></pre>



<a name="0x2_multibase_ErrorUnsupportedBase"></a>

Error when an unsupported encoding base is used


<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_ErrorUnsupportedBase">ErrorUnsupportedBase</a>: u64 = 2;
</code></pre>



<a name="0x2_multibase_SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH"></a>

The length of Secp256k1 compressed public keys in bytes


<pre><code><b>const</b> <a href="multibase.md#0x2_multibase_SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH">SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH</a>: u64 = 33;
</code></pre>



<a name="0x2_multibase_base58btc_name"></a>

## Function `base58btc_name`

Returns the name of base58btc encoding


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_base58btc_name">base58btc_name</a>(): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_base32_name"></a>

## Function `base32_name`

Returns the name of base32 encoding


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_base32_name">base32_name</a>(): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_base64pad_name"></a>

## Function `base64pad_name`

Returns the name of base64pad encoding (RFC4648 with padding)


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_base64pad_name">base64pad_name</a>(): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_base16_name"></a>

## Function `base16_name`

Returns the name of base16/hex encoding


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_base16_name">base16_name</a>(): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_hex_name"></a>

## Function `hex_name`

Returns the alternate name (hex) for base16 encoding


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_hex_name">hex_name</a>(): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_encode_base58btc"></a>

## Function `encode_base58btc`

Encodes bytes using base58btc and adds the multibase prefix 'z'

@param bytes - The raw bytes to encode
@return - A multibase encoded string with 'z' prefix


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_encode_base58btc">encode_base58btc</a>(bytes: &<a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_encode_base64pad"></a>

## Function `encode_base64pad`

Encodes bytes using base64pad (RFC4648 with padding) and adds the multibase prefix 'M'

@param bytes - The raw bytes to encode
@return - A multibase encoded string with 'M' prefix


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_encode_base64pad">encode_base64pad</a>(bytes: &<a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_encode_base16"></a>

## Function `encode_base16`

Encodes bytes using base16 (hex) and adds the multibase prefix 'f'

@param bytes - The raw bytes to encode
@return - A multibase encoded string with 'f' prefix


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_encode_base16">encode_base16</a>(bytes: &<a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_encode"></a>

## Function `encode`

Encodes bytes using a specified encoding format

@param bytes - The raw bytes to encode
@param encoding - The encoding format to use (e.g., "base58btc", "base64pad")
@return - Option containing a multibase encoded string, or none if encoding is unsupported


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_encode">encode</a>(bytes: &<a href="">vector</a>&lt;u8&gt;, encoding: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x2_multibase_encode_ed25519_key"></a>

## Function `encode_ed25519_key`

Encodes an Ed25519 public key using base58btc with multibase prefix

@param pubkey - The raw Ed25519 public key bytes
@return - A multibase encoded string with 'z' prefix


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_encode_ed25519_key">encode_ed25519_key</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_encode_secp256k1_key"></a>

## Function `encode_secp256k1_key`

Encodes a Secp256k1 compressed public key using base58btc with multibase prefix

@param pubkey - The raw Secp256k1 compressed public key bytes (33 bytes)
@return - A multibase encoded string with 'z' prefix


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_encode_secp256k1_key">encode_secp256k1_key</a>(pubkey: &<a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_multibase_decode"></a>

## Function `decode`

Decodes a multibase-encoded string to its raw bytes

@param encoded_str - The multibase encoded string
@return - Option containing the decoded bytes, or none if decoding fails


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_decode">decode</a>(encoded_str: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_multibase_decode_ed25519_key"></a>

## Function `decode_ed25519_key`

Decodes a multibase-encoded Ed25519 public key

@param pk_mb_str - The multibase encoded Ed25519 public key string
@return - Option containing the decoded public key bytes, or none if decoding fails


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_decode_ed25519_key">decode_ed25519_key</a>(pk_mb_str: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_multibase_decode_secp256k1_key"></a>

## Function `decode_secp256k1_key`

Decodes a multibase-encoded Secp256k1 compressed public key

@param pk_mb_str - The multibase encoded Secp256k1 public key string
@return - Option containing the decoded public key bytes, or none if decoding fails


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_decode_secp256k1_key">decode_secp256k1_key</a>(pk_mb_str: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_multibase_get_prefix_for_encoding"></a>

## Function `get_prefix_for_encoding`

Gets the multibase prefix character for a given encoding

@param encoding_name - The name of the encoding
@return - Option containing the prefix byte, or none if encoding is unknown


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_get_prefix_for_encoding">get_prefix_for_encoding</a>(encoding_name: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u8&gt;
</code></pre>



<a name="0x2_multibase_get_encoding_from_prefix"></a>

## Function `get_encoding_from_prefix`

Gets the encoding name from a multibase prefix character

@param prefix - The multibase prefix byte
@return - Option containing the encoding name, or none if prefix is unknown


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_get_encoding_from_prefix">get_encoding_from_prefix</a>(prefix: u8): <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x2_multibase_extract_prefix"></a>

## Function `extract_prefix`

Extracts the multibase prefix from an encoded string

@param encoded_str - The multibase encoded string
@return - Option containing the prefix byte, or none if string is empty


<pre><code><b>public</b> <b>fun</b> <a href="multibase.md#0x2_multibase_extract_prefix">extract_prefix</a>(encoded_str: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;u8&gt;
</code></pre>
