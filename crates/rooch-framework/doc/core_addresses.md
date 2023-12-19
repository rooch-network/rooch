
<a name="0x3_core_addresses"></a>

# Module `0x3::core_addresses`



-  [Constants](#@Constants_0)
-  [Function `assert_rooch_genesis`](#0x3_core_addresses_assert_rooch_genesis)
-  [Function `assert_rooch_genesis_address`](#0x3_core_addresses_assert_rooch_genesis_address)
-  [Function `is_rooch_genesis_address`](#0x3_core_addresses_is_rooch_genesis_address)
-  [Function `assert_rooch_framework`](#0x3_core_addresses_assert_rooch_framework)
-  [Function `assert_framework_reserved_address`](#0x3_core_addresses_assert_framework_reserved_address)
-  [Function `assert_framework_reserved`](#0x3_core_addresses_assert_framework_reserved)
-  [Function `is_framework_reserved_address`](#0x3_core_addresses_is_framework_reserved_address)
-  [Function `is_rooch_framework_address`](#0x3_core_addresses_is_rooch_framework_address)
-  [Function `assert_vm`](#0x3_core_addresses_assert_vm)
-  [Function `is_vm`](#0x3_core_addresses_is_vm)
-  [Function `is_vm_address`](#0x3_core_addresses_is_vm_address)
-  [Function `is_reserved_address`](#0x3_core_addresses_is_reserved_address)
-  [Function `genesis_address`](#0x3_core_addresses_genesis_address)


<pre><code><b>use</b> <a href="">0x1::signer</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_core_addresses_ErrorNotAssociationAddress"></a>

The address/account did not correspond to the association address


<pre><code><b>const</b> <a href="core_addresses.md#0x3_core_addresses_ErrorNotAssociationAddress">ErrorNotAssociationAddress</a>: u64 = 2;
</code></pre>



<a name="0x3_core_addresses_ErrorNotFrameworkReservedAddress"></a>

The address is not framework reserved address


<pre><code><b>const</b> <a href="core_addresses.md#0x3_core_addresses_ErrorNotFrameworkReservedAddress">ErrorNotFrameworkReservedAddress</a>: u64 = 5;
</code></pre>



<a name="0x3_core_addresses_ErrorNotGenesisAddress"></a>

The address/account did not correspond to the genesis address


<pre><code><b>const</b> <a href="core_addresses.md#0x3_core_addresses_ErrorNotGenesisAddress">ErrorNotGenesisAddress</a>: u64 = 1;
</code></pre>



<a name="0x3_core_addresses_ErrorNotRoochFrameworkAddress"></a>

The address/account did not correspond to the core framework address


<pre><code><b>const</b> <a href="core_addresses.md#0x3_core_addresses_ErrorNotRoochFrameworkAddress">ErrorNotRoochFrameworkAddress</a>: u64 = 4;
</code></pre>



<a name="0x3_core_addresses_ErrorNotVM"></a>

The operation can only be performed by the VM


<pre><code><b>const</b> <a href="core_addresses.md#0x3_core_addresses_ErrorNotVM">ErrorNotVM</a>: u64 = 3;
</code></pre>



<a name="0x3_core_addresses_assert_rooch_genesis"></a>

## Function `assert_rooch_genesis`



<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_assert_rooch_genesis">assert_rooch_genesis</a>(<a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<a name="0x3_core_addresses_assert_rooch_genesis_address"></a>

## Function `assert_rooch_genesis_address`



<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_assert_rooch_genesis_address">assert_rooch_genesis_address</a>(addr: <b>address</b>)
</code></pre>



<a name="0x3_core_addresses_is_rooch_genesis_address"></a>

## Function `is_rooch_genesis_address`



<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_is_rooch_genesis_address">is_rooch_genesis_address</a>(addr: <b>address</b>): bool
</code></pre>



<a name="0x3_core_addresses_assert_rooch_framework"></a>

## Function `assert_rooch_framework`



<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_assert_rooch_framework">assert_rooch_framework</a>(<a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<a name="0x3_core_addresses_assert_framework_reserved_address"></a>

## Function `assert_framework_reserved_address`



<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_assert_framework_reserved_address">assert_framework_reserved_address</a>(<a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<a name="0x3_core_addresses_assert_framework_reserved"></a>

## Function `assert_framework_reserved`



<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_assert_framework_reserved">assert_framework_reserved</a>(addr: <b>address</b>)
</code></pre>



<a name="0x3_core_addresses_is_framework_reserved_address"></a>

## Function `is_framework_reserved_address`

Return true if <code>addr</code> is 0x0 or under the on chain governance's control.


<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_is_framework_reserved_address">is_framework_reserved_address</a>(addr: <b>address</b>): bool
</code></pre>



<a name="0x3_core_addresses_is_rooch_framework_address"></a>

## Function `is_rooch_framework_address`

Return true if <code>addr</code> is 0x3.


<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_is_rooch_framework_address">is_rooch_framework_address</a>(addr: <b>address</b>): bool
</code></pre>



<a name="0x3_core_addresses_assert_vm"></a>

## Function `assert_vm`

Assert that the signer has the VM reserved address.


<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_assert_vm">assert_vm</a>(<a href="account.md#0x3_account">account</a>: &<a href="">signer</a>)
</code></pre>



<a name="0x3_core_addresses_is_vm"></a>

## Function `is_vm`

Return true if <code>addr</code> is a reserved address for the VM to call system modules.


<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_is_vm">is_vm</a>(<a href="account.md#0x3_account">account</a>: &<a href="">signer</a>): bool
</code></pre>



<a name="0x3_core_addresses_is_vm_address"></a>

## Function `is_vm_address`

Return true if <code>addr</code> is a reserved address for the VM to call system modules.


<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_is_vm_address">is_vm_address</a>(addr: <b>address</b>): bool
</code></pre>



<a name="0x3_core_addresses_is_reserved_address"></a>

## Function `is_reserved_address`

Return true if <code>addr</code> is either the VM address or an Rooch Framework address.


<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_is_reserved_address">is_reserved_address</a>(addr: <b>address</b>): bool
</code></pre>



<a name="0x3_core_addresses_genesis_address"></a>

## Function `genesis_address`

The address of the genesis


<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x3_core_addresses_genesis_address">genesis_address</a>(): <b>address</b>
</code></pre>
