
<a name="0x4_bitseed"></a>

# Module `0x4::bitseed`



-  [Function `inscribe_generate`](#0x4_bitseed_inscribe_generate)


<pre><code><b>use</b> <a href="">0x2::wasm</a>;
<b>use</b> <a href="ord.md#0x4_ord">0x4::ord</a>;
</code></pre>



<a name="0x4_bitseed_inscribe_generate"></a>

## Function `inscribe_generate`



<pre><code><b>public</b> <b>fun</b> <a href="bitseed.md#0x4_bitseed_inscribe_generate">inscribe_generate</a>(wasm_bytes: <a href="">vector</a>&lt;u8&gt;, deploy_args: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, seed: <a href="">vector</a>&lt;u8&gt;, user_input: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>
