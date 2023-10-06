
<a name="0x3_account_authentication"></a>

# Module `0x3::account_authentication`

This module contains the resources and functions that are used for account authentication.
Migrate their from the account module for simplyfying the account module.


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


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
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



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>authentication_key: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_account_authentication_AuthenticationKeys"></a>

## Resource `AuthenticationKeys`

A resource that holds the authentication keys for this account.


<pre><code><b>struct</b> <a href="account_authentication.md#0x3_account_authentication_AuthenticationKeys">AuthenticationKeys</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>authentication_keys: <a href="_TypeTable">type_table::TypeTable</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_account_authentication_InstalledAuthValidator"></a>

## Resource `InstalledAuthValidator`

A resource tha holds the auth validator ids for this account has installed.


<pre><code><b>struct</b> <a href="account_authentication.md#0x3_account_authentication_InstalledAuthValidator">InstalledAuthValidator</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>validators: <a href="">vector</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

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



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_init_authentication_keys">init_authentication_keys</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_init_authentication_keys">init_authentication_keys</a>(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
   <b>let</b> authentication_keys = <a href="account_authentication.md#0x3_account_authentication_AuthenticationKeys">AuthenticationKeys</a> {
      authentication_keys: <a href="_new">type_table::new</a>(ctx),
   };
   <a href="_global_move_to">account_storage::global_move_to</a>&lt;<a href="account_authentication.md#0x3_account_authentication_AuthenticationKeys">AuthenticationKeys</a>&gt;(ctx, <a href="account.md#0x3_account">account</a>, authentication_keys);
}
</code></pre>



</details>

<a name="0x3_account_authentication_get_authentication_key"></a>

## Function `get_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_get_authentication_key">get_authentication_key</a>&lt;ValidatorType&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, account_addr: <b>address</b>): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_get_authentication_key">get_authentication_key</a>&lt;ValidatorType&gt;(ctx: &StorageContext, account_addr: <b>address</b>): Option&lt;<a href="">vector</a>&lt;u8&gt;&gt; {
   <b>if</b>(!<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account_authentication.md#0x3_account_authentication_AuthenticationKeys">AuthenticationKeys</a>&gt;(ctx, account_addr)){
      <a href="_none">option::none</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;()
   }<b>else</b>{
      <b>let</b> authentication_keys = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="account_authentication.md#0x3_account_authentication_AuthenticationKeys">AuthenticationKeys</a>&gt;(ctx, account_addr);
      <b>if</b>(<a href="_contains">type_table::contains</a>&lt;<a href="account_authentication.md#0x3_account_authentication_AuthenticationKey">AuthenticationKey</a>&lt;ValidatorType&gt;&gt;(&authentication_keys.authentication_keys)){
         <a href="_some">option::some</a>(<a href="_borrow">type_table::borrow</a>&lt;<a href="account_authentication.md#0x3_account_authentication_AuthenticationKey">AuthenticationKey</a>&lt;ValidatorType&gt;&gt;(&authentication_keys.authentication_keys).authentication_key)
      }<b>else</b>{
         <a href="_none">option::none</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;()
      }
   }
}
</code></pre>



</details>

<a name="0x3_account_authentication_rotate_authentication_key"></a>

## Function `rotate_authentication_key`

This function is used to rotate a resource account's authentication key, only the module which define the <code>ValidatorType</code> can call this function.


<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_rotate_authentication_key">rotate_authentication_key</a>&lt;ValidatorType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, account_addr: <b>address</b>, new_auth_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_rotate_authentication_key">rotate_authentication_key</a>&lt;ValidatorType&gt;(ctx: &<b>mut</b> StorageContext, account_addr: <b>address</b>, new_auth_key: <a href="">vector</a>&lt;u8&gt;) {

   <b>assert</b>!(
      <a href="_length">vector::length</a>(&new_auth_key) &lt;= <a href="account_authentication.md#0x3_account_authentication_MAX_AUTHENTICATION_KEY_LENGTH">MAX_AUTHENTICATION_KEY_LENGTH</a>,
      <a href="_invalid_argument">error::invalid_argument</a>(<a href="account_authentication.md#0x3_account_authentication_ErrorMalformedAuthenticationKey">ErrorMalformedAuthenticationKey</a>)
   );
   //We need <b>to</b> ensure the <a href="account_authentication.md#0x3_account_authentication_AuthenticationKeys">AuthenticationKeys</a> resource <b>exists</b> before we can rotate the authentication key.
   <b>let</b> authentication_keys = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="account_authentication.md#0x3_account_authentication_AuthenticationKeys">AuthenticationKeys</a>&gt;(ctx, account_addr);
   <b>if</b>(<a href="_contains">type_table::contains</a>&lt;<a href="account_authentication.md#0x3_account_authentication_AuthenticationKey">AuthenticationKey</a>&lt;ValidatorType&gt;&gt;(&authentication_keys.authentication_keys)){
      <b>let</b> authentication_key = <a href="_borrow_mut">type_table::borrow_mut</a>&lt;<a href="account_authentication.md#0x3_account_authentication_AuthenticationKey">AuthenticationKey</a>&lt;ValidatorType&gt;&gt;(&<b>mut</b> authentication_keys.authentication_keys);
      authentication_key.authentication_key = new_auth_key;
   }<b>else</b>{
      <b>let</b> authentication_key = <a href="account_authentication.md#0x3_account_authentication_AuthenticationKey">AuthenticationKey</a>&lt;ValidatorType&gt; {
         authentication_key: new_auth_key,
      };
      <a href="_add">type_table::add</a>(&<b>mut</b> authentication_keys.authentication_keys, authentication_key);
   };
}
</code></pre>



</details>

<a name="0x3_account_authentication_remove_authentication_key"></a>

## Function `remove_authentication_key`

