
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


<pre><code><b>use</b> <a href="../../moveos/moveos-stdlib/move-stdlib/doc/ascii.md#0x1_ascii">0x1::ascii</a>;
<b>use</b> <a href="../../moveos/moveos-stdlib/move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/account_storage.md#0x2_account_storage">0x2::account_storage</a>;
<b>use</b> <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/storage_context.md#0x2_storage_context">0x2::storage_context</a>;
<b>use</b> <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/table.md#0x2_table">0x2::table</a>;
<b>use</b> <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/tx_context.md#0x2_tx_context">0x2::tx_context</a>;
<b>use</b> <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_info.md#0x2_type_info">0x2::type_info</a>;
<b>use</b> <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_table.md#0x2_type_table">0x2::type_table</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
</code></pre>



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
<code>validators: <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/table.md#0x2_table_Table">table::Table</a>&lt;u64, <a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>validators_with_type: <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_table.md#0x2_type_table_TypeTable">type_table::TypeTable</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

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


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, sender: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_genesis_init">genesis_init</a>(ctx: &<b>mut</b> StorageContext, sender: &<a href="../../moveos/moveos-stdlib/move-stdlib/doc/signer.md#0x1_signer">signer</a>){
    <b>let</b> registry = <a href="auth_validator_registry.md#0x3_auth_validator_registry_ValidatorRegistry">ValidatorRegistry</a> {
        validator_num: 0,
        validators: <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/table.md#0x2_table_new">table::new</a>(<a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/storage_context.md#0x2_storage_context_tx_context_mut">storage_context::tx_context_mut</a>(ctx)),
        validators_with_type: <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_table.md#0x2_type_table_new">type_table::new</a>(<a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/storage_context.md#0x2_storage_context_tx_context_mut">storage_context::tx_context_mut</a>(ctx)),
    };
    <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/account_storage.md#0x2_account_storage_global_move_to">account_storage::global_move_to</a>(ctx, sender, registry);
}
</code></pre>



</details>

<a name="0x3_auth_validator_registry_register"></a>

## Function `register`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register">register</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): u64
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



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_internal">register_internal</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_internal">register_internal</a>&lt;ValidatorType: store&gt;(ctx: &<b>mut</b> StorageContext) : u64{
    <b>let</b> <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_info.md#0x2_type_info">type_info</a> = <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_info.md#0x2_type_info_type_of">type_info::type_of</a>&lt;ValidatorType&gt;();
    <b>let</b> module_address = <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_info.md#0x2_type_info_account_address">type_info::account_address</a>(&<a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_info.md#0x2_type_info">type_info</a>);
    //TODO consider change <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_info.md#0x2_type_info_module_name">type_info::module_name</a> <b>to</b> <a href="../../moveos/moveos-stdlib/move-stdlib/doc/ascii.md#0x1_ascii_String">ascii::String</a>.
    <b>let</b> module_name = std::ascii::string(<a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_info.md#0x2_type_info_module_name">type_info::module_name</a>(&<a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_info.md#0x2_type_info">type_info</a>));

    <b>let</b> registry = <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/account_storage.md#0x2_account_storage_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="auth_validator_registry.md#0x3_auth_validator_registry_ValidatorRegistry">ValidatorRegistry</a>&gt;(ctx, @rooch_framework);
    <b>let</b> id = registry.validator_num;

    <b>assert</b>!(!<a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_table.md#0x2_type_table_contains">type_table::contains</a>&lt;<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidatorWithType">AuthValidatorWithType</a>&lt;ValidatorType&gt;&gt;(&registry.validators_with_type), <a href="../../moveos/moveos-stdlib/move-stdlib/doc/error.md#0x1_error_already_exists">error::already_exists</a>(<a href="auth_validator_registry.md#0x3_auth_validator_registry_ErrorValidatorAlreadyRegistered">ErrorValidatorAlreadyRegistered</a>));

    <b>let</b> validator_with_type = <a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidatorWithType">AuthValidatorWithType</a>&lt;ValidatorType&gt;{
        id,
    };
    <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_table.md#0x2_type_table_add">type_table::add</a>(&<b>mut</b> registry.validators_with_type, validator_with_type);

    <b>let</b> validator = <a href="auth_validator.md#0x3_auth_validator_new_auth_validator">auth_validator::new_auth_validator</a>(
        id,
        module_address,
        module_name,
    );
    <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/table.md#0x2_table_add">table::add</a>(&<b>mut</b> registry.validators, id, validator);

    registry.validator_num = registry.validator_num + 1;
    id
}
</code></pre>



</details>

<a name="0x3_auth_validator_registry_borrow_validator"></a>

## Function `borrow_validator`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator">borrow_validator</a>(ctx: &<a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, id: u64): &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator">borrow_validator</a>(ctx: &StorageContext, id: u64): &AuthValidator {
    <b>let</b> registry = <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/account_storage.md#0x2_account_storage_global_borrow">account_storage::global_borrow</a>&lt;<a href="auth_validator_registry.md#0x3_auth_validator_registry_ValidatorRegistry">ValidatorRegistry</a>&gt;(ctx, @rooch_framework);
    <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/table.md#0x2_table_borrow">table::borrow</a>(&registry.validators, id)
}
</code></pre>



</details>

<a name="0x3_auth_validator_registry_borrow_validator_by_type"></a>

## Function `borrow_validator_by_type`



<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator_by_type">borrow_validator_by_type</a>&lt;ValidatorType: store&gt;(ctx: &<a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): &<a href="auth_validator.md#0x3_auth_validator_AuthValidator">auth_validator::AuthValidator</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator_by_type">borrow_validator_by_type</a>&lt;ValidatorType: store&gt;(ctx: &StorageContext): &AuthValidator {
    <b>let</b> registry = <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/account_storage.md#0x2_account_storage_global_borrow">account_storage::global_borrow</a>&lt;<a href="auth_validator_registry.md#0x3_auth_validator_registry_ValidatorRegistry">ValidatorRegistry</a>&gt;(ctx, @rooch_framework);
    <b>assert</b>!(<a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_table.md#0x2_type_table_contains">type_table::contains</a>&lt;<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidatorWithType">AuthValidatorWithType</a>&lt;ValidatorType&gt;&gt;(&registry.validators_with_type), <a href="../../moveos/moveos-stdlib/move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="auth_validator_registry.md#0x3_auth_validator_registry_ErrorValidatorUnregistered">ErrorValidatorUnregistered</a>));
    <b>let</b> validator_with_type = <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/type_table.md#0x2_type_table_borrow">type_table::borrow</a>&lt;<a href="auth_validator_registry.md#0x3_auth_validator_registry_AuthValidatorWithType">AuthValidatorWithType</a>&lt;ValidatorType&gt;&gt;(&registry.validators_with_type);
    <b>assert</b>!(<a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/table.md#0x2_table_contains">table::contains</a>(&registry.validators, validator_with_type.id), <a href="../../moveos/moveos-stdlib/move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="auth_validator_registry.md#0x3_auth_validator_registry_ErrorValidatorUnregistered">ErrorValidatorUnregistered</a>));
    <a href="../../moveos/moveos-stdlib/moveos-stdlib/doc/table.md#0x2_table_borrow">table::borrow</a>(&registry.validators, validator_with_type.id)
}
</code></pre>



</details>
