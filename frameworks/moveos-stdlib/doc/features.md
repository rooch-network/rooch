
<a name="0x2_features"></a>

# Module `0x2::features`

Defines feature flags for Rooch frameworks. Those are used in implementations of features in
the moveos-stdlib, rooch-framework and other frameworks.

============================================================================================
Feature Flag Definitions

Each feature flag should come with documentation which justifies the need of the flag.
Introduction of a new feature flag requires approval of framework owners. Be frugal when
introducing new feature flags, as too many can make it hard to understand the code.

Note that removing a feature flag still requires the function which tests for the feature
to stay around for compatibility reasons, as it is a public function. However, once the
feature flag is disabled, those functions can constantly return true.


-  [Resource `FeatureStore`](#0x2_features_FeatureStore)
-  [Constants](#@Constants_0)
-  [Function `change_feature_flags`](#0x2_features_change_feature_flags)
-  [Function `is_enabled`](#0x2_features_is_enabled)
-  [Function `get_module_template_feature`](#0x2_features_get_module_template_feature)
-  [Function `module_template_enabled`](#0x2_features_module_template_enabled)
-  [Function `ensuer_module_template_enabled`](#0x2_features_ensuer_module_template_enabled)
-  [Function `get_all_features`](#0x2_features_get_all_features)


<pre><code><b>use</b> <a href="core_addresses.md#0x2_core_addresses">0x2::core_addresses</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_features_FeatureStore"></a>

## Resource `FeatureStore`

The enabled features, represented by a bitset stored on chain.


<pre><code><b>struct</b> <a href="features.md#0x2_features_FeatureStore">FeatureStore</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_features_EAPI_DISABLED"></a>



<pre><code><b>const</b> <a href="features.md#0x2_features_EAPI_DISABLED">EAPI_DISABLED</a>: u64 = 2;
</code></pre>



<a name="0x2_features_EINVALID_FEATURE"></a>



<pre><code><b>const</b> <a href="features.md#0x2_features_EINVALID_FEATURE">EINVALID_FEATURE</a>: u64 = 1;
</code></pre>



<a name="0x2_features_MODULE_TEMPLATE"></a>

Whether allowing replacing module's address, module identifier, struct identifier
and constant address.
This feature is used for creating a new module through a module template bytes,
thus developers can used to publish new modules in Move.


<pre><code><b>const</b> <a href="features.md#0x2_features_MODULE_TEMPLATE">MODULE_TEMPLATE</a>: u64 = 1;
</code></pre>



<a name="0x2_features_change_feature_flags"></a>

## Function `change_feature_flags`

Enable or disable features. Only the framework signers can call this function.


<pre><code><b>public</b> <b>fun</b> <a href="features.md#0x2_features_change_feature_flags">change_feature_flags</a>(framework: &<a href="">signer</a>, enable: <a href="">vector</a>&lt;u64&gt;, disable: <a href="">vector</a>&lt;u64&gt;)
</code></pre>



<a name="0x2_features_is_enabled"></a>

## Function `is_enabled`

Check whether the feature is enabled.
All features are enabled for system reserved accounts.


<pre><code><b>public</b> <b>fun</b> <a href="features.md#0x2_features_is_enabled">is_enabled</a>(feature: u64): bool
</code></pre>



<a name="0x2_features_get_module_template_feature"></a>

## Function `get_module_template_feature`



<pre><code><b>public</b> <b>fun</b> <a href="features.md#0x2_features_get_module_template_feature">get_module_template_feature</a>(): u64
</code></pre>



<a name="0x2_features_module_template_enabled"></a>

## Function `module_template_enabled`



<pre><code><b>public</b> <b>fun</b> <a href="features.md#0x2_features_module_template_enabled">module_template_enabled</a>(): bool
</code></pre>



<a name="0x2_features_ensuer_module_template_enabled"></a>

## Function `ensuer_module_template_enabled`



<pre><code><b>public</b> <b>fun</b> <a href="features.md#0x2_features_ensuer_module_template_enabled">ensuer_module_template_enabled</a>()
</code></pre>



<a name="0x2_features_get_all_features"></a>

## Function `get_all_features`

Helper for getting all features.
Update this once new feature added.


<pre><code><b>public</b> <b>fun</b> <a href="features.md#0x2_features_get_all_features">get_all_features</a>(): <a href="">vector</a>&lt;u64&gt;
</code></pre>
