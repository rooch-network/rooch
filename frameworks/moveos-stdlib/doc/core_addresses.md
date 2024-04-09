
<a name="0x2_core_addresses"></a>

# Module `0x2::core_addresses`



-  [Constants](#@Constants_0)
-  [Function `assert_vm`](#0x2_core_addresses_assert_vm)
-  [Function `is_vm`](#0x2_core_addresses_is_vm)
-  [Function `is_vm_address`](#0x2_core_addresses_is_vm_address)
-  [Function `assert_system_reserved`](#0x2_core_addresses_assert_system_reserved)
-  [Function `assert_system_reserved_address`](#0x2_core_addresses_assert_system_reserved_address)
-  [Function `is_system_reserved_address`](#0x2_core_addresses_is_system_reserved_address)
-  [Function `is_reserved_address`](#0x2_core_addresses_is_reserved_address)


<pre><code><b>use</b> <a href="">0x1::signer</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_core_addresses_ErrorNotSystemReservedAddress"></a>

The address is not rooch reserved address


<pre><code><b>const</b> <a href="core_addresses.md#0x2_core_addresses_ErrorNotSystemReservedAddress">ErrorNotSystemReservedAddress</a>: u64 = 2;
</code></pre>



<a name="0x2_core_addresses_ErrorNotVM"></a>

The operation can only be performed by the VM


<pre><code><b>const</b> <a href="core_addresses.md#0x2_core_addresses_ErrorNotVM">ErrorNotVM</a>: u64 = 1;
</code></pre>



<a name="0x2_core_addresses_assert_vm"></a>

## Function `assert_vm`

Assert that the signer has the VM reserved address.


<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x2_core_addresses_assert_vm">assert_vm</a>(<a href="account.md#0x2_account">account</a>: &<a href="">signer</a>)
</code></pre>



<a name="0x2_core_addresses_is_vm"></a>

## Function `is_vm`

Return true if <code>addr</code> is a reserved address for the VM to call system modules.


<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x2_core_addresses_is_vm">is_vm</a>(<a href="account.md#0x2_account">account</a>: &<a href="">signer</a>): bool
</code></pre>



<a name="0x2_core_addresses_is_vm_address"></a>

## Function `is_vm_address`

Return true if <code>addr</code> is a reserved address for the VM to call system modules.


<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x2_core_addresses_is_vm_address">is_vm_address</a>(addr: <b>address</b>): bool
</code></pre>



<a name="0x2_core_addresses_assert_system_reserved"></a>

## Function `assert_system_reserved`



<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x2_core_addresses_assert_system_reserved">assert_system_reserved</a>(<a href="account.md#0x2_account">account</a>: &<a href="">signer</a>)
</code></pre>



<a name="0x2_core_addresses_assert_system_reserved_address"></a>

## Function `assert_system_reserved_address`



<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x2_core_addresses_assert_system_reserved_address">assert_system_reserved_address</a>(addr: <b>address</b>)
</code></pre>



<a name="0x2_core_addresses_is_system_reserved_address"></a>

## Function `is_system_reserved_address`

Return true if <code>addr</code> is 0x0 or under the on chain governance's control.


<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x2_core_addresses_is_system_reserved_address">is_system_reserved_address</a>(addr: <b>address</b>): bool
</code></pre>



<a name="0x2_core_addresses_is_reserved_address"></a>

## Function `is_reserved_address`

Return true if <code>addr</code> is either the VM address or an Rooch system address.


<pre><code><b>public</b> <b>fun</b> <a href="core_addresses.md#0x2_core_addresses_is_reserved_address">is_reserved_address</a>(addr: <b>address</b>): bool
</code></pre>
