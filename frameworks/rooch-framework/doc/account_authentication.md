
<a name="0x3_account_authentication"></a>

# Module `0x3::account_authentication`

This module contains the resources and functions that are used for account authentication.
Migrated from the account module for simplyfying the account module.


-  [Resource `InstalledAuthValidator`](#0x3_account_authentication_InstalledAuthValidator)
-  [Constants](#@Constants_0)
-  [Function `is_auth_validator_installed`](#0x3_account_authentication_is_auth_validator_installed)
-  [Function `install_auth_validator`](#0x3_account_authentication_install_auth_validator)
-  [Function `install_auth_validator_entry`](#0x3_account_authentication_install_auth_validator_entry)


<pre><code><b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::features</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry">0x3::auth_validator_registry</a>;
</code></pre>



<a name="0x3_account_authentication_InstalledAuthValidator"></a>

## Resource `InstalledAuthValidator`

A resource that holds the auth validator ids for this account has installed.


<pre><code><b>struct</b> <a href="account_authentication.md#0x3_account_authentication_InstalledAuthValidator">InstalledAuthValidator</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_account_authentication_ErrorAuthValidatorAlreadyInstalled"></a>

The authentication validator is already installed


<pre><code><b>const</b> <a href="account_authentication.md#0x3_account_authentication_ErrorAuthValidatorAlreadyInstalled">ErrorAuthValidatorAlreadyInstalled</a>: u64 = 1;
</code></pre>



<a name="0x3_account_authentication_is_auth_validator_installed"></a>

## Function `is_auth_validator_installed`

Return if the authentication validator is installed for the account at <code>account_addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_is_auth_validator_installed">is_auth_validator_installed</a>(account_addr: <b>address</b>, auth_validator_id: u64): bool
</code></pre>



<a name="0x3_account_authentication_install_auth_validator"></a>

## Function `install_auth_validator`



<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_install_auth_validator">install_auth_validator</a>&lt;ValidatorType: store&gt;(account_signer: &<a href="">signer</a>)
</code></pre>



<a name="0x3_account_authentication_install_auth_validator_entry"></a>

## Function `install_auth_validator_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_install_auth_validator_entry">install_auth_validator_entry</a>&lt;ValidatorType: store&gt;(account_signer: &<a href="">signer</a>)
</code></pre>
