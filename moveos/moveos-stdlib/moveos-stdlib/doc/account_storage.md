
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


<pre><code><b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="context.md#0x2_context">0x2::context</a>;
<b>use</b> <a href="move_module.md#0x2_move_module">0x2::move_module</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="table.md#0x2_table">0x2::table</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
<b>use</b> <a href="type_table.md#0x2_type_table">0x2::type_table</a>;
</code></pre>



<a name="0x2_account_storage_AccountStorage"></a>

## Resource `AccountStorage`



<pre><code><b>struct</b> <a href="account_storage.md#0x2_account_storage_AccountStorage">AccountStorage</a> <b>has</b> key
</code></pre>



<a name="0x2_account_storage_ModuleUpgradeFlag"></a>

## Struct `ModuleUpgradeFlag`



<pre><code><b>struct</b> <a href="account_storage.md#0x2_account_storage_ModuleUpgradeFlag">ModuleUpgradeFlag</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_account_storage_ErrorAccountAlreadyExists"></a>

The account with the given address already exists


<pre><code><b>const</b> <a href="account_storage.md#0x2_account_storage_ErrorAccountAlreadyExists">ErrorAccountAlreadyExists</a>: u64 = 1;
</code></pre>



<a name="0x2_account_storage_ErrorResourceAlreadyExists"></a>

The resource with the given type already exists


<pre><code><b>const</b> <a href="account_storage.md#0x2_account_storage_ErrorResourceAlreadyExists">ErrorResourceAlreadyExists</a>: u64 = 2;
</code></pre>



<a name="0x2_account_storage_ErrorResourceNotExists"></a>

The resource with the given type not exists


<pre><code><b>const</b> <a href="account_storage.md#0x2_account_storage_ErrorResourceNotExists">ErrorResourceNotExists</a>: u64 = 3;
</code></pre>



<a name="0x2_account_storage_NamedTableModule"></a>



<pre><code><b>const</b> <a href="account_storage.md#0x2_account_storage_NamedTableModule">NamedTableModule</a>: u64 = 1;
</code></pre>



<a name="0x2_account_storage_NamedTableResource"></a>



<pre><code><b>const</b> <a href="account_storage.md#0x2_account_storage_NamedTableResource">NamedTableResource</a>: u64 = 0;
</code></pre>



<a name="0x2_account_storage_named_table_id"></a>

## Function `named_table_id`



<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_named_table_id">named_table_id</a>(account: <b>address</b>, table_type: u64): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_account_storage_create_account_storage"></a>

## Function `create_account_storage`

Create a new account storage space


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_create_account_storage">create_account_storage</a>(ctx: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, account: <b>address</b>)
</code></pre>



<a name="0x2_account_storage_exist_account_storage"></a>

## Function `exist_account_storage`

check if account storage eixst


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_exist_account_storage">exist_account_storage</a>(ctx: &<a href="context.md#0x2_context_Context">context::Context</a>, account: <b>address</b>): bool
</code></pre>



<a name="0x2_account_storage_ensure_account_storage"></a>

## Function `ensure_account_storage`



<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_ensure_account_storage">ensure_account_storage</a>(ctx: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, account: <b>address</b>)
</code></pre>



<a name="0x2_account_storage_global_borrow"></a>

## Function `global_borrow`

Borrow a resource from the account's storage
This function equates to <code><b>borrow_global</b>&lt;T&gt;(<b>address</b>)</code> instruction in Move


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_borrow">global_borrow</a>&lt;T: key&gt;(ctx: &<a href="context.md#0x2_context_Context">context::Context</a>, account: <b>address</b>): &T
</code></pre>



<a name="0x2_account_storage_global_borrow_mut"></a>

## Function `global_borrow_mut`

Borrow a mut resource from the account's storage
This function equates to <code><b>borrow_global_mut</b>&lt;T&gt;(<b>address</b>)</code> instruction in Move


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_borrow_mut">global_borrow_mut</a>&lt;T: key&gt;(ctx: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, account: <b>address</b>): &<b>mut</b> T
</code></pre>



<a name="0x2_account_storage_global_move_to"></a>

## Function `global_move_to`

Move a resource to the account's storage
This function equates to <code><b>move_to</b>&lt;T&gt;(&<a href="">signer</a>, resource)</code> instruction in Move


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_move_to">global_move_to</a>&lt;T: key&gt;(ctx: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, account: &<a href="">signer</a>, resource: T)
</code></pre>



<a name="0x2_account_storage_global_move_from"></a>

## Function `global_move_from`

Move a resource from the account's storage
This function equates to <code><b>move_from</b>&lt;T&gt;(<b>address</b>)</code> instruction in Move


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_move_from">global_move_from</a>&lt;T: key&gt;(ctx: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, account: <b>address</b>): T
</code></pre>



<a name="0x2_account_storage_global_exists"></a>

## Function `global_exists`

Check if the account has a resource of the given type
This function equates to <code><b>exists</b>&lt;T&gt;(<b>address</b>)</code> instruction in Move


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_global_exists">global_exists</a>&lt;T: key&gt;(ctx: &<a href="context.md#0x2_context_Context">context::Context</a>, account: <b>address</b>): bool
</code></pre>



<a name="0x2_account_storage_exists_module"></a>

## Function `exists_module`

Check if the account has a module with the given name


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_exists_module">exists_module</a>(ctx: &<a href="context.md#0x2_context_Context">context::Context</a>, account: <b>address</b>, name: <a href="_String">string::String</a>): bool
</code></pre>



<a name="0x2_account_storage_publish_modules"></a>

## Function `publish_modules`

Publish modules to the account's storage


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_publish_modules">publish_modules</a>(ctx: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, account: &<a href="">signer</a>, modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;)
</code></pre>



<a name="0x2_account_storage_publish_modules_entry"></a>

## Function `publish_modules_entry`

Entry function to publish modules
The order of modules must be sorted by dependency order.


<pre><code><b>public</b> entry <b>fun</b> <a href="account_storage.md#0x2_account_storage_publish_modules_entry">publish_modules_entry</a>(ctx: &<b>mut</b> <a href="context.md#0x2_context_Context">context::Context</a>, account: &<a href="">signer</a>, modules: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;)
</code></pre>
