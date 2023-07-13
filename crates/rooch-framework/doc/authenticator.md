
<a name="0x3_authenticator"></a>

# Module `0x3::authenticator`



-  [Struct `AuthenticatorInfo`](#0x3_authenticator_AuthenticatorInfo)
-  [Struct `Authenticator`](#0x3_authenticator_Authenticator)
-  [Constants](#@Constants_0)
-  [Function `is_builtin_scheme`](#0x3_authenticator_is_builtin_scheme)


<pre><code></code></pre>



<a name="0x3_authenticator_AuthenticatorInfo"></a>

## Struct `AuthenticatorInfo`



<pre><code><b>struct</b> <a href="authenticator.md#0x3_authenticator_AuthenticatorInfo">AuthenticatorInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code><a href="authenticator.md#0x3_authenticator">authenticator</a>: <a href="authenticator.md#0x3_authenticator_Authenticator">authenticator::Authenticator</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_authenticator_Authenticator"></a>

## Struct `Authenticator`



<pre><code><b>struct</b> <a href="authenticator.md#0x3_authenticator_Authenticator">Authenticator</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>scheme: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>payload: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_authenticator_EUnsupportedScheme"></a>



<pre><code><b>const</b> <a href="authenticator.md#0x3_authenticator_EUnsupportedScheme">EUnsupportedScheme</a>: u64 = 1000;
</code></pre>



<a name="0x3_authenticator_SCHEME_ED25519"></a>



<pre><code><b>const</b> <a href="authenticator.md#0x3_authenticator_SCHEME_ED25519">SCHEME_ED25519</a>: u64 = 0;
</code></pre>



<a name="0x3_authenticator_SCHEME_MULTIED25519"></a>



<pre><code><b>const</b> <a href="authenticator.md#0x3_authenticator_SCHEME_MULTIED25519">SCHEME_MULTIED25519</a>: u64 = 1;
</code></pre>



<a name="0x3_authenticator_SCHEME_SECP256K1"></a>



<pre><code><b>const</b> <a href="authenticator.md#0x3_authenticator_SCHEME_SECP256K1">SCHEME_SECP256K1</a>: u64 = 2;
</code></pre>



<a name="0x3_authenticator_is_builtin_scheme"></a>

## Function `is_builtin_scheme`



<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_is_builtin_scheme">is_builtin_scheme</a>(scheme: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_is_builtin_scheme">is_builtin_scheme</a>(scheme: u64): bool {
   scheme == <a href="authenticator.md#0x3_authenticator_SCHEME_ED25519">SCHEME_ED25519</a> || scheme == <a href="authenticator.md#0x3_authenticator_SCHEME_MULTIED25519">SCHEME_MULTIED25519</a> || scheme == <a href="authenticator.md#0x3_authenticator_SCHEME_SECP256K1">SCHEME_SECP256K1</a>
}
</code></pre>



</details>
