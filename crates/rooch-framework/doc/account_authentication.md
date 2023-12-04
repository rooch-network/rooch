
<a name="0x3_account_authentication"></a>

# Module `0x3::account_authentication`

This module contains the resources and functions that are used for account authentication.
Migrated from the account module for simplyfying the account module.


-  [Resource `AuthenticationKey`](#0x3_account_authentication_AuthenticationKey)
-  [Resource `AuthenticationKeys`](#0x3_account_authentication_AuthenticationKeys)
-  [Resource `InstalledAuthValidator`](#0x3_account_authentication_InstalledAuthValidator)
-  [Constants](#@Constants_0)
-  [Function `init_authentication_keys`](#0x3_account_authentication_init_authentication_keys)
-  [Function `get_authentication_key`](#0x3_account_authentication_get_authentication_key)
-  [Function `rotate_authentication_key`](#0x3_account_authentication_rotate_authentication_key)
-  [Function `remove_authentication_key`](#0x3_account_authentication_remove_authentication_key)
-  [Function `is_auth_validator_installed`](#0x3_account_authentication_is_auth_validator_installed)
-  [Function `install_auth_validator`](#0x3_account_authentication_install_auth_validator)
-  [Function `install_auth_validator_entry`](#0x3_account_authentication_install_auth_validator_entry)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::type_table</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry">0x3::auth_validator_registry</a>;
</code></pre>



<a name="0x3_account_authentication_AuthenticationKey"></a>

## Resource `AuthenticationKey`

A resource that holds the authentication key for this account.
ValidatorType is a phantom type parameter that is used to distinguish between different auth validator types.


<pre><code><b>struct</b> <a href="account_authentication.md#0x3_account_authentication_AuthenticationKey">AuthenticationKey</a>&lt;ValidatorType&gt; <b>has</b> drop, key
</code></pre>



<a name="0x3_account_authentication_AuthenticationKeys"></a>

## Resource `AuthenticationKeys`

A resource that holds the authentication keys for this account.


<pre><code><b>struct</b> <a href="account_authentication.md#0x3_account_authentication_AuthenticationKeys">AuthenticationKeys</a> <b>has</b> key
</code></pre>



<a name="0x3_account_authentication_InstalledAuthValidator"></a>

## Resource `InstalledAuthValidator`

A resource tha holds the auth validator ids for this account has installed.


<pre><code><b>struct</b> <a href="account_authentication.md#0x3_account_authentication_InstalledAuthValidator">InstalledAuthValidator</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_account_authentication_ErrorAuthValidatorAlreadyInstalled"></a>

The authentication validator is already installed


<pre><code><b>const</b> <a href="account_authentication.md#0x3_account_authentication_ErrorAuthValidatorAlreadyInstalled">ErrorAuthValidatorAlreadyInstalled</a>: u64 = 1;
</code></pre>



<a name="0x3_account_authentication_ErrorAuthenticationKeyAlreadyExists"></a>

The authentication key already exists in the specified validator


<pre><code><b>const</b> <a href="account_authentication.md#0x3_account_authentication_ErrorAuthenticationKeyAlreadyExists">ErrorAuthenticationKeyAlreadyExists</a>: u64 = 5;
</code></pre>



<a name="0x3_account_authentication_ErrorAuthenticationKeyNotFound"></a>

The authentication key has not been found for the specified validator


<pre><code><b>const</b> <a href="account_authentication.md#0x3_account_authentication_ErrorAuthenticationKeyNotFound">ErrorAuthenticationKeyNotFound</a>: u64 = 4;
</code></pre>



<a name="0x3_account_authentication_ErrorAuthenticationKeysResourceNotFound"></a>

The authentication keys resource has not been found for the account address


<pre><code><b>const</b> <a href="account_authentication.md#0x3_account_authentication_ErrorAuthenticationKeysResourceNotFound">ErrorAuthenticationKeysResourceNotFound</a>: u64 = 3;
</code></pre>



<a name="0x3_account_authentication_ErrorMalformedAuthenticationKey"></a>

The provided authentication key has an invalid length


<pre><code><b>const</b> <a href="account_authentication.md#0x3_account_authentication_ErrorMalformedAuthenticationKey">ErrorMalformedAuthenticationKey</a>: u64 = 2;
</code></pre>



<a name="0x3_account_authentication_MAX_AUTHENTICATION_KEY_LENGTH"></a>

max authentication key length


<pre><code><b>const</b> <a href="account_authentication.md#0x3_account_authentication_MAX_AUTHENTICATION_KEY_LENGTH">MAX_AUTHENTICATION_KEY_LENGTH</a>: u64 = 256;
</code></pre>



<a name="0x3_account_authentication_init_authentication_keys"></a>

## Function `init_authentication_keys`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_init_authentication_keys">init_authentication_keys</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<a name="0x3_account_authentication_get_authentication_key"></a>

## Function `get_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_get_authentication_key">get_authentication_key</a>&lt;ValidatorType&gt;(ctx: &<a href="_Context">context::Context</a>, account_addr: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_account_authentication_rotate_authentication_key"></a>

## Function `rotate_authentication_key`

This function is used to rotate a resource account's authentication key, only the module which define the <code>ValidatorType</code> can call this function.


<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_rotate_authentication_key">rotate_authentication_key</a>&lt;ValidatorType&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, account_addr: <b>address</b>, new_auth_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_account_authentication_remove_authentication_key"></a>

## Function `remove_authentication_key`

This function is used to remove a resource account's authentication key, only the module which define the <code>ValidatorType</code> can call this function.


<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_remove_authentication_key">remove_authentication_key</a>&lt;ValidatorType&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, account_addr: <b>address</b>): <a href="account_authentication.md#0x3_account_authentication_AuthenticationKey">account_authentication::AuthenticationKey</a>&lt;ValidatorType&gt;
</code></pre>



<a name="0x3_account_authentication_is_auth_validator_installed"></a>

## Function `is_auth_validator_installed`

Return if the authentication validator is installed for the account at <code>account_addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_is_auth_validator_installed">is_auth_validator_installed</a>(ctx: &<a href="_Context">context::Context</a>, account_addr: <b>address</b>, auth_validator_id: u64): bool
</code></pre>



<a name="0x3_account_authentication_install_auth_validator"></a>

## Function `install_auth_validator`



<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_install_auth_validator">install_auth_validator</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, account_signer: &<a href="">signer</a>)
</code></pre>



<a name="0x3_account_authentication_install_auth_validator_entry"></a>

## Function `install_auth_validator_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_install_auth_validator_entry">install_auth_validator_entry</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, account_signer: &<a href="">signer</a>)
</code></pre>
