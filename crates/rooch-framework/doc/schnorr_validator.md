
<a name="0x3_schnorr_validator"></a>

# Module `0x3::schnorr_validator`

This module implements the schnorr validator scheme.


-  [Struct `SchnorrValidator`](#0x3_schnorr_validator_SchnorrValidator)
-  [Constants](#@Constants_0)
-  [Function `scheme`](#0x3_schnorr_validator_scheme)
-  [Function `schnorr_public_key`](#0x3_schnorr_validator_schnorr_public_key)
-  [Function `schnorr_signature`](#0x3_schnorr_validator_schnorr_signature)
-  [Function `schnorr_authentication_key`](#0x3_schnorr_validator_schnorr_authentication_key)
-  [Function `schnorr_public_key_to_address`](#0x3_schnorr_validator_schnorr_public_key_to_address)
-  [Function `get_authentication_key`](#0x3_schnorr_validator_get_authentication_key)
-  [Function `validate`](#0x3_schnorr_validator_validate)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="account_authentication.md#0x3_account_authentication">0x3::account_authentication</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="hash.md#0x3_hash">0x3::hash</a>;
<b>use</b> <a href="schnorr.md#0x3_schnorr">0x3::schnorr</a>;
</code></pre>



<a name="0x3_schnorr_validator_SchnorrValidator"></a>

## Struct `SchnorrValidator`



<pre><code><b>struct</b> <a href="schnorr_validator.md#0x3_schnorr_validator_SchnorrValidator">SchnorrValidator</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_schnorr_validator_KECCAK256"></a>

Hash function name that are valid for verify.


<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_KECCAK256">KECCAK256</a>: u8 = 0;
</code></pre>



<a name="0x3_schnorr_validator_SHA256"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_SHA256">SHA256</a>: u8 = 1;
</code></pre>



<a name="0x3_schnorr_validator_SCHEME_SCHNORR"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_SCHEME_SCHNORR">SCHEME_SCHNORR</a>: u64 = 4;
</code></pre>



<a name="0x3_schnorr_validator_V_SCHNORR_HASH_LENGTH"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_HASH_LENGTH">V_SCHNORR_HASH_LENGTH</a>: u64 = 1;
</code></pre>



<a name="0x3_schnorr_validator_V_SCHNORR_PUBKEY_LENGTH"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_PUBKEY_LENGTH">V_SCHNORR_PUBKEY_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x3_schnorr_validator_V_SCHNORR_SCHEME_LENGTH"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SCHEME_LENGTH">V_SCHNORR_SCHEME_LENGTH</a>: u64 = 1;
</code></pre>



<a name="0x3_schnorr_validator_V_SCHNORR_SIG_LENGTH"></a>



<pre><code><b>const</b> <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SIG_LENGTH">V_SCHNORR_SIG_LENGTH</a>: u64 = 64;
</code></pre>



<a name="0x3_schnorr_validator_scheme"></a>

## Function `scheme`



<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_scheme">scheme</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_scheme">scheme</a>(): u64 {
   <a href="schnorr_validator.md#0x3_schnorr_validator_SCHEME_SCHNORR">SCHEME_SCHNORR</a>
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_schnorr_public_key"></a>

## Function `schnorr_public_key`



<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_public_key">schnorr_public_key</a>(payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_public_key">schnorr_public_key</a>(payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
   <b>let</b> public_key = <a href="_empty">vector::empty</a>&lt;u8&gt;();
   <b>let</b> i = <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SCHEME_LENGTH">V_SCHNORR_SCHEME_LENGTH</a> + <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SIG_LENGTH">V_SCHNORR_SIG_LENGTH</a>;
   <b>while</b> (i &lt; <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SCHEME_LENGTH">V_SCHNORR_SCHEME_LENGTH</a> + <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SIG_LENGTH">V_SCHNORR_SIG_LENGTH</a> + <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_PUBKEY_LENGTH">V_SCHNORR_PUBKEY_LENGTH</a>) {
      <b>let</b> value = <a href="_borrow">vector::borrow</a>(payload, i);
      <a href="_push_back">vector::push_back</a>(&<b>mut</b> public_key, *value);
      i = i + 1;
   };

   public_key
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_schnorr_signature"></a>

## Function `schnorr_signature`



<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_signature">schnorr_signature</a>(payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_signature">schnorr_signature</a>(payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
   <b>let</b> sign = <a href="_empty">vector::empty</a>&lt;u8&gt;();
   <b>let</b> i = <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SCHEME_LENGTH">V_SCHNORR_SCHEME_LENGTH</a>;
   <b>while</b> (i &lt; <a href="schnorr_validator.md#0x3_schnorr_validator_V_SCHNORR_SIG_LENGTH">V_SCHNORR_SIG_LENGTH</a> + 1) {
      <b>let</b> value = <a href="_borrow">vector::borrow</a>(payload, i);
      <a href="_push_back">vector::push_back</a>(&<b>mut</b> sign, *value);
      i = i + 1;
   };

   sign
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_schnorr_authentication_key"></a>

## Function `schnorr_authentication_key`

Get the authentication key of the given authenticator.


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_authentication_key">schnorr_authentication_key</a>(payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_authentication_key">schnorr_authentication_key</a>(payload: &<a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
   <b>let</b> public_key = <a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_public_key">schnorr_public_key</a>(payload);
   <b>let</b> addr = <a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_public_key_to_address">schnorr_public_key_to_address</a>(public_key);
   moveos_std::bcs::to_bytes(&addr)
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_schnorr_public_key_to_address"></a>

## Function `schnorr_public_key_to_address`



<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_public_key_to_address">schnorr_public_key_to_address</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_public_key_to_address">schnorr_public_key_to_address</a>(public_key: <a href="">vector</a>&lt;u8&gt;): <b>address</b> {
   <b>let</b> bytes = <a href="_singleton">vector::singleton</a>((<a href="schnorr_validator.md#0x3_schnorr_validator_SCHEME_SCHNORR">SCHEME_SCHNORR</a> <b>as</b> u8));
   <a href="_append">vector::append</a>(&<b>mut</b> bytes, public_key);
   moveos_std::bcs::to_address(hash::blake2b256(&bytes))
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_get_authentication_key"></a>

## Function `get_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_authentication_key">get_authentication_key</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_get_authentication_key">get_authentication_key</a>(ctx: &StorageContext, addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt; {
   <b>let</b> auth_key_option = <a href="account_authentication.md#0x3_account_authentication_get_authentication_key">account_authentication::get_authentication_key</a>&lt;<a href="schnorr_validator.md#0x3_schnorr_validator_SchnorrValidator">SchnorrValidator</a>&gt;(ctx, addr);
   <b>if</b>(<a href="_is_some">option::is_some</a>(&auth_key_option)){
      <a href="_extract">option::extract</a>(&<b>mut</b> auth_key_option)
   }<b>else</b>{
     //<b>if</b> AuthenticationKey does not exist, <b>return</b> addr <b>as</b> authentication key
     moveos_std::bcs::to_bytes(&addr)
   }
}
</code></pre>



</details>

<a name="0x3_schnorr_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_validate">validate</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="schnorr_validator.md#0x3_schnorr_validator_validate">validate</a>(ctx: &StorageContext, payload: <a href="">vector</a>&lt;u8&gt;){
   // TODO handle non-<a href="ed25519.md#0x3_ed25519">ed25519</a> auth key and <b>address</b> relationship
   // <b>let</b> auth_key = <a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_authentication_key">schnorr_authentication_key</a>(&payload);
   // <b>let</b> auth_key_in_account = <a href="schnorr_validator.md#0x3_schnorr_validator_get_authentication_key">get_authentication_key</a>(ctx, <a href="_sender">storage_context::sender</a>(ctx));
   // <b>assert</b>!(
   //    auth_key_in_account == auth_key,
   //    <a href="auth_validator.md#0x3_auth_validator_error_invalid_account_auth_key">auth_validator::error_invalid_account_auth_key</a>()
   // );
   <b>assert</b>!(
      <a href="schnorr.md#0x3_schnorr_verify">schnorr::verify</a>(
         &<a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_signature">schnorr_signature</a>(&payload),
         &<a href="schnorr_validator.md#0x3_schnorr_validator_schnorr_public_key">schnorr_public_key</a>(&payload),
         &<a href="_tx_hash">storage_context::tx_hash</a>(ctx),
         <a href="schnorr_validator.md#0x3_schnorr_validator_SHA256">SHA256</a>,
      ),
      <a href="auth_validator.md#0x3_auth_validator_error_invalid_account_auth_key">auth_validator::error_invalid_account_auth_key</a>()
   );
}
</code></pre>



</details>
