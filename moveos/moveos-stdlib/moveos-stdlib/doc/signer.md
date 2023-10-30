
<a name="0x2_signer"></a>

# Module `0x2::signer`



-  [Function `module_signer`](#0x2_signer_module_signer)
-  [Function `address_of`](#0x2_signer_address_of)


<pre><code><b>use</b> <a href="">0x1::signer</a>;
</code></pre>



<a name="0x2_signer_module_signer"></a>

## Function `module_signer`

Returns the signer of the module address of the generic type <code>T</code>.
This is safe because the generic type <code>T</code> is private, meaning it can only be used within the module that defines it.


<pre><code><b>public</b> <b>fun</b> <a href="signer.md#0x2_signer_module_signer">module_signer</a>&lt;T&gt;(): <a href="">signer</a>
</code></pre>



<a name="0x2_signer_address_of"></a>

## Function `address_of`

Returns the address of the signer.


<pre><code><b>public</b> <b>fun</b> <a href="signer.md#0x2_signer_address_of">address_of</a>(<a href="">signer</a>: &<a href="">signer</a>): <b>address</b>
</code></pre>
