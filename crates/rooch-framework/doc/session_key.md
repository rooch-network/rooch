
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
<b>use</b> <a href="">0x2::account_storage</a>;
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



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>module_address: <b>address</b></code>
</dt>
<dd>
 The scope module address, the address can not support <code>*</code>
</dd>
<dt>
<code>module_name: <a href="_String">ascii::String</a></code>
</dt>
<dd>
 The scope module name, <code>*</code> means all modules in the module address
</dd>
<dt>
<code>function_name: <a href="_String">ascii::String</a></code>
</dt>
<dd>
 The scope function name, <code>*</code> means all functions in the module
</dd>
</dl>


</details>

<a name="0x3_session_key_SessionKey"></a>

## Struct `SessionKey`



<pre><code><b>struct</b> <a href="session_key.md#0x3_session_key_SessionKey">SessionKey</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>authentication_key: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>
 The session key's authentication key, it also is the session key's id
</dd>
<dt>
<code>scopes: <a href="">vector</a>&lt;<a href="session_key.md#0x3_session_key_SessionScope">session_key::SessionScope</a>&gt;</code>
</dt>
<dd>
 The session key's scopes
</dd>
<dt>
<code>create_time: u64</code>
</dt>
<dd>
 The session key's create time, current timestamp in seconds
</dd>
<dt>
<code>last_active_time: u64</code>
</dt>
<dd>
 The session key's last active time, in seconds
</dd>
<dt>
<code>max_inactive_interval: u64</code>
</dt>
<dd>
 The session key's max inactive time period, in seconds
 If the session key is not active in this time period, it will be expired
 If the max_inactive_interval is 0, the session key will never be expired
</dd>
</dl>


</details>

<a name="0x3_session_key_SessionKeys"></a>

## Resource `SessionKeys`



<pre><code><b>struct</b> <a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>keys: <a href="_Table">table::Table</a>&lt;<a href="">vector</a>&lt;u8&gt;, <a href="session_key.md#0x3_session_key_SessionKey">session_key::SessionKey</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

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



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_new_session_scope">new_session_scope</a>(module_address: <b>address</b>, module_name: std::ascii::String, function_name: std::ascii::String) : <a href="session_key.md#0x3_session_key_SessionScope">SessionScope</a> {
    <a href="session_key.md#0x3_session_key_SessionScope">SessionScope</a> {
        module_address: module_address,
        module_name: module_name,
        function_name: function_name,
    }
}
</code></pre>



</details>

<a name="0x3_session_key_is_expired_session_key"></a>

## Function `is_expired_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_is_expired_session_key">is_expired_session_key</a>(ctx: &<a href="_Context">context::Context</a>, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_is_expired_session_key">is_expired_session_key</a>(ctx: &Context, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;) : bool {
    <b>let</b> session_key_option = <a href="session_key.md#0x3_session_key_get_session_key">get_session_key</a>(ctx, account_address, authentication_key);
    <b>if</b> (<a href="_is_none">option::is_none</a>(&session_key_option)){
        <b>return</b> <b>true</b>
    };

    <b>let</b> <a href="session_key.md#0x3_session_key">session_key</a> = <a href="_extract">option::extract</a>(&<b>mut</b> session_key_option);
    <a href="session_key.md#0x3_session_key_is_expired">is_expired</a>(ctx, &<a href="session_key.md#0x3_session_key">session_key</a>)
}
</code></pre>



</details>

<a name="0x3_session_key_exists_session_key"></a>

## Function `exists_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_exists_session_key">exists_session_key</a>(ctx: &<a href="_Context">context::Context</a>, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_exists_session_key">exists_session_key</a>(ctx: &Context, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;) : bool {
    <a href="_is_some">option::is_some</a>(&<a href="session_key.md#0x3_session_key_get_session_key">get_session_key</a>(ctx, account_address, authentication_key))
}
</code></pre>



</details>

<a name="0x3_session_key_get_session_key"></a>

## Function `get_session_key`

