
<a name="0x3_account"></a>

# Module `0x3::account`



-  [Resource `Account`](#0x3_account_Account)
-  [Resource `Balance`](#0x3_account_Balance)
-  [Resource `ResourceAccount`](#0x3_account_ResourceAccount)
-  [Struct `SignerCapability`](#0x3_account_SignerCapability)
-  [Constants](#@Constants_0)
-  [Function `create_account_entry`](#0x3_account_create_account_entry)
-  [Function `create_account`](#0x3_account_create_account)
-  [Function `create_framework_reserved_account`](#0x3_account_create_framework_reserved_account)
-  [Function `sequence_number`](#0x3_account_sequence_number)
-  [Function `sequence_number_for_sender`](#0x3_account_sequence_number_for_sender)
-  [Function `increment_sequence_number`](#0x3_account_increment_sequence_number)
-  [Function `balance`](#0x3_account_balance)
-  [Function `get_authentication_key`](#0x3_account_get_authentication_key)
-  [Function `signer_address`](#0x3_account_signer_address)
-  [Function `is_resource_account`](#0x3_account_is_resource_account)
-  [Function `exists_at`](#0x3_account_exists_at)
-  [Function `create_resource_account`](#0x3_account_create_resource_account)
-  [Function `create_resource_address`](#0x3_account_create_resource_address)
-  [Function `rotate_authentication_key_internal`](#0x3_account_rotate_authentication_key_internal)
-  [Function `create_signer_with_capability`](#0x3_account_create_signer_with_capability)
-  [Function `get_signer_capability_address`](#0x3_account_get_signer_capability_address)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="../doc/hash.md#0x1_hash">0x1::hash</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
</code></pre>



<a name="0x3_account_Account"></a>

## Resource `Account`

Resource representing an account.


<pre><code><b>struct</b> <a href="account.md#0x3_account_Account">Account</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>authentication_key: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_account_Balance"></a>

## Resource `Balance`

A resource that holds the tokens stored in this account


<pre><code><b>struct</b> <a href="account.md#0x3_account_Balance">Balance</a>&lt;TokenType&gt; <b>has</b> key
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

<a name="0x3_account_ResourceAccount"></a>

## Resource `ResourceAccount`



<pre><code><b>struct</b> <a href="account.md#0x3_account_ResourceAccount">ResourceAccount</a> <b>has</b> key
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

<a name="0x3_account_SignerCapability"></a>

## Struct `SignerCapability`



<pre><code><b>struct</b> <a href="account.md#0x3_account_SignerCapability">SignerCapability</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>addr: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_account_MAX_U64"></a>



<pre><code><b>const</b> <a href="account.md#0x3_account_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x3_account_EAccountAlreadyExists"></a>

Account already exists


<pre><code><b>const</b> <a href="account.md#0x3_account_EAccountAlreadyExists">EAccountAlreadyExists</a>: u64 = 1;
</code></pre>



<a name="0x3_account_AUTHENTICATION_KEY_LENGTH"></a>

authentication key length


<pre><code><b>const</b> <a href="account.md#0x3_account_AUTHENTICATION_KEY_LENGTH">AUTHENTICATION_KEY_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x3_account_CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER"></a>



<pre><code><b>const</b> <a href="account.md#0x3_account_CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER">CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER</a>: <a href="">vector</a>&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
</code></pre>



<a name="0x3_account_DERIVE_RESOURCE_ACCOUNT_SCHEME"></a>

Scheme identifier used when hashing an account's address together with a seed to derive the address (not the
authentication key) of a resource account. This is an abuse of the notion of a scheme identifier which, for now,
serves to domain separate hashes used to derive resource account addresses from hashes used to derive
authentication keys. Without such separation, an adversary could create (and get a signer for) a resource account
whose address matches an existing address of a MultiEd25519 wallet.


<pre><code><b>const</b> <a href="account.md#0x3_account_DERIVE_RESOURCE_ACCOUNT_SCHEME">DERIVE_RESOURCE_ACCOUNT_SCHEME</a>: u8 = 255;
</code></pre>



<a name="0x3_account_EAccountIsAlreadyResourceAccount"></a>

Resource Account can't derive resource account


<pre><code><b>const</b> <a href="account.md#0x3_account_EAccountIsAlreadyResourceAccount">EAccountIsAlreadyResourceAccount</a>: u64 = 7;
</code></pre>



<a name="0x3_account_EAccountNotExist"></a>

Account does not exist


<pre><code><b>const</b> <a href="account.md#0x3_account_EAccountNotExist">EAccountNotExist</a>: u64 = 2;
</code></pre>



<a name="0x3_account_EAddressReseved"></a>

Cannot create account because address is reserved


<pre><code><b>const</b> <a href="account.md#0x3_account_EAddressReseved">EAddressReseved</a>: u64 = 5;
</code></pre>



<a name="0x3_account_EMalformedAuthenticationKey"></a>

The provided authentication key has an invalid length


<pre><code><b>const</b> <a href="account.md#0x3_account_EMalformedAuthenticationKey">EMalformedAuthenticationKey</a>: u64 = 4;
</code></pre>



<a name="0x3_account_ENoValidFrameworkReservedAddress"></a>

Address to create is not a valid reserved address for Rooch framework


<pre><code><b>const</b> <a href="account.md#0x3_account_ENoValidFrameworkReservedAddress">ENoValidFrameworkReservedAddress</a>: u64 = 11;
</code></pre>



<a name="0x3_account_EResourceAccountAlreadyUsed"></a>

An attempt to create a resource account on an account that has a committed transaction


<pre><code><b>const</b> <a href="account.md#0x3_account_EResourceAccountAlreadyUsed">EResourceAccountAlreadyUsed</a>: u64 = 6;
</code></pre>



<a name="0x3_account_ESequenceNumberTooBig"></a>

Sequence number exceeds the maximum value for a u64


<pre><code><b>const</b> <a href="account.md#0x3_account_ESequenceNumberTooBig">ESequenceNumberTooBig</a>: u64 = 3;
</code></pre>



<a name="0x3_account_ZERO_AUTH_KEY"></a>



<pre><code><b>const</b> <a href="account.md#0x3_account_ZERO_AUTH_KEY">ZERO_AUTH_KEY</a>: <a href="">vector</a>&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
</code></pre>



<a name="0x3_account_create_account_entry"></a>

## Function `create_account_entry`

A entry function to create an account under <code>new_address</code>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x3_account_create_account_entry">create_account_entry</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, new_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x3_account_create_account_entry">create_account_entry</a>(ctx: &<b>mut</b> StorageContext, new_address: <b>address</b>){
   <a href="account.md#0x3_account_create_account">Self::create_account</a>(ctx, new_address);
}
</code></pre>



</details>

<a name="0x3_account_create_account"></a>

## Function `create_account`

Publishes a new <code><a href="account.md#0x3_account_Account">Account</a></code> resource under <code>new_address</code>. A signer representing <code>new_address</code>
is returned. This way, the caller of this function can publish additional resources under
<code>new_address</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_create_account">create_account</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, new_address: <b>address</b>): <a href="">signer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_create_account">create_account</a>(ctx: &<b>mut</b> StorageContext, new_address: <b>address</b>): <a href="">signer</a> {
   <b>assert</b>!(
      new_address != @vm_reserved && new_address != @rooch_framework,
      <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x3_account_EAddressReseved">EAddressReseved</a>)
   );

   // there cannot be an <a href="account.md#0x3_account_Account">Account</a> resource under new_addr already.
   <b>assert</b>!(!<b>exists</b>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(new_address), <a href="_already_exists">error::already_exists</a>(<a href="account.md#0x3_account_EAccountAlreadyExists">EAccountAlreadyExists</a>));

   <a href="account.md#0x3_account_create_account_unchecked">create_account_unchecked</a>(ctx, new_address)
}
</code></pre>



</details>

<a name="0x3_account_create_framework_reserved_account"></a>

## Function `create_framework_reserved_account`

create the account for system reserved addresses


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_create_framework_reserved_account">create_framework_reserved_account</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): (<a href="">signer</a>, <a href="account.md#0x3_account_SignerCapability">account::SignerCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_create_framework_reserved_account">create_framework_reserved_account</a>(ctx: &<b>mut</b> StorageContext, addr: <b>address</b>): (<a href="">signer</a>, <a href="account.md#0x3_account_SignerCapability">SignerCapability</a>) {
   <b>assert</b>!(
      addr == @0x1 ||
          addr == @0x2 ||
          addr == @0x3 ||
          addr == @0x4 ||
          addr == @0x5 ||
          addr == @0x6 ||
          addr == @0x7 ||
          addr == @0x8 ||
          addr == @0x9 ||
          addr == @0xa,
      <a href="_permission_denied">error::permission_denied</a>(<a href="account.md#0x3_account_ENoValidFrameworkReservedAddress">ENoValidFrameworkReservedAddress</a>),
   );
   <b>let</b> <a href="">signer</a> = <a href="account.md#0x3_account_create_account_unchecked">create_account_unchecked</a>(ctx, addr);
   <b>let</b> signer_cap = <a href="account.md#0x3_account_SignerCapability">SignerCapability</a> { addr };
   (<a href="">signer</a>, signer_cap)
}
</code></pre>



</details>

<a name="0x3_account_sequence_number"></a>

## Function `sequence_number`

Return the current sequence number at <code>addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_sequence_number">sequence_number</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_sequence_number">sequence_number</a>(ctx: &StorageContext, addr: <b>address</b>): u64 {
   // <b>if</b> <a href="account.md#0x3_account">account</a> does not exist, <b>return</b> 0 <b>as</b> sequence number
   // TODO: refactor this after we decide how <b>to</b> handle <a href="account.md#0x3_account">account</a> create.
   <b>if</b> (!<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(ctx, addr)) {
      <b>return</b> 0
   };
   <b>let</b> <a href="account.md#0x3_account">account</a> = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(ctx, addr);
   <a href="account.md#0x3_account_sequence_number_for_account">sequence_number_for_account</a>(<a href="account.md#0x3_account">account</a>)
}
</code></pre>



</details>

<a name="0x3_account_sequence_number_for_sender"></a>

## Function `sequence_number_for_sender`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_sequence_number_for_sender">sequence_number_for_sender</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_sequence_number_for_sender">sequence_number_for_sender</a>(ctx: &StorageContext): u64 {
   <b>let</b> sender = <a href="_sender">storage_context::sender</a>(ctx);
   <a href="account.md#0x3_account_sequence_number">sequence_number</a>(ctx, sender)
}
</code></pre>



</details>

<a name="0x3_account_increment_sequence_number"></a>

## Function `increment_sequence_number`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_increment_sequence_number">increment_sequence_number</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_increment_sequence_number">increment_sequence_number</a>(ctx: &<b>mut</b> StorageContext) {
   <b>let</b> sender = <a href="_sender">storage_context::sender</a>(ctx);

   <b>let</b> sequence_number = &<b>mut</b> <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(ctx, sender).sequence_number;

   <b>assert</b>!(
      (*sequence_number <b>as</b> u128) &lt; <a href="account.md#0x3_account_MAX_U64">MAX_U64</a>,
      <a href="_out_of_range">error::out_of_range</a>(<a href="account.md#0x3_account_ESequenceNumberTooBig">ESequenceNumberTooBig</a>)
   );

   *sequence_number = *sequence_number + 1;
}
</code></pre>



</details>

<a name="0x3_account_balance"></a>

## Function `balance`

Return the current TokenType balance of the account at <code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_balance">balance</a>&lt;TokenType: store&gt;(_addr: <b>address</b>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_balance">balance</a>&lt;TokenType: store&gt;(_addr: <b>address</b>): u128 {
   //TODO token standard, <b>with</b> balance precesion(u64|u128|u256)
   0u128
}
</code></pre>



</details>

<a name="0x3_account_get_authentication_key"></a>

## Function `get_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_get_authentication_key">get_authentication_key</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_get_authentication_key">get_authentication_key</a>(ctx: &StorageContext, addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt; {
   //<b>if</b> <a href="account.md#0x3_account">account</a> does not exist, <b>return</b> addr <b>as</b> authentication key
   <b>if</b>(!<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(ctx, addr)){
      <a href="_to_bytes">bcs::to_bytes</a>(&addr)
   }<b>else</b>{
      <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(ctx, addr).authentication_key
   }
}
</code></pre>



</details>

<a name="0x3_account_signer_address"></a>

## Function `signer_address`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_signer_address">signer_address</a>(cap: &<a href="account.md#0x3_account_SignerCapability">account::SignerCapability</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_signer_address">signer_address</a>(cap: &<a href="account.md#0x3_account_SignerCapability">SignerCapability</a>): <b>address</b> {
   cap.addr
}
</code></pre>



</details>

<a name="0x3_account_is_resource_account"></a>

## Function `is_resource_account`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_is_resource_account">is_resource_account</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_is_resource_account">is_resource_account</a>(ctx: &StorageContext, addr: <b>address</b>): bool {
   // for resource <a href="account.md#0x3_account">account</a> , <a href="account.md#0x3_account">account</a> storage maybe not exist when create,
   // so need check <a href="account.md#0x3_account">account</a> storage eixst befor call <b>global</b> exist function
   <b>if</b>(<a href="_exist_account_storage">account_storage::exist_account_storage</a>(ctx, addr)){
      <a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account.md#0x3_account_ResourceAccount">ResourceAccount</a>&gt;(ctx, addr)
   } <b>else</b> {
      <b>false</b>
   }
}
</code></pre>



</details>

<a name="0x3_account_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_exists_at">exists_at</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_exists_at">exists_at</a>(ctx: &StorageContext, addr: <b>address</b>): bool {
   <b>if</b>(<a href="_exist_account_storage">account_storage::exist_account_storage</a>(ctx, addr)){
      <a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(ctx, addr)
   } <b>else</b> {
      <b>false</b>
   }
}
</code></pre>



</details>

<a name="0x3_account_create_resource_account"></a>

## Function `create_resource_account`

A resource account is used to manage resources independent of an account managed by a user.
In Rooch a resource account is created based upon the sha3 256 of the source's address and additional seed data.
A resource account can only be created once


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_resource_account">create_resource_account</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, source: &<a href="">signer</a>): (<a href="">signer</a>, <a href="account.md#0x3_account_SignerCapability">account::SignerCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_resource_account">create_resource_account</a>(ctx: &<b>mut</b> StorageContext, source: &<a href="">signer</a>): (<a href="">signer</a>, <a href="account.md#0x3_account_SignerCapability">SignerCapability</a>) {
   <b>let</b> source_addr = <a href="_address_of">signer::address_of</a>(source);
   <b>let</b> seed = <a href="account.md#0x3_account_generate_seed_bytes">generate_seed_bytes</a>(ctx, &source_addr);
   <b>let</b> resource_addr = <a href="account.md#0x3_account_create_resource_address">create_resource_address</a>(&source_addr, seed);
   <b>assert</b>!(!<a href="account.md#0x3_account_is_resource_account">is_resource_account</a>(ctx, resource_addr), <a href="_invalid_state">error::invalid_state</a>(<a href="account.md#0x3_account_EAccountIsAlreadyResourceAccount">EAccountIsAlreadyResourceAccount</a>));
   <b>let</b> resource_signer = <b>if</b> (<a href="account.md#0x3_account_exists_at">exists_at</a>(ctx, resource_addr)) {
      <b>let</b> <a href="account.md#0x3_account">account</a> = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(ctx, resource_addr);
      <b>assert</b>!(<a href="account.md#0x3_account">account</a>.sequence_number == 0, <a href="_invalid_state">error::invalid_state</a>(<a href="account.md#0x3_account_EResourceAccountAlreadyUsed">EResourceAccountAlreadyUsed</a>));
      <a href="account.md#0x3_account_create_signer">create_signer</a>(resource_addr)
   } <b>else</b> {
      <a href="account.md#0x3_account_create_account_unchecked">create_account_unchecked</a>(ctx, resource_addr)
   };

   // By default, only the <a href="account.md#0x3_account_SignerCapability">SignerCapability</a> should have control over the resource <a href="account.md#0x3_account">account</a> and not the auth key.
   // If the source <a href="account.md#0x3_account">account</a> wants direct control via auth key, they would need <b>to</b> explicitly rotate the auth key
   // of the resource <a href="account.md#0x3_account">account</a> using the <a href="account.md#0x3_account_SignerCapability">SignerCapability</a>.
   <a href="account.md#0x3_account_rotate_authentication_key_internal">rotate_authentication_key_internal</a>(ctx,&resource_signer, <a href="account.md#0x3_account_ZERO_AUTH_KEY">ZERO_AUTH_KEY</a>);
   // <b>move_to</b>(&resource_signer, <a href="account.md#0x3_account_ResourceAccount">ResourceAccount</a> {});
   <a href="_global_move_to">account_storage::global_move_to</a>&lt;<a href="account.md#0x3_account_ResourceAccount">ResourceAccount</a>&gt;(ctx,
      &resource_signer,
      <a href="account.md#0x3_account_ResourceAccount">ResourceAccount</a> {}
   );

   <b>let</b> signer_cap = <a href="account.md#0x3_account_SignerCapability">SignerCapability</a> { addr: resource_addr };
   (resource_signer, signer_cap)
}
</code></pre>



</details>

<a name="0x3_account_create_resource_address"></a>

## Function `create_resource_address`

This is a helper function to compute resource addresses. Computation of the address
involves the use of a cryptographic hash operation and should be use thoughtfully.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_resource_address">create_resource_address</a>(source: &<b>address</b>, seed: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_resource_address">create_resource_address</a>(source: &<b>address</b>, seed: <a href="">vector</a>&lt;u8&gt;): <b>address</b> {
   <b>let</b> bytes = <a href="_to_bytes">bcs::to_bytes</a>(source);
   <a href="_append">vector::append</a>(&<b>mut</b> bytes, seed);
   <a href="_push_back">vector::push_back</a>(&<b>mut</b> bytes, <a href="account.md#0x3_account_DERIVE_RESOURCE_ACCOUNT_SCHEME">DERIVE_RESOURCE_ACCOUNT_SCHEME</a>);
   bcs::to_address(<a href="../doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(bytes))
}
</code></pre>



</details>

<a name="0x3_account_rotate_authentication_key_internal"></a>

## Function `rotate_authentication_key_internal`

This function is used to rotate a resource account's authentication key to 0, so that no private key can control
the resource account.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_rotate_authentication_key_internal">rotate_authentication_key_internal</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, new_auth_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_rotate_authentication_key_internal">rotate_authentication_key_internal</a>(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, new_auth_key: <a href="">vector</a>&lt;u8&gt;) {
   <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
   <b>assert</b>!(<a href="account.md#0x3_account_exists_at">exists_at</a>(ctx, addr), <a href="_not_found">error::not_found</a>(<a href="account.md#0x3_account_EAccountNotExist">EAccountNotExist</a>));
   <b>assert</b>!(
      <a href="_length">vector::length</a>(&new_auth_key) == <a href="account.md#0x3_account_AUTHENTICATION_KEY_LENGTH">AUTHENTICATION_KEY_LENGTH</a>,
      <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x3_account_EMalformedAuthenticationKey">EMalformedAuthenticationKey</a>)
   );
   <b>let</b> account_resource = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(ctx, addr);
   account_resource.authentication_key = new_auth_key;
}
</code></pre>



</details>

<a name="0x3_account_create_signer_with_capability"></a>

## Function `create_signer_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_signer_with_capability">create_signer_with_capability</a>(capability: &<a href="account.md#0x3_account_SignerCapability">account::SignerCapability</a>): <a href="">signer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_signer_with_capability">create_signer_with_capability</a>(capability: &<a href="account.md#0x3_account_SignerCapability">SignerCapability</a>): <a href="">signer</a> {
   <b>let</b> addr = &capability.addr;
   <a href="account.md#0x3_account_create_signer">create_signer</a>(*addr)
}
</code></pre>



</details>

<a name="0x3_account_get_signer_capability_address"></a>

## Function `get_signer_capability_address`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_get_signer_capability_address">get_signer_capability_address</a>(capability: &<a href="account.md#0x3_account_SignerCapability">account::SignerCapability</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_get_signer_capability_address">get_signer_capability_address</a>(capability: &<a href="account.md#0x3_account_SignerCapability">SignerCapability</a>): <b>address</b> {
   capability.addr
}
</code></pre>



</details>
