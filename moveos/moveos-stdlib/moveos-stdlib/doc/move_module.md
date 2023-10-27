
<a name="0x2_move_module"></a>

# Module `0x2::move_module`

<code><a href="move_module.md#0x2_move_module">move_module</a></code> provides some basic functions for handle Move module in Move.


-  [Struct `MoveModule`](#0x2_move_module_MoveModule)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x2_move_module_new)
-  [Function `module_name`](#0x2_move_module_module_name)
-  [Function `sort_and_verify_modules`](#0x2_move_module_sort_and_verify_modules)
-  [Function `check_comatibility`](#0x2_move_module_check_comatibility)
-  [Function `binding_module_address`](#0x2_move_module_binding_module_address)
-  [Function `replace_module_identiner`](#0x2_move_module_replace_module_identiner)
-  [Function `replace_struct_identifier`](#0x2_move_module_replace_struct_identifier)
-  [Function `request_init_functions`](#0x2_move_module_request_init_functions)
-  [Function `replace_address_identifiers`](#0x2_move_module_replace_address_identifiers)
-  [Function `replace_identifiers`](#0x2_move_module_replace_identifiers)
-  [Function `replace_addresses_constant`](#0x2_move_module_replace_addresses_constant)
-  [Function `replace_bytes_constant`](#0x2_move_module_replace_bytes_constant)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
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



<a name="0x2_move_module_ErrorLengthNotMatch"></a>

Vector length not match


<pre><code><b>const</b> <a href="move_module.md#0x2_move_module_ErrorLengthNotMatch">ErrorLengthNotMatch</a>: u64 = 4;
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

<a name="0x2_move_module_binding_module_address"></a>

## Function `binding_module_address`

Binding given module's address to the new address


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_binding_module_address">binding_module_address</a>(modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, old_address: <b>address</b>, new_address: <b>address</b>): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_binding_module_address">binding_module_address</a>(
    modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>&gt;,
    old_address: <b>address</b>,
    new_address: <b>address</b>,
): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>&gt; {
    <b>let</b> bytes_vec = <a href="_empty">vector::empty</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;();
    <b>let</b> i = 0u64;
    <b>let</b> len = <a href="_length">vector::length</a>(&modules);
    <b>while</b> (i &lt; len) {
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> bytes_vec, <a href="_pop_back">vector::pop_back</a>(&<b>mut</b> modules).byte_codes);
        i = i + 1;
    };
    <b>let</b> old_addresses = <a href="_singleton">vector::singleton</a>(old_address);
    <b>let</b> new_addresses = <a href="_singleton">vector::singleton</a>(new_address);

    <b>let</b> rebinded_bytes = <a href="move_module.md#0x2_move_module_replace_address_identifiers">replace_address_identifiers</a>(bytes_vec, old_addresses, new_addresses);
    <b>let</b> rebinded_bytes = <a href="move_module.md#0x2_move_module_replace_addresses_constant">replace_addresses_constant</a>(rebinded_bytes, old_addresses, new_addresses);
    <b>let</b> rebinded_modules = <a href="_empty">vector::empty</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>&gt;();
    i = 0u64;
    <b>let</b> len = <a href="_length">vector::length</a>(&rebinded_bytes);
    <b>while</b> (i &lt; len) {
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> rebinded_modules, <a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a> {
            byte_codes: <a href="_pop_back">vector::pop_back</a>(&<b>mut</b> rebinded_bytes),
        });
        i = i + 1;
    };
    <a href="_destroy_empty">vector::destroy_empty</a>(rebinded_bytes);
    rebinded_modules
}
</code></pre>



</details>

<a name="0x2_move_module_replace_module_identiner"></a>

## Function `replace_module_identiner`

Replace given module's identifier to the new ones


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_replace_module_identiner">replace_module_identiner</a>(modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, old_names: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, new_names: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_replace_module_identiner">replace_module_identiner</a> (
    modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>&gt;,
    old_names: <a href="">vector</a>&lt;String&gt;,
    new_names: <a href="">vector</a>&lt;String&gt;,
): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>&gt; {
    <b>assert</b>!(
        <a href="_length">vector::length</a>(&old_names) == <a href="_length">vector::length</a>(&new_names),
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="move_module.md#0x2_move_module_ErrorLengthNotMatch">ErrorLengthNotMatch</a>)
    );
    <b>let</b> bytes_vec = <a href="_empty">vector::empty</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;();
    <b>let</b> i = 0u64;
    <b>let</b> len = <a href="_length">vector::length</a>(&modules);
    <b>while</b> (i &lt; len) {
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> bytes_vec, <a href="_pop_back">vector::pop_back</a>(&<b>mut</b> modules).byte_codes);
        i = i + 1;
    };

    <b>let</b> rebinded_bytes = <a href="move_module.md#0x2_move_module_replace_identifiers">replace_identifiers</a>(bytes_vec, old_names, new_names);
    <b>let</b> rebinded_modules = <a href="_empty">vector::empty</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>&gt;();
    i = 0u64;
    <b>let</b> len = <a href="_length">vector::length</a>(&rebinded_bytes);
    <b>while</b> (i &lt; len) {
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> rebinded_modules, <a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a> {
            byte_codes: <a href="_pop_back">vector::pop_back</a>(&<b>mut</b> rebinded_bytes),
        });
        i = i + 1;
    };
    <a href="_destroy_empty">vector::destroy_empty</a>(rebinded_bytes);
    rebinded_modules
}
</code></pre>



</details>

<a name="0x2_move_module_replace_struct_identifier"></a>

## Function `replace_struct_identifier`

Replace given struct's identifier to the new ones


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_replace_struct_identifier">replace_struct_identifier</a>(modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, old_names: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, new_names: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_replace_struct_identifier">replace_struct_identifier</a>(
    modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>&gt;,
    old_names: <a href="">vector</a>&lt;String&gt;,
    new_names: <a href="">vector</a>&lt;String&gt;,
): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a>&gt; {
    <a href="move_module.md#0x2_move_module_replace_module_identiner">replace_module_identiner</a>(modules, old_names, new_names)
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

<a name="0x2_move_module_replace_address_identifiers"></a>

## Function `replace_address_identifiers`

Native function to replace addresses identifier in module binary where the length of
<code>old_addresses</code> must equal to that of <code>new_addresses</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_address_identifiers">replace_address_identifiers</a>(bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, old_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;, new_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_address_identifiers">replace_address_identifiers</a>(
    bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;,
    old_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;,
    new_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;,
): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;;
</code></pre>



</details>

<a name="0x2_move_module_replace_identifiers"></a>

## Function `replace_identifiers`

Native function to replace the name identifier <code>old_name</code> to <code>new_name</code> in module binary.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_identifiers">replace_identifiers</a>(bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, old_idents: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, new_idents: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_identifiers">replace_identifiers</a>(
    bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;,
    old_idents: <a href="">vector</a>&lt;String&gt;,
    new_idents: <a href="">vector</a>&lt;String&gt;,
): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;;
</code></pre>



</details>

<a name="0x2_move_module_replace_addresses_constant"></a>

## Function `replace_addresses_constant`

Native function to replace constant addresses in module binary where the length of
<code>old_addresses</code> must equal to that of <code>new_addresses</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_addresses_constant">replace_addresses_constant</a>(bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, old_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;, new_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_addresses_constant">replace_addresses_constant</a>(
    bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;,
    old_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;,
    new_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;,
): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;;
</code></pre>



</details>

<a name="0x2_move_module_replace_bytes_constant"></a>

## Function `replace_bytes_constant`

Native function to replace constant bytes in module binary where the length of
<code>old_bytes</code> must equal to that of <code>new_bytes</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_bytes_constant">replace_bytes_constant</a>(bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, old_bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, new_bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_bytes_constant">replace_bytes_constant</a>(
    bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;,
    old_bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;,
    new_bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;,
): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;;
</code></pre>



</details>
