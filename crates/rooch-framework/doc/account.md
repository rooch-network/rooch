
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
<b>use</b> <a href="">0x2::account_storage</a>;
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



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_account_ResourceAccount"></a>

## Resource `ResourceAccount`

ResourceAccount can only be stored under address, not in other structs.


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

SignerCapability can only be stored in other structs, not under address.
So that the capability is always controlled by contracts, not by some EOA.


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



<a name="0x3_account_ErrorAccountAlreadyExists"></a>

Account already exists


<pre><code><b>const</b> <a href="account.md#0x3_account_ErrorAccountAlreadyExists">ErrorAccountAlreadyExists</a>: u64 = 1;
</code></pre>



<a name="0x3_account_CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER"></a>



<pre><code><b>const</b> <a href="account.md#0x3_account_CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER">CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER</a>: <a href="">vector</a>&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
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



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x3_account_create_account_entry">create_account_entry</a>(ctx: &<b>mut</b> Context, new_address: <b>address</b>){
   // If <a href="account.md#0x3_account">account</a> already <b>exists</b>, do nothing
   // Because <b>if</b> the new <b>address</b> is the same <b>as</b> the sender, the <a href="account.md#0x3_account">account</a> must already created in the `<a href="transaction_validator.md#0x3_transaction_validator_pre_execute">transaction_validator::pre_execute</a>` function
   <b>if</b>(!<a href="account.md#0x3_account_exists_at">exists_at</a>(ctx, new_address)){
      <a href="account.md#0x3_account_create_account">create_account</a>(ctx, new_address);
   };
}
</code></pre>



</details>

<a name="0x3_account_create_account"></a>

## Function `create_account`

Publishes a new <code><a href="account.md#0x3_account_Account">Account</a></code> resource under <code>new_address</code>. A signer representing <code>new_address</code>
is returned. This way, the caller of this function can publish additional resources under
<code>new_address</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_create_account">create_account</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, new_address: <b>address</b>): <a href="">signer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_create_account">create_account</a>(ctx: &<b>mut</b> Context, new_address: <b>address</b>): <a href="">signer</a> {
   <b>assert</b>!(
      new_address != @vm_reserved,
      <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x3_account_ErrorAddressReseved">ErrorAddressReseved</a>)
   );

   // Make sure the <a href="account.md#0x3_account_Account">Account</a> is not already created.
   <b>assert</b>!(
      !<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(ctx, new_address),
      <a href="_already_exists">error::already_exists</a>(<a href="account.md#0x3_account_ErrorAccountAlreadyExists">ErrorAccountAlreadyExists</a>)
   );

   <b>let</b> new_account = <a href="account.md#0x3_account_create_account_unchecked">create_account_unchecked</a>(ctx, new_address);
   new_account
}
</code></pre>



</details>

<a name="0x3_account_create_framework_reserved_account"></a>

## Function `create_framework_reserved_account`

create the account for system reserved addresses


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_create_framework_reserved_account">create_framework_reserved_account</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, addr: <b>address</b>): (<a href="">signer</a>, <a href="account.md#0x3_account_SignerCapability">account::SignerCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_create_framework_reserved_account">create_framework_reserved_account</a>(ctx: &<b>mut</b> Context, addr: <b>address</b>): (<a href="">signer</a>, <a href="account.md#0x3_account_SignerCapability">SignerCapability</a>) {
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
      <a href="_permission_denied">error::permission_denied</a>(<a href="account.md#0x3_account_ErrorNotValidFrameworkReservedAddress">ErrorNotValidFrameworkReservedAddress</a>),
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


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_sequence_number">sequence_number</a>(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_sequence_number">sequence_number</a>(ctx: &Context, addr: <b>address</b>): u64 {
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



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_sequence_number_for_sender">sequence_number_for_sender</a>(ctx: &<a href="_Context">context::Context</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_sequence_number_for_sender">sequence_number_for_sender</a>(ctx: &Context): u64 {
   <b>let</b> sender = <a href="_sender">context::sender</a>(ctx);
   <a href="account.md#0x3_account_sequence_number">sequence_number</a>(ctx, sender)
}
</code></pre>



</details>

<a name="0x3_account_increment_sequence_number"></a>

## Function `increment_sequence_number`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_increment_sequence_number">increment_sequence_number</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x3_account_increment_sequence_number">increment_sequence_number</a>(ctx: &<b>mut</b> Context) {
   <b>let</b> sender = <a href="_sender">context::sender</a>(ctx);

   <b>let</b> sequence_number = &<b>mut</b> <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(ctx, sender).sequence_number;

   <b>assert</b>!(
      (*sequence_number <b>as</b> u128) &lt; <a href="account.md#0x3_account_MAX_U64">MAX_U64</a>,
      <a href="_out_of_range">error::out_of_range</a>(<a href="account.md#0x3_account_ErrorSequenceNumberTooBig">ErrorSequenceNumberTooBig</a>)
   );

   *sequence_number = *sequence_number + 1;
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



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_is_resource_account">is_resource_account</a>(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_is_resource_account">is_resource_account</a>(ctx: &Context, addr: <b>address</b>): bool {
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



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_exists_at">exists_at</a>(ctx: &<a href="_Context">context::Context</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_exists_at">exists_at</a>(ctx: &Context, addr: <b>address</b>): bool {
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


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_resource_account">create_resource_account</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, source: &<a href="">signer</a>): (<a href="">signer</a>, <a href="account.md#0x3_account_SignerCapability">account::SignerCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_create_resource_account">create_resource_account</a>(ctx: &<b>mut</b> Context, source: &<a href="">signer</a>): (<a href="">signer</a>, <a href="account.md#0x3_account_SignerCapability">SignerCapability</a>) {
   <b>let</b> source_addr = <a href="_address_of">signer::address_of</a>(source);
   <b>let</b> seed = <a href="account.md#0x3_account_generate_seed_bytes">generate_seed_bytes</a>(ctx, &source_addr);
   <b>let</b> resource_addr = <a href="account.md#0x3_account_create_resource_address">create_resource_address</a>(&source_addr, seed);
   <b>assert</b>!(!<a href="account.md#0x3_account_is_resource_account">is_resource_account</a>(ctx, resource_addr), <a href="_invalid_state">error::invalid_state</a>(<a href="account.md#0x3_account_ErrorAccountIsAlreadyResourceAccount">ErrorAccountIsAlreadyResourceAccount</a>));
   <b>let</b> resource_signer = <b>if</b> (<a href="account.md#0x3_account_exists_at">exists_at</a>(ctx, resource_addr)) {
      <b>let</b> <a href="account.md#0x3_account">account</a> = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(ctx, resource_addr);
      <b>assert</b>!(<a href="account.md#0x3_account">account</a>.sequence_number == 0, <a href="_invalid_state">error::invalid_state</a>(<a href="account.md#0x3_account_ErrorResourceAccountAlreadyUsed">ErrorResourceAccountAlreadyUsed</a>));
      <a href="account.md#0x3_account_create_signer">create_signer</a>(resource_addr)
   } <b>else</b> {
      <a href="account.md#0x3_account_create_account_unchecked">create_account_unchecked</a>(ctx, resource_addr)
   };

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
   <a href="_push_back">vector::push_back</a>(&<b>mut</b> bytes, <a href="account.md#0x3_account_SCHEME_DERIVE_RESOURCE_ACCOUNT">SCHEME_DERIVE_RESOURCE_ACCOUNT</a>);
   bcs::to_address(<a href="_sha3_256">hash::sha3_256</a>(bytes))
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
