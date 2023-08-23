
<a name="0x3_account"></a>

# Module `0x3::account`



-  [Resource `Account`](#0x3_account_Account)
-  [Resource `AutoAcceptCoin`](#0x3_account_AutoAcceptCoin)
-  [Struct `DepositEvent`](#0x3_account_DepositEvent)
-  [Struct `WithdrawEvent`](#0x3_account_WithdrawEvent)
-  [Struct `AcceptCoinEvent`](#0x3_account_AcceptCoinEvent)
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
-  [Function `is_account_accept_coin`](#0x3_account_is_account_accept_coin)
-  [Function `can_auto_accept_coin`](#0x3_account_can_auto_accept_coin)
-  [Function `do_accept_coin`](#0x3_account_do_accept_coin)
-  [Function `set_auto_accept_coin`](#0x3_account_set_auto_accept_coin)
-  [Function `withdraw`](#0x3_account_withdraw)
-  [Function `deposit`](#0x3_account_deposit)
-  [Function `transfer`](#0x3_account_transfer)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="../doc/hash.md#0x1_hash">0x1::hash</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="account_authentication.md#0x3_account_authentication">0x3::account_authentication</a>;
<b>use</b> <a href="coin.md#0x3_coin">0x3::coin</a>;
<b>use</b> <a href="gas_coin.md#0x3_gas_coin">0x3::gas_coin</a>;
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

<a name="0x3_account_AutoAcceptCoin"></a>

## Resource `AutoAcceptCoin`



<pre><code><b>struct</b> <a href="account.md#0x3_account_AutoAcceptCoin">AutoAcceptCoin</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>enable: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_account_DepositEvent"></a>

## Struct `DepositEvent`

Event emitted when some amount of a coin is deposited into an account.


<pre><code><b>struct</b> <a href="account.md#0x3_account_DepositEvent">DepositEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_type_info: <a href="_TypeInfo">type_info::TypeInfo</a></code>
</dt>
<dd>
 The type info for the coin that was sent
</dd>
<dt>
<code>amount: u256</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_account_WithdrawEvent"></a>

## Struct `WithdrawEvent`

Event emitted when some amount of a coin is withdrawn from an account.


<pre><code><b>struct</b> <a href="account.md#0x3_account_WithdrawEvent">WithdrawEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_type_info: <a href="_TypeInfo">type_info::TypeInfo</a></code>
</dt>
<dd>
 The type info for the coin that was sent
</dd>
<dt>
<code>amount: u256</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_account_AcceptCoinEvent"></a>

## Struct `AcceptCoinEvent`

Event for accept coin


<pre><code><b>struct</b> <a href="account.md#0x3_account_AcceptCoinEvent">AcceptCoinEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin_type_info: <a href="_TypeInfo">type_info::TypeInfo</a></code>
</dt>
<dd>
 full info of coin
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



<a name="0x3_account_EAccountNotAcceptCoin"></a>

Account hasn't accept <code>CoinType</code>


<pre><code><b>const</b> <a href="account.md#0x3_account_EAccountNotAcceptCoin">EAccountNotAcceptCoin</a>: u64 = 15;
</code></pre>



<a name="0x3_account_EAccountNotExist"></a>

Account does not exist


<pre><code><b>const</b> <a href="account.md#0x3_account_EAccountNotExist">EAccountNotExist</a>: u64 = 2;
</code></pre>



<a name="0x3_account_EAccountWithCoinFrozen"></a>

CoinStore is frozen. Coins cannot be deposited or withdrawn


<pre><code><b>const</b> <a href="account.md#0x3_account_EAccountWithCoinFrozen">EAccountWithCoinFrozen</a>: u64 = 13;
</code></pre>



<a name="0x3_account_EAddressReseved"></a>

Cannot create account because address is reserved


<pre><code><b>const</b> <a href="account.md#0x3_account_EAddressReseved">EAddressReseved</a>: u64 = 5;
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
   <b>assert</b>!(
      !<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account.md#0x3_account_Account">Account</a>&gt;(ctx, new_address),
      <a href="_already_exists">error::already_exists</a>(<a href="account.md#0x3_account_EAccountAlreadyExists">EAccountAlreadyExists</a>)
   );

   <b>let</b> new_account = <a href="account.md#0x3_account_create_account_unchecked">create_account_unchecked</a>(ctx, new_address);
   // Make sure all <a href="account.md#0x3_account">account</a> accept GasCoin.
   <a href="account.md#0x3_account_do_accept_coin">do_accept_coin</a>&lt;GasCoin&gt;(ctx, &new_account);
   new_account
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

<a name="0x3_account_is_account_accept_coin"></a>

## Function `is_account_accept_coin`

Return whether the account at <code>addr</code> accept <code>Coin</code> type coins


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_is_account_accept_coin">is_account_accept_coin</a>&lt;CoinType&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_is_account_accept_coin">is_account_accept_coin</a>&lt;CoinType&gt;(ctx: &StorageContext, addr: <b>address</b>): bool {
   <b>if</b> (<a href="account.md#0x3_account_can_auto_accept_coin">can_auto_accept_coin</a>(ctx, addr)) {
      <b>true</b>
   } <b>else</b> {
      <a href="coin.md#0x3_coin_exist_coin_store">coin::exist_coin_store</a>&lt;CoinType&gt;(ctx, addr)
   }
}
</code></pre>



</details>

<a name="0x3_account_can_auto_accept_coin"></a>

## Function `can_auto_accept_coin`

Check whether the address can auto accept coin.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_can_auto_accept_coin">can_auto_accept_coin</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_can_auto_accept_coin">can_auto_accept_coin</a>(ctx: &StorageContext, addr: <b>address</b>): bool {
   <b>if</b> (<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account.md#0x3_account_AutoAcceptCoin">AutoAcceptCoin</a>&gt;(ctx, addr)) {
      <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="account.md#0x3_account_AutoAcceptCoin">AutoAcceptCoin</a>&gt;(ctx, addr).enable
   } <b>else</b> {
      <b>false</b>
   }
}
</code></pre>



</details>

<a name="0x3_account_do_accept_coin"></a>

## Function `do_accept_coin`

Add a balance of <code>Coin</code> type to the sending account.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_do_accept_coin">do_accept_coin</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_do_accept_coin">do_accept_coin</a>&lt;CoinType&gt;(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>) {
   <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
   <b>if</b> (!<a href="coin.md#0x3_coin_exist_coin_store">coin::exist_coin_store</a>&lt;CoinType&gt;(ctx, addr)) {
      <a href="coin.md#0x3_coin_initialize_coin_store">coin::initialize_coin_store</a>&lt;CoinType&gt;(ctx, <a href="account.md#0x3_account">account</a>);

      <b>let</b> coin_type_info = <a href="_type_of">type_info::type_of</a>&lt;CoinType&gt;();
      <a href="_emit">event::emit</a>&lt;<a href="account.md#0x3_account_AcceptCoinEvent">AcceptCoinEvent</a>&gt;(ctx,
         <a href="account.md#0x3_account_AcceptCoinEvent">AcceptCoinEvent</a> {
            coin_type_info,
         },
      );
   }
}
</code></pre>



</details>

<a name="0x3_account_set_auto_accept_coin"></a>

## Function `set_auto_accept_coin`

Configure whether auto-accept coins.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_set_auto_accept_coin">set_auto_accept_coin</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, enable: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_set_auto_accept_coin">set_auto_accept_coin</a>(ctx: &<b>mut</b> StorageContext, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, enable: bool)  {
   <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
   <b>if</b> (<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="account.md#0x3_account_AutoAcceptCoin">AutoAcceptCoin</a>&gt;(ctx, addr)) {
      <b>let</b> config = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="account.md#0x3_account_AutoAcceptCoin">AutoAcceptCoin</a>&gt;(ctx, addr);
      config.enable = enable;
   } <b>else</b> {
      <a href="_global_move_to">account_storage::global_move_to</a>&lt;<a href="account.md#0x3_account_AutoAcceptCoin">AutoAcceptCoin</a>&gt;(ctx, <a href="account.md#0x3_account">account</a>, <a href="account.md#0x3_account_AutoAcceptCoin">AutoAcceptCoin</a>{ enable });
   };
}
</code></pre>



</details>

<a name="0x3_account_withdraw"></a>

## Function `withdraw`

Withdraw specifed <code>amount</code> of coin <code>CoinType</code> from the signing account.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_withdraw">withdraw</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>, amount: u256): <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_withdraw">withdraw</a>&lt;CoinType&gt;(
   ctx: &<b>mut</b> StorageContext,
   <a href="account.md#0x3_account">account</a>: &<a href="">signer</a>,
   amount: u256,
): Coin&lt;CoinType&gt; {
   <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x3_account">account</a>);
   <b>assert</b>!(
      <a href="account.md#0x3_account_is_account_accept_coin">is_account_accept_coin</a>&lt;CoinType&gt;(ctx, addr),
      <a href="_not_found">error::not_found</a>(<a href="account.md#0x3_account_EAccountNotAcceptCoin">EAccountNotAcceptCoin</a>),
   );

   <b>assert</b>!(
       !<a href="coin.md#0x3_coin_is_coin_store_frozen">coin::is_coin_store_frozen</a>&lt;CoinType&gt;(ctx, addr),
       <a href="_permission_denied">error::permission_denied</a>(<a href="account.md#0x3_account_EAccountWithCoinFrozen">EAccountWithCoinFrozen</a> ),
   );

   <b>let</b> coin_type_info = <a href="_type_of">type_info::type_of</a>&lt;CoinType&gt;();
   <a href="_emit">event::emit</a>&lt;<a href="account.md#0x3_account_WithdrawEvent">WithdrawEvent</a>&gt;(ctx, <a href="account.md#0x3_account_WithdrawEvent">WithdrawEvent</a> {
      coin_type_info,
      amount,
   });

   <a href="coin.md#0x3_coin_extract_coin">coin::extract_coin</a>(ctx, addr, amount)
}
</code></pre>



</details>

<a name="0x3_account_deposit"></a>

## Function `deposit`

Deposit the coin balance into the recipient's account and emit an event.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_deposit">deposit</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: <a href="coin.md#0x3_coin_Coin">coin::Coin</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_deposit">deposit</a>&lt;CoinType&gt;(ctx: &<b>mut</b> StorageContext, addr: <b>address</b>, <a href="coin.md#0x3_coin">coin</a>: Coin&lt;CoinType&gt;) {
   <a href="account.md#0x3_account_try_accept_coin">try_accept_coin</a>&lt;CoinType&gt;(ctx, addr);
   <b>assert</b>!(
      <a href="account.md#0x3_account_is_account_accept_coin">is_account_accept_coin</a>&lt;CoinType&gt;(ctx, addr),
      <a href="_not_found">error::not_found</a>(<a href="account.md#0x3_account_EAccountNotAcceptCoin">EAccountNotAcceptCoin</a>),
   );

   <b>assert</b>!(
       !<a href="coin.md#0x3_coin_is_coin_store_frozen">coin::is_coin_store_frozen</a>&lt;CoinType&gt;(ctx, addr),
       <a href="_permission_denied">error::permission_denied</a>(<a href="account.md#0x3_account_EAccountWithCoinFrozen">EAccountWithCoinFrozen</a>),
   );

   <b>let</b> coin_type_info = <a href="_type_of">type_info::type_of</a>&lt;CoinType&gt;();
   <a href="_emit">event::emit</a>&lt;<a href="account.md#0x3_account_DepositEvent">DepositEvent</a>&gt;(ctx, <a href="account.md#0x3_account_DepositEvent">DepositEvent</a> {
      coin_type_info,
      amount: <a href="coin.md#0x3_coin_value">coin::value</a>(&<a href="coin.md#0x3_coin">coin</a>),
   });

   <a href="coin.md#0x3_coin_merge_coin">coin::merge_coin</a>(ctx, addr, <a href="coin.md#0x3_coin">coin</a>);
}
</code></pre>



</details>

<a name="0x3_account_transfer"></a>

## Function `transfer`

Transfer <code>amount</code> of coins <code>CoinType</code> from <code>from</code> to <code><b>to</b></code>.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_transfer">transfer</a>&lt;CoinType&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, from: &<a href="">signer</a>, <b>to</b>: <b>address</b>, amount: u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x3_account_transfer">transfer</a>&lt;CoinType&gt;(
   ctx: &<b>mut</b> StorageContext,
   from: &<a href="">signer</a>,
   <b>to</b>: <b>address</b>,
   amount: u256,
) {
   <b>let</b> <a href="coin.md#0x3_coin">coin</a> = <a href="account.md#0x3_account_withdraw">withdraw</a>&lt;CoinType&gt;(ctx, from, amount);
   <a href="account.md#0x3_account_deposit">deposit</a>(ctx, <b>to</b>, <a href="coin.md#0x3_coin">coin</a>);
}
</code></pre>



</details>
