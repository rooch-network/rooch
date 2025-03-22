
<a name="0x2_account"></a>

# Module `0x2::account`



-  [Resource `Account`](#0x2_account_Account)
-  [Resource `AccountCap`](#0x2_account_AccountCap)
-  [Constants](#@Constants_0)
-  [Function `create_account_by_system`](#0x2_account_create_account_by_system)
-  [Function `create_account`](#0x2_account_create_account)
-  [Function `create_account_and_return_cap`](#0x2_account_create_account_and_return_cap)
-  [Function `sequence_number`](#0x2_account_sequence_number)
-  [Function `increment_sequence_number_for_system`](#0x2_account_increment_sequence_number_for_system)
-  [Function `exists_at`](#0x2_account_exists_at)
-  [Function `create_signer_for_system`](#0x2_account_create_signer_for_system)
-  [Function `create_signer_with_account`](#0x2_account_create_signer_with_account)
-  [Function `create_signer_with_account_cap`](#0x2_account_create_signer_with_account_cap)
-  [Function `account_object_id`](#0x2_account_account_object_id)
-  [Function `account_address`](#0x2_account_account_address)
-  [Function `account_cap_address`](#0x2_account_account_cap_address)
-  [Function `account_sequence_number`](#0x2_account_account_sequence_number)
-  [Function `account_borrow_resource`](#0x2_account_account_borrow_resource)
-  [Function `account_borrow_mut_resource`](#0x2_account_account_borrow_mut_resource)
-  [Function `account_move_resource_to`](#0x2_account_account_move_resource_to)
-  [Function `account_move_resource_from`](#0x2_account_account_move_resource_from)
-  [Function `account_exists_resource`](#0x2_account_account_exists_resource)
-  [Function `destroy_account`](#0x2_account_destroy_account)
-  [Function `destroy_account_cap`](#0x2_account_destroy_account_cap)
-  [Function `borrow_account`](#0x2_account_borrow_account)
-  [Function `borrow_mut_account`](#0x2_account_borrow_mut_account)
-  [Function `borrow_resource`](#0x2_account_borrow_resource)
-  [Function `borrow_mut_resource`](#0x2_account_borrow_mut_resource)
-  [Function `move_resource_to`](#0x2_account_move_resource_to)
-  [Function `move_resource_from`](#0x2_account_move_resource_from)
-  [Function `exists_resource`](#0x2_account_exists_resource)


<pre><code><b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="core_addresses.md#0x2_core_addresses">0x2::core_addresses</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
<b>use</b> <a href="type_table.md#0x2_type_table">0x2::type_table</a>;
</code></pre>



<a name="0x2_account_Account"></a>

## Resource `Account`

Account is a struct that holds the sequence number for an address


<pre><code><b>struct</b> <a href="account.md#0x2_account_Account">Account</a> <b>has</b> key
</code></pre>



<a name="0x2_account_AccountCap"></a>

## Resource `AccountCap`

AccountCap is a capability for Account
The contract that has AccountCap can access the Account object


<pre><code><b>struct</b> <a href="account.md#0x2_account_AccountCap">AccountCap</a> <b>has</b> store, key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_account_MAX_U64"></a>



<pre><code><b>const</b> <a href="account.md#0x2_account_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x2_account_ErrorAccountAlreadyExists"></a>

Account already exists


<pre><code><b>const</b> <a href="account.md#0x2_account_ErrorAccountAlreadyExists">ErrorAccountAlreadyExists</a>: u64 = 1;
</code></pre>



<a name="0x2_account_ErrorAddressReserved"></a>

Cannot create account because address is reserved


<pre><code><b>const</b> <a href="account.md#0x2_account_ErrorAddressReserved">ErrorAddressReserved</a>: u64 = 3;
</code></pre>



<a name="0x2_account_ErrorDeprecateFunction"></a>

The function is deprecated


<pre><code><b>const</b> <a href="account.md#0x2_account_ErrorDeprecateFunction">ErrorDeprecateFunction</a>: u64 = 7;
</code></pre>



<a name="0x2_account_ErrorNotValidSystemReservedAddress"></a>

Address to create is not a valid reserved address


<pre><code><b>const</b> <a href="account.md#0x2_account_ErrorNotValidSystemReservedAddress">ErrorNotValidSystemReservedAddress</a>: u64 = 4;
</code></pre>



<a name="0x2_account_ErrorResourceAlreadyExists"></a>

The resource with the given type already exists


<pre><code><b>const</b> <a href="account.md#0x2_account_ErrorResourceAlreadyExists">ErrorResourceAlreadyExists</a>: u64 = 5;
</code></pre>



<a name="0x2_account_ErrorResourceNotExists"></a>

The resource with the given type not exists


<pre><code><b>const</b> <a href="account.md#0x2_account_ErrorResourceNotExists">ErrorResourceNotExists</a>: u64 = 6;
</code></pre>



<a name="0x2_account_ErrorSequenceNumberTooBig"></a>

Sequence number exceeds the maximum value for a u64


<pre><code><b>const</b> <a href="account.md#0x2_account_ErrorSequenceNumberTooBig">ErrorSequenceNumberTooBig</a>: u64 = 2;
</code></pre>



<a name="0x2_account_create_account_by_system"></a>

## Function `create_account_by_system`

Create a new account for the given address, only callable by the system account


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_create_account_by_system">create_account_by_system</a>(system: &<a href="">signer</a>, new_address: <b>address</b>): <a href="">signer</a>
</code></pre>



<a name="0x2_account_create_account"></a>

## Function `create_account`

This function is deprecated, please use <code>create_account_and_return_cap</code> instead


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_create_account">create_account</a>(): <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;
</code></pre>



<a name="0x2_account_create_account_and_return_cap"></a>

## Function `create_account_and_return_cap`

Create a new account and return the AccountCap


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_create_account_and_return_cap">create_account_and_return_cap</a>(): <a href="account.md#0x2_account_AccountCap">account::AccountCap</a>
</code></pre>



<a name="0x2_account_sequence_number"></a>

## Function `sequence_number`

Return the current sequence number at <code>addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_sequence_number">sequence_number</a>(addr: <b>address</b>): u64
</code></pre>



<a name="0x2_account_increment_sequence_number_for_system"></a>

## Function `increment_sequence_number_for_system`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_increment_sequence_number_for_system">increment_sequence_number_for_system</a>(system: &<a href="">signer</a>, sender: <b>address</b>)
</code></pre>



<a name="0x2_account_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_exists_at">exists_at</a>(addr: <b>address</b>): bool
</code></pre>



<a name="0x2_account_create_signer_for_system"></a>

## Function `create_signer_for_system`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_create_signer_for_system">create_signer_for_system</a>(system: &<a href="">signer</a>, addr: <b>address</b>): <a href="">signer</a>
</code></pre>



<a name="0x2_account_create_signer_with_account"></a>

## Function `create_signer_with_account`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_create_signer_with_account">create_signer_with_account</a>(<a href="account.md#0x2_account">account</a>: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;): <a href="">signer</a>
</code></pre>



<a name="0x2_account_create_signer_with_account_cap"></a>

## Function `create_signer_with_account_cap`

Create a signer with the given account capability


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_create_signer_with_account_cap">create_signer_with_account_cap</a>(cap: &<b>mut</b> <a href="account.md#0x2_account_AccountCap">account::AccountCap</a>): <a href="">signer</a>
</code></pre>



<a name="0x2_account_account_object_id"></a>

## Function `account_object_id`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_object_id">account_object_id</a>(<a href="account.md#0x2_account">account</a>: <b>address</b>): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_account_account_address"></a>

## Function `account_address`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_address">account_address</a>(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;): <b>address</b>
</code></pre>



<a name="0x2_account_account_cap_address"></a>

## Function `account_cap_address`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_cap_address">account_cap_address</a>(self: &<a href="account.md#0x2_account_AccountCap">account::AccountCap</a>): <b>address</b>
</code></pre>



<a name="0x2_account_account_sequence_number"></a>

## Function `account_sequence_number`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_sequence_number">account_sequence_number</a>(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;): u64
</code></pre>



<a name="0x2_account_account_borrow_resource"></a>

## Function `account_borrow_resource`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_borrow_resource">account_borrow_resource</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;): &T
</code></pre>



<a name="0x2_account_account_borrow_mut_resource"></a>

## Function `account_borrow_mut_resource`



<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_borrow_mut_resource">account_borrow_mut_resource</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;): &<b>mut</b> T
</code></pre>



<a name="0x2_account_account_move_resource_to"></a>

## Function `account_move_resource_to`



<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_move_resource_to">account_move_resource_to</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;, resource: T)
</code></pre>



<a name="0x2_account_account_move_resource_from"></a>

## Function `account_move_resource_from`



<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_move_resource_from">account_move_resource_from</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;): T
</code></pre>



<a name="0x2_account_account_exists_resource"></a>

## Function `account_exists_resource`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_exists_resource">account_exists_resource</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;): bool
</code></pre>



<a name="0x2_account_destroy_account"></a>

## Function `destroy_account`

Deprecated: Direct destruction of account objects is not allowed.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_destroy_account">destroy_account</a>(_account_obj: <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;)
</code></pre>



<a name="0x2_account_destroy_account_cap"></a>

## Function `destroy_account_cap`

Destroy the account capability


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_destroy_account_cap">destroy_account_cap</a>(account_cap: <a href="account.md#0x2_account_AccountCap">account::AccountCap</a>)
</code></pre>



<a name="0x2_account_borrow_account"></a>

## Function `borrow_account`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_borrow_account">borrow_account</a>(<a href="account.md#0x2_account">account</a>: <b>address</b>): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;
</code></pre>



<a name="0x2_account_borrow_mut_account"></a>

## Function `borrow_mut_account`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_borrow_mut_account">borrow_mut_account</a>(<a href="account.md#0x2_account">account</a>: &<a href="">signer</a>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;
</code></pre>



<a name="0x2_account_borrow_resource"></a>

## Function `borrow_resource`

Borrow a resource from the account's storage
This function equates to <code><b>borrow_global</b>&lt;T&gt;(<b>address</b>)</code> instruction in Move
But we remove the restriction of the caller must be the module of T


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_borrow_resource">borrow_resource</a>&lt;T: key&gt;(<a href="account.md#0x2_account">account</a>: <b>address</b>): &T
</code></pre>



<a name="0x2_account_borrow_mut_resource"></a>

## Function `borrow_mut_resource`

Borrow a mut resource from the account's storage
This function equates to <code><b>borrow_global_mut</b>&lt;T&gt;(<b>address</b>)</code> instruction in Move


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="account.md#0x2_account_borrow_mut_resource">borrow_mut_resource</a>&lt;T: key&gt;(<a href="account.md#0x2_account">account</a>: <b>address</b>): &<b>mut</b> T
</code></pre>



<a name="0x2_account_move_resource_to"></a>

## Function `move_resource_to`

Move a resource to the account's resource object
This function equates to <code><b>move_to</b>&lt;T&gt;(&<a href="">signer</a>, resource)</code> instruction in Move


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="account.md#0x2_account_move_resource_to">move_resource_to</a>&lt;T: key&gt;(<a href="account.md#0x2_account">account</a>: &<a href="">signer</a>, resource: T)
</code></pre>



<a name="0x2_account_move_resource_from"></a>

## Function `move_resource_from`

Move a resource from the account's storage
This function equates to <code><b>move_from</b>&lt;T&gt;(<b>address</b>)</code> instruction in Move


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="account.md#0x2_account_move_resource_from">move_resource_from</a>&lt;T: key&gt;(<a href="account.md#0x2_account">account</a>: <b>address</b>): T
</code></pre>



<a name="0x2_account_exists_resource"></a>

## Function `exists_resource`

Check if the account has a resource of the given type
This function equates to <code><b>exists</b>&lt;T&gt;(<b>address</b>)</code> instruction in Move
But we remove the restriction of the caller must be the module of T


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_exists_resource">exists_resource</a>&lt;T: key&gt;(<a href="account.md#0x2_account">account</a>: <b>address</b>): bool
</code></pre>
