
<a name="0xa_btc_script"></a>

# Module `0xa::btc_script`



-  [Function `send_btc`](#0xa_btc_script_send_btc)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x3::bitcoin_address</a>;
<b>use</b> <a href="">0x4::script_buf</a>;
<b>use</b> <a href="">0x4::utxo</a>;
</code></pre>



<a name="0xa_btc_script_send_btc"></a>

## Function `send_btc`



<pre><code><b>public</b> <b>fun</b> <a href="btc_script.md#0xa_btc_script_send_btc">send_btc</a>(<a href="">signer</a>: &<a href="">signer</a>, assets: <a href="">vector</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;, recipient: &<a href="_String">string::String</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>
