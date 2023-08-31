
<a name="0x3_gas_price"></a>

# Module `0x3::gas_price`



-  [Function `get_gas_price_per_unit`](#0x3_gas_price_get_gas_price_per_unit)


<pre><code></code></pre>



<a name="0x3_gas_price_get_gas_price_per_unit"></a>

## Function `get_gas_price_per_unit`

Returns the gas price per unit of gas.


<pre><code><b>public</b> <b>fun</b> <a href="gas_price.md#0x3_gas_price_get_gas_price_per_unit">get_gas_price_per_unit</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="gas_price.md#0x3_gas_price_get_gas_price_per_unit">get_gas_price_per_unit</a>(): u64 {
    //TODO we should provide a algorithm <b>to</b> cordanate the gas price based on the network throughput
    <b>return</b> 1
}
</code></pre>



</details>