Get the session key of the account_address by the authentication key


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_get_session_key">get_session_key</a>(ctx: &<a href="_Context">context::Context</a>, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;<a href="session_key.md#0x3_session_key_SessionKey">session_key::SessionKey</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_get_session_key">get_session_key</a>(ctx: &Context, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;) : Option&lt;<a href="session_key.md#0x3_session_key_SessionKey">SessionKey</a>&gt; {
    <b>if</b> (!<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, account_address)){
        <b>return</b> <a href="_none">option::none</a>()
    };
    <b>let</b> session_keys = <a href="_global_borrow">account_storage::global_borrow</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, account_address);
    <b>if</b> (!<a href="_contains">table::contains</a>(&session_keys.keys, authentication_key)){
        <b>return</b> <a href="_none">option::none</a>()
    }<b>else</b>{
        <a href="_some">option::some</a>(*<a href="_borrow">table::borrow</a>(&session_keys.keys, authentication_key))
    }
}
</code></pre>



</details>

<a name="0x3_session_key_create_session_key"></a>

## Function `create_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key">create_session_key</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scopes: <a href="">vector</a>&lt;<a href="session_key.md#0x3_session_key_SessionScope">session_key::SessionScope</a>&gt;, max_inactive_interval: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key">create_session_key</a>(ctx: &<b>mut</b> Context, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scopes: <a href="">vector</a>&lt;<a href="session_key.md#0x3_session_key_SessionScope">SessionScope</a>&gt;, max_inactive_interval: u64) {
    //Can not create new session key by the other session key
    <b>assert</b>!(!<a href="auth_validator.md#0x3_auth_validator_is_validate_via_session_key">auth_validator::is_validate_via_session_key</a>(ctx), <a href="_permission_denied">error::permission_denied</a>(<a href="session_key.md#0x3_session_key_ErrorSessionKeyCreatePermissionDenied">ErrorSessionKeyCreatePermissionDenied</a>));
    <b>let</b> sender_addr = <a href="_address_of">signer::address_of</a>(sender);
    <b>assert</b>!(!<a href="session_key.md#0x3_session_key_exists_session_key">exists_session_key</a>(ctx, sender_addr, authentication_key), <a href="_already_exists">error::already_exists</a>(<a href="session_key.md#0x3_session_key_ErrorSessionKeyAlreadyExists">ErrorSessionKeyAlreadyExists</a>));
    <b>let</b> now_seconds = <a href="timestamp.md#0x3_timestamp_now_seconds">timestamp::now_seconds</a>(ctx);
    <b>let</b> <a href="session_key.md#0x3_session_key">session_key</a> = <a href="session_key.md#0x3_session_key_SessionKey">SessionKey</a> {
        authentication_key: authentication_key,
        scopes: scopes,
        create_time: now_seconds,
        last_active_time: now_seconds,
        max_inactive_interval: max_inactive_interval,
    };
    <b>if</b> (!<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender_addr)){
        <b>let</b> keys = <a href="_new">table::new</a>&lt;<a href="">vector</a>&lt;u8&gt;, <a href="session_key.md#0x3_session_key_SessionKey">SessionKey</a>&gt;(ctx);
        <a href="_global_move_to">account_storage::global_move_to</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender, <a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>{keys});
    };

    <b>let</b> session_keys = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender_addr);
    <a href="_add">table::add</a>(&<b>mut</b> session_keys.keys, authentication_key, <a href="session_key.md#0x3_session_key">session_key</a>);
}
</code></pre>



</details>

<a name="0x3_session_key_create_session_key_entry"></a>

## Function `create_session_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key_entry">create_session_key_entry</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scope_module_address: <b>address</b>, scope_module_name: <a href="_String">ascii::String</a>, scope_function_name: <a href="_String">ascii::String</a>, max_inactive_interval: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key_entry">create_session_key_entry</a>(ctx: &<b>mut</b> Context, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scope_module_address: <b>address</b>, scope_module_name: std::ascii::String, scope_function_name: std::ascii::String, max_inactive_interval: u64) {
    <a href="session_key.md#0x3_session_key_create_session_key">create_session_key</a>(ctx, sender, authentication_key, <a href="_singleton">vector::singleton</a>(<a href="session_key.md#0x3_session_key_SessionScope">SessionScope</a>{
        module_address: scope_module_address,
        module_name: scope_module_name,
        function_name: scope_function_name,
    }), max_inactive_interval);
}
</code></pre>



</details>

<a name="0x3_session_key_create_session_key_with_multi_scope_entry"></a>

