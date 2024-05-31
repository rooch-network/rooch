
<a name="0x3_builtin_validators"></a>

# Module `0x3::builtin_validators`



-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_builtin_validators_genesis_init)
-  [Function `is_builtin_auth_validator`](#0x3_builtin_validators_is_builtin_auth_validator)


<pre><code><b>use</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry">0x3::auth_validator_registry</a>;
<b>use</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator">0x3::bitcoin_validator</a>;
<b>use</b> <a href="session_validator.md#0x3_session_validator">0x3::session_validator</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_builtin_validators_ErrorGenesisInit"></a>



<pre><code><b>const</b> <a href="builtin_validators.md#0x3_builtin_validators_ErrorGenesisInit">ErrorGenesisInit</a>: u64 = 1;
</code></pre>



<a name="0x3_builtin_validators_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="builtin_validators.md#0x3_builtin_validators_genesis_init">genesis_init</a>(_genesis_account: &<a href="">signer</a>)
</code></pre>



<a name="0x3_builtin_validators_is_builtin_auth_validator"></a>

## Function `is_builtin_auth_validator`



<pre><code><b>public</b> <b>fun</b> <a href="builtin_validators.md#0x3_builtin_validators_is_builtin_auth_validator">is_builtin_auth_validator</a>(auth_validator_id: u64): bool
</code></pre>
