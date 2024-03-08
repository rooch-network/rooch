
<a name="0x2_wasm"></a>

# Module `0x2::wasm`



-  [Function `create_wasm_instance`](#0x2_wasm_create_wasm_instance)
-  [Function `create_wasm_args`](#0x2_wasm_create_wasm_args)
-  [Function `execute_wasm_instance`](#0x2_wasm_execute_wasm_instance)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="context.md#0x2_context">0x2::context</a>;
</code></pre>



<a name="0x2_wasm_create_wasm_instance"></a>

## Function `create_wasm_instance`



<pre><code><b>public</b> <b>fun</b> <a href="wasm.md#0x2_wasm_create_wasm_instance">create_wasm_instance</a>(_ctx: &<a href="context.md#0x2_context_Context">context::Context</a>, bytecode: <a href="">vector</a>&lt;u8&gt;): u64
</code></pre>



<a name="0x2_wasm_create_wasm_args"></a>

## Function `create_wasm_args`



<pre><code><b>public</b> <b>fun</b> <a href="wasm.md#0x2_wasm_create_wasm_args">create_wasm_args</a>(instance_id: u64, func_name: <a href="">vector</a>&lt;u8&gt;, args: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <a href="">vector</a>&lt;u32&gt;
</code></pre>



<a name="0x2_wasm_execute_wasm_instance"></a>

## Function `execute_wasm_instance`



<pre><code><b>public</b> <b>fun</b> <a href="wasm.md#0x2_wasm_execute_wasm_instance">execute_wasm_instance</a>(_ctx: &<a href="context.md#0x2_context_Context">context::Context</a>, instance_id: u64, func_name: <a href="_String">string::String</a>, args: <a href="">vector</a>&lt;u32&gt;): bool
</code></pre>
