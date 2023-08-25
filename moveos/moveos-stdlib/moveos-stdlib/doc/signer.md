
<a name="0x2_signer"></a>

# Module `0x2::signer`



-  [Function `module_signer`](#0x2_signer_module_signer)


<pre><code></code></pre>



<a name="0x2_signer_module_signer"></a>

## Function `module_signer`

Returns the signer of the module address of the generic type <code>T</code>.
This is safe because the generic type <code>T</code> is private, meaning it can only be used within the module that defines it.


<pre><code><b>public</b> <b>fun</b> <a href="signer.md#0x2_signer_module_signer">module_signer</a>&lt;T&gt;(): <a href="signer.md#0x2_signer">signer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="signer.md#0x2_signer_module_signer">module_signer</a>&lt;T&gt;(): <a href="signer.md#0x2_signer">signer</a>;
</code></pre>



</details>
