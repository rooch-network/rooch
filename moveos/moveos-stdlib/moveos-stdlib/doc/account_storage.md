
<a name="0x2_account_storage"></a>

# Module `0x2::account_storage`

AccountStorage is part of the StorageAbstraction
It is used to store the account's resources and modules


-  [Resource `AccountStorage`](#0x2_account_storage_AccountStorage)
-  [Constants](#@Constants_0)
-  [Function `named_table_id`](#0x2_account_storage_named_table_id)
-  [Function `create_account_storage`](#0x2_account_storage_create_account_storage)
-  [Function `borrow_resource`](#0x2_account_storage_borrow_resource)
-  [Function `borrow_mut_resource`](#0x2_account_storage_borrow_mut_resource)
-  [Function `move_resource_to`](#0x2_account_storage_move_resource_to)
-  [Function `move_resource_from`](#0x2_account_storage_move_resource_from)
-  [Function `exists_resource`](#0x2_account_storage_exists_resource)
-  [Function `transfer`](#0x2_account_storage_transfer)
-  [Function `exists_module`](#0x2_account_storage_exists_module)
-  [Function `publish_modules`](#0x2_account_storage_publish_modules)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
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



<a name="@Constants_0"></a>

## Constants


<a name="0x2_account_storage_ErrorResourceAlreadyExists"></a>

The resource with the given type already exists


<pre><code><b>const</b> <a href="account_storage.md#0x2_account_storage_ErrorResourceAlreadyExists">ErrorResourceAlreadyExists</a>: u64 = 1;
</code></pre>



<a name="0x2_account_storage_ErrorResourceNotExists"></a>

The resource with the given type not exists


<pre><code><b>const</b> <a href="account_storage.md#0x2_account_storage_ErrorResourceNotExists">ErrorResourceNotExists</a>: u64 = 2;
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


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account_storage.md#0x2_account_storage_create_account_storage">create_account_storage</a>(account: <b>address</b>): <a href="account_storage.md#0x2_account_storage_AccountStorage">account_storage::AccountStorage</a>
</code></pre>



<a name="0x2_account_storage_borrow_resource"></a>

## Function `borrow_resource`



<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_borrow_resource">borrow_resource</a>&lt;T: key&gt;(self: &<a href="account_storage.md#0x2_account_storage_AccountStorage">account_storage::AccountStorage</a>): &T
</code></pre>



<a name="0x2_account_storage_borrow_mut_resource"></a>

## Function `borrow_mut_resource`



<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_borrow_mut_resource">borrow_mut_resource</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="account_storage.md#0x2_account_storage_AccountStorage">account_storage::AccountStorage</a>): &<b>mut</b> T
</code></pre>



<a name="0x2_account_storage_move_resource_to"></a>

## Function `move_resource_to`



<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_move_resource_to">move_resource_to</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="account_storage.md#0x2_account_storage_AccountStorage">account_storage::AccountStorage</a>, resource: T)
</code></pre>



<a name="0x2_account_storage_move_resource_from"></a>

## Function `move_resource_from`



<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_move_resource_from">move_resource_from</a>&lt;T: key&gt;(self: &<b>mut</b> <a href="account_storage.md#0x2_account_storage_AccountStorage">account_storage::AccountStorage</a>): T
</code></pre>



<a name="0x2_account_storage_exists_resource"></a>

## Function `exists_resource`



<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_exists_resource">exists_resource</a>&lt;T: key&gt;(self: &<a href="account_storage.md#0x2_account_storage_AccountStorage">account_storage::AccountStorage</a>): bool
</code></pre>



<a name="0x2_account_storage_transfer"></a>

## Function `transfer`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account_storage.md#0x2_account_storage_transfer">transfer</a>(obj: <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="account_storage.md#0x2_account_storage_AccountStorage">account_storage::AccountStorage</a>&gt;, account: <b>address</b>)
</code></pre>



<a name="0x2_account_storage_exists_module"></a>

## Function `exists_module`

Check if the account has a module with the given name


<pre><code><b>public</b> <b>fun</b> <a href="account_storage.md#0x2_account_storage_exists_module">exists_module</a>(self: &<a href="account_storage.md#0x2_account_storage_AccountStorage">account_storage::AccountStorage</a>, name: <a href="_String">string::String</a>): bool
</code></pre>



<a name="0x2_account_storage_publish_modules"></a>

## Function `publish_modules`

Publish modules to the account's storage
Return true if the modules are upgraded


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account_storage.md#0x2_account_storage_publish_modules">publish_modules</a>(self: &<b>mut</b> <a href="account_storage.md#0x2_account_storage_AccountStorage">account_storage::AccountStorage</a>, account_address: <b>address</b>, modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;): bool
</code></pre>