## Function `create_session_key_with_multi_scope_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key_with_multi_scope_entry">create_session_key_with_multi_scope_entry</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scope_module_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;, scope_module_names: <a href="">vector</a>&lt;<a href="_String">ascii::String</a>&gt;, scope_function_names: <a href="">vector</a>&lt;<a href="_String">ascii::String</a>&gt;, max_inactive_interval: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key_with_multi_scope_entry">create_session_key_with_multi_scope_entry</a>(
    ctx: &<b>mut</b> Context,
    sender: &<a href="">signer</a>,
    authentication_key: <a href="">vector</a>&lt;u8&gt;,
    scope_module_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;,
    scope_module_names: <a href="">vector</a>&lt;std::ascii::String&gt;,
    scope_function_names: <a href="">vector</a>&lt;std::ascii::String&gt;,
    max_inactive_interval: u64) {
    <b>assert</b>!(
        <a href="_length">vector::length</a>&lt;<b>address</b>&gt;(&scope_module_addresses) == <a href="_length">vector::length</a>&lt;std::ascii::String&gt;(&scope_module_names) &&
        <a href="_length">vector::length</a>&lt;std::ascii::String&gt;(&scope_module_names) == <a href="_length">vector::length</a>&lt;std::ascii::String&gt;(&scope_function_names),
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="session_key.md#0x3_session_key_ErrorSessionScopePartLengthNotMatch">ErrorSessionScopePartLengthNotMatch</a>)
    );

    <b>let</b> idx = 0;
    <b>let</b> scopes = <a href="_empty">vector::empty</a>&lt;<a href="session_key.md#0x3_session_key_SessionScope">SessionScope</a>&gt;();

    <b>while</b>(idx &lt; <a href="_length">vector::length</a>(&scope_module_addresses)){
        <b>let</b> scope_module_address = <a href="_borrow">vector::borrow</a>(&scope_module_addresses, idx);
        <b>let</b> scope_module_name = <a href="_borrow">vector::borrow</a>(&scope_module_names, idx);
        <b>let</b> scope_function_name = <a href="_borrow">vector::borrow</a>(&scope_function_names, idx);

        <a href="_push_back">vector::push_back</a>(&<b>mut</b> scopes, <a href="session_key.md#0x3_session_key_SessionScope">SessionScope</a>{
            module_address: *scope_module_address,
            module_name: *scope_module_name,
            function_name: *scope_function_name,
        });

        idx = idx + 1;
    };

    <a href="session_key.md#0x3_session_key_create_session_key">create_session_key</a>(ctx, sender, authentication_key, scopes, max_inactive_interval);
}
</code></pre>



</details>

<a name="0x3_session_key_validate"></a>

## Function `validate`

Validate the current tx via the session key
If the authentication key is not a session key, return option::none
If the session key is expired or invalid, abort the tx, otherwise return option::some(authentication key)


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_validate">validate</a>(ctx: &<a href="_Context">context::Context</a>, auth_validator_id: u64, authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_validate">validate</a>(ctx: &Context, auth_validator_id: u64, authenticator_payload: <a href="">vector</a>&lt;u8&gt;) : Option&lt;<a href="">vector</a>&lt;u8&gt;&gt; {
    <b>let</b> sender_addr = <a href="_sender">context::sender</a>(ctx);
    <b>if</b> (!<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender_addr)){
        <b>return</b> <a href="_none">option::none</a>()
    };
    // We only support <b>native</b> validator for <a href="session_key.md#0x3_session_key_SessionKey">SessionKey</a> now
    <b>if</b>(auth_validator_id != <a href="native_validator.md#0x3_native_validator_auth_validator_id">native_validator::auth_validator_id</a>()){
        <b>return</b> <a href="_none">option::none</a>()
    };

    <b>let</b> auth_key = <a href="native_validator.md#0x3_native_validator_get_authentication_key_from_authenticator_payload">native_validator::get_authentication_key_from_authenticator_payload</a>(&authenticator_payload);

    <b>let</b> session_key_option = <a href="session_key.md#0x3_session_key_get_session_key">get_session_key</a>(ctx, sender_addr, auth_key);
    <b>if</b> (<a href="_is_none">option::is_none</a>(&session_key_option)){
        <b>return</b> <a href="_none">option::none</a>()
    };
    <b>let</b> <a href="session_key.md#0x3_session_key">session_key</a> = <a href="_extract">option::extract</a>(&<b>mut</b> session_key_option);
    <b>assert</b>!(!<a href="session_key.md#0x3_session_key_is_expired">is_expired</a>(ctx, &<a href="session_key.md#0x3_session_key">session_key</a>), <a href="_permission_denied">error::permission_denied</a>(<a href="session_key.md#0x3_session_key_ErrorSessionIsExpired">ErrorSessionIsExpired</a>));

    <b>assert</b>!(<a href="session_key.md#0x3_session_key_in_session_scope">in_session_scope</a>(ctx, &<a href="session_key.md#0x3_session_key">session_key</a>), <a href="_permission_denied">error::permission_denied</a>(<a href="session_key.md#0x3_session_key_ErrorFunctionCallBeyondSessionScope">ErrorFunctionCallBeyondSessionScope</a>));

    <a href="native_validator.md#0x3_native_validator_validate_signature">native_validator::validate_signature</a>(&authenticator_payload, &<a href="_tx_hash">context::tx_hash</a>(ctx));
    <a href="_some">option::some</a>(auth_key)
}
</code></pre>



