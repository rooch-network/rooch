
<a name="0x2_move_module"></a>

# Module `0x2::move_module`

<code><a href="move_module.md#0x2_move_module">move_module</a></code> provides some basic functions for handle Move module in Move.


-  [Struct `MoveModule`](#0x2_move_module_MoveModule)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x2_move_module_new)
-  [Function `module_name`](#0x2_move_module_module_name)
-  [Function `sort_and_verify_modules`](#0x2_move_module_sort_and_verify_modules)
-  [Function `check_comatibility`](#0x2_move_module_check_comatibility)
-  [Function `request_init_functions`](#0x2_move_module_request_init_functions)


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

<a name="@Constants_0"></a>

## Constants


<a name="0x2_move_module_ErrorAddressNotMatchWithSigner"></a>

Module address is not the same as the signer


<pre><code><b>const</b> <a href="move_module.md#0x2_move_module_ErrorAddressNotMatchWithSigner">ErrorAddressNotMatchWithSigner</a>: u64 = 1;
</code></pre>



<a name="0x2_move_module_ErrorModuleIncompatible"></a>

Module incompatible with the old ones.


<pre><code><b>const</b> <a href="move_module.md#0x2_move_module_ErrorModuleIncompatible">ErrorModuleIncompatible</a>: u64 = 3;
</code></pre>



<a name="0x2_move_module_ErrorModuleVerificationError"></a>

Module verification error


<pre><code><b>const</b> <a href="move_module.md#0x2_move_module_ErrorModuleVerificationError">ErrorModuleVerificationError</a>: u64 = 2;
</code></pre>



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
    <a href="move_module.md#0x2_move_module_module_name_inner">module_name_inner</a>(&<a href="move_module.md#0x2_move_module">move_module</a>.byte_codes)
}
</code></pre>



</details>

<a name="0x2_move_module_sort_and_verify_modules"></a>

## Function `sort_and_verify_modules`

Sort modules by dependency order and then verify.
Return their names and names of the modules with init function if sorted dependency order.
This function will ensure the module's bytecode is valid and the module id is matching the account address.
Return
1. Module names of all the modules. Order of names is not matching the input, but sorted by module dependency order
2. Module names of the modules with init function.


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_sort_and_verify_modules">sort_and_verify_modules</a>(modules: &<a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, account_address: <b>address</b>): (<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_sort_and_verify_modules">sort_and_verify_modules</a>(modules: &<a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>&gt;, account_address: <b>address</b>): (<a href="">vector</a>&lt;String&gt;, <a href="">vector</a>&lt;String&gt;) {
    <b>let</b> bytes_vec = <a href="_empty">vector::empty</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;();
    <b>let</b> i = 0u64;
    <b>let</b> len = <a href="_length">vector::length</a>(modules);
    <b>while</b> (i &lt; len) {
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> bytes_vec, <a href="_borrow">vector::borrow</a>(modules, i).byte_codes);
        i = i + 1;
    };
    <a href="move_module.md#0x2_move_module_sort_and_verify_modules_inner">sort_and_verify_modules_inner</a>(bytes_vec, account_address)
}
</code></pre>



</details>

<a name="0x2_move_module_check_comatibility"></a>

## Function `check_comatibility`

Check module compatibility when upgrading
Abort if the new module is not compatible with the old module.


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_check_comatibility">check_comatibility</a>(new_module: &<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>, old_module: &<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_check_comatibility">check_comatibility</a>(new_module: &<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>, old_module: &<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>) {
    <a href="move_module.md#0x2_move_module_check_compatibililty_inner">check_compatibililty_inner</a>(new_module.byte_codes, old_module.byte_codes);
}
</code></pre>



</details>

<a name="0x2_move_module_request_init_functions"></a>

## Function `request_init_functions`

Request to call the init functions of the given modules
module_names: names of modules which have a init function
account_address: address of all the modules


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_request_init_functions">request_init_functions</a>(module_names: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, account_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_request_init_functions">request_init_functions</a>(module_names: <a href="">vector</a>&lt;String&gt;, account_address: <b>address</b>);
</code></pre>



</details>
