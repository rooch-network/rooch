
<a name="0x2_account_storage"></a>

# Module `0x2::account_storage`

AccountStorage is part of the StorageAbstraction
It is used to store the account's resources and modules


-  [Resource `AccountStorage`](#0x2_account_storage_AccountStorage)
-  [Struct `ModuleUpgradeFlag`](#0x2_account_storage_ModuleUpgradeFlag)
-  [Constants](#@Constants_0)
-  [Function `named_table_id`](#0x2_account_storage_named_table_id)
-  [Function `create_account_storage`](#0x2_account_storage_create_account_storage)
-  [Function `exist_account_storage`](#0x2_account_storage_exist_account_storage)
-  [Function `ensure_account_storage`](#0x2_account_storage_ensure_account_storage)
-  [Function `global_borrow`](#0x2_account_storage_global_borrow)
-  [Function `global_borrow_mut`](#0x2_account_storage_global_borrow_mut)
-  [Function `global_move_to`](#0x2_account_storage_global_move_to)
-  [Function `global_move_from`](#0x2_account_storage_global_move_from)
-  [Function `global_exists`](#0x2_account_storage_global_exists)
-  [Function `exists_module`](#0x2_account_storage_exists_module)
-  [Function `publish_modules`](#0x2_account_storage_publish_modules)
-  [Function `publish_modules_entry`](#0x2_account_storage_publish_modules_entry)


<pre><code><b>use</b> <a href="../doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="move_module.md#0x2_move_module">0x2::move_module</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
<b>use</b> <a href="object_storage.md#0x2_object_storage">0x2::object_storage</a>;
<b>use</b> <a href="storage_context.md#0x2_storage_context">0x2::storage_context</a>;
<b>use</b> <a href="table.md#0x2_table">0x2::table</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
<b>use</b> <a href="type_table.md#0x2_type_table">0x2::type_table</a>;
</code></pre>



<a name="0x2_account_storage_AccountStorage"></a>

## Resource `AccountStorage`



<pre><code><b>struct</b> <a href="account_storage.md#0x2_account_storage_AccountStorage">AccountStorage</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>resources: <a href="type_table.md#0x2_type_table_TypeTable">type_table::TypeTable</a></code>
</dt>
<dd>

</dd>
<dt>
<code>modules: <a href="table.md#0x2_table_Table">table::Table</a>&lt;<a href="_String">string::String</a>, <a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x2_account_storage_ModuleUpgradeFlag"></a>

## Struct `ModuleUpgradeFlag`



<pre><code><b>struct</b> <a href="account_storage.md#0x2_account_storage_ModuleUpgradeFlag">ModuleUpgradeFlag</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>is_upgrade: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x2_account_storage_EAccountAlreadyExists"></a>

The account with the given address already exists


<pre><code><b>const</b> <a href="account_storage.md#0x2_account_storage_EAccountAlreadyExists">EAccountAlreadyExists</a>: u64 = 0;
</code></pre>



<a name="0x2_account_storage_EResourceAlreadyExists"></a>

The resource with the given type already exists


<pre><code><b>const</b> <a href="account_storage.md#0x2_account_storage_EResourceAlreadyExists">EResourceAlreadyExists</a>: u64 = 1;
</code></pre>



<a name="0x2_account_storage_EResourceNotExists"></a>

The resource with the given type not exists


<pre><code><b>const</b> <a href="account_storage.md#0x2_account_storage_EResourceNotExists">EResourceNotExists</a>: u64 = 2;
</code></pre>



<a name="0x2_account_storage_NamedTableModule"></a>



<pre><code><b>const</b> <a href="account_storage.md#0x2_account_storage_NamedTableModule">NamedTableModule</a>: u64 = 1;
</code></pre>



<a name="0x2_account_storage_NamedTableResource"></a>



<pre><code><b>const</b> <a href="account_storage.md#0x2_account_storage_NamedTableResource">NamedTableResource</a>: u64 = 0;
</code></pre>



<a name="0x2_account_storage_named_table_id"></a>

## Function `named_table_id`



<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_named_table_id">named_table_id</a>(account: <b>address</b>, table_type: u64): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_named_table_id">named_table_id</a>(account: <b>address</b>, table_type: u64): ObjectID{
    <a href="object_id.md#0x2_object_id_address_to_object_id">object_id::address_to_object_id</a>(<a href="tx_context.md#0x2_tx_context_derive_id">tx_context::derive_id</a>(<a href="../doc/bcs.md#0x1_bcs_to_bytes">bcs::to_bytes</a>(&account), table_type))
}
</code></pre>



</details>

<a name="0x2_account_storage_create_account_storage"></a>

## Function `create_account_storage`

Create a new account storage space


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_create_account_storage">create_account_storage</a>(ctx: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, account: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_create_account_storage">create_account_storage</a>(ctx: &<b>mut</b> StorageContext, account: <b>address</b>) {
    <b>let</b> <a href="object_id.md#0x2_object_id">object_id</a> = <a href="object_id.md#0x2_object_id_address_to_object_id">object_id::address_to_object_id</a>(account);
    <b>let</b> <a href="account_storage.md#0x2_account_storage">account_storage</a> = <a href="account_storage.md#0x2_account_storage_AccountStorage">AccountStorage</a> {
        resources: <a href="type_table.md#0x2_type_table_new_with_id">type_table::new_with_id</a>(<a href="account_storage.md#0x2_account_storage_named_table_id">named_table_id</a>(account, <a href="account_storage.md#0x2_account_storage_NamedTableResource">NamedTableResource</a>)),
        modules: <a href="table.md#0x2_table_new_with_id">table::new_with_id</a>(<a href="account_storage.md#0x2_account_storage_named_table_id">named_table_id</a>(account, <a href="account_storage.md#0x2_account_storage_NamedTableModule">NamedTableModule</a>)),
    };
    <b>let</b> <a href="object_storage.md#0x2_object_storage">object_storage</a> = <a href="storage_context.md#0x2_storage_context_object_storage_mut">storage_context::object_storage_mut</a>(ctx);
    <b>assert</b>!(!<a href="object_storage.md#0x2_object_storage_contains">object_storage::contains</a>(<a href="object_storage.md#0x2_object_storage">object_storage</a>, <a href="object_id.md#0x2_object_id">object_id</a>), <a href="account_storage.md#0x2_account_storage_EAccountAlreadyExists">EAccountAlreadyExists</a>);
    <b>let</b> <a href="object.md#0x2_object">object</a> = <a href="object.md#0x2_object_new_with_id">object::new_with_id</a>(<a href="object_id.md#0x2_object_id">object_id</a>, account, <a href="account_storage.md#0x2_account_storage">account_storage</a>);
    <a href="object_storage.md#0x2_object_storage_add">object_storage::add</a>(<a href="object_storage.md#0x2_object_storage">object_storage</a>, <a href="object.md#0x2_object">object</a>);
}
</code></pre>



</details>

<a name="0x2_account_storage_exist_account_storage"></a>

## Function `exist_account_storage`

check if account storage eixst


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_exist_account_storage">exist_account_storage</a>(ctx: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, account: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_exist_account_storage">exist_account_storage</a>(ctx: &StorageContext, account: <b>address</b>): bool {
    <b>let</b> <a href="object_id.md#0x2_object_id">object_id</a> = <a href="object_id.md#0x2_object_id_address_to_object_id">object_id::address_to_object_id</a>(account);
    <b>let</b> <a href="object_storage.md#0x2_object_storage">object_storage</a> = <a href="storage_context.md#0x2_storage_context_object_storage">storage_context::object_storage</a>(ctx);
    <a href="object_storage.md#0x2_object_storage_contains">object_storage::contains</a>(<a href="object_storage.md#0x2_object_storage">object_storage</a>, <a href="object_id.md#0x2_object_id">object_id</a>)
}
</code></pre>



</details>

<a name="0x2_account_storage_ensure_account_storage"></a>

## Function `ensure_account_storage`



<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_ensure_account_storage">ensure_account_storage</a>(ctx: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, account: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_ensure_account_storage">ensure_account_storage</a>(ctx: &<b>mut</b> StorageContext, account: <b>address</b>) {
    <b>if</b> (!<a href="account_storage.md#0x2_account_storage_exist_account_storage">exist_account_storage</a>(ctx, account)) {
        <a href="account_storage.md#0x2_account_storage_create_account_storage">create_account_storage</a>(ctx, account);
    }
}
</code></pre>



</details>

<a name="0x2_account_storage_global_borrow"></a>

## Function `global_borrow`

Borrow a resource from the account's storage
This function equates to <code><b>borrow_global</b>&lt;T&gt;(<b>address</b>)</code> instruction in Move


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_borrow">global_borrow</a>&lt;T: key&gt;(ctx: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, account: <b>address</b>): &T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_borrow">global_borrow</a>&lt;T: key&gt;(ctx: &StorageContext, account: <b>address</b>): &T {
    <b>let</b> <a href="object_storage.md#0x2_object_storage">object_storage</a> = <a href="storage_context.md#0x2_storage_context_object_storage">storage_context::object_storage</a>(ctx);
    <b>let</b> <a href="account_storage.md#0x2_account_storage">account_storage</a> = <a href="account_storage.md#0x2_account_storage_borrow_account_storage">borrow_account_storage</a>(<a href="object_storage.md#0x2_object_storage">object_storage</a>, account);
    <a href="account_storage.md#0x2_account_storage_borrow_resource_from_account_storage">borrow_resource_from_account_storage</a>&lt;T&gt;(<a href="account_storage.md#0x2_account_storage">account_storage</a>)
}
</code></pre>



</details>

<a name="0x2_account_storage_global_borrow_mut"></a>

## Function `global_borrow_mut`

Borrow a mut resource from the account's storage
This function equates to <code><b>borrow_global_mut</b>&lt;T&gt;(<b>address</b>)</code> instruction in Move


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_borrow_mut">global_borrow_mut</a>&lt;T: key&gt;(ctx: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, account: <b>address</b>): &<b>mut</b> T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_borrow_mut">global_borrow_mut</a>&lt;T: key&gt;(ctx: &<b>mut</b> StorageContext, account: <b>address</b>): &<b>mut</b> T {
    <b>let</b> <a href="object_storage.md#0x2_object_storage">object_storage</a> = <a href="storage_context.md#0x2_storage_context_object_storage_mut">storage_context::object_storage_mut</a>(ctx);
    <b>let</b> <a href="account_storage.md#0x2_account_storage">account_storage</a> = <a href="account_storage.md#0x2_account_storage_borrow_account_storage_mut">borrow_account_storage_mut</a>(<a href="object_storage.md#0x2_object_storage">object_storage</a>, account);
    <a href="account_storage.md#0x2_account_storage_borrow_mut_resource_from_account_storage">borrow_mut_resource_from_account_storage</a>&lt;T&gt;(<a href="account_storage.md#0x2_account_storage">account_storage</a>)
}
</code></pre>



</details>

<a name="0x2_account_storage_global_move_to"></a>

## Function `global_move_to`

Move a resource to the account's storage
This function equates to <code><b>move_to</b>&lt;T&gt;(&<a href="signer.md#0x2_signer">signer</a>, resource)</code> instruction in Move


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_move_to">global_move_to</a>&lt;T: key&gt;(ctx: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, account: &<a href="signer.md#0x2_signer">signer</a>, resource: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_move_to">global_move_to</a>&lt;T: key&gt;(ctx: &<b>mut</b> StorageContext, account: &<a href="signer.md#0x2_signer">signer</a>, resource: T){
    <b>let</b> account_address = signer::address_of(account);
    //Auto create the account storage when <b>move</b> resource <b>to</b> the account
    <a href="account_storage.md#0x2_account_storage_ensure_account_storage">ensure_account_storage</a>(ctx, account_address);
    <b>let</b> <a href="account_storage.md#0x2_account_storage">account_storage</a> = <a href="account_storage.md#0x2_account_storage_borrow_account_storage_mut">borrow_account_storage_mut</a>(<a href="storage_context.md#0x2_storage_context_object_storage_mut">storage_context::object_storage_mut</a>(ctx), account_address);
    <a href="account_storage.md#0x2_account_storage_add_resource_to_account_storage">add_resource_to_account_storage</a>(<a href="account_storage.md#0x2_account_storage">account_storage</a>, resource);
}
</code></pre>



</details>

<a name="0x2_account_storage_global_move_from"></a>

## Function `global_move_from`

Move a resource from the account's storage
This function equates to <code><b>move_from</b>&lt;T&gt;(<b>address</b>)</code> instruction in Move


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_move_from">global_move_from</a>&lt;T: key&gt;(ctx: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, account: <b>address</b>): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_move_from">global_move_from</a>&lt;T: key&gt;(ctx: &<b>mut</b> StorageContext, account: <b>address</b>): T {
    <b>let</b> <a href="account_storage.md#0x2_account_storage">account_storage</a> = <a href="account_storage.md#0x2_account_storage_borrow_account_storage_mut">borrow_account_storage_mut</a>(<a href="storage_context.md#0x2_storage_context_object_storage_mut">storage_context::object_storage_mut</a>(ctx), account);
    <a href="account_storage.md#0x2_account_storage_remove_resource_from_account_storage">remove_resource_from_account_storage</a>&lt;T&gt;(<a href="account_storage.md#0x2_account_storage">account_storage</a>)
}
</code></pre>



</details>

<a name="0x2_account_storage_global_exists"></a>

## Function `global_exists`

Check if the account has a resource of the given type
This function equates to <code><b>exists</b>&lt;T&gt;(<b>address</b>)</code> instruction in Move


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_exists">global_exists</a>&lt;T: key&gt;(ctx: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, account: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_exists">global_exists</a>&lt;T: key&gt;(ctx: &StorageContext, account: <b>address</b>) : bool {
    <b>if</b> (<a href="account_storage.md#0x2_account_storage_exist_account_storage">exist_account_storage</a>(ctx, account)) {
        <b>let</b> <a href="account_storage.md#0x2_account_storage">account_storage</a> = <a href="account_storage.md#0x2_account_storage_borrow_account_storage">borrow_account_storage</a>(<a href="storage_context.md#0x2_storage_context_object_storage">storage_context::object_storage</a>(ctx), account);
        <a href="account_storage.md#0x2_account_storage_exists_resource_at_account_storage">exists_resource_at_account_storage</a>&lt;T&gt;(<a href="account_storage.md#0x2_account_storage">account_storage</a>)
    }<b>else</b>{
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x2_account_storage_exists_module"></a>

## Function `exists_module`

Check if the account has a module with the given name


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_exists_module">exists_module</a>(ctx: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, account: <b>address</b>, name: <a href="_String">string::String</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_exists_module">exists_module</a>(ctx: &StorageContext, account: <b>address</b>, name: String): bool {
    <b>let</b> <a href="account_storage.md#0x2_account_storage">account_storage</a> = <a href="account_storage.md#0x2_account_storage_borrow_account_storage">borrow_account_storage</a>(<a href="storage_context.md#0x2_storage_context_object_storage">storage_context::object_storage</a>(ctx), account);
    <a href="account_storage.md#0x2_account_storage_exists_module_at_account_storage">exists_module_at_account_storage</a>(<a href="account_storage.md#0x2_account_storage">account_storage</a>, name)
}
</code></pre>



</details>

<a name="0x2_account_storage_publish_modules"></a>

## Function `publish_modules`

Publish modules to the account's storage


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_publish_modules">publish_modules</a>(ctx: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, account: &<a href="signer.md#0x2_signer">signer</a>, modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_publish_modules">publish_modules</a>(ctx: &<b>mut</b> StorageContext, account: &<a href="signer.md#0x2_signer">signer</a>, modules: <a href="">vector</a>&lt;MoveModule&gt;) {
    <b>let</b> account_address = signer::address_of(account);
    <b>let</b> <a href="account_storage.md#0x2_account_storage">account_storage</a> = <a href="account_storage.md#0x2_account_storage_borrow_account_storage_mut">borrow_account_storage_mut</a>(<a href="storage_context.md#0x2_storage_context_object_storage_mut">storage_context::object_storage_mut</a>(ctx), account_address);
    <b>let</b> i = 0;
    <b>let</b> len = <a href="_length">vector::length</a>(&modules);
    <b>let</b> (module_names, module_names_with_init_fn) = <a href="move_module.md#0x2_move_module_verify_modules">move_module::verify_modules</a>(&modules, account_address);

    <b>let</b> upgrade_flag = <b>false</b>;
    <b>while</b> (i &lt; len) {
        <b>let</b> name = <a href="_pop_back">vector::pop_back</a>(&<b>mut</b> module_names);
        <b>let</b> m = <a href="_pop_back">vector::pop_back</a>(&<b>mut</b> modules);

        // The <b>module</b> already <b>exists</b>, which means we are upgrading the <b>module</b>
        <b>if</b> (<a href="table.md#0x2_table_contains">table::contains</a>(&<a href="account_storage.md#0x2_account_storage">account_storage</a>.modules, name)) {
            <b>let</b> old_m = <a href="table.md#0x2_table_remove">table::remove</a>(&<b>mut</b> <a href="account_storage.md#0x2_account_storage">account_storage</a>.modules, name);
            <a href="move_module.md#0x2_move_module_check_comatibility">move_module::check_comatibility</a>(&m, &old_m);
            upgrade_flag = <b>true</b>;
        } <b>else</b> {
            // request init function invoking
            <a href="move_module.md#0x2_move_module_request_init_functions">move_module::request_init_functions</a>(module_names_with_init_fn, account_address);
        };
        <a href="table.md#0x2_table_add">table::add</a>(&<b>mut</b> <a href="account_storage.md#0x2_account_storage">account_storage</a>.modules, name, m);
        i = i + 1;
    };

    // Store <a href="account_storage.md#0x2_account_storage_ModuleUpgradeFlag">ModuleUpgradeFlag</a> in <a href="tx_context.md#0x2_tx_context">tx_context</a> which will be fetched in VM in Rust,
    // and then announce <b>to</b> the VM that the code loading cache should be considered outdated.
    <b>let</b> tx_ctx = <a href="storage_context.md#0x2_storage_context_tx_context_mut">storage_context::tx_context_mut</a>(ctx);
    <b>if</b> (!<a href="tx_context.md#0x2_tx_context_contains">tx_context::contains</a>&lt;<a href="account_storage.md#0x2_account_storage_ModuleUpgradeFlag">ModuleUpgradeFlag</a>&gt;(tx_ctx)) {
        <a href="tx_context.md#0x2_tx_context_add">tx_context::add</a>(tx_ctx, <a href="account_storage.md#0x2_account_storage_ModuleUpgradeFlag">ModuleUpgradeFlag</a> { is_upgrade: upgrade_flag });
    }
}
</code></pre>



</details>

<a name="0x2_account_storage_publish_modules_entry"></a>

## Function `publish_modules_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="account_storage.md#0x2_account_storage_publish_modules_entry">publish_modules_entry</a>(ctx: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, account: &<a href="signer.md#0x2_signer">signer</a>, modules: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account_storage.md#0x2_account_storage_publish_modules_entry">publish_modules_entry</a>(ctx: &<b>mut</b> StorageContext, account: &<a href="signer.md#0x2_signer">signer</a>, modules: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;) {
    <b>let</b> n_modules = <a href="_length">vector::length</a>(&modules);
    <b>let</b> i = 0;
    <b>let</b> module_vec = <a href="_empty">vector::empty</a>&lt;MoveModule&gt;();
    <b>while</b> (i &lt; n_modules) {
        <b>let</b> code_bytes = <a href="_pop_back">vector::pop_back</a>(&<b>mut</b> modules);
        <b>let</b> m = <a href="move_module.md#0x2_move_module_new">move_module::new</a>(code_bytes);
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> module_vec, m);
        i = i + 1;
    };
    <a href="account_storage.md#0x2_account_storage_publish_modules">publish_modules</a>(ctx, account, module_vec);
}
</code></pre>



</details>
