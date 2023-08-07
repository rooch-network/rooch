
<a name="0x3_session_key"></a>

# Module `0x3::session_key`



-  [Struct `SessionScope`](#0x3_session_key_SessionScope)
-  [Struct `SessionKey`](#0x3_session_key_SessionKey)
-  [Resource `SessionKeys`](#0x3_session_key_SessionKeys)
-  [Constants](#@Constants_0)
-  [Function `is_expired`](#0x3_session_key_is_expired)
-  [Function `exists_session_key`](#0x3_session_key_exists_session_key)
-  [Function `get_session_key`](#0x3_session_key_get_session_key)
-  [Function `create_session_key`](#0x3_session_key_create_session_key)
-  [Function `create_session_key_entry`](#0x3_session_key_create_session_key_entry)
-  [Function `validate`](#0x3_session_key_validate)
-  [Function `active_session_key`](#0x3_session_key_active_session_key)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::account_storage</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="ed25519_validator.md#0x3_ed25519_validator">0x3::ed25519_validator</a>;
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

</dd>
<dt>
<code>scheme: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>scopes: <a href="">vector</a>&lt;<a href="session_key.md#0x3_session_key_SessionScope">session_key::SessionScope</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>expiration_time: u64</code>
</dt>
<dd>
 The session key's expiration time period, in seconds, 0 means never expired
</dd>
<dt>
<code>last_active_time: u64</code>
</dt>
<dd>
 The session key's last active time
</dd>
<dt>
<code>max_inactive_interval: u64</code>
</dt>
<dd>
 The session key's max inactive time period, in seconds
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


<a name="0x3_session_key_ESessionIsExpired"></a>



<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ESessionIsExpired">ESessionIsExpired</a>: u64 = 4;
</code></pre>



<a name="0x3_session_key_ESessionKeyAlreadyExists"></a>



<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ESessionKeyAlreadyExists">ESessionKeyAlreadyExists</a>: u64 = 2;
</code></pre>



<a name="0x3_session_key_ESessionKeyCreatePermissionDenied"></a>



<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ESessionKeyCreatePermissionDenied">ESessionKeyCreatePermissionDenied</a>: u64 = 1;
</code></pre>



<a name="0x3_session_key_ESessionKeyIsInvalid"></a>



<pre><code><b>const</b> <a href="session_key.md#0x3_session_key_ESessionKeyIsInvalid">ESessionKeyIsInvalid</a>: u64 = 3;
</code></pre>



<a name="0x3_session_key_is_expired"></a>

## Function `is_expired`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_is_expired">is_expired</a>(_ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, _session_key: &<a href="session_key.md#0x3_session_key_SessionKey">session_key::SessionKey</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_is_expired">is_expired</a>(_ctx: &StorageContext, _session_key: &<a href="session_key.md#0x3_session_key_SessionKey">SessionKey</a>) : bool {
    //TODO check the session key is expired or not after the timestamp is supported
    <b>return</b> <b>false</b>
}
</code></pre>



</details>

<a name="0x3_session_key_exists_session_key"></a>

## Function `exists_session_key`



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_exists_session_key">exists_session_key</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_exists_session_key">exists_session_key</a>(ctx: &StorageContext, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;) : bool {
    <a href="_is_some">option::is_some</a>(&<a href="session_key.md#0x3_session_key_get_session_key">get_session_key</a>(ctx, account_address, authentication_key))
}
</code></pre>



</details>

<a name="0x3_session_key_get_session_key"></a>

## Function `get_session_key`

Get the session key of the account_address by the authentication key


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_get_session_key">get_session_key</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;<a href="session_key.md#0x3_session_key_SessionKey">session_key::SessionKey</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_get_session_key">get_session_key</a>(ctx: &StorageContext, account_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;) : Option&lt;<a href="session_key.md#0x3_session_key_SessionKey">SessionKey</a>&gt; {
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



<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key">create_session_key</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scheme: u64, scopes: <a href="">vector</a>&lt;<a href="session_key.md#0x3_session_key_SessionScope">session_key::SessionScope</a>&gt;, expiration_time: u64, max_inactive_interval: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key">create_session_key</a>(ctx: &<b>mut</b> StorageContext, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scheme: u64, scopes: <a href="">vector</a>&lt;<a href="session_key.md#0x3_session_key_SessionScope">SessionScope</a>&gt;, expiration_time: u64, max_inactive_interval: u64) {
    //Can not create new session key by the other session key
    <b>assert</b>!(!<a href="auth_validator.md#0x3_auth_validator_is_validate_via_session_key">auth_validator::is_validate_via_session_key</a>(ctx), <a href="_permission_denied">error::permission_denied</a>(<a href="session_key.md#0x3_session_key_ESessionKeyCreatePermissionDenied">ESessionKeyCreatePermissionDenied</a>));
    <b>let</b> sender_addr = <a href="_address_of">signer::address_of</a>(sender);
    <b>assert</b>!(!<a href="session_key.md#0x3_session_key_exists_session_key">exists_session_key</a>(ctx, sender_addr, authentication_key), <a href="_already_exists">error::already_exists</a>(<a href="session_key.md#0x3_session_key_ESessionKeyAlreadyExists">ESessionKeyAlreadyExists</a>));

    <b>let</b> <a href="session_key.md#0x3_session_key">session_key</a> = <a href="session_key.md#0x3_session_key_SessionKey">SessionKey</a> {
        authentication_key: authentication_key,
        scheme: scheme,
        scopes: scopes,
        expiration_time: expiration_time,
        //TODO set the last active time <b>to</b> now
        last_active_time: 0,
        max_inactive_interval: max_inactive_interval,
    };
    <b>if</b> (!<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender_addr)){
        <b>let</b> keys = <a href="_new">table::new</a>&lt;<a href="">vector</a>&lt;u8&gt;, <a href="session_key.md#0x3_session_key_SessionKey">SessionKey</a>&gt;(<a href="_tx_context_mut">storage_context::tx_context_mut</a>(ctx));
        <a href="_global_move_to">account_storage::global_move_to</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender, <a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>{keys});
    };

    <b>let</b> session_keys = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender_addr);
    <a href="_add">table::add</a>(&<b>mut</b> session_keys.keys, authentication_key, <a href="session_key.md#0x3_session_key">session_key</a>);
}
</code></pre>



</details>

<a name="0x3_session_key_create_session_key_entry"></a>

## Function `create_session_key_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key_entry">create_session_key_entry</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scheme: u64, scope_module_address: <b>address</b>, scope_module_name: <a href="_String">ascii::String</a>, scope_function_name: <a href="_String">ascii::String</a>, expiration_time: u64, max_inactive_interval: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="session_key.md#0x3_session_key_create_session_key_entry">create_session_key_entry</a>(ctx: &<b>mut</b> StorageContext, sender: &<a href="">signer</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;, scheme: u64, scope_module_address: <b>address</b>, scope_module_name: std::ascii::String, scope_function_name: std::ascii::String,expiration_time: u64, max_inactive_interval: u64) {
    <a href="session_key.md#0x3_session_key_create_session_key">create_session_key</a>(ctx, sender, authentication_key, scheme, <a href="_singleton">vector::singleton</a>(<a href="session_key.md#0x3_session_key_SessionScope">SessionScope</a>{
        module_address: scope_module_address,
        module_name: scope_module_name,
        function_name: scope_function_name,
    }), expiration_time, max_inactive_interval);
}
</code></pre>



</details>

<a name="0x3_session_key_validate"></a>

## Function `validate`

Validate the current tx via the session key
If the authentication key is not a session key, return option::none
If the session key is expired or invalid, abort the tx, otherwise return option::some(authentication key)


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_validate">validate</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, scheme: u64, authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_validate">validate</a>(ctx: &StorageContext, scheme: u64, authenticator_payload: <a href="">vector</a>&lt;u8&gt;) : Option&lt;<a href="">vector</a>&lt;u8&gt;&gt; {
    <b>let</b> sender_addr = <a href="_sender">storage_context::sender</a>(ctx);
    <b>if</b> (!<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender_addr)){
        <b>return</b> <a href="_none">option::none</a>()
    };
    <b>let</b> auth_key = <b>if</b>(scheme == <a href="ed25519_validator.md#0x3_ed25519_validator_scheme">ed25519_validator::scheme</a>()){
        <a href="ed25519_validator.md#0x3_ed25519_validator_get_authentication_key_from_payload">ed25519_validator::get_authentication_key_from_payload</a>(&authenticator_payload)
    }<b>else</b>{
        //TODO support other built-in validators
        <b>return</b> <a href="_none">option::none</a>()
    };

    <b>let</b> session_key_option = <a href="session_key.md#0x3_session_key_get_session_key">get_session_key</a>(ctx, sender_addr, auth_key);
    <b>if</b> (<a href="_is_none">option::is_none</a>(&session_key_option)){
        <b>return</b> <a href="_none">option::none</a>()
    };
    <b>let</b> <a href="session_key.md#0x3_session_key">session_key</a> = <a href="_extract">option::extract</a>(&<b>mut</b> session_key_option);
    <b>assert</b>!(!<a href="session_key.md#0x3_session_key_is_expired">is_expired</a>(ctx, &<a href="session_key.md#0x3_session_key">session_key</a>), <a href="_permission_denied">error::permission_denied</a>(<a href="session_key.md#0x3_session_key_ESessionIsExpired">ESessionIsExpired</a>));
    <b>assert</b>!(<a href="session_key.md#0x3_session_key">session_key</a>.scheme == scheme, <a href="_invalid_argument">error::invalid_argument</a>(<a href="session_key.md#0x3_session_key_ESessionKeyIsInvalid">ESessionKeyIsInvalid</a>));
    //TODO validate session scopes

    <b>if</b>(scheme == <a href="ed25519_validator.md#0x3_ed25519_validator_scheme">ed25519_validator::scheme</a>()){
        <a href="ed25519_validator.md#0x3_ed25519_validator_validate_signature">ed25519_validator::validate_signature</a>(&authenticator_payload, &<a href="_tx_hash">storage_context::tx_hash</a>(ctx));
    }<b>else</b>{
        //TODO support other built-in validators
        <b>abort</b> 1
    };
    <a href="_some">option::some</a>(auth_key)
}
</code></pre>



</details>

<a name="0x3_session_key_active_session_key"></a>

## Function `active_session_key`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_active_session_key">active_session_key</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, authentication_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_key.md#0x3_session_key_active_session_key">active_session_key</a>(ctx: &<b>mut</b> StorageContext, authentication_key: <a href="">vector</a>&lt;u8&gt;) {
    <b>let</b> sender_addr = <a href="_sender">storage_context::sender</a>(ctx);
    <b>assert</b>!(<a href="_global_exists">account_storage::global_exists</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender_addr), <a href="_not_found">error::not_found</a>(<a href="session_key.md#0x3_session_key_ESessionKeyIsInvalid">ESessionKeyIsInvalid</a>));
    <b>let</b> session_keys = <a href="_global_borrow_mut">account_storage::global_borrow_mut</a>&lt;<a href="session_key.md#0x3_session_key_SessionKeys">SessionKeys</a>&gt;(ctx, sender_addr);
    <b>assert</b>!(<a href="_contains">table::contains</a>(&session_keys.keys, authentication_key), <a href="_not_found">error::not_found</a>(<a href="session_key.md#0x3_session_key_ESessionKeyIsInvalid">ESessionKeyIsInvalid</a>));
    <b>let</b> <a href="session_key.md#0x3_session_key">session_key</a> = <a href="_borrow_mut">table::borrow_mut</a>(&<b>mut</b> session_keys.keys, authentication_key);
    //TODO set the last active time <b>to</b> now when the timestamp is supported
    <a href="session_key.md#0x3_session_key">session_key</a>.last_active_time = <a href="session_key.md#0x3_session_key">session_key</a>.last_active_time + 1;
}
</code></pre>



</details>
