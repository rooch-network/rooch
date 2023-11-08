
<a name="0x3_session_key"></a>

# Module `0x3::session_key`



-  [Struct `SessionScope`](#0x3_session_key_SessionScope)
-  [Struct `SessionKey`](#0x3_session_key_SessionKey)
-  [Resource `SessionKeys`](#0x3_session_key_SessionKeys)
-  [Constants](#@Constants_0)
-  [Function `new_session_scope`](#0x3_session_key_new_session_scope)
-  [Function `is_expired_session_key`](#0x3_session_key_is_expired_session_key)
-  [Function `exists_session_key`](#0x3_session_key_exists_session_key)
-  [Function `get_session_key`](#0x3_session_key_get_session_key)
-  [Function `create_session_key`](#0x3_session_key_create_session_key)
-  [Function `create_session_key_entry`](#0x3_session_key_create_session_key_entry)
-  [Function `create_session_key_with_multi_scope_entry`](#0x3_session_key_create_session_key_with_multi_scope_entry)
-  [Function `validate`](#0x3_session_key_validate)
-  [Function `active_session_key`](#0x3_session_key_active_session_key)
-  [Function `remove_session_key`](#0x3_session_key_remove_session_key)
-  [Function `remove_session_key_entry`](#0x3_session_key_remove_session_key_entry)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::tx_meta</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="native_validator.md#0x3_native_validator">0x3::native_validator</a>;
<b>use</b> <a href="timestamp.md#0x3_timestamp">0x3::timestamp</a>;
</code></pre>



<a name="0x3_session_key_SessionScope"></a>

## Struct `SessionScope`

The session's scope


<pre><code><b>struct</b> <a href="session_key.md#0x3_session_key_SessionScope">SessionScope</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_session_key_SessionKey"></a>

## Struct `SessionKey`



<pre><code><b>struct</b> <a href="session_key.md#0x3_session_key_SessionKey">SessionKey</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_session_key_SessionKeys"></a>

## Resource `SessionKeys`



<pre><code><b>struct</b> <a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a> <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_session_key_ErrorFunctionCallBeyondSessionScope"></a>

The function call is beyond the session's scope


<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ErrorFunctionCallBeyondSessionScope">ErrorFunctionCallBeyondSessionScope</a>: u64 = 5;
</code></pre>



<a name="0x3_session_key_ErrorSessionIsExpired"></a>

The session is expired


<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ErrorSessionIsExpired">ErrorSessionIsExpired</a>: u64 = 4;
</code></pre>



<a name="0x3_session_key_ErrorSessionKeyAlreadyExists"></a>

The session key already exists


<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ErrorSessionKeyAlreadyExists">ErrorSessionKeyAlreadyExists</a>: u64 = 2;
</code></pre>



<a name="0x3_session_key_ErrorSessionKeyCreatePermissionDenied"></a>

Create session key in this context is not allowed


<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ErrorSessionKeyCreatePermissionDenied">ErrorSessionKeyCreatePermissionDenied</a>: u64 = 1;
</code></pre>



<a name="0x3_session_key_ErrorSessionKeyIsInvalid"></a>

The session key is invalid


<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ErrorSessionKeyIsInvalid">ErrorSessionKeyIsInvalid</a>: u64 = 3;
</code></pre>



<a name="0x3_session_key_ErrorSessionScopePartLengthNotMatch"></a>

The lengths of the parts of the session's scope do not match.


<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ErrorSessionScopePartLengthNotMatch">ErrorSessionScopePartLengthNotMatch</a>: u64 = 6;
</code></pre>



<a name="0x3_session_key_new_session_scope"></a>

## Function `new_session_scope`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_new_session_scope">new_session_scope</a>(module_address: <b>address</b>, module_name: <a href="_String">ascii::String</a>, function_name: <a href="_String">ascii::String</a>): <a href="session_key.md#0x3_session_key_SessionScope">session_key::SessionScope</a>
</code></pre>



<a name="0x3_session_key_is_expired_session_key"></a>

## Function `is_expired_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_is_expired_session_key">is_expired_session_key</a>(ctx: &<a href="_Context">context::Context</a>, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<a name="0x3_session_key_exists_session_key"></a>

## Function `exists_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_exists_session_key">exists_session_key</a>(ctx: &<a href="_Context">context::Context</a>, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<a name="0x3_session_key_get_session_key"></a>

## Function `get_session_key`

Get the session key of the account_address by the authentication key


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_get_session_key">get_session_key</a>(ctx: &<a href="_Context">context::Context</a>, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;<a href="session_key.md#0x3_session_key_SessionKey">session_key::SessionKey</a>&gt;
</code></pre>



<a name="0x3_session_key_create_session_key"></a>

## Function `create_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key">create_session_key</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scopes: <a href="">vector</a>&lt;<a href="session_key.md#0x3_session_key_SessionScope">session_key::SessionScope</a>&gt;, max_inactive_interval: u64)
</code></pre>



<a name="0x3_session_key_create_session_key_entry"></a>

## Function `create_session_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key_entry">create_session_key_entry</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scope_module_address: <b>address</b>, scope_module_name: <a href="_String">ascii::String</a>, scope_function_name: <a href="_String">ascii::String</a>, max_inactive_interval: u64)
</code></pre>



<a name="0x3_session_key_create_session_key_with_multi_scope_entry"></a>

## Function `create_session_key_with_multi_scope_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key_with_multi_scope_entry">create_session_key_with_multi_scope_entry</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scope_module_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;, scope_module_names: <a href="">vector</a>&lt;<a href="_String">ascii::String</a>&gt;, scope_function_names: <a href="">vector</a>&lt;<a href="_String">ascii::String</a>&gt;, max_inactive_interval: u64)
</code></pre>



<a name="0x3_session_key_validate"></a>

## Function `validate`

Validate the current tx via the session key
If the authentication key is not a session key, return option::none
If the session key is expired or invalid, abort the tx, otherwise return option::some(authentication key)


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_validate">validate</a>(ctx: &<a href="_Context">context::Context</a>, auth_validator_id: u64, authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x3_session_key_active_session_key"></a>

## Function `active_session_key`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_active_session_key">active_session_key</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_session_key_remove_session_key"></a>

## Function `remove_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_remove_session_key">remove_session_key</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_session_key_remove_session_key_entry"></a>

## Function `remove_session_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_remove_session_key_entry">remove_session_key_entry</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>
