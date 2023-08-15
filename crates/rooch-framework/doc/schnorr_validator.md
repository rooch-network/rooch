
<a name="0x3_schnorr_validator"></a>

# Module `0x3::schnorr_validator`

This module implements the schnorr validator scheme.


-  [Struct `SchnorrValidator`](#0x3_schnorr_validator_SchnorrValidator)
-  [Constants](#@Constants_0)
-  [Function `scheme`](#0x3_schnorr_validator_scheme)
-  [Function `rotate_authentication_key_entry`](#0x3_schnorr_validator_rotate_authentication_key_entry)
-  [Function `remove_authentication_key_entry`](#0x3_schnorr_validator_remove_authentication_key_entry)
-  [Function `get_public_key_from_authenticator_payload`](#0x3_schnorr_validator_get_public_key_from_authenticator_payload)
-  [Function `get_signature_from_authenticator_payload`](#0x3_schnorr_validator_get_signature_from_authenticator_payload)
-  [Function `get_authentication_key_from_authenticator_payload`](#0x3_schnorr_validator_get_authentication_key_from_authenticator_payload)
-  [Function `public_key_to_address`](#0x3_schnorr_validator_public_key_to_address)
-  [Function `public_key_to_authentication_key`](#0x3_schnorr_validator_public_key_to_authentication_key)
-  [Function `get_authentication_key_option_from_account`](#0x3_schnorr_validator_get_authentication_key_option_from_account)
-  [Function `is_authentication_key_in_account`](#0x3_schnorr_validator_is_authentication_key_in_account)
-  [Function `get_authentication_key_from_account`](#0x3_schnorr_validator_get_authentication_key_from_account)
-  [Function `validate_signature`](#0x3_schnorr_validator_validate_signature)
-  [Function `validate`](#0x3_schnorr_validator_validate)


<pre><code><b>use</b> <a href="">0x1::debug</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="account_authentication.md#0x3_account_authentication">0x3::account_authentication</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="hash.md#0x3_hash">0x3::hash</a>;
<b>use</b> <a href="schnorr.md#0x3_schnorr">0x3::schnorr</a>;
</code></pre>



<a name="0x3_schnorr_validator_SchnorrValidator"></a>

## Struct `SchnorrValidator`



<pre><code><b>struct</b> <a href="schnorr_validator.md#0x3_schnorr_validator_SchnorrValidator">SchnorrValidator</a> <b>has</b> drop, store
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


<a name="0x3_schnorr_validator_KECCAK256"></a>

Hash function name that are valid for verify.


<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_KECCAK256">KECCAK256</a>: u8 = 0;
</code></pre>



<a name="0x3_schnorr_validator_SHA256"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_SHA256">SHA256</a>: u8 = 1;
</code></pre>



<a name="0x3_schnorr_validator_EInvalidPublicKeyLength"></a>

error code


<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_EInvalidPublicKeyLength">EInvalidPublicKeyLength</a>: u64 = 0;
</code></pre>



<a name="0x3_schnorr_validator_SCHEME_SCHNORR"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_SCHEME_SCHNORR">SCHEME_SCHNORR</a>: u64 = 4;
</code></pre>



<a name="0x3_schnorr_validator_V_AUTHENTICATION_KEY_LENGTH"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_V_AUTHENTICATION_KEY_LENGTH">V_AUTHENTICATION_KEY_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x3_schnorr_validator_V_SCHNORR_HASH_LENGTH"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_HASH_LENGTH">V_SCHNORR_HASH_LENGTH</a>: u64 = 1;
</code></pre>



<a name="0x3_schnorr_validator_V_SCHNORR_PUBKEY_LENGTH"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_PUBKEY_LENGTH">V_SCHNORR_PUBKEY_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x3_schnorr_validator_V_SCHNORR_SCHEME_LENGTH"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SCHEME_LENGTH">V_SCHNORR_SCHEME_LENGTH</a>: u64 = 1;
</code></pre>



<a name="0x3_schnorr_validator_V_SCHNORR_SIG_LENGTH"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SIG_LENGTH">V_SCHNORR_SIG_LENGTH</a>: u64 = 64;
</code></pre>



<a name="0x3_schnorr_validator_scheme"></a>

## Function `scheme`



<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_scheme">scheme</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_scheme">scheme</a>(): u64 {
    <a href="schnorr_validator.md#0x3_schnorr_validator_SCHEME_SCHNORR">SCHEME_SCHNORR</a>
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_rotate_authentication_key_entry"></a>

## Function `rotate_authentication_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_rotate_authentication_key_entry">rotate_authentication_key_entry</a>&lt;T&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, public_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_rotate_authentication_key_entry">rotate_authentication_key_entry</a>&lt;T&gt;(
    ctx: &<b>mut</b> StorageContext,
    <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>,
    public_key: <a href="">vector</a>&lt;u8&gt;
) {
    // compare newly passed <b>public</b> key <b>with</b> <a href="schnorr.md#0x3_schnorr">schnorr</a> <b>public</b> key length <b>to</b> ensure it's compatible
    <b>assert</b>!(
        <a href="_length">vector::length</a>(&public_key) == <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_PUBKEY_LENGTH">V_SCHNORR_PUBKEY_LENGTH</a>,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="schnorr_validator.md#0x3_schnorr_validator_EInvalidPublicKeyLength">EInvalidPublicKeyLength</a>)
    );

    // User can rotate the authentication key arbitrarily, so we do not need <b>to</b> check the new <b>public</b> key <b>with</b> the <a href="account.md#0x3_account">account</a> <b>address</b>.
    <b>let</b> authentication_key = <a href="schnorr_validator.md#0x3_schnorr_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key);
    <b>let</b> account_addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    <a href="schnorr_validator.md#0x3_schnorr_validator_rotate_authentication_key">rotate_authentication_key</a>(ctx, account_addr, authentication_key);
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_remove_authentication_key_entry"></a>

## Function `remove_authentication_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_remove_authentication_key_entry">remove_authentication_key_entry</a>&lt;T&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_remove_authentication_key_entry">remove_authentication_key_entry</a>&lt;T&gt;(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="account_authentication.md#0x3_account_authentication_remove_authentication_key">account_authentication::remove_authentication_key</a>&lt;<a href="schnorr_validator.md#0x3_schnorr_validator_SchnorrValidator">SchnorrValidator</a>&gt;(ctx, <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>));
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_get_public_key_from_authenticator_payload"></a>

## Function `get_public_key_from_authenticator_payload`



<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> public_key = <a href="_empty">vector::empty</a>&lt;u8&gt;();
    <b>let</b> i = <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SCHEME_LENGTH">V_SCHNORR_SCHEME_LENGTH</a> + <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SIG_LENGTH">V_SCHNORR_SIG_LENGTH</a>;
    <b>while</b> (i &lt; <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SCHEME_LENGTH">V_SCHNORR_SCHEME_LENGTH</a> + <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SIG_LENGTH">V_SCHNORR_SIG_LENGTH</a> + <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_PUBKEY_LENGTH">V_SCHNORR_PUBKEY_LENGTH</a>) {
        <b>let</b> value = <a href="_borrow">vector::borrow</a>(authenticator_payload, i);
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> public_key, *value);
        i = i + 1;
    };

    public_key
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_get_signature_from_authenticator_payload"></a>

## Function `get_signature_from_authenticator_payload`



<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_signature_from_authenticator_payload">get_signature_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_signature_from_authenticator_payload">get_signature_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> sign = <a href="_empty">vector::empty</a>&lt;u8&gt;();
    <b>let</b> i = <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SCHEME_LENGTH">V_SCHNORR_SCHEME_LENGTH</a>;
    <b>while</b> (i &lt; <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SIG_LENGTH">V_SCHNORR_SIG_LENGTH</a> + 1) {
        <b>let</b> value = <a href="_borrow">vector::borrow</a>(authenticator_payload, i);
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> sign, *value);
        i = i + 1;
    };

    sign
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_get_authentication_key_from_authenticator_payload"></a>

## Function `get_authentication_key_from_authenticator_payload`

Get the authentication key of the given authenticator from authenticator_payload.


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_authentication_key_from_authenticator_payload">get_authentication_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_authentication_key_from_authenticator_payload">get_authentication_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> public_key = <a href="schnorr_validator.md#0x3_schnorr_validator_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload);
    <b>let</b> addr = <a href="schnorr_validator.md#0x3_schnorr_validator_public_key_to_address">public_key_to_address</a>(public_key);
    moveos_std::bcs::to_bytes(&addr)
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_public_key_to_address"></a>

## Function `public_key_to_address`

TODO: define NostrAddress or BTCLightningAddress as the return type
TODO: Schnorr public keys can be used to generate Nostr and possibly BTC Lightning addresses, and we need to determine which one to use.


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_public_key_to_address">public_key_to_address</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_public_key_to_address">public_key_to_address</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <b>address</b> {
    moveos_std::bcs::to_address(<a href="schnorr_validator.md#0x3_schnorr_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key))
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_public_key_to_authentication_key"></a>

## Function `public_key_to_authentication_key`

Get the authentication key of the given public key.


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> bytes = <a href="_singleton">vector::singleton</a>((<a href="schnorr_validator.md#0x3_schnorr_validator_SCHEME_SCHNORR">SCHEME_SCHNORR</a> <b>as</b> u8));
    <a href="_append">vector::append</a>(&<b>mut</b> bytes, public_key);
    hash::blake2b256(&bytes)
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_get_authentication_key_option_from_account"></a>

## Function `get_authentication_key_option_from_account`

Get the authentication key option of the given account.


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_authentication_key_option_from_account">get_authentication_key_option_from_account</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_authentication_key_option_from_account">get_authentication_key_option_from_account</a>(ctx: &StorageContext, addr: <b>address</b>): Option&lt;<a href="">vector</a>&lt;u8&gt;&gt; {
    <a href="account_authentication.md#0x3_account_authentication_get_authentication_key">account_authentication::get_authentication_key</a>&lt;<a href="schnorr_validator.md#0x3_schnorr_validator_SchnorrValidator">SchnorrValidator</a>&gt;(ctx, addr)
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_is_authentication_key_in_account"></a>

## Function `is_authentication_key_in_account`

The authentication key exists in account or not.


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_is_authentication_key_in_account">is_authentication_key_in_account</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_is_authentication_key_in_account">is_authentication_key_in_account</a>(ctx: &StorageContext, addr: <b>address</b>): bool {
    <a href="_is_some">option::is_some</a>(&<a href="schnorr_validator.md#0x3_schnorr_validator_get_authentication_key_option_from_account">get_authentication_key_option_from_account</a>(ctx, addr))
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_get_authentication_key_from_account"></a>

## Function `get_authentication_key_from_account`

Extract the authentication key of the authentication key option.


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_authentication_key_from_account">get_authentication_key_from_account</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_authentication_key_from_account">get_authentication_key_from_account</a>(ctx: &StorageContext, addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt; {
    <a href="_extract">option::extract</a>(&<b>mut</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_authentication_key_option_from_account">get_authentication_key_option_from_account</a>(ctx, addr))
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_validate_signature"></a>

## Function `validate_signature`

Only validate the authenticator's signature.


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_validate_signature">validate_signature</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;, tx_hash: &<a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_validate_signature">validate_signature</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;, tx_hash: &<a href="">vector</a>&lt;u8&gt;) {
    <b>assert</b>!(
        <a href="schnorr.md#0x3_schnorr_verify">schnorr::verify</a>(
            &<a href="schnorr_validator.md#0x3_schnorr_validator_get_signature_from_authenticator_payload">get_signature_from_authenticator_payload</a>(authenticator_payload),
            &<a href="schnorr_validator.md#0x3_schnorr_validator_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload),
            tx_hash,
            <a href="schnorr_validator.md#0x3_schnorr_validator_SHA256">SHA256</a>
        ),
        <a href="auth_validator.md#0x3_auth_validator_error_invalid_authenticator">auth_validator::error_invalid_authenticator</a>()
    );
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_validate">validate</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, authenticator_payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_validate">validate</a>(ctx: &StorageContext, authenticator_payload: <a href="">vector</a>&lt;u8&gt;) {
    <b>let</b> tx_hash = <a href="_tx_hash">storage_context::tx_hash</a>(ctx);
    <a href="schnorr_validator.md#0x3_schnorr_validator_validate_signature">validate_signature</a>(&authenticator_payload, &tx_hash);

    // TODO compare the auth_key from the payload <b>with</b> the auth_key from the <a href="account.md#0x3_account">account</a>
}
</code></pre>



</details>
