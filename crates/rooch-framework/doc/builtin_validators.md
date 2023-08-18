
<a name="0x3_builtin_validators"></a>

# Module `0x3::builtin_validators`



-  [Constants](#@Constants_0)
-  [Function `genesis_init`](#0x3_builtin_validators_genesis_init)
-  [Function `is_builtin_scheme`](#0x3_builtin_validators_is_builtin_scheme)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry">0x3::auth_validator_registry</a>;
<b>use</b> <a href="bitcoin_validator.md#0x3_bitcoin_validator">0x3::bitcoin_validator</a>;
<b>use</b> <a href="ethereum_validator.md#0x3_ethereum_validator">0x3::ethereum_validator</a>;
<b>use</b> <a href="multi_ed25519_validator.md#0x3_multi_ed25519_validator">0x3::multi_ed25519_validator</a>;
<b>use</b> <a href="native_validator.md#0x3_native_validator">0x3::native_validator</a>;
<b>use</b> <a href="nostr_validator.md#0x3_nostr_validator">0x3::nostr_validator</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_builtin_validators_E_GENESIS_INIT"></a>



<pre><code><b>const</b> <a href="builtin_validators.md#0x3_builtin_validators_E_GENESIS_INIT">E_GENESIS_INIT</a>: u64 = 1;
</code></pre>



<a name="0x3_builtin_validators_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="builtin_validators.md#0x3_builtin_validators_genesis_init">genesis_init</a>(ctx: &<b>mut</b> <a href="_StorageContext">storage_context::StorageContext</a>, _genesis_account: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="builtin_validators.md#0x3_builtin_validators_genesis_init">genesis_init</a>(ctx: &<b>mut</b> StorageContext, _genesis_account: &<a href="">signer</a>){
    // SCHEME_NATIVE: u64 = 0;
    <b>let</b> id = <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_internal">auth_validator_registry::register_internal</a>&lt;<a href="native_validator.md#0x3_native_validator_NativeValidator">native_validator::NativeValidator</a>&gt;(ctx);
    <b>assert</b>!(id == <a href="native_validator.md#0x3_native_validator_scheme">native_validator::scheme</a>(), std::error::internal(<a href="builtin_validators.md#0x3_builtin_validators_E_GENESIS_INIT">E_GENESIS_INIT</a>));

    // SCHEME_MULTIED25519: u64 = 1;
    <b>let</b> id = <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_internal">auth_validator_registry::register_internal</a>&lt;<a href="multi_ed25519_validator.md#0x3_multi_ed25519_validator_MultiEd25519Validator">multi_ed25519_validator::MultiEd25519Validator</a>&gt;(ctx);
    <b>assert</b>!(id == <a href="multi_ed25519_validator.md#0x3_multi_ed25519_validator_scheme">multi_ed25519_validator::scheme</a>(), std::error::internal(<a href="builtin_validators.md#0x3_builtin_validators_E_GENESIS_INIT">E_GENESIS_INIT</a>));

    // SCHEME_BITCOIN: u64 = 2;
    <b>let</b> id = <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_internal">auth_validator_registry::register_internal</a>&lt;<a href="bitcoin_validator.md#0x3_bitcoin_validator_BitcoinValidator">bitcoin_validator::BitcoinValidator</a>&gt;(ctx);
    <b>assert</b>!(id == <a href="bitcoin_validator.md#0x3_bitcoin_validator_scheme">bitcoin_validator::scheme</a>(), std::error::internal(<a href="builtin_validators.md#0x3_builtin_validators_E_GENESIS_INIT">E_GENESIS_INIT</a>));

    // SCHEME_ETHEREUM: u64 = 3;
    <b>let</b> id = <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_internal">auth_validator_registry::register_internal</a>&lt;<a href="ethereum_validator.md#0x3_ethereum_validator_EthereumValidator">ethereum_validator::EthereumValidator</a>&gt;(ctx);
    <b>assert</b>!(id == <a href="ethereum_validator.md#0x3_ethereum_validator_scheme">ethereum_validator::scheme</a>(), std::error::internal(<a href="builtin_validators.md#0x3_builtin_validators_E_GENESIS_INIT">E_GENESIS_INIT</a>));

    // SCHEME_NOSTR: u64 = 4;
    <b>let</b> id = <a href="auth_validator_registry.md#0x3_auth_validator_registry_register_internal">auth_validator_registry::register_internal</a>&lt;<a href="nostr_validator.md#0x3_nostr_validator_NostrValidator">nostr_validator::NostrValidator</a>&gt;(ctx);
    <b>assert</b>!(id == <a href="nostr_validator.md#0x3_nostr_validator_scheme">nostr_validator::scheme</a>(), std::error::internal(<a href="builtin_validators.md#0x3_builtin_validators_E_GENESIS_INIT">E_GENESIS_INIT</a>));
}
</code></pre>



</details>

<a name="0x3_builtin_validators_is_builtin_scheme"></a>

## Function `is_builtin_scheme`



<pre><code><b>public</b> <b>fun</b> <a href="builtin_validators.md#0x3_builtin_validators_is_builtin_scheme">is_builtin_scheme</a>(scheme: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="builtin_validators.md#0x3_builtin_validators_is_builtin_scheme">is_builtin_scheme</a>(scheme: u64): bool {
    scheme == <a href="native_validator.md#0x3_native_validator_scheme">native_validator::scheme</a>()
    || scheme == <a href="multi_ed25519_validator.md#0x3_multi_ed25519_validator_scheme">multi_ed25519_validator::scheme</a>()
    || scheme == <a href="bitcoin_validator.md#0x3_bitcoin_validator_scheme">bitcoin_validator::scheme</a>()
    || scheme == <a href="ethereum_validator.md#0x3_ethereum_validator_scheme">ethereum_validator::scheme</a>()
    || scheme == <a href="nostr_validator.md#0x3_nostr_validator_scheme">nostr_validator::scheme</a>()
}
</code></pre>



</details>