</details>

<a name="0x3_session_key_active_session_key"></a>

## Function `active_session_key`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_active_session_key">active_session_key</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_active_session_key">active_session_key</a>(ctx: &<b>mut</b> Context, authentication_key: <a href="">vector</a>&lt;u8&gt;) {
    <b>let</b> sender_addr = <a href="_sender">context::sender</a>(ctx);
    <b>let</b> now_seconds = <a href="timestamp.md#0x3_timestamp_now_seconds">timestamp::now_seconds</a>(ctx);
    <b>assert</b>!(<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender_addr), <a href="_not_found">error::not_found</a>(<a href="session_key.md#0x3_session_key_ErrorSessionKeyIsInvalid">ErrorSessionKeyIsInvalid</a>));
    <b>let</b> session_keys = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender_addr);
    <b>assert</b>!(<a href="_contains">table::contains</a>(&session_keys.keys, authentication_key), <a href="_not_found">error::not_found</a>(<a href="session_key.md#0x3_session_key_ErrorSessionKeyIsInvalid">ErrorSessionKeyIsInvalid</a>));
    <b>let</b> <a href="session_key.md#0x3_session_key">session_key</a> = <a href="_borrow_mut">table::borrow_mut</a>(&<b>mut</b> session_keys.keys, authentication_key);
    <a href="session_key.md#0x3_session_key">session_key</a>.last_active_time = now_seconds;
}
</code></pre>



</details>

<a name="0x3_session_key_remove_session_key"></a>

## Function `remove_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_remove_session_key">remove_session_key</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_remove_session_key">remove_session_key</a>(ctx: &<b>mut</b> Context, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;) {
    <b>let</b> sender_addr = <a href="_address_of">signer::address_of</a>(sender);
    <b>assert</b>!(<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender_addr), <a href="_not_found">error::not_found</a>(<a href="session_key.md#0x3_session_key_ErrorSessionKeyIsInvalid">ErrorSessionKeyIsInvalid</a>));
    <b>let</b> session_keys = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender_addr);
    <b>assert</b>!(<a href="_contains">table::contains</a>(&session_keys.keys, authentication_key), <a href="_not_found">error::not_found</a>(<a href="session_key.md#0x3_session_key_ErrorSessionKeyIsInvalid">ErrorSessionKeyIsInvalid</a>));
    <a href="_remove">table::remove</a>(&<b>mut</b> session_keys.keys, authentication_key);
}
</code></pre>



</details>

<a name="0x3_session_key_remove_session_key_entry"></a>

## Function `remove_session_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_remove_session_key_entry">remove_session_key_entry</a>(ctx: &<b>mut</b> <a href="_Context">context::Context</a>, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_remove_session_key_entry">remove_session_key_entry</a>(ctx: &<b>mut</b> Context, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;) {
    <a href="session_key.md#0x3_session_key_remove_session_key">remove_session_key</a>(ctx, sender, authentication_key);
}
</code></pre>



</details>
