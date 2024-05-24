
<a name="0x2_account"></a>

# Module `0x2::account`



-  [Resource `Account`](#0x2_account_Account)
-  [Resource `AccountHolder`](#0x2_account_AccountHolder)
-  [Struct `SignerCapability`](#0x2_account_SignerCapability)
-  [Constants](#@Constants_0)
-  [Function `create_account_by_system`](#0x2_account_create_account_by_system)
-  [Function `create_system_reserved_account`](#0x2_account_create_system_reserved_account)
-  [Function `sequence_number`](#0x2_account_sequence_number)
-  [Function `increment_sequence_number_for_system`](#0x2_account_increment_sequence_number_for_system)
-  [Function `signer_address`](#0x2_account_signer_address)
-  [Function `exists_at`](#0x2_account_exists_at)
-  [Function `create_signer_for_system`](#0x2_account_create_signer_for_system)
-  [Function `create_signer`](#0x2_account_create_signer)
-  [Function `create_signer_with_capability`](#0x2_account_create_signer_with_capability)
-  [Function `get_signer_capability_address`](#0x2_account_get_signer_capability_address)
-  [Function `account_object_id`](#0x2_account_account_object_id)
-  [Function `create_account_object`](#0x2_account_create_account_object)
-  [Function `create_account_holder_object`](#0x2_account_create_account_holder_object)
-  [Function `account_borrow_resource`](#0x2_account_account_borrow_resource)
-  [Function `account_holder_borrow_resource`](#0x2_account_account_holder_borrow_resource)
-  [Function `account_borrow_mut_resource`](#0x2_account_account_borrow_mut_resource)
-  [Function `account_holder_borrow_mut_resource`](#0x2_account_account_holder_borrow_mut_resource)
-  [Function `account_move_resource_to`](#0x2_account_account_move_resource_to)
-  [Function `account_holder_move_resource_to`](#0x2_account_account_holder_move_resource_to)
-  [Function `account_move_resource_from`](#0x2_account_account_move_resource_from)
-  [Function `account_exists_resource`](#0x2_account_account_exists_resource)
-  [Function `account_holder_exists_resource`](#0x2_account_account_holder_exists_resource)
-  [Function `transfer`](#0x2_account_transfer)
-  [Function `borrow_account`](#0x2_account_borrow_account)
-  [Function `borrow_mut_account_holder`](#0x2_account_borrow_mut_account_holder)
-  [Function `borrow_resource`](#0x2_account_borrow_resource)
-  [Function `borrow_mut_resource`](#0x2_account_borrow_mut_resource)
-  [Function `move_resource_to`](#0x2_account_move_resource_to)
-  [Function `move_resource_from`](#0x2_account_move_resource_from)
-  [Function `exists_resource`](#0x2_account_exists_resource)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="core_addresses.md#0x2_core_addresses">0x2::core_addresses</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="type_table.md#0x2_type_table">0x2::type_table</a>;
</code></pre>



<a name="0x2_account_Account"></a>

## Resource `Account`

Account is part of the StorageAbstraction
It is also used to store the account's resources


<pre><code><b>struct</b> <a href="account.md#0x2_account_Account">Account</a> <b>has</b> key
</code></pre>



<a name="0x2_account_AccountHolder"></a>

## Resource `AccountHolder`



<pre><code><b>struct</b> <a href="account.md#0x2_account_AccountHolder">AccountHolder</a> <b>has</b> key
</code></pre>



<a name="0x2_account_SignerCapability"></a>

## Struct `SignerCapability`

SignerCapability can only be stored in other structs, not under address.
So that the capability is always controlled by contracts, not by some EOA.


<pre><code><b>struct</b> <a href="account.md#0x2_account_SignerCapability">SignerCapability</a> <b>has</b> store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_account_MAX_U64"></a>



<pre><code><b>const</b> <a href="account.md#0x2_account_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x2_account_CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER"></a>



<pre><code><b>const</b> <a href="account.md#0x2_account_CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER">CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER</a>: <a href="">vector</a>&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
</code></pre>



<a name="0x2_account_ErrorAccountAlreadyExists"></a>

Account already exists


<pre><code><b>const</b> <a href="account.md#0x2_account_ErrorAccountAlreadyExists">ErrorAccountAlreadyExists</a>: u64 = 1;
</code></pre>



<a name="0x2_account_ErrorAddressReserved"></a>

Cannot create account because address is reserved


<pre><code><b>const</b> <a href="account.md#0x2_account_ErrorAddressReserved">ErrorAddressReserved</a>: u64 = 3;
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



<a name="0x2_account_ZERO_AUTH_KEY"></a>



<pre><code><b>const</b> <a href="account.md#0x2_account_ZERO_AUTH_KEY">ZERO_AUTH_KEY</a>: <a href="">vector</a>&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
</code></pre>



<a name="0x2_account_create_account_by_system"></a>

## Function `create_account_by_system`

Publishes a new <code><a href="account.md#0x2_account_Account">Account</a></code> resource under <code>new_address</code> via system. A signer representing <code>new_address</code>
is returned. This way, the caller of this function can publish additional resources under
<code>new_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_create_account_by_system">create_account_by_system</a>(system: &<a href="">signer</a>, new_address: <b>address</b>): <a href="">signer</a>
</code></pre>



<a name="0x2_account_create_system_reserved_account"></a>

## Function `create_system_reserved_account`

create the account for system reserved addresses


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_create_system_reserved_account">create_system_reserved_account</a>(system: &<a href="">signer</a>, addr: <b>address</b>): (<a href="">signer</a>, <a href="account.md#0x2_account_SignerCapability">account::SignerCapability</a>)
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



<a name="0x2_account_signer_address"></a>

## Function `signer_address`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_signer_address">signer_address</a>(cap: &<a href="account.md#0x2_account_SignerCapability">account::SignerCapability</a>): <b>address</b>
</code></pre>



<a name="0x2_account_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_exists_at">exists_at</a>(addr: <b>address</b>): bool
</code></pre>



<a name="0x2_account_create_signer_for_system"></a>

## Function `create_signer_for_system`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_create_signer_for_system">create_signer_for_system</a>(system: &<a href="">signer</a>, addr: <b>address</b>): <a href="">signer</a>
</code></pre>



<a name="0x2_account_create_signer"></a>

## Function `create_signer`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x2_account_create_signer">create_signer</a>(addr: <b>address</b>): <a href="">signer</a>
</code></pre>



<a name="0x2_account_create_signer_with_capability"></a>

## Function `create_signer_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_create_signer_with_capability">create_signer_with_capability</a>(capability: &<a href="account.md#0x2_account_SignerCapability">account::SignerCapability</a>): <a href="">signer</a>
</code></pre>



<a name="0x2_account_get_signer_capability_address"></a>

## Function `get_signer_capability_address`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_get_signer_capability_address">get_signer_capability_address</a>(capability: &<a href="account.md#0x2_account_SignerCapability">account::SignerCapability</a>): <b>address</b>
</code></pre>



<a name="0x2_account_account_object_id"></a>

## Function `account_object_id`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_object_id">account_object_id</a>(<a href="account.md#0x2_account">account</a>: <b>address</b>): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_account_create_account_object"></a>

## Function `create_account_object`

Create a new account object space


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x2_account_create_account_object">create_account_object</a>(<a href="account.md#0x2_account">account</a>: <b>address</b>)
</code></pre>



<a name="0x2_account_create_account_holder_object"></a>

## Function `create_account_holder_object`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_create_account_holder_object">create_account_holder_object</a>(<a href="account.md#0x2_account">account</a>: <b>address</b>)
</code></pre>



<a name="0x2_account_account_borrow_resource"></a>

## Function `account_borrow_resource`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_borrow_resource">account_borrow_resource</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;): &T
</code></pre>



<a name="0x2_account_account_holder_borrow_resource"></a>

## Function `account_holder_borrow_resource`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_holder_borrow_resource">account_holder_borrow_resource</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_AccountHolder">account::AccountHolder</a>&gt;): &T
</code></pre>



<a name="0x2_account_account_borrow_mut_resource"></a>

## Function `account_borrow_mut_resource`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_borrow_mut_resource">account_borrow_mut_resource</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;): &<b>mut</b> T
</code></pre>



<a name="0x2_account_account_holder_borrow_mut_resource"></a>

## Function `account_holder_borrow_mut_resource`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_holder_borrow_mut_resource">account_holder_borrow_mut_resource</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_AccountHolder">account::AccountHolder</a>&gt;): &<b>mut</b> T
</code></pre>



<a name="0x2_account_account_move_resource_to"></a>

## Function `account_move_resource_to`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_move_resource_to">account_move_resource_to</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;, resource: T)
</code></pre>



<a name="0x2_account_account_holder_move_resource_to"></a>

## Function `account_holder_move_resource_to`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_holder_move_resource_to">account_holder_move_resource_to</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_AccountHolder">account::AccountHolder</a>&gt;, resource: T)
</code></pre>



<a name="0x2_account_account_move_resource_from"></a>

## Function `account_move_resource_from`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_move_resource_from">account_move_resource_from</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;): T
</code></pre>



<a name="0x2_account_account_exists_resource"></a>

## Function `account_exists_resource`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_exists_resource">account_exists_resource</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;): bool
</code></pre>



<a name="0x2_account_account_holder_exists_resource"></a>

## Function `account_holder_exists_resource`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_account_holder_exists_resource">account_holder_exists_resource</a>&lt;T: key&gt;(self: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_AccountHolder">account::AccountHolder</a>&gt;): bool
</code></pre>



<a name="0x2_account_transfer"></a>

## Function `transfer`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x2_account_transfer">transfer</a>(obj: <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;, <a href="account.md#0x2_account">account</a>: <b>address</b>)
</code></pre>



<a name="0x2_account_borrow_account"></a>

## Function `borrow_account`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_borrow_account">borrow_account</a>(<a href="account.md#0x2_account">account</a>: <b>address</b>): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_Account">account::Account</a>&gt;
</code></pre>



<a name="0x2_account_borrow_mut_account_holder"></a>

## Function `borrow_mut_account_holder`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x2_account_borrow_mut_account_holder">borrow_mut_account_holder</a>(<a href="account.md#0x2_account">account</a>: <b>address</b>): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account.md#0x2_account_AccountHolder">account::AccountHolder</a>&gt;
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
