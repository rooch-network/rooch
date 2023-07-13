
<a name="0x3_auth_validator_registry"></a>

# Module `0x3::auth_validator_registry`



-  [Struct `AuthValidator`](#0x3_auth_validator_registry_AuthValidator)
-  [Resource `AuthValidatorWithType`](#0x3_auth_validator_registry_AuthValidatorWithType)
-  [Resource `ValidatorRegistry`](#0x3_auth_validator_registry_ValidatorRegistry)
-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_auth_validator_registry_genesis_init)
-  [Function `register`](#0x3_auth_validator_registry_register)
-  [Function `register_internal`](#0x3_auth_validator_registry_register_internal)
-  [Function `borrow_validator`](#0x3_auth_validator_registry_borrow_validator)
-  [Function `borrow_validator_by_type`](#0x3_auth_validator_registry_borrow_validator_by_type)
-  [Function `validator_id`](#0x3_auth_validator_registry_validator_id)
-  [Function `validator_module_address`](#0x3_auth_validator_registry_validator_module_address)
-  [Function `validator_module_name`](#0x3_auth_validator_registry_validator_module_name)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x2::type_info</a>;
<b>use</b> <a href="">0x2::type_table</a>;
</code></pre>



<a name="0x3_auth_validator_registry_AuthValidator"></a>

## Struct `AuthValidator`



<pre><code><b>struct</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">AuthValidator</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>module_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>module_name: <a href="_String">ascii::String</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_auth_validator_registry_AuthValidatorWithType"></a>

## Resource `AuthValidatorWithType`



<pre><code><b>struct</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidatorWithType">AuthValidatorWithType</a>&lt;ValidatorType: store&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_auth_validator_registry_ValidatorRegistry"></a>

## Resource `ValidatorRegistry`



<pre><code><b>struct</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_ValidatorRegistry">ValidatorRegistry</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>validator_num: u64</code>
</dt>
<dd>
How many validators are registered
</dd>
<dt>
<code>validators: <a href="_Table">table::Table</a>&lt;u64, <a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">auth_validator_registry::AuthValidator</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>validators_with_type: <a href="_TypeTable">type_table::TypeTable</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_auth_validator_registry_E_VALIDATOR_UNREGISTERED"></a>



<pre><code><b>const</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_E_VALIDATOR_UNREGISTERED">E_VALIDATOR_UNREGISTERED</a>: u64 = 1;
</code></pre>



<a name="0x3_auth_validator_registry_genesis_init"></a>

## Function `genesis_init`

Init function called by genesis.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, sender: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_genesis_init">genesis_init</a>(ctx: &<b>mut</b> StorageContext, sender: &<a href="">signer</a>){
    <b>let</b> registry = <a href="auth_validator_registry.md#0x3_auth_validator_registry_ValidatorRegistry">ValidatorRegistry</a> {
        validator_num: 0,
        validators: <a href="_new">table::new</a>(<a href="_tx_context_mut">storage_context::tx_context_mut</a>(ctx)),
        validators_with_type: <a href="_new">type_table::new</a>(<a href="_tx_context_mut">storage_context::tx_context_mut</a>(ctx)),
    };
    <a href="_global_move_to">account_storage::global_move_to</a>(ctx, sender, registry);
}
</code></pre>



</details>

<a name="0x3_auth_validator_registry_register"></a>

## Function `register`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register">register</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register">register</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> StorageContext) : u64{
    <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_internal">register_internal</a>&lt;ValidatorType&gt;(ctx)
}
</code></pre>



</details>

<a name="0x3_auth_validator_registry_register_internal"></a>

## Function `register_internal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_internal">register_internal</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_internal">register_internal</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> StorageContext) : u64{
    <b>let</b> <a href="">type_info</a> = <a href="_type_of">type_info::type_of</a>&lt;ValidatorType&gt;();
    <b>let</b> module_address = <a href="_account_address">type_info::account_address</a>(&<a href="">type_info</a>);
    //TODO consider change <a href="_module_name">type_info::module_name</a> <b>to</b> <a href="_String">ascii::String</a>.
    <b>let</b> module_name = std::ascii::string(<a href="_module_name">type_info::module_name</a>(&<a href="">type_info</a>));

    <b>let</b> registry = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="auth_validator_registry.md#0x3_auth_validator_registry_ValidatorRegistry">ValidatorRegistry</a>&gt;(ctx, @rooch_framework);
    <b>let</b> id = registry.validator_num;

    <b>let</b> validator_with_type = <a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidatorWithType">AuthValidatorWithType</a>&lt;ValidatorType&gt;{
        id,
    };
    <a href="_add">type_table::add</a>(&<b>mut</b> registry.validators_with_type, validator_with_type);

    <b>let</b> validator = <a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">AuthValidator</a> {
        id,
        module_address: module_address,
        module_name: module_name,
    };
    <a href="_add">table::add</a>(&<b>mut</b> registry.validators, id, validator);

    registry.validator_num = registry.validator_num + 1;
    id
}
</code></pre>



</details>

<a name="0x3_auth_validator_registry_borrow_validator"></a>

## Function `borrow_validator`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator">borrow_validator</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, id: u64): &<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">auth_validator_registry::AuthValidator</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator">borrow_validator</a>(ctx: &StorageContext, id: u64): &<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">AuthValidator</a> {
    <b>let</b> registry = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="auth_validator_registry.md#0x3_auth_validator_registry_ValidatorRegistry">ValidatorRegistry</a>&gt;(ctx, @rooch_framework);
    <a href="_borrow">table::borrow</a>(&registry.validators, id)
}
</code></pre>



</details>

<a name="0x3_auth_validator_registry_borrow_validator_by_type"></a>

## Function `borrow_validator_by_type`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator_by_type">borrow_validator_by_type</a>&lt;ValidatorType: store&gt;(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>): &<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">auth_validator_registry::AuthValidator</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator_by_type">borrow_validator_by_type</a>&lt;ValidatorType: store&gt;(ctx: &StorageContext): &<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">AuthValidator</a> {
    <b>let</b> registry = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="auth_validator_registry.md#0x3_auth_validator_registry_ValidatorRegistry">ValidatorRegistry</a>&gt;(ctx, @rooch_framework);
    <b>assert</b>!(<a href="_contains">type_table::contains</a>&lt;<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidatorWithType">AuthValidatorWithType</a>&lt;ValidatorType&gt;&gt;(&registry.validators_with_type), <a href="_not_found">error::not_found</a>(<a href="auth_validator_registry.md#0x3_auth_validator_registry_E_VALIDATOR_UNREGISTERED">E_VALIDATOR_UNREGISTERED</a>));
    <b>let</b> validator_with_type = <a href="_borrow">type_table::borrow</a>&lt;<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidatorWithType">AuthValidatorWithType</a>&lt;ValidatorType&gt;&gt;(&registry.validators_with_type);
    <b>assert</b>!(<a href="_contains">table::contains</a>(&registry.validators, validator_with_type.id), <a href="_not_found">error::not_found</a>(<a href="auth_validator_registry.md#0x3_auth_validator_registry_E_VALIDATOR_UNREGISTERED">E_VALIDATOR_UNREGISTERED</a>));
    <a href="_borrow">table::borrow</a>(&registry.validators, validator_with_type.id)
}
</code></pre>



</details>

<a name="0x3_auth_validator_registry_validator_id"></a>

## Function `validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_validator_id">validator_id</a>(validator: &<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">auth_validator_registry::AuthValidator</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_validator_id">validator_id</a>(validator: &<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">AuthValidator</a>): u64 {
    validator.id
}
</code></pre>



</details>

<a name="0x3_auth_validator_registry_validator_module_address"></a>

## Function `validator_module_address`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_validator_module_address">validator_module_address</a>(validator: &<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">auth_validator_registry::AuthValidator</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_validator_module_address">validator_module_address</a>(validator: &<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">AuthValidator</a>): <b>address</b> {
    validator.module_address
}
</code></pre>



</details>

<a name="0x3_auth_validator_registry_validator_module_name"></a>

## Function `validator_module_name`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_validator_module_name">validator_module_name</a>(validator: &<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">auth_validator_registry::AuthValidator</a>): <a href="_String">ascii::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_validator_module_name">validator_module_name</a>(validator: &<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidator">AuthValidator</a>): std::ascii::String {
    validator.module_name
}
</code></pre>



</details>
