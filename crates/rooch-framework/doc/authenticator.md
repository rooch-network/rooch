
<a name="0x3_authenticator"></a>

# Module `0x3::authenticator`



-  [Struct `AuthenticatorInfo`](#0x3_authenticator_AuthenticatorInfo)
-  [Struct `Authenticator`](#0x3_authenticator_Authenticator)
-  [Constants](#@Constants_0)


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