This function is used to remove a resource account's authentication key, only the module which define the <code>ValidatorType</code> can call this function.


<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_remove_authentication_key">remove_authentication_key</a>&lt;ValidatorType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, account_addr: <b>address</b>): <a href="account_authentication.md#0x3_account_authentication_AuthenticationKey">account_authentication::AuthenticationKey</a>&lt;ValidatorType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_remove_authentication_key">remove_authentication_key</a>&lt;ValidatorType&gt;(ctx: &<b>mut</b> StorageContext, account_addr: <b>address</b>): <a href="account_authentication.md#0x3_account_authentication_AuthenticationKey">AuthenticationKey</a>&lt;ValidatorType&gt; {
   <b>assert</b>!(
      <a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account_authentication.md#0x3_account_authentication_AuthenticationKeys">AuthenticationKeys</a>&gt;(ctx, account_addr),
      <a href="_not_found">error::not_found</a>(<a href="account_authentication.md#0x3_account_authentication_ErrorAuthenticationKeysResourceNotFound">ErrorAuthenticationKeysResourceNotFound</a>)
   );
   <b>let</b> authentication_keys = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="account_authentication.md#0x3_account_authentication_AuthenticationKeys">AuthenticationKeys</a>&gt;(ctx, account_addr);
   <b>assert</b>!(
      <a href="_contains">type_table::contains</a>&lt;<a href="account_authentication.md#0x3_account_authentication_AuthenticationKey">AuthenticationKey</a>&lt;ValidatorType&gt;&gt;(&authentication_keys.authentication_keys),
      <a href="_not_found">error::not_found</a>(<a href="account_authentication.md#0x3_account_authentication_ErrorAuthenticationKeyNotFound">ErrorAuthenticationKeyNotFound</a>)
   );

   <b>let</b> removed_authentication_key = <a href="_remove">type_table::remove</a>&lt;<a href="account_authentication.md#0x3_account_authentication_AuthenticationKey">AuthenticationKey</a>&lt;ValidatorType&gt;&gt;(&<b>mut</b> authentication_keys.authentication_keys);
   removed_authentication_key
}
</code></pre>



</details>

<a name="0x3_account_authentication_is_auth_validator_installed"></a>

## Function `is_auth_validator_installed`

Return the authentication validator is installed for the account at <code>account_addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_is_auth_validator_installed">is_auth_validator_installed</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, account_addr: <b>address</b>, auth_validator_id: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_is_auth_validator_installed">is_auth_validator_installed</a>(ctx: &StorageContext, account_addr: <b>address</b>, auth_validator_id: u64): bool {
   <b>if</b>(<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account_authentication.md#0x3_account_authentication_InstalledAuthValidator">InstalledAuthValidator</a>&gt;(ctx, account_addr)){
      <b>let</b> installed_auth_validator = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="account_authentication.md#0x3_account_authentication_InstalledAuthValidator">InstalledAuthValidator</a>&gt;(ctx, account_addr);
      <a href="_contains">vector::contains</a>(&installed_auth_validator.validators, &auth_validator_id)
   }<b>else</b>{
      <b>false</b>
   }
}
</code></pre>



</details>

<a name="0x3_account_authentication_install_auth_validator"></a>

## Function `install_auth_validator`



<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_install_auth_validator">install_auth_validator</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, account_signer: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_install_auth_validator">install_auth_validator</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> StorageContext, account_signer: &<a href="">signer</a>) {
   <b>let</b> validator = <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator_by_type">auth_validator_registry::borrow_validator_by_type</a>&lt;ValidatorType&gt;(ctx);
   <b>let</b> validator_id = <a href="auth_validator.md#0x3_auth_validator_validator_id">auth_validator::validator_id</a>(validator);
   <b>let</b> account_addr = <a href="_address_of">signer::address_of</a>(account_signer);

   <b>assert</b>!(
      !<a href="account_authentication.md#0x3_account_authentication_is_auth_validator_installed">is_auth_validator_installed</a>(ctx, account_addr, validator_id),
      <a href="_already_exists">error::already_exists</a>(<a href="account_authentication.md#0x3_account_authentication_ErrorAuthValidatorAlreadyInstalled">ErrorAuthValidatorAlreadyInstalled</a>));


   <b>if</b>(!<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account_authentication.md#0x3_account_authentication_InstalledAuthValidator">InstalledAuthValidator</a>&gt;(ctx, account_addr)){
      <b>let</b> installed_auth_validator = <a href="account_authentication.md#0x3_account_authentication_InstalledAuthValidator">InstalledAuthValidator</a> {
         validators: <a href="_empty">vector::empty</a>(),
      };
      <a href="_global_move_to">account_storage::global_move_to</a>&lt;<a href="account_authentication.md#0x3_account_authentication_InstalledAuthValidator">InstalledAuthValidator</a>&gt;(ctx, account_signer, installed_auth_validator);
   };
   <b>let</b> installed_auth_validator = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="account_authentication.md#0x3_account_authentication_InstalledAuthValidator">InstalledAuthValidator</a>&gt;(ctx, account_addr);
   <a href="_push_back">vector::push_back</a>(&<b>mut</b> installed_auth_validator.validators, validator_id);
}
</code></pre>



</details>

<a name="0x3_account_authentication_install_auth_validator_entry"></a>

## Function `install_auth_validator_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_install_auth_validator_entry">install_auth_validator_entry</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, account_signer: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account_authentication.md#0x3_account_authentication_install_auth_validator_entry">install_auth_validator_entry</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> StorageContext, account_signer: &<a href="">signer</a>) {
   <a href="account_authentication.md#0x3_account_authentication_install_auth_validator">install_auth_validator</a>&lt;ValidatorType&gt;(ctx, account_signer);
}
</code></pre>



</details>
