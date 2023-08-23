
<a name="0x3_bitcoin_validator"></a>

# Module `0x3::bitcoin_validator`

This module implements bitcoin validator with the ECDSA signature over Secp256k1 crypto scheme.


-  [Struct `BitcoinValidator`](#0x3_bitcoin_validator_BitcoinValidator)
-  [Constants](#@Constants_0)
-  [Function `scheme`](#0x3_bitcoin_validator_scheme)
-  [Function `rotate_authentication_key_entry`](#0x3_bitcoin_validator_rotate_authentication_key_entry)
-  [Function `remove_authentication_key_entry`](#0x3_bitcoin_validator_remove_authentication_key_entry)
-  [Function `get_authentication_key_from_authenticator_payload`](#0x3_bitcoin_validator_get_authentication_key_from_authenticator_payload)
-  [Function `public_key_to_address`](#0x3_bitcoin_validator_public_key_to_address)
-  [Function `public_key_to_authentication_key`](#0x3_bitcoin_validator_public_key_to_authentication_key)
-  [Function `get_authentication_key_option_from_account`](#0x3_bitcoin_validator_get_authentication_key_option_from_account)
-  [Function `is_authentication_key_in_account`](#0x3_bitcoin_validator_is_authentication_key_in_account)
-  [Function `get_authentication_key_from_account`](#0x3_bitcoin_validator_get_authentication_key_from_account)
-  [Function `validate_signature`](#0x3_bitcoin_validator_validate_signature)
-  [Function `validate`](#0x3_bitcoin_validator_validate)


<pre><code><b>use</b> <a href="">0x1::debug</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="account_authentication.md#0x3_account_authentication">0x3::account_authentication</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="bitcoin_address.md#0x3_bitcoin_address">0x3::bitcoin_address</a>;
<b>use</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1">0x3::ecdsa_k1</a>;
</code></pre>



<a name="0x3_bitcoin_validator_BitcoinValidator"></a>

## Struct `BitcoinValidator`



<pre><code><b>struct</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_BitcoinValidator">BitcoinValidator</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_bitcoin_validator_BITCOIN_SCHEME"></a>

there defines scheme for each blockchain


<pre><code><b>const</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_BITCOIN_SCHEME">BITCOIN_SCHEME</a>: u64 = 2;
</code></pre>



<a name="0x3_bitcoin_validator_EInvalidPublicKeyLength"></a>

error code


<pre><code><b>const</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_EInvalidPublicKeyLength">EInvalidPublicKeyLength</a>: u64 = 0;
</code></pre>



<a name="0x3_bitcoin_validator_scheme"></a>

## Function `scheme`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_scheme">scheme</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_scheme">scheme</a>(): u64 {
    <a href="bitcoin_validator.md#0x3_bitcoin_validator_BITCOIN_SCHEME">BITCOIN_SCHEME</a>
}
</code></pre>



</details>

<a name="0x3_bitcoin_validator_rotate_authentication_key_entry"></a>

## Function `rotate_authentication_key_entry`

<code>rotate_authentication_key_entry</code> only supports rotating authentication key to a Bitcoin legacy address
becuase ecdsa k1 scheme only supports key generation of 33-bytes compressed public key at this time.


<pre><code><b>public</b> entry <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_rotate_authentication_key_entry">rotate_authentication_key_entry</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, public_key: <a href="">vector</a>&lt;u8&gt;, decimal_prefix_or_version: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_rotate_authentication_key_entry">rotate_authentication_key_entry</a>(
    ctx: &<b>mut</b> StorageContext,
    <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>,
    public_key: <a href="">vector</a>&lt;u8&gt;,
    decimal_prefix_or_version: u8,
) {
    // compare newly passed <b>public</b> key <b>with</b> Bitcoin <b>public</b> key length <b>to</b> ensure it's compatible
    <b>assert</b>!(
        <a href="_length">vector::length</a>(&public_key) == <a href="ecdsa_k1.md#0x3_ecdsa_k1_public_key_length">ecdsa_k1::public_key_length</a>()
        || <a href="_length">vector::length</a>(&public_key) == 20 // TODO support key generation of 20-bytes <b>public</b> key for Bitcoin bech32 addresses
        || <a href="_length">vector::length</a>(&public_key) == 32, // TODO support key generation of 32-bytes <b>public</b> key for Bitcoin bech32 addresses
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="bitcoin_validator.md#0x3_bitcoin_validator_EInvalidPublicKeyLength">EInvalidPublicKeyLength</a>)
    );

    // User can rotate the authentication key arbitrarily, so we do not need <b>to</b> check the new <b>public</b> key <b>with</b> the <a href="account.md#0x3_account">account</a> <b>address</b>.
    <b>let</b> authentication_key = <a href="bitcoin_validator.md#0x3_bitcoin_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key, decimal_prefix_or_version);
    <b>let</b> account_addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    <a href="bitcoin_validator.md#0x3_bitcoin_validator_rotate_authentication_key">rotate_authentication_key</a>(ctx, account_addr, authentication_key);
}
</code></pre>



</details>

<a name="0x3_bitcoin_validator_remove_authentication_key_entry"></a>

## Function `remove_authentication_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_remove_authentication_key_entry">remove_authentication_key_entry</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_remove_authentication_key_entry">remove_authentication_key_entry</a>(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="account_authentication.md#0x3_account_authentication_remove_authentication_key">account_authentication::remove_authentication_key</a>&lt;<a href="bitcoin_validator.md#0x3_bitcoin_validator_BitcoinValidator">BitcoinValidator</a>&gt;(ctx, <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>));
}
</code></pre>



</details>

<a name="0x3_bitcoin_validator_get_authentication_key_from_authenticator_payload"></a>

## Function `get_authentication_key_from_authenticator_payload`

Get the authentication key of the given authenticator from authenticator_payload.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_get_authentication_key_from_authenticator_payload">get_authentication_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;, decimal_prefix_or_version: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_get_authentication_key_from_authenticator_payload">get_authentication_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;, decimal_prefix_or_version: u8): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> public_key = <a href="ecdsa_k1.md#0x3_ecdsa_k1_get_public_key_from_authenticator_payload">ecdsa_k1::get_public_key_from_authenticator_payload</a>(authenticator_payload);
    <b>let</b> addr = <a href="bitcoin_validator.md#0x3_bitcoin_validator_public_key_to_address">public_key_to_address</a>(public_key, decimal_prefix_or_version);
    <a href="bitcoin_address.md#0x3_bitcoin_address_into_bytes">bitcoin_address::into_bytes</a>(addr)
}
</code></pre>



</details>

<a name="0x3_bitcoin_validator_public_key_to_address"></a>

## Function `public_key_to_address`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_public_key_to_address">public_key_to_address</a>(public_key: <a href="">vector</a>&lt;u8&gt;, decimal_prefix_or_version: u8): <a href="bitcoin_address.md#0x3_bitcoin_address_BTCAddress">bitcoin_address::BTCAddress</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_public_key_to_address">public_key_to_address</a>(public_key: <a href="">vector</a>&lt;u8&gt;, decimal_prefix_or_version: u8): BTCAddress {
    // Determine the <b>public</b> key length, 33-bytes for a legacy <b>address</b> and 32- and 20-bytes for a bech32 <b>address</b>.
    <b>if</b> (<a href="_length">vector::length</a>(&public_key) == <a href="ecdsa_k1.md#0x3_ecdsa_k1_public_key_length">ecdsa_k1::public_key_length</a>()) {
        <b>let</b> decimal_prefix = decimal_prefix_or_version;
        <a href="bitcoin_address.md#0x3_bitcoin_address_new_legacy">bitcoin_address::new_legacy</a>(&public_key, decimal_prefix)
    } <b>else</b> {
        <b>let</b> version = decimal_prefix_or_version;
        <a href="bitcoin_address.md#0x3_bitcoin_address_new_bech32">bitcoin_address::new_bech32</a>(&public_key, version)
    }
}
</code></pre>



</details>

<a name="0x3_bitcoin_validator_public_key_to_authentication_key"></a>

## Function `public_key_to_authentication_key`

Get the authentication key of the given public key.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key: <a href="">vector</a>&lt;u8&gt;, decimal_prefix_or_version: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key: <a href="">vector</a>&lt;u8&gt;, decimal_prefix_or_version: u8): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> addr = <a href="bitcoin_validator.md#0x3_bitcoin_validator_public_key_to_address">public_key_to_address</a>(public_key, decimal_prefix_or_version);
    <a href="bitcoin_address.md#0x3_bitcoin_address_into_bytes">bitcoin_address::into_bytes</a>(addr)
}
</code></pre>



</details>

<a name="0x3_bitcoin_validator_get_authentication_key_option_from_account"></a>

## Function `get_authentication_key_option_from_account`

Get the authentication key option of the given account.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_get_authentication_key_option_from_account">get_authentication_key_option_from_account</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_get_authentication_key_option_from_account">get_authentication_key_option_from_account</a>(ctx: &StorageContext, addr: <b>address</b>): Option&lt;<a href="">vector</a>&lt;u8&gt;&gt; {
    <a href="account_authentication.md#0x3_account_authentication_get_authentication_key">account_authentication::get_authentication_key</a>&lt;<a href="bitcoin_validator.md#0x3_bitcoin_validator_BitcoinValidator">BitcoinValidator</a>&gt;(ctx, addr)
}
</code></pre>



</details>

<a name="0x3_bitcoin_validator_is_authentication_key_in_account"></a>

## Function `is_authentication_key_in_account`

The authentication key exists in account or not.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_is_authentication_key_in_account">is_authentication_key_in_account</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_is_authentication_key_in_account">is_authentication_key_in_account</a>(ctx: &StorageContext, addr: <b>address</b>): bool {
    <a href="_is_some">option::is_some</a>(&<a href="bitcoin_validator.md#0x3_bitcoin_validator_get_authentication_key_option_from_account">get_authentication_key_option_from_account</a>(ctx, addr))
}
</code></pre>



</details>

<a name="0x3_bitcoin_validator_get_authentication_key_from_account"></a>

## Function `get_authentication_key_from_account`

Extract the authentication key of the authentication key option.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_get_authentication_key_from_account">get_authentication_key_from_account</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_get_authentication_key_from_account">get_authentication_key_from_account</a>(ctx: &StorageContext, addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt; {
    <a href="_extract">option::extract</a>(&<b>mut</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_get_authentication_key_option_from_account">get_authentication_key_option_from_account</a>(ctx, addr))
}
</code></pre>



</details>

<a name="0x3_bitcoin_validator_validate_signature"></a>

## Function `validate_signature`

Only validate the authenticator's signature.


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_validate_signature">validate_signature</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;, tx_hash: &<a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_validate_signature">validate_signature</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;, tx_hash: &<a href="">vector</a>&lt;u8&gt;) {
    <b>assert</b>!(
        <a href="ecdsa_k1.md#0x3_ecdsa_k1_verify">ecdsa_k1::verify</a>(
            &<a href="ecdsa_k1.md#0x3_ecdsa_k1_get_signature_from_authenticator_payload">ecdsa_k1::get_signature_from_authenticator_payload</a>(authenticator_payload),
            &<a href="ecdsa_k1.md#0x3_ecdsa_k1_get_public_key_from_authenticator_payload">ecdsa_k1::get_public_key_from_authenticator_payload</a>(authenticator_payload),
            tx_hash,
            <a href="ecdsa_k1.md#0x3_ecdsa_k1_sha256">ecdsa_k1::sha256</a>()
        ),
        <a href="auth_validator.md#0x3_auth_validator_error_invalid_authenticator">auth_validator::error_invalid_authenticator</a>()
    );
}
</code></pre>



</details>

<a name="0x3_bitcoin_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_validate">validate</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, authenticator_payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator_validate">validate</a>(ctx: &StorageContext, authenticator_payload: <a href="">vector</a>&lt;u8&gt;) {
    <b>let</b> tx_hash = <a href="_tx_hash">storage_context::tx_hash</a>(ctx);
    <a href="bitcoin_validator.md#0x3_bitcoin_validator_validate_signature">validate_signature</a>(&authenticator_payload, &tx_hash);

    // TODO compare the auth_key from the payload <b>with</b> the auth_key from the <a href="account.md#0x3_account">account</a>
}
</code></pre>



</details>
