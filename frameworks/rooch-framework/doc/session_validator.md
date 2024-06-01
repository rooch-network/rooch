
<a name="0x3_session_validator"></a>

# Module `0x3::session_validator`

This module implements the session auth validator.


-  [Struct `SessionValidator`](#0x3_session_validator_SessionValidator)
-  [Constants](#@Constants_0)
-  [Function `auth_validator_id`](#0x3_session_validator_auth_validator_id)
-  [Function `validate`](#0x3_session_validator_validate)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::hash</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="ed25519.md#0x3_ed25519">0x3::ed25519</a>;
<b>use</b> <a href="session_key.md#0x3_session_key">0x3::session_key</a>;
</code></pre>



<a name="0x3_session_validator_SessionValidator"></a>

## Struct `SessionValidator`



<pre><code><b>struct</b> <a href="session_validator.md#0x3_session_validator_SessionValidator">SessionValidator</a> <b>has</b> drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_session_validator_SESSION_VALIDATOR_ID"></a>

there defines auth validator id for each auth validator


<pre><code><b>const</b> <a href="session_validator.md#0x3_session_validator_SESSION_VALIDATOR_ID">SESSION_VALIDATOR_ID</a>: u64 = 0;
</code></pre>



<a name="0x3_session_validator_SIGNATURE_SCHEME_ED25519"></a>



<pre><code><b>const</b> <a href="session_validator.md#0x3_session_validator_SIGNATURE_SCHEME_ED25519">SIGNATURE_SCHEME_ED25519</a>: u8 = 0;
</code></pre>



<a name="0x3_session_validator_auth_validator_id"></a>

## Function `auth_validator_id`



<pre><code><b>public</b> <b>fun</b> <a href="session_validator.md#0x3_session_validator_auth_validator_id">auth_validator_id</a>(): u64
</code></pre>



<a name="0x3_session_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="session_validator.md#0x3_session_validator_validate">validate</a>(authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>
