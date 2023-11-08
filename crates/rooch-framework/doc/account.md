
<a name="0x3_account"></a>

# Module `0x3::account`



-  [Resource `Account`](#0x3_account_Account)
-  [Resource `ResourceAccount`](#0x3_account_ResourceAccount)
-  [Struct `SignerCapability`](#0x3_account_SignerCapability)
-  [Constants](#@Constants_0)
-  [Function `create_account_entry`](#0x3_account_create_account_entry)
-  [Function `create_account`](#0x3_account_create_account)
-  [Function `create_framework_reserved_account`](#0x3_account_create_framework_reserved_account)
-  [Function `sequence_number`](#0x3_account_sequence_number)
-  [Function `sequence_number_for_sender`](#0x3_account_sequence_number_for_sender)
-  [Function `increment_sequence_number`](#0x3_account_increment_sequence_number)
-  [Function `signer_address`](#0x3_account_signer_address)
-  [Function `is_resource_account`](#0x3_account_is_resource_account)
-  [Function `exists_at`](#0x3_account_exists_at)
-  [Function `create_resource_account`](#0x3_account_create_resource_account)
-  [Function `create_resource_address`](#0x3_account_create_resource_address)
-  [Function `create_signer_with_capability`](#0x3_account_create_signer_with_capability)
-  [Function `get_signer_capability_address`](#0x3_account_get_signer_capability_address)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="account_authentication.md#0x3_account_authentication">0x3::account_authentication</a>;
<b>use</b> <a href="account_coin_store.md#0x3_account_coin_store">0x3::account_coin_store</a>;
</code></pre>



<a name="0x3_account_Account"></a>

## Resource `Account`

Resource representing an account.


<pre><code><b>struct</b> <a href="account.md#0x3_account_Account">Account</a> <b>has</b> store, key
</code></pre>



<a name="0x3_account_ResourceAccount"></a>

## Resource `ResourceAccount`

ResourceAccount can only be stored under address, not in other structs.


<pre><code><b>struct</b> <a href="account.md#0x3_account_ResourceAccount">ResourceAccount</a> <b>has</b> key
</code></pre>



<a name="0x3_account_SignerCapability"></a>

## Struct `SignerCapability`

SignerCapability can only be stored in other structs, not under address.
So that the capability is always controlled by contracts, not by some EOA.


<pre><code><b>struct</b> <a href="account.md#0x3_account_SignerCapability">SignerCapability</a> <b>has</b> store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_account_MAX_U64"></a>



<pre><code><b>const</b> <a href="account.md#0x3_account_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x3_account_CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER"></a>



<pre><code><b>const</b> <a href="account.md#0x3_account_CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER">CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER</a>: <a href="">vector</a>&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
</code></pre>



<a name="0x3_account_ErrorAccountAlreadyExists"></a>

Account already exists


<pre><code><b>const</b> <a href="account.md#0x3_account_ErrorAccountAlreadyExists">ErrorAccountAlreadyExists</a>: u64 = 1;
</code></pre>



<a name="0x3_account_ErrorAccountIsAlreadyResourceAccount"></a>

Resource Account can't derive resource account


<pre><code><b>const</b> <a href="account.md#0x3_account_ErrorAccountIsAlreadyResourceAccount">ErrorAccountIsAlreadyResourceAccount</a>: u64 = 7;
</code></pre>



<a name="0x3_account_ErrorAccountNotExist"></a>

Account does not exist


<pre><code><b>const</b> <a href="account.md#0x3_account_ErrorAccountNotExist">ErrorAccountNotExist</a>: u64 = 2;
</code></pre>



<a name="0x3_account_ErrorAddressReseved"></a>

Cannot create account because address is reserved


<pre><code><b>const</b> <a href="account.md#0x3_account_ErrorAddressReseved">ErrorAddressReseved</a>: u64 = 5;
</code></pre>



<a name="0x3_account_ErrorNotValidFrameworkReservedAddress"></a>

Address to create is not a valid reserved address for Rooch framework


<pre><code><b>const</b> <a href="account.md#0x3_account_ErrorNotValidFrameworkReservedAddress">ErrorNotValidFrameworkReservedAddress</a>: u64 = 11;
</code></pre>



<a name="0x3_account_ErrorResourceAccountAlreadyUsed"></a>

An attempt to create a resource account on an account that has a committed transaction


<pre><code><b>const</b> <a href="account.md#0x3_account_ErrorResourceAccountAlreadyUsed">ErrorResourceAccountAlreadyUsed</a>: u64 = 6;
</code></pre>



<a name="0x3_account_ErrorSequenceNumberTooBig"></a>

Sequence number exceeds the maximum value for a u64


<pre><code><b>const</b> <a href="account.md#0x3_account_ErrorSequenceNumberTooBig">ErrorSequenceNumberTooBig</a>: u64 = 3;
</code></pre>



<a name="0x3_account_SCHEME_DERIVE_RESOURCE_ACCOUNT"></a>

Scheme identifier used when hashing an account's address together with a seed to derive the address (not the
authentication key) of a resource account. This is an abuse of the notion of a scheme identifier which, for now,
serves to domain separate hashes used to derive resource account addresses from hashes used to derive
authentication keys. Without such separation, an adversary could create (and get a signer for) a resource account
whose address matches an existing address of a MultiEd25519 wallet.


<pre><code><b>const</b> <a href="account.md#0x3_account_SCHEME_DERIVE_RESOURCE_ACCOUNT">SCHEME_DERIVE_RESOURCE_ACCOUNT</a>: u8 = 255;
</code></pre>



<a name="0x3_account_ZERO_AUTH_KEY"></a>



<pre><code><b>const</b> <a href="account.md#0x3_account_ZERO_AUTH_KEY">ZERO_AUTH_KEY</a>: <a href="">vector</a>&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
</code></pre>



<a name="0x3_account_create_account_entry"></a>

## Function `create_account_entry`

A entry function to create an account under <code>new_address</code>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x3_account_create_account_entry">create_account_entry</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, new_address: <b>address</b>)
</code></pre>



<a name="0x3_account_create_account"></a>

## Function `create_account`

Publishes a new <code><a href="account.md#0x3_account_Account">Account</a></code> resource under <code>new_address</code>. A signer representing <code>new_address</code>
is returned. This way, the caller of this function can publish additional resources under
<code>new_address</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_create_account">create_account</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, new_address: <b>address</b>): <a href="">signer</a>
</code></pre>



<a name="0x3_account_create_framework_reserved_account"></a>

## Function `create_framework_reserved_account`

create the account for system reserved addresses


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_create_framework_reserved_account">create_framework_reserved_account</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, addr: <b>address</b>): (<a href="">signer</a>, <a href="account.md#0x3_account_SignerCapability">account::SignerCapability</a>)
</code></pre>



<a name="0x3_account_sequence_number"></a>

## Function `sequence_number`

Return the current sequence number at <code>addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_sequence_number">sequence_number</a>(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): u64
</code></pre>



<a name="0x3_account_sequence_number_for_sender"></a>

## Function `sequence_number_for_sender`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_sequence_number_for_sender">sequence_number_for_sender</a>(ctx: &<a href="_Context">context::Context</a>): u64
</code></pre>



<a name="0x3_account_increment_sequence_number"></a>

## Function `increment_sequence_number`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_increment_sequence_number">increment_sequence_number</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>)
</code></pre>



<a name="0x3_account_signer_address"></a>

## Function `signer_address`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_signer_address">signer_address</a>(cap: &<a href="account.md#0x3_account_SignerCapability">account::SignerCapability</a>): <b>address</b>
</code></pre>



<a name="0x3_account_is_resource_account"></a>

## Function `is_resource_account`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_is_resource_account">is_resource_account</a>(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): bool
</code></pre>



<a name="0x3_account_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_exists_at">exists_at</a>(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): bool
</code></pre>



<a name="0x3_account_create_resource_account"></a>

## Function `create_resource_account`

A resource account is used to manage resources independent of an account managed by a user.
In Rooch a resource account is created based upon the sha3 256 of the source's address and additional seed data.
A resource account can only be created once


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_resource_account">create_resource_account</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, source: &<a href="">signer</a>): (<a href="">signer</a>, <a href="account.md#0x3_account_SignerCapability">account::SignerCapability</a>)
</code></pre>



<a name="0x3_account_create_resource_address"></a>

## Function `create_resource_address`

This is a helper function to compute resource addresses. Computation of the address
involves the use of a cryptographic hash operation and should be use thoughtfully.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_resource_address">create_resource_address</a>(source: &<b>address</b>, seed: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<a name="0x3_account_create_signer_with_capability"></a>

## Function `create_signer_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_signer_with_capability">create_signer_with_capability</a>(capability: &<a href="account.md#0x3_account_SignerCapability">account::SignerCapability</a>): <a href="">signer</a>
</code></pre>



<a name="0x3_account_get_signer_capability_address"></a>

## Function `get_signer_capability_address`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_get_signer_capability_address">get_signer_capability_address</a>(capability: &<a href="account.md#0x3_account_SignerCapability">account::SignerCapability</a>): <b>address</b>
</code></pre>
