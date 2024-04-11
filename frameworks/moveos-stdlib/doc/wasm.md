
<a name="0x2_wasm"></a>

# Module `0x2::wasm`



-  [Struct `WASMInstance`](#0x2_wasm_WASMInstance)
-  [Function `get_instance_id`](#0x2_wasm_get_instance_id)
-  [Function `create_wasm_instance`](#0x2_wasm_create_wasm_instance)
-  [Function `create_cbor_values`](#0x2_wasm_create_cbor_values)
-  [Function `add_length_with_data`](#0x2_wasm_add_length_with_data)
-  [Function `create_memory_wasm_args`](#0x2_wasm_create_memory_wasm_args)
-  [Function `execute_wasm_function`](#0x2_wasm_execute_wasm_function)
-  [Function `read_data_length`](#0x2_wasm_read_data_length)
-  [Function `read_data_from_heap`](#0x2_wasm_read_data_from_heap)
-  [Function `release_wasm_instance`](#0x2_wasm_release_wasm_instance)


<pre><code></code></pre>



<a name="0x2_wasm_WASMInstance"></a>

## Struct `WASMInstance`



<pre><code><b>struct</b> <a href="wasm.md#0x2_wasm_WASMInstance">WASMInstance</a>
</code></pre>



<a name="0x2_wasm_get_instance_id"></a>

## Function `get_instance_id`



<pre><code><b>public</b> <b>fun</b> <a href="wasm.md#0x2_wasm_get_instance_id">get_instance_id</a>(instance: &<a href="wasm.md#0x2_wasm_WASMInstance">wasm::WASMInstance</a>): u64
</code></pre>



<a name="0x2_wasm_create_wasm_instance"></a>

## Function `create_wasm_instance`



<pre><code><b>public</b> <b>fun</b> <a href="wasm.md#0x2_wasm_create_wasm_instance">create_wasm_instance</a>(bytecode: <a href="">vector</a>&lt;u8&gt;): <a href="wasm.md#0x2_wasm_WASMInstance">wasm::WASMInstance</a>
</code></pre>



<a name="0x2_wasm_create_cbor_values"></a>

## Function `create_cbor_values`



<pre><code><b>public</b> <b>fun</b> <a href="wasm.md#0x2_wasm_create_cbor_values">create_cbor_values</a>(value: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_wasm_add_length_with_data"></a>

## Function `add_length_with_data`



<pre><code><b>public</b> <b>fun</b> <a href="wasm.md#0x2_wasm_add_length_with_data">add_length_with_data</a>(value: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_wasm_create_memory_wasm_args"></a>

## Function `create_memory_wasm_args`



<pre><code><b>public</b> <b>fun</b> <a href="wasm.md#0x2_wasm_create_memory_wasm_args">create_memory_wasm_args</a>(instance_id: u64, func_name: <a href="">vector</a>&lt;u8&gt;, args: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <a href="">vector</a>&lt;u64&gt;
</code></pre>



<a name="0x2_wasm_execute_wasm_function"></a>

## Function `execute_wasm_function`



<pre><code><b>public</b> <b>fun</b> <a href="wasm.md#0x2_wasm_execute_wasm_function">execute_wasm_function</a>(instance_id: u64, func_name: <a href="">vector</a>&lt;u8&gt;, args: <a href="">vector</a>&lt;u64&gt;): u64
</code></pre>



<a name="0x2_wasm_read_data_length"></a>

## Function `read_data_length`



<pre><code><b>public</b> <b>fun</b> <a href="wasm.md#0x2_wasm_read_data_length">read_data_length</a>(instance_id: u64, data_ptr: u64): u32
</code></pre>



<a name="0x2_wasm_read_data_from_heap"></a>

## Function `read_data_from_heap`



<pre><code><b>public</b> <b>fun</b> <a href="wasm.md#0x2_wasm_read_data_from_heap">read_data_from_heap</a>(instance_id: u64, data_ptr: u32, data_length: u32): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_wasm_release_wasm_instance"></a>

## Function `release_wasm_instance`



<pre><code><b>public</b> <b>fun</b> <a href="wasm.md#0x2_wasm_release_wasm_instance">release_wasm_instance</a>(instance: <a href="wasm.md#0x2_wasm_WASMInstance">wasm::WASMInstance</a>): bool
</code></pre>
