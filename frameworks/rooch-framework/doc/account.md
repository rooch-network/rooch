
<a name="0x3_account"></a>

# Module `0x3::account`



-  [Struct `AccountPlaceholder`](#0x3_account_AccountPlaceholder)
-  [Constants](#@Constants_0)
-  [Function `create_account`](#0x3_account_create_account)
-  [Function `create_system_account`](#0x3_account_create_system_account)


<pre><code><b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::core_addresses</a>;
<b>use</b> <a href="">0x2::signer</a>;
</code></pre>



<a name="0x3_account_AccountPlaceholder"></a>

## Struct `AccountPlaceholder`

Just using to get Account module signer


<pre><code><b>struct</b> <a href="account.md#0x3_account_AccountPlaceholder">AccountPlaceholder</a>
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_account_ErrorAddressReserved"></a>

Cannot create account because address is reserved


<pre><code><b>const</b> <a href="account.md#0x3_account_ErrorAddressReserved">ErrorAddressReserved</a>: u64 = 1;
</code></pre>



<a name="0x3_account_ErrorAddressNotReserved"></a>



<pre><code><b>const</b> <a href="account.md#0x3_account_ErrorAddressNotReserved">ErrorAddressNotReserved</a>: u64 = 2;
</code></pre>



<a name="0x3_account_create_account"></a>

## Function `create_account`

Create a new account with the given address, the address must not be reserved


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_create_account">create_account</a>(new_address: <b>address</b>): <a href="">signer</a>
</code></pre>



<a name="0x3_account_create_system_account"></a>

## Function `create_system_account`

Create a new account with the given address, the address must be reserved as system address


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_create_system_account">create_system_account</a>(new_address: <b>address</b>): <a href="">signer</a>
</code></pre>
