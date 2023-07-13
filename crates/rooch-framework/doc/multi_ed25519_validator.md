
<a name="0x3_multi_ed25519_validator"></a>

# Module `0x3::multi_ed25519_validator`

This module implements the multi-ed25519 validator scheme.


-  [Struct `MultiEd25519Validator`](#0x3_multi_ed25519_validator_MultiEd25519Validator)
-  [Constants](#@Constants_0)
-  [Function `validate`](#0x3_multi_ed25519_validator_validate)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
</code></pre>



<a name="0x3_multi_ed25519_validator_MultiEd25519Validator"></a>

## Struct `MultiEd25519Validator`



<pre><code><b>struct</b> <a href="multi_ed25519_validator.md#0x3_multi_ed25519_validator_MultiEd25519Validator">MultiEd25519Validator</a> <b>has</b> store
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


<a name="0x3_multi_ed25519_validator_SCHEME_MULTIED25519"></a>



<pre><code><b>const</b> <a href="multi_ed25519_validator.md#0x3_multi_ed25519_validator_SCHEME_MULTIED25519">SCHEME_MULTIED25519</a>: u64 = 1;
</code></pre>



<a name="0x3_multi_ed25519_validator_validate"></a>

## Function `validate`



<pre><code><b>public</b> <b>fun</b> <a href="multi_ed25519_validator.md#0x3_multi_ed25519_validator_validate">validate</a>(_ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, _payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multi_ed25519_validator.md#0x3_multi_ed25519_validator_validate">validate</a>(_ctx: &StorageContext, _payload: <a href="">vector</a>&lt;u8&gt;){
   //TODO
   <b>abort</b> std::error::not_implemented(1)
}
</code></pre>



</details>
