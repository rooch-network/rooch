
<a name="0x3_authenticator"></a>

# Module `0x3::authenticator`



-  [Struct `AuthenticatorInfo`](#0x3_authenticator_AuthenticatorInfo)
-  [Struct `Authenticator`](#0x3_authenticator_Authenticator)
-  [Struct `Ed25519Authenticator`](#0x3_authenticator_Ed25519Authenticator)
-  [Struct `MultiEd25519Authenticator`](#0x3_authenticator_MultiEd25519Authenticator)
-  [Struct `Secp256k1Authenticator`](#0x3_authenticator_Secp256k1Authenticator)
-  [Constants](#@Constants_0)
-  [Function `check_authenticator`](#0x3_authenticator_check_authenticator)
-  [Function `scheme`](#0x3_authenticator_scheme)
-  [Function `decode_authenticator_info`](#0x3_authenticator_decode_authenticator_info)
-  [Function `decode_ed25519_authenticator`](#0x3_authenticator_decode_ed25519_authenticator)
-  [Function `ed25519_public`](#0x3_authenticator_ed25519_public)
-  [Function `ed25519_signature`](#0x3_authenticator_ed25519_signature)
-  [Function `decode_multied25519_authenticator`](#0x3_authenticator_decode_multied25519_authenticator)
-  [Function `decode_secp256k1_authenticator`](#0x3_authenticator_decode_secp256k1_authenticator)
-  [Function `secp256k1_signature`](#0x3_authenticator_secp256k1_signature)


<pre><code><b>use</b> <a href="">0x2::bcs</a>;
</code></pre>



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

<a name="0x3_authenticator_Ed25519Authenticator"></a>

## Struct `Ed25519Authenticator`



<pre><code><b>struct</b> <a href="authenticator.md#0x3_authenticator_Ed25519Authenticator">Ed25519Authenticator</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>signature: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_authenticator_MultiEd25519Authenticator"></a>

## Struct `MultiEd25519Authenticator`



<pre><code><b>struct</b> <a href="authenticator.md#0x3_authenticator_MultiEd25519Authenticator">MultiEd25519Authenticator</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>public_key: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>signature: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_authenticator_Secp256k1Authenticator"></a>

## Struct `Secp256k1Authenticator`



<pre><code><b>struct</b> <a href="authenticator.md#0x3_authenticator_Secp256k1Authenticator">Secp256k1Authenticator</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>signature: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_authenticator_ED25519_PUBKEY_LENGTH"></a>



<pre><code><b>const</b> <a href="authenticator.md#0x3_authenticator_ED25519_PUBKEY_LENGTH">ED25519_PUBKEY_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x3_authenticator_ED25519_SCHEME_LENGTH"></a>



<pre><code><b>const</b> <a href="authenticator.md#0x3_authenticator_ED25519_SCHEME_LENGTH">ED25519_SCHEME_LENGTH</a>: u64 = 1;
</code></pre>



<a name="0x3_authenticator_ED25519_SIG_LENGTH"></a>



<pre><code><b>const</b> <a href="authenticator.md#0x3_authenticator_ED25519_SIG_LENGTH">ED25519_SIG_LENGTH</a>: u64 = 64;
</code></pre>



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



<a name="0x3_authenticator_check_authenticator"></a>

## Function `check_authenticator`

Check if we can handle the given authenticator info.
If not, just abort


<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_check_authenticator">check_authenticator</a>(<a href="authenticator.md#0x3_authenticator">authenticator</a>: &<a href="authenticator.md#0x3_authenticator_Authenticator">authenticator::Authenticator</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_check_authenticator">check_authenticator</a>(<a href="authenticator.md#0x3_authenticator">authenticator</a>: &<a href="authenticator.md#0x3_authenticator_Authenticator">Authenticator</a>) {
   <b>assert</b>!(<a href="authenticator.md#0x3_authenticator_is_builtin_scheme">is_builtin_scheme</a>(<a href="authenticator.md#0x3_authenticator">authenticator</a>.scheme), <a href="authenticator.md#0x3_authenticator_EUnsupportedScheme">EUnsupportedScheme</a>);
}
</code></pre>



</details>

<a name="0x3_authenticator_scheme"></a>

## Function `scheme`



<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_scheme">scheme</a>(self: &<a href="authenticator.md#0x3_authenticator_Authenticator">authenticator::Authenticator</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_scheme">scheme</a>(self: &<a href="authenticator.md#0x3_authenticator_Authenticator">Authenticator</a>): u64 {
   self.scheme
}
</code></pre>



</details>

<a name="0x3_authenticator_decode_authenticator_info"></a>

## Function `decode_authenticator_info`



<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_decode_authenticator_info">decode_authenticator_info</a>(data: <a href="">vector</a>&lt;u8&gt;): (u64, <a href="authenticator.md#0x3_authenticator_Authenticator">authenticator::Authenticator</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_decode_authenticator_info">decode_authenticator_info</a>(data: <a href="">vector</a>&lt;u8&gt;): (u64, <a href="authenticator.md#0x3_authenticator_Authenticator">Authenticator</a>) {
   <b>let</b> info = moveos_std::bcs::from_bytes&lt;<a href="authenticator.md#0x3_authenticator_AuthenticatorInfo">AuthenticatorInfo</a>&gt;(data);
   <b>let</b> <a href="authenticator.md#0x3_authenticator_AuthenticatorInfo">AuthenticatorInfo</a> { sequence_number, <a href="authenticator.md#0x3_authenticator">authenticator</a> } = info;
   (sequence_number, <a href="authenticator.md#0x3_authenticator">authenticator</a>)
}
</code></pre>



</details>

<a name="0x3_authenticator_decode_ed25519_authenticator"></a>

## Function `decode_ed25519_authenticator`



<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_decode_ed25519_authenticator">decode_ed25519_authenticator</a>(<a href="authenticator.md#0x3_authenticator">authenticator</a>: <a href="authenticator.md#0x3_authenticator_Authenticator">authenticator::Authenticator</a>): <a href="authenticator.md#0x3_authenticator_Ed25519Authenticator">authenticator::Ed25519Authenticator</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_decode_ed25519_authenticator">decode_ed25519_authenticator</a>(<a href="authenticator.md#0x3_authenticator">authenticator</a>: <a href="authenticator.md#0x3_authenticator_Authenticator">Authenticator</a>): <a href="authenticator.md#0x3_authenticator_Ed25519Authenticator">Ed25519Authenticator</a> {
   <b>assert</b>!(<a href="authenticator.md#0x3_authenticator">authenticator</a>.scheme == <a href="authenticator.md#0x3_authenticator_SCHEME_ED25519">SCHEME_ED25519</a>, <a href="authenticator.md#0x3_authenticator_EUnsupportedScheme">EUnsupportedScheme</a>);
   moveos_std::bcs::from_bytes&lt;<a href="authenticator.md#0x3_authenticator_Ed25519Authenticator">Ed25519Authenticator</a>&gt;(<a href="authenticator.md#0x3_authenticator">authenticator</a>.payload)
}
</code></pre>



</details>

<a name="0x3_authenticator_ed25519_public"></a>

## Function `ed25519_public`



<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_ed25519_public">ed25519_public</a>(self: &<a href="authenticator.md#0x3_authenticator_Ed25519Authenticator">authenticator::Ed25519Authenticator</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_ed25519_public">ed25519_public</a>(self: &<a href="authenticator.md#0x3_authenticator_Ed25519Authenticator">Ed25519Authenticator</a>): <a href="">vector</a>&lt;u8&gt; {
   <b>let</b> public_key = <a href="_empty">vector::empty</a>&lt;u8&gt;();
   <b>let</b> i = <a href="authenticator.md#0x3_authenticator_ED25519_SCHEME_LENGTH">ED25519_SCHEME_LENGTH</a> + <a href="authenticator.md#0x3_authenticator_ED25519_SIG_LENGTH">ED25519_SIG_LENGTH</a>;
   <b>while</b> (i &lt; <a href="authenticator.md#0x3_authenticator_ED25519_SCHEME_LENGTH">ED25519_SCHEME_LENGTH</a> + <a href="authenticator.md#0x3_authenticator_ED25519_SIG_LENGTH">ED25519_SIG_LENGTH</a> + <a href="authenticator.md#0x3_authenticator_ED25519_PUBKEY_LENGTH">ED25519_PUBKEY_LENGTH</a>) {
      <b>let</b> value = <a href="_borrow">vector::borrow</a>(&self.signature, i);
      <a href="_push_back">vector::push_back</a>(&<b>mut</b> public_key, *value);
      i = i + 1;
   };

   public_key
}
</code></pre>



</details>

<a name="0x3_authenticator_ed25519_signature"></a>

## Function `ed25519_signature`



<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_ed25519_signature">ed25519_signature</a>(self: &<a href="authenticator.md#0x3_authenticator_Ed25519Authenticator">authenticator::Ed25519Authenticator</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_ed25519_signature">ed25519_signature</a>(self: &<a href="authenticator.md#0x3_authenticator_Ed25519Authenticator">Ed25519Authenticator</a>): <a href="">vector</a>&lt;u8&gt; {
   <b>let</b> sign = <a href="_empty">vector::empty</a>&lt;u8&gt;();
   <b>let</b> i = <a href="authenticator.md#0x3_authenticator_ED25519_SCHEME_LENGTH">ED25519_SCHEME_LENGTH</a>;
   <b>while</b> (i &lt; <a href="authenticator.md#0x3_authenticator_ED25519_SIG_LENGTH">ED25519_SIG_LENGTH</a> + 1) {
      <b>let</b> value = <a href="_borrow">vector::borrow</a>(&self.signature, i);
      <a href="_push_back">vector::push_back</a>(&<b>mut</b> sign, *value);
      i = i + 1;
   };

   sign
}
</code></pre>



</details>

<a name="0x3_authenticator_decode_multied25519_authenticator"></a>

## Function `decode_multied25519_authenticator`



<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_decode_multied25519_authenticator">decode_multied25519_authenticator</a>(<a href="authenticator.md#0x3_authenticator">authenticator</a>: <a href="authenticator.md#0x3_authenticator_Authenticator">authenticator::Authenticator</a>): <a href="authenticator.md#0x3_authenticator_MultiEd25519Authenticator">authenticator::MultiEd25519Authenticator</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_decode_multied25519_authenticator">decode_multied25519_authenticator</a>(<a href="authenticator.md#0x3_authenticator">authenticator</a>: <a href="authenticator.md#0x3_authenticator_Authenticator">Authenticator</a>): <a href="authenticator.md#0x3_authenticator_MultiEd25519Authenticator">MultiEd25519Authenticator</a> {
   <b>assert</b>!(<a href="authenticator.md#0x3_authenticator">authenticator</a>.scheme == <a href="authenticator.md#0x3_authenticator_SCHEME_MULTIED25519">SCHEME_MULTIED25519</a>, <a href="authenticator.md#0x3_authenticator_EUnsupportedScheme">EUnsupportedScheme</a>);
   moveos_std::bcs::from_bytes&lt;<a href="authenticator.md#0x3_authenticator_MultiEd25519Authenticator">MultiEd25519Authenticator</a>&gt;(<a href="authenticator.md#0x3_authenticator">authenticator</a>.payload)
}
</code></pre>



</details>

<a name="0x3_authenticator_decode_secp256k1_authenticator"></a>

## Function `decode_secp256k1_authenticator`



<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_decode_secp256k1_authenticator">decode_secp256k1_authenticator</a>(<a href="authenticator.md#0x3_authenticator">authenticator</a>: <a href="authenticator.md#0x3_authenticator_Authenticator">authenticator::Authenticator</a>): <a href="authenticator.md#0x3_authenticator_Secp256k1Authenticator">authenticator::Secp256k1Authenticator</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_decode_secp256k1_authenticator">decode_secp256k1_authenticator</a>(<a href="authenticator.md#0x3_authenticator">authenticator</a>: <a href="authenticator.md#0x3_authenticator_Authenticator">Authenticator</a>): <a href="authenticator.md#0x3_authenticator_Secp256k1Authenticator">Secp256k1Authenticator</a> {
   <b>assert</b>!(<a href="authenticator.md#0x3_authenticator">authenticator</a>.scheme == <a href="authenticator.md#0x3_authenticator_SCHEME_SECP256K1">SCHEME_SECP256K1</a>, <a href="authenticator.md#0x3_authenticator_EUnsupportedScheme">EUnsupportedScheme</a>);
   moveos_std::bcs::from_bytes&lt;<a href="authenticator.md#0x3_authenticator_Secp256k1Authenticator">Secp256k1Authenticator</a>&gt;(<a href="authenticator.md#0x3_authenticator">authenticator</a>.payload)
}
</code></pre>



</details>

<a name="0x3_authenticator_secp256k1_signature"></a>

## Function `secp256k1_signature`



<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_secp256k1_signature">secp256k1_signature</a>(self: &<a href="authenticator.md#0x3_authenticator_Secp256k1Authenticator">authenticator::Secp256k1Authenticator</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="authenticator.md#0x3_authenticator_secp256k1_signature">secp256k1_signature</a>(self: &<a href="authenticator.md#0x3_authenticator_Secp256k1Authenticator">Secp256k1Authenticator</a>): <a href="">vector</a>&lt;u8&gt; {
   self.signature
}
</code></pre>



</details>
