
<a name="0x2_module_store"></a>

# Module `0x2::module_store`

<code><a href="module_store.md#0x2_module_store">module_store</a></code> provide object to manage packages and modules.


-  [Resource `Allowlist`](#0x2_module_store_Allowlist)
-  [Resource `ModuleStore`](#0x2_module_store_ModuleStore)
-  [Resource `Package`](#0x2_module_store_Package)
-  [Struct `PackageData`](#0x2_module_store_PackageData)
-  [Resource `UpgradeCap`](#0x2_module_store_UpgradeCap)
-  [Struct `UpgradeEvent`](#0x2_module_store_UpgradeEvent)
-  [Constants](#@Constants_0)
-  [Function `module_store_id`](#0x2_module_store_module_store_id)
-  [Function `init_module_store`](#0x2_module_store_init_module_store)
-  [Function `issue_upgrade_cap_by_system`](#0x2_module_store_issue_upgrade_cap_by_system)
-  [Function `issue_upgrade_cap`](#0x2_module_store_issue_upgrade_cap)
-  [Function `is_upgrade_cap_issued`](#0x2_module_store_is_upgrade_cap_issued)
-  [Function `borrow_module_store`](#0x2_module_store_borrow_module_store)
-  [Function `borrow_mut_module_store`](#0x2_module_store_borrow_mut_module_store)
-  [Function `package_obj_id`](#0x2_module_store_package_obj_id)
-  [Function `exists_package`](#0x2_module_store_exists_package)
-  [Function `exists_module`](#0x2_module_store_exists_module)
-  [Function `publish_package_entry`](#0x2_module_store_publish_package_entry)
-  [Function `package_version`](#0x2_module_store_package_version)
-  [Function `publish_modules_internal`](#0x2_module_store_publish_modules_internal)
-  [Function `freeze_package`](#0x2_module_store_freeze_package)
-  [Function `add_to_allowlist`](#0x2_module_store_add_to_allowlist)
-  [Function `remove_from_allowlist`](#0x2_module_store_remove_from_allowlist)
-  [Function `is_in_allowlist`](#0x2_module_store_is_in_allowlist)
-  [Function `has_upgrade_permission`](#0x2_module_store_has_upgrade_permission)
-  [Function `ensure_upgrade_permission`](#0x2_module_store_ensure_upgrade_permission)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="core_addresses.md#0x2_core_addresses">0x2::core_addresses</a>;
<b>use</b> <a href="event.md#0x2_event">0x2::event</a>;
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

Used to store packages.
A package is an Object, and the package id is the module address.
Packages are child objects of ModuleStore.


<pre><code><b>struct</b> <a href="module_store.md#0x2_module_store_ModuleStore">ModuleStore</a> <b>has</b> key
</code></pre>



<a name="0x2_module_store_Package"></a>

## Resource `Package`

Used to store modules.
Modules are the Package's dynamic fields, with the module name as the key.


<pre><code><b>struct</b> <a href="module_store.md#0x2_module_store_Package">Package</a> <b>has</b> key
</code></pre>



<a name="0x2_module_store_PackageData"></a>

## Struct `PackageData`

This is a data struct to store package data, which is the same with the Rust definition.
When building package, the package data will be stored in this struct and be serialized,
we then deserialize package in Move.


<pre><code>#[data_struct]
<b>struct</b> <a href="module_store.md#0x2_module_store_PackageData">PackageData</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x2_module_store_UpgradeCap"></a>

## Resource `UpgradeCap`

Package upgrade capability


<pre><code><b>struct</b> <a href="module_store.md#0x2_module_store_UpgradeCap">UpgradeCap</a> <b>has</b> store, key
</code></pre>



<a name="0x2_module_store_UpgradeEvent"></a>

## Struct `UpgradeEvent`

Event for package upgrades. New published modules will also trigger this event.


<pre><code><b>struct</b> <a href="module_store.md#0x2_module_store_UpgradeEvent">UpgradeEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_module_store_ErrorNoUpgradePermission"></a>

Have no permission to upgrade package


<pre><code><b>const</b> <a href="module_store.md#0x2_module_store_ErrorNoUpgradePermission">ErrorNoUpgradePermission</a>: u64 = 2;
</code></pre>



<a name="0x2_module_store_ErrorNotAllowToPublish"></a>

Not allow to publish module


<pre><code><b>const</b> <a href="module_store.md#0x2_module_store_ErrorNotAllowToPublish">ErrorNotAllowToPublish</a>: u64 = 1;
</code></pre>



<a name="0x2_module_store_ErrorUpgradeCapIssued"></a>

Upgrade cap issued already


<pre><code><b>const</b> <a href="module_store.md#0x2_module_store_ErrorUpgradeCapIssued">ErrorUpgradeCapIssued</a>: u64 = 3;
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



<a name="0x2_module_store_issue_upgrade_cap_by_system"></a>

## Function `issue_upgrade_cap_by_system`

Issue an UpgradeCap for any package by the system accounts.


<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_issue_upgrade_cap_by_system">issue_upgrade_cap_by_system</a>(system: &<a href="">signer</a>, package_id: <b>address</b>, owner: <b>address</b>)
</code></pre>



<a name="0x2_module_store_issue_upgrade_cap"></a>

## Function `issue_upgrade_cap`

Issue an UpgradeCap for package under the sender's account. Then transfer the ownership to the owner.
This is used to issue an upgrade cap before first publishing.


<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_issue_upgrade_cap">issue_upgrade_cap</a>(sender: &<a href="">signer</a>, owner: <b>address</b>)
</code></pre>



<a name="0x2_module_store_is_upgrade_cap_issued"></a>

## Function `is_upgrade_cap_issued`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_is_upgrade_cap_issued">is_upgrade_cap_issued</a>(package_id: <b>address</b>): bool
</code></pre>



<a name="0x2_module_store_borrow_module_store"></a>

## Function `borrow_module_store`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_borrow_module_store">borrow_module_store</a>(): &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="module_store.md#0x2_module_store_ModuleStore">module_store::ModuleStore</a>&gt;
</code></pre>



<a name="0x2_module_store_borrow_mut_module_store"></a>

## Function `borrow_mut_module_store`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_borrow_mut_module_store">borrow_mut_module_store</a>(): &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="module_store.md#0x2_module_store_ModuleStore">module_store::ModuleStore</a>&gt;
</code></pre>



<a name="0x2_module_store_package_obj_id"></a>

## Function `package_obj_id`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_package_obj_id">package_obj_id</a>(package_id: <b>address</b>): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x2_module_store_exists_package"></a>

## Function `exists_package`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_exists_package">exists_package</a>(package_id: <b>address</b>): bool
</code></pre>



<a name="0x2_module_store_exists_module"></a>

## Function `exists_module`

Check if module exists
package_id: the address of the package
name: the name of the module


<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_exists_module">exists_module</a>(package_id: <b>address</b>, name: <a href="_String">string::String</a>): bool
</code></pre>



<a name="0x2_module_store_publish_package_entry"></a>

## Function `publish_package_entry`

Entry function to publish package
The order of modules must be sorted by dependency order.


<pre><code><b>public</b> entry <b>fun</b> <a href="module_store.md#0x2_module_store_publish_package_entry">publish_package_entry</a>(<a href="account.md#0x2_account">account</a>: &<a href="">signer</a>, package_bytes: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x2_module_store_package_version"></a>

## Function `package_version`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_package_version">package_version</a>(package_id: <b>address</b>): u64
</code></pre>



<a name="0x2_module_store_publish_modules_internal"></a>

## Function `publish_modules_internal`

Publish modules to the module object's storage
Return true if the modules are upgraded


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="module_store.md#0x2_module_store_publish_modules_internal">publish_modules_internal</a>(module_store_object: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="module_store.md#0x2_module_store_ModuleStore">module_store::ModuleStore</a>&gt;, package_id: <b>address</b>, modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;): bool
</code></pre>



<a name="0x2_module_store_freeze_package"></a>

## Function `freeze_package`



<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_freeze_package">freeze_package</a>(package: <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="module_store.md#0x2_module_store_Package">module_store::Package</a>&gt;)
</code></pre>



<a name="0x2_module_store_add_to_allowlist"></a>

## Function `add_to_allowlist`

Add a package id to the allowlist. Only package id in allowlist can publish modules.
This is only valid when module_publishing_allowlist_enabled feature is enabled.


<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_add_to_allowlist">add_to_allowlist</a>(<a href="account.md#0x2_account">account</a>: &<a href="">signer</a>, package_id: <b>address</b>)
</code></pre>



<a name="0x2_module_store_remove_from_allowlist"></a>

## Function `remove_from_allowlist`

Remove a package id from the allowlist.


<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_remove_from_allowlist">remove_from_allowlist</a>(<a href="account.md#0x2_account">account</a>: &<a href="">signer</a>, package_id: <b>address</b>)
</code></pre>



<a name="0x2_module_store_is_in_allowlist"></a>

## Function `is_in_allowlist`

Check if a package id is in the allowlist.


<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_is_in_allowlist">is_in_allowlist</a>(package_id: <b>address</b>): bool
</code></pre>



<a name="0x2_module_store_has_upgrade_permission"></a>

## Function `has_upgrade_permission`

Check if the account has the permission to upgrade the package with the package_id.


<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_has_upgrade_permission">has_upgrade_permission</a>(package_id: <b>address</b>, <a href="account.md#0x2_account">account</a>: <b>address</b>): bool
</code></pre>



<a name="0x2_module_store_ensure_upgrade_permission"></a>

## Function `ensure_upgrade_permission`

Ensure the account has the permission to upgrade the package with the package_id.


<pre><code><b>public</b> <b>fun</b> <a href="module_store.md#0x2_module_store_ensure_upgrade_permission">ensure_upgrade_permission</a>(package_id: <b>address</b>, <a href="account.md#0x2_account">account</a>: &<a href="">signer</a>)
</code></pre>
