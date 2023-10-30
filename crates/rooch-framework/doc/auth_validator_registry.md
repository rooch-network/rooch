
<a name="0x3_auth_validator_registry"></a>

# Module `0x3::auth_validator_registry`



-  [Resource `AuthValidatorWithType`](#0x3_auth_validator_registry_AuthValidatorWithType)
-  [Resource `ValidatorRegistry`](#0x3_auth_validator_registry_ValidatorRegistry)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_auth_validator_registry_genesis_init)
-  [Function `register`](#0x3_auth_validator_registry_register)
-  [Function `register_internal`](#0x3_auth_validator_registry_register_internal)
-  [Function `borrow_validator`](#0x3_auth_validator_registry_borrow_validator)
-  [Function `borrow_validator_by_type`](#0x3_auth_validator_registry_borrow_validator_by_type)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="">0x2::type_table</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
</code></pre>



<a name="0x3_auth_validator_registry_AuthValidatorWithType"></a>

## Resource `AuthValidatorWithType`



<pre><code><b>struct</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidatorWithType">AuthValidatorWithType</a>&lt;ValidatorType: store&gt; <b>has</b> key
</code></pre>



<a name="0x3_auth_validator_registry_ValidatorRegistry"></a>

## Resource `ValidatorRegistry`



<pre><code><b>struct</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_ValidatorRegistry">ValidatorRegistry</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_auth_validator_registry_ErrorValidatorAlreadyRegistered"></a>



<pre><code><b>const</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_ErrorValidatorAlreadyRegistered">ErrorValidatorAlreadyRegistered</a>: u64 = 2;
</code></pre>



<a name="0x3_auth_validator_registry_ErrorValidatorUnregistered"></a>



<pre><code><b>const</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_ErrorValidatorUnregistered">ErrorValidatorUnregistered</a>: u64 = 1;
</code></pre>



<a name="0x3_auth_validator_registry_genesis_init"></a>

## Function `genesis_init`

Init function called by genesis.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, sender: &<a href="">signer</a>)
</code></pre>



<a name="0x3_auth_validator_registry_register"></a>

## Function `register`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register">register</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>): u64
</code></pre>



<a name="0x3_auth_validator_registry_register_internal"></a>

## Function `register_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_internal">register_internal</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> <a href="_Context">context::Context</a>): u64
</code></pre>



<a name="0x3_auth_validator_registry_borrow_validator"></a>

## Function `borrow_validator`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator">borrow_validator</a>(ctx: &<a href="_Context">context::Context</a>, id: u64): &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>
</code></pre>



<a name="0x3_auth_validator_registry_borrow_validator_by_type"></a>

## Function `borrow_validator_by_type`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator_by_type">borrow_validator_by_type</a>&lt;ValidatorType: store&gt;(ctx: &<a href="_Context">context::Context</a>): &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>
</code></pre>
