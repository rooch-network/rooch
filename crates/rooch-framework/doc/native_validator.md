
<a name="0x3_native_validator"></a>

# Module `0x3::native_validator`

This module implements the native validator.


-  [Struct `NativeValidator`](#0x3_native_validator_NativeValidator)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0x3_native_validator_auth_validator_id)
-  [Function `rotate_authentication_key_entry`](#0x3_native_validator_rotate_authentication_key_entry)
-  [Function `remove_authentication_key_entry`](#0x3_native_validator_remove_authentication_key_entry)
-  [Function `get_public_key_from_authenticator_payload`](#0x3_native_validator_get_public_key_from_authenticator_payload)
-  [Function `get_signature_from_authenticator_payload`](#0x3_native_validator_get_signature_from_authenticator_payload)
-  [Function `get_authentication_key_from_authenticator_payload`](#0x3_native_validator_get_authentication_key_from_authenticator_payload)
-  [Function `public_key_to_address`](#0x3_native_validator_public_key_to_address)
-  [Function `public_key_to_authentication_key`](#0x3_native_validator_public_key_to_authentication_key)
-  [Function `get_authentication_key_with_default`](#0x3_native_validator_get_authentication_key_with_default)
-  [Function `default_authentication_key`](#0x3_native_validator_default_authentication_key)
-  [Function `validate_signature`](#0x3_native_validator_validate_signature)
-  [Function `validate`](#0x3_native_validator_validate)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="account_authentication.md#0x3_account_authentication">0x3::account_authentication</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="ed25519.md#0x3_ed25519">0x3::ed25519</a>;
<b>use</b> <a href="hash.md#0x3_hash">0x3::hash</a>;
</code></pre>



<a name="0x3_native_validator_NativeValidator"></a>

## Struct `NativeValidator`



<pre><code><b>struct</b> <a href="native_validator.md#0x3_native_validator_NativeValidator">NativeValidator</a> <b>has</b> drop, store
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


<a name="0x3_native_validator_ErrorInvalidPublicKeyLength"></a>

error code


<pre><code><b>const</b> <a href="native_validator.md#0x3_native_validator_ErrorInvalidPublicKeyLength">ErrorInvalidPublicKeyLength</a>: u64 = 0;
</code></pre>



<a name="0x3_native_validator_NATIVE_VALIDATOR_ID"></a>

there defines auth validator id for each blockchain


<pre><code><b>const</b> <a href="native_validator.md#0x3_native_validator_NATIVE_VALIDATOR_ID">NATIVE_VALIDATOR_ID</a>: u64 = 0;
</code></pre>



<a name="0x3_native_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_auth_validator_id">auth_validator_id</a>(): u64 {
    <a href="native_validator.md#0x3_native_validator_NATIVE_VALIDATOR_ID">NATIVE_VALIDATOR_ID</a>
}
</code></pre>



</details>

<a name="0x3_native_validator_rotate_authentication_key_entry"></a>

## Function `rotate_authentication_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="native_validator.md#0x3_native_validator_rotate_authentication_key_entry">rotate_authentication_key_entry</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, public_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="native_validator.md#0x3_native_validator_rotate_authentication_key_entry">rotate_authentication_key_entry</a>(
    ctx: &<b>mut</b> StorageContext,
    <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>,
    public_key: <a href="">vector</a>&lt;u8&gt;
) {
    // compare newly passed <b>public</b> key <b>with</b> Rooch <b>public</b> key length <b>to</b> ensure it's compatible
    <b>assert</b>!(
        <a href="_length">vector::length</a>(&public_key) == <a href="ed25519.md#0x3_ed25519_public_key_length">ed25519::public_key_length</a>(),
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="native_validator.md#0x3_native_validator_ErrorInvalidPublicKeyLength">ErrorInvalidPublicKeyLength</a>)
    );

    // User can rotate the authentication key arbitrarily, so we do not need <b>to</b> check the new <b>public</b> key <b>with</b> the <a href="account.md#0x3_account">account</a> <b>address</b>.
    <b>let</b> authentication_key = <a href="native_validator.md#0x3_native_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key);
    <b>let</b> account_addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
    <a href="native_validator.md#0x3_native_validator_rotate_authentication_key">rotate_authentication_key</a>(ctx, account_addr, authentication_key);
}
</code></pre>



</details>

<a name="0x3_native_validator_remove_authentication_key_entry"></a>

## Function `remove_authentication_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="native_validator.md#0x3_native_validator_remove_authentication_key_entry">remove_authentication_key_entry</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="native_validator.md#0x3_native_validator_remove_authentication_key_entry">remove_authentication_key_entry</a>(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
    <a href="account_authentication.md#0x3_account_authentication_remove_authentication_key">account_authentication::remove_authentication_key</a>&lt;<a href="native_validator.md#0x3_native_validator_NativeValidator">NativeValidator</a>&gt;(ctx, <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>));
}
</code></pre>



</details>

<a name="0x3_native_validator_get_public_key_from_authenticator_payload"></a>

## Function `get_public_key_from_authenticator_payload`



<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> public_key = <a href="_empty">vector::empty</a>&lt;u8&gt;();
    // TODO remove auth validator id from payload here <b>to</b> 0
    <b>let</b> i = 1 + <a href="ed25519.md#0x3_ed25519_signature_length">ed25519::signature_length</a>();
    <b>let</b> public_key_position = 1 + <a href="ed25519.md#0x3_ed25519_signature_length">ed25519::signature_length</a>() + <a href="ed25519.md#0x3_ed25519_public_key_length">ed25519::public_key_length</a>();
    <b>while</b> (i &lt; public_key_position) {
        <b>let</b> value = <a href="_borrow">vector::borrow</a>(authenticator_payload, i);
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> public_key, *value);
        i = i + 1;
    };
    public_key
}
</code></pre>



</details>

<a name="0x3_native_validator_get_signature_from_authenticator_payload"></a>

## Function `get_signature_from_authenticator_payload`



<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_get_signature_from_authenticator_payload">get_signature_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_get_signature_from_authenticator_payload">get_signature_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> sign = <a href="_empty">vector::empty</a>&lt;u8&gt;();
    // TODO remove auth validator id from payload here <b>to</b> 0
    <b>let</b> i = 1;
    <b>let</b> signature_position = <a href="ed25519.md#0x3_ed25519_signature_length">ed25519::signature_length</a>() + 1;
    <b>while</b> (i &lt; signature_position) {
        <b>let</b> value = <a href="_borrow">vector::borrow</a>(authenticator_payload, i);
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> sign, *value);
        i = i + 1;
    };
    sign
}
</code></pre>



</details>

<a name="0x3_native_validator_get_authentication_key_from_authenticator_payload"></a>

## Function `get_authentication_key_from_authenticator_payload`

Get the authentication key of the given authenticator from authenticator_payload.


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_get_authentication_key_from_authenticator_payload">get_authentication_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_get_authentication_key_from_authenticator_payload">get_authentication_key_from_authenticator_payload</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> public_key = <a href="native_validator.md#0x3_native_validator_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload);
    <b>let</b> addr = <a href="native_validator.md#0x3_native_validator_public_key_to_address">public_key_to_address</a>(public_key);
    moveos_std::bcs::to_bytes(&addr)
}
</code></pre>



</details>

<a name="0x3_native_validator_public_key_to_address"></a>

## Function `public_key_to_address`



<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_public_key_to_address">public_key_to_address</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_public_key_to_address">public_key_to_address</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <b>address</b> {
    moveos_std::bcs::to_address(<a href="native_validator.md#0x3_native_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key))
}
</code></pre>



</details>

<a name="0x3_native_validator_public_key_to_authentication_key"></a>

## Function `public_key_to_authentication_key`

Get the authentication key of the given public key.


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_public_key_to_authentication_key">public_key_to_authentication_key</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> bytes = <a href="_singleton">vector::singleton</a>((<a href="native_validator.md#0x3_native_validator_auth_validator_id">auth_validator_id</a>() <b>as</b> u8));
    <a href="_append">vector::append</a>(&<b>mut</b> bytes, public_key);
    hash::blake2b256(&bytes)
}
</code></pre>



</details>

<a name="0x3_native_validator_get_authentication_key_with_default"></a>

## Function `get_authentication_key_with_default`

Get the authentication key of the given account, if it not exist, return the account address as authentication key.


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_get_authentication_key_with_default">get_authentication_key_with_default</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_get_authentication_key_with_default">get_authentication_key_with_default</a>(ctx: &StorageContext, addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt; {
    <b>let</b> auth_key_option = <a href="account_authentication.md#0x3_account_authentication_get_authentication_key">account_authentication::get_authentication_key</a>&lt;<a href="native_validator.md#0x3_native_validator_NativeValidator">NativeValidator</a>&gt;(ctx, addr);
    <b>if</b> (<a href="_is_some">option::is_some</a>(&auth_key_option)) {
        <a href="_extract">option::extract</a>(&<b>mut</b> auth_key_option)
    }<b>else</b> {
        <a href="native_validator.md#0x3_native_validator_default_authentication_key">default_authentication_key</a>(addr)
    }
}
</code></pre>



</details>

<a name="0x3_native_validator_default_authentication_key"></a>

## Function `default_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_default_authentication_key">default_authentication_key</a>(addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_default_authentication_key">default_authentication_key</a>(addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt; {
    moveos_std::bcs::to_bytes(&addr)
}
</code></pre>



</details>

<a name="0x3_native_validator_validate_signature"></a>

## Function `validate_signature`

Only validate the authenticator's signature.


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_validate_signature">validate_signature</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;, tx_hash: &<a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_validate_signature">validate_signature</a>(authenticator_payload: &<a href="">vector</a>&lt;u8&gt;, tx_hash: &<a href="">vector</a>&lt;u8&gt;) {
    <b>assert</b>!(
        <a href="ed25519.md#0x3_ed25519_verify">ed25519::verify</a>(
            &<a href="native_validator.md#0x3_native_validator_get_signature_from_authenticator_payload">get_signature_from_authenticator_payload</a>(authenticator_payload),
            &<a href="native_validator.md#0x3_native_validator_get_public_key_from_authenticator_payload">get_public_key_from_authenticator_payload</a>(authenticator_payload),
            tx_hash
        ),
        <a href="auth_validator.md#0x3_auth_validator_error_invalid_authenticator">auth_validator::error_invalid_authenticator</a>()
    );
}
</code></pre>



</details>

<a name="0x3_native_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_validate">validate</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, authenticator_payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="native_validator.md#0x3_native_validator_validate">validate</a>(ctx: &StorageContext, authenticator_payload: <a href="">vector</a>&lt;u8&gt;) {
    <b>let</b> tx_hash = <a href="_tx_hash">storage_context::tx_hash</a>(ctx);
    <a href="native_validator.md#0x3_native_validator_validate_signature">validate_signature</a>(&authenticator_payload, &tx_hash);

    <b>let</b> auth_key_from_authenticator_payload = <a href="native_validator.md#0x3_native_validator_get_authentication_key_from_authenticator_payload">get_authentication_key_from_authenticator_payload</a>(&authenticator_payload);
    <b>let</b> auth_key_in_account = <a href="native_validator.md#0x3_native_validator_get_authentication_key_with_default">get_authentication_key_with_default</a>(ctx, <a href="_sender">storage_context::sender</a>(ctx));
    <b>assert</b>!(
        auth_key_in_account == auth_key_from_authenticator_payload,
        <a href="auth_validator.md#0x3_auth_validator_error_invalid_account_auth_key">auth_validator::error_invalid_account_auth_key</a>()
    );
}
</code></pre>



</details>
