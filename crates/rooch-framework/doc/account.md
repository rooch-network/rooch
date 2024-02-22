
<a name="0x3_account"></a>

# Module `0x3::account`



-  [Struct `AccountPlaceholder`](#0x3_account_AccountPlaceholder)
-  [Function `create_account_entry`](#0x3_account_create_account_entry)
-  [Function `create_account`](#0x3_account_create_account)
-  [Function `create_resource_account`](#0x3_account_create_resource_account)


<pre><code><b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="account_authentication.md#0x3_account_authentication">0x3::account_authentication</a>;
<b>use</b> <a href="account_coin_store.md#0x3_account_coin_store">0x3::account_coin_store</a>;
</code></pre>



<a name="0x3_account_AccountPlaceholder"></a>

## Struct `AccountPlaceholder`

Just using to get Account module signer


<pre><code><b>struct</b> <a href="account.md#0x3_account_AccountPlaceholder">AccountPlaceholder</a>
</code></pre>



<a name="0x3_account_create_account_entry"></a>

## Function `create_account_entry`

A entry function to create an account under <code>new_address</code>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x3_account_create_account_entry">create_account_entry</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, new_address: <b>address</b>)
</code></pre>



<a name="0x3_account_create_account"></a>

## Function `create_account`

Publishes a new <code>Account</code> resource under <code>new_address</code>. A signer representing <code>new_address</code>
is returned. This way, the caller of this function can publish additional resources under
<code>new_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_account">create_account</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, new_address: <b>address</b>): <a href="">signer</a>
</code></pre>



<a name="0x3_account_create_resource_account"></a>

## Function `create_resource_account`

A resource account is used to manage resources independent of an account managed by a user.
In Rooch a resource account is created based upon the sha3 256 of the source's address and additional seed data.
A resource account can only be created once


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_resource_account">create_resource_account</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, source: &<a href="">signer</a>): (<a href="">signer</a>, <a href="_SignerCapability">account::SignerCapability</a>)
</code></pre>
