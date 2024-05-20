
<a name="0x2_module_store"></a>

# Module `0x2::module_store`

<code><a href="module_store.md#0x2_module_store">module_store</a></code> provide object to manage packages and modules.


-  [Resource `Allowlist`](#0x2_module_store_Allowlist)
-  [Resource `ModuleStore`](#0x2_module_store_ModuleStore)
-  [Constants](#@Constants_0)
-  [Function `module_store_id`](#0x2_module_store_module_store_id)
-  [Function `init_module_store`](#0x2_module_store_init_module_store)
-  [Function `borrow_module_store`](#0x2_module_store_borrow_module_store)
-  [Function `borrow_mut_module_store`](#0x2_module_store_borrow_mut_module_store)
-  [Function `exists_module`](#0x2_module_store_exists_module)
-  [Function `exists_module_id`](#0x2_module_store_exists_module_id)
-  [Function `publish_modules`](#0x2_module_store_publish_modules)
-  [Function `publish_modules_entry`](#0x2_module_store_publish_modules_entry)
-  [Function `publish_modules_internal`](#0x2_module_store_publish_modules_internal)
-  [Function `borrow_allowlist`](#0x2_module_store_borrow_allowlist)
-  [Function `borrow_mut_allowlist`](#0x2_module_store_borrow_mut_allowlist)
-  [Function `add_to_allowlist`](#0x2_module_store_add_to_allowlist)
-  [Function `remove_from_allowlist`](#0x2_module_store_remove_from_allowlist)
-  [Function `is_in_allowlist`](#0x2_module_store_is_in_allowlist)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="core_addresses.md#0x2_core_addresses">0x2::core_addresses</a>;
<b>use</b> <a href="features.md#0x2_features">0x2::features</a>;
<b>use</b> <a href="move_module.md#0x2_move_module">0x2::move_module</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="signer.md#0x2_signer">0x2::signer</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_module_store_Allowlist"></a>

## Resource `Allowlist`

Allowlist for module function invocation


<pre><code><b>struct</b> <a href="module_store.md#0x2_module_store_Allowlist">Allowlist</a> <b>has</b> store, key
</code></pre>



<a name="0x2_module_store_ModuleStore"></a>

## Resource `ModuleStore`

It is used to store the modules


<pre><code><b>struct</b> <a href="module_store.md#0x2_module_store_ModuleStore">ModuleStore</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_module_store_ErrorNotAllowToPublish"></a>

Not allow to publish module


<pre><code><b>const</b> <a href="module_store.md#0x2_module_store_ErrorNotAllowToPublish">ErrorNotAllowToPublish</a>: u64 = 1;
</code></pre>



<a name="0x2_module_store_module_store_id"></a>

## Function `module_store_id`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_module_store_id">module_store_id</a>(): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_module_store_init_module_store"></a>

## Function `init_module_store`

Create a new module object space


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="module_store.md#0x2_module_store_init_module_store">init_module_store</a>()
</code></pre>



<a name="0x2_module_store_borrow_module_store"></a>

## Function `borrow_module_store`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_borrow_module_store">borrow_module_store</a>(): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="module_store.md#0x2_module_store_ModuleStore">module_store::ModuleStore</a>&gt;
</code></pre>



<a name="0x2_module_store_borrow_mut_module_store"></a>

## Function `borrow_mut_module_store`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_borrow_mut_module_store">borrow_mut_module_store</a>(): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="module_store.md#0x2_module_store_ModuleStore">module_store::ModuleStore</a>&gt;
</code></pre>



<a name="0x2_module_store_exists_module"></a>

## Function `exists_module`

Check if the module object has a module with the given name


<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_exists_module">exists_module</a>(module_object: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="module_store.md#0x2_module_store_ModuleStore">module_store::ModuleStore</a>&gt;, <a href="account.md#0x2_account">account</a>: <b>address</b>, name: <a href="_String">string::String</a>): bool
</code></pre>



<a name="0x2_module_store_exists_module_id"></a>

## Function `exists_module_id`

Check if the module object has a module with the given id


<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_exists_module_id">exists_module_id</a>(module_object: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="module_store.md#0x2_module_store_ModuleStore">module_store::ModuleStore</a>&gt;, module_id: <a href="_String">string::String</a>): bool
</code></pre>



<a name="0x2_module_store_publish_modules"></a>

## Function `publish_modules`

Publish modules to the account's storage


<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_publish_modules">publish_modules</a>(<a href="module_store.md#0x2_module_store">module_store</a>: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="module_store.md#0x2_module_store_ModuleStore">module_store::ModuleStore</a>&gt;, <a href="account.md#0x2_account">account</a>: &<a href="">signer</a>, modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;)
</code></pre>



<a name="0x2_module_store_publish_modules_entry"></a>

## Function `publish_modules_entry`

Entry function to publish modules
The order of modules must be sorted by dependency order.


<pre><code><b>public</b> entry <b>fun</b> <a href="module_store.md#0x2_module_store_publish_modules_entry">publish_modules_entry</a>(<a href="account.md#0x2_account">account</a>: &<a href="">signer</a>, modules: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;)
</code></pre>



<a name="0x2_module_store_publish_modules_internal"></a>

## Function `publish_modules_internal`

Publish modules to the module object's storage
Return true if the modules are upgraded


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="module_store.md#0x2_module_store_publish_modules_internal">publish_modules_internal</a>(module_object: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="module_store.md#0x2_module_store_ModuleStore">module_store::ModuleStore</a>&gt;, account_address: <b>address</b>, modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;): bool
</code></pre>



<a name="0x2_module_store_borrow_allowlist"></a>

## Function `borrow_allowlist`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_borrow_allowlist">borrow_allowlist</a>(): &<a href="module_store.md#0x2_module_store_Allowlist">module_store::Allowlist</a>
</code></pre>



<a name="0x2_module_store_borrow_mut_allowlist"></a>

## Function `borrow_mut_allowlist`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_borrow_mut_allowlist">borrow_mut_allowlist</a>(): &<b>mut</b> <a href="module_store.md#0x2_module_store_Allowlist">module_store::Allowlist</a>
</code></pre>



<a name="0x2_module_store_add_to_allowlist"></a>

## Function `add_to_allowlist`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_add_to_allowlist">add_to_allowlist</a>(allowlist: &<b>mut</b> <a href="module_store.md#0x2_module_store_Allowlist">module_store::Allowlist</a>, <a href="account.md#0x2_account">account</a>: &<a href="">signer</a>, publisher: <b>address</b>)
</code></pre>



<a name="0x2_module_store_remove_from_allowlist"></a>

## Function `remove_from_allowlist`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_remove_from_allowlist">remove_from_allowlist</a>(allowlist: &<b>mut</b> <a href="module_store.md#0x2_module_store_Allowlist">module_store::Allowlist</a>, <a href="account.md#0x2_account">account</a>: &<a href="">signer</a>, publisher: <b>address</b>)
</code></pre>



<a name="0x2_module_store_is_in_allowlist"></a>

## Function `is_in_allowlist`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_is_in_allowlist">is_in_allowlist</a>(allowlist: &<a href="module_store.md#0x2_module_store_Allowlist">module_store::Allowlist</a>, publisher: <b>address</b>): bool
</code></pre>
