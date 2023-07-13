
<a name="0x3_genesis"></a>

# Module `0x3::genesis`



-  [Constants](#@Constants_0)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry">0x3::auth_validator_registry</a>;
<b>use</b> <a href="ed25519_validator.md#0x3_ed25519_validator">0x3::ed25519_validator</a>;
<b>use</b> <a href="multi_ed25519_validator.md#0x3_multi_ed25519_validator">0x3::multi_ed25519_validator</a>;
<b>use</b> <a href="secp256k1_validator.md#0x3_secp256k1_validator">0x3::secp256k1_validator</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_genesis_E_GENESIS_INIT"></a>



<pre><code><b>const</b> <a href="genesis.md#0x3_genesis_E_GENESIS_INIT">E_GENESIS_INIT</a>: u64 = 1;
</code></pre>
