
<a name="0x2_move_module"></a>

# Module `0x2::move_module`

<code><a href="move_module.md#0x2_move_module">move_module</a></code> provides some basic functions for handle Move module in Move.


-  [Struct `MoveModule`](#0x2_move_module_MoveModule)
-  [Function `new`](#0x2_move_module_new)
-  [Function `module_name`](#0x2_move_module_module_name)
-  [Function `verify_modules`](#0x2_move_module_verify_modules)


<pre><code><b>use</b> <a href="">0x1::string</a>;
</code></pre>



<a name="0x2_move_module_MoveModule"></a>

## Struct `MoveModule`



<pre><code><b>struct</b> <a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>byte_codes: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x2_move_module_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_new">new</a>(byte_codes: <a href="">vector</a>&lt;u8&gt;): <a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_new">new</a>(byte_codes: <a href="">vector</a>&lt;u8&gt;) : <a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a> {
    <a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a> {
        byte_codes,
    }
}
</code></pre>



</details>

<a name="0x2_move_module_module_name"></a>

## Function `module_name`



<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_module_name">module_name</a>(<a href="move_module.md#0x2_move_module">move_module</a>: &<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_module_name">module_name</a>(<a href="move_module.md#0x2_move_module">move_module</a>: &<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>): String {
    //TODO implement <b>native</b> <b>module</b> name
    <a href="move_module.md#0x2_move_module_module_name_inner">module_name_inner</a>(&<a href="move_module.md#0x2_move_module">move_module</a>.byte_codes)
}
</code></pre>



</details>

<a name="0x2_move_module_verify_modules"></a>

## Function `verify_modules`



<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_verify_modules">verify_modules</a>(modules: &<a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, account_address: <b>address</b>): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_verify_modules">verify_modules</a>(modules: &<a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>&gt;, account_address: <b>address</b>): <a href="">vector</a>&lt;String&gt; {
    <b>let</b> bytes_vec = <a href="_empty">vector::empty</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;();
    <b>let</b> i = 0u64;
    <b>let</b> len = <a href="_length">vector::length</a>(modules);
    <b>while</b> (i &lt; len) {
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> bytes_vec, <a href="_borrow">vector::borrow</a>(modules, i).byte_codes);
        i = i + 1;
    };
    <a href="move_module.md#0x2_move_module_verify_modules_inner">verify_modules_inner</a>(bytes_vec, account_address)
}
</code></pre>



</details>
