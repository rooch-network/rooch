
<a name="0x3_auth_validator_registry"></a>

# Module `0x3::auth_validator_registry`



-  [Resource `AuthValidatorWithType`](#0x3_auth_validator_registry_AuthValidatorWithType)
-  [Resource `ValidatorRegistry`](#0x3_auth_validator_registry_ValidatorRegistry)
-  [Constants](#@Constants_0)
-  [Function `system_reserved_validator_id`](#0x3_auth_validator_registry_system_reserved_validator_id)
-  [Function `genesis_init`](#0x3_auth_validator_registry_genesis_init)
-  [Function `register`](#0x3_auth_validator_registry_register)
-  [Function `register_by_system`](#0x3_auth_validator_registry_register_by_system)
-  [Function `register_by_system_with_id`](#0x3_auth_validator_registry_register_by_system_with_id)
-  [Function `register_internal`](#0x3_auth_validator_registry_register_internal)
-  [Function `is_registered`](#0x3_auth_validator_registry_is_registered)
-  [Function `borrow_validator`](#0x3_auth_validator_registry_borrow_validator)
-  [Function `borrow_validator_by_type`](#0x3_auth_validator_registry_borrow_validator_by_type)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::core_addresses</a>;
<b>use</b> <a href="">0x2::features</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="">0x2::type_table</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
</code></pre>



<a name="0x3_auth_validator_registry_AuthValidatorWithType"></a>

## Resource `AuthValidatorWithType`



<pre><code><b>struct</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidatorWithType">AuthValidatorWithType</a>&lt;ValidatorType: store&gt; <b>has</b> store, key
</code></pre>



<a name="0x3_auth_validator_registry_ValidatorRegistry"></a>

## Resource `ValidatorRegistry`



<pre><code><b>struct</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_ValidatorRegistry">ValidatorRegistry</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_auth_validator_registry_ErrorDeprecated"></a>



<pre><code><b>const</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_ErrorDeprecated">ErrorDeprecated</a>: u64 = 3;
</code></pre>



<a name="0x3_auth_validator_registry_ErrorInvalidValidatorId"></a>



<pre><code><b>const</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_ErrorInvalidValidatorId">ErrorInvalidValidatorId</a>: u64 = 4;
</code></pre>



<a name="0x3_auth_validator_registry_ErrorValidatorAlreadyRegistered"></a>



<pre><code><b>const</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_ErrorValidatorAlreadyRegistered">ErrorValidatorAlreadyRegistered</a>: u64 = 2;
</code></pre>



<a name="0x3_auth_validator_registry_ErrorValidatorUnregistered"></a>



<pre><code><b>const</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_ErrorValidatorUnregistered">ErrorValidatorUnregistered</a>: u64 = 1;
</code></pre>



<a name="0x3_auth_validator_registry_SYSTEM_RESERVED_VALIDATOR_ID"></a>

From 0 to 99 are reserved for system validators.


<pre><code><b>const</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_SYSTEM_RESERVED_VALIDATOR_ID">SYSTEM_RESERVED_VALIDATOR_ID</a>: u64 = 100;
</code></pre>



<a name="0x3_auth_validator_registry_system_reserved_validator_id"></a>

## Function `system_reserved_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_system_reserved_validator_id">system_reserved_validator_id</a>(): u64
</code></pre>



<a name="0x3_auth_validator_registry_genesis_init"></a>

## Function `genesis_init`

Init function called by genesis.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_genesis_init">genesis_init</a>(sender: &<a href="">signer</a>)
</code></pre>



<a name="0x3_auth_validator_registry_register"></a>

## Function `register`

Register a new validator. This feature not enabled in the mainnet.


<pre><code>#[private_generics(#[ValidatorType])]
<b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register">register</a>&lt;ValidatorType: store&gt;(): u64
</code></pre>



<a name="0x3_auth_validator_registry_register_by_system"></a>

## Function `register_by_system`

Deprecated.


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_by_system">register_by_system</a>&lt;ValidatorType: store&gt;(_system: &<a href="">signer</a>): u64
</code></pre>



<a name="0x3_auth_validator_registry_register_by_system_with_id"></a>

## Function `register_by_system_with_id`

Register a new validator by system with a specific id. This function is only called by system.


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_by_system_with_id">register_by_system_with_id</a>&lt;ValidatorType: store&gt;(system: &<a href="">signer</a>, id: u64): u64
</code></pre>



<a name="0x3_auth_validator_registry_register_internal"></a>

## Function `register_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_internal">register_internal</a>&lt;ValidatorType: store&gt;(id: <a href="_Option">option::Option</a>&lt;u64&gt;): u64
</code></pre>



<a name="0x3_auth_validator_registry_is_registered"></a>

## Function `is_registered`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_is_registered">is_registered</a>&lt;ValidatorType: store&gt;(): bool
</code></pre>



<a name="0x3_auth_validator_registry_borrow_validator"></a>

## Function `borrow_validator`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator">borrow_validator</a>(id: u64): &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>
</code></pre>



<a name="0x3_auth_validator_registry_borrow_validator_by_type"></a>

## Function `borrow_validator_by_type`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator_by_type">borrow_validator_by_type</a>&lt;ValidatorType: store&gt;(): &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>
</code></pre>
