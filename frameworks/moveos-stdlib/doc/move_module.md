
<a name="0x2_move_module"></a>

# Module `0x2::move_module`

<code><a href="move_module.md#0x2_move_module">move_module</a></code> wraps module bytes and provides some basic functions for handle Move module in Move.


-  [Struct `MoveModule`](#0x2_move_module_MoveModule)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x2_move_module_new)
-  [Function `new_batch`](#0x2_move_module_new_batch)
-  [Function `into_byte_codes_batch`](#0x2_move_module_into_byte_codes_batch)
-  [Function `module_id`](#0x2_move_module_module_id)
-  [Function `sort_and_verify_modules`](#0x2_move_module_sort_and_verify_modules)
-  [Function `check_comatibility`](#0x2_move_module_check_comatibility)
-  [Function `binding_module_address`](#0x2_move_module_binding_module_address)
-  [Function `replace_module_identiner`](#0x2_move_module_replace_module_identiner)
-  [Function `replace_struct_identifier`](#0x2_move_module_replace_struct_identifier)
-  [Function `replace_constant_string`](#0x2_move_module_replace_constant_string)
-  [Function `replace_constant_address`](#0x2_move_module_replace_constant_address)
-  [Function `replace_constant_u8`](#0x2_move_module_replace_constant_u8)
-  [Function `replace_constant_u64`](#0x2_move_module_replace_constant_u64)
-  [Function `replace_constant_u256`](#0x2_move_module_replace_constant_u256)
-  [Function `module_id_from_name`](#0x2_move_module_module_id_from_name)
-  [Function `sort_and_verify_modules_inner`](#0x2_move_module_sort_and_verify_modules_inner)
-  [Function `request_init_functions`](#0x2_move_module_request_init_functions)
-  [Function `replace_address_identifiers`](#0x2_move_module_replace_address_identifiers)
-  [Function `replace_identifiers`](#0x2_move_module_replace_identifiers)
-  [Function `replace_addresses_constant`](#0x2_move_module_replace_addresses_constant)
-  [Function `replace_bytes_constant`](#0x2_move_module_replace_bytes_constant)
-  [Function `replace_u8_constant`](#0x2_move_module_replace_u8_constant)
-  [Function `replace_u64_constant`](#0x2_move_module_replace_u64_constant)
-  [Function `replace_u256_constant`](#0x2_move_module_replace_u256_constant)


<pre><code><b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="features.md#0x2_features">0x2::features</a>;
</code></pre>



<a name="0x2_move_module_MoveModule"></a>

## Struct `MoveModule`



<pre><code><b>struct</b> <a href="move_module.md#0x2_move_module_MoveModule">MoveModule</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



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



<a name="0x2_move_module_new_batch"></a>

## Function `new_batch`



<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_new_batch">new_batch</a>(byte_codes_batch: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;
</code></pre>



<a name="0x2_move_module_into_byte_codes_batch"></a>

## Function `into_byte_codes_batch`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_into_byte_codes_batch">into_byte_codes_batch</a>(modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_move_module_module_id"></a>

## Function `module_id`



<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_module_id">module_id</a>(<a href="move_module.md#0x2_move_module">move_module</a>: &<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_move_module_sort_and_verify_modules"></a>

## Function `sort_and_verify_modules`

Sort modules by dependency order and then verify.
Return their names and names of the modules with init function if sorted dependency order.
This function will ensure the module's bytecode is valid and the module id is matching the module object address.
Return
1. Module names of all the modules. Order of names is not matching the input, but sorted by module dependency order
2. Module names of the modules with init function.
3. Indices in input modules of each sorted modules.


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_sort_and_verify_modules">sort_and_verify_modules</a>(modules: &<a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, account_address: <b>address</b>): (<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, <a href="">vector</a>&lt;u64&gt;)
</code></pre>



<a name="0x2_move_module_check_comatibility"></a>

## Function `check_comatibility`

Check module compatibility when upgrading
Abort if the new module is not compatible with the old module.


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_check_comatibility">check_comatibility</a>(new_module: &<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>, old_module: &<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>)
</code></pre>



<a name="0x2_move_module_binding_module_address"></a>

## Function `binding_module_address`

Binding given module's address to the new address


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_binding_module_address">binding_module_address</a>(modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, old_address: <b>address</b>, new_address: <b>address</b>): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;
</code></pre>



<a name="0x2_move_module_replace_module_identiner"></a>

## Function `replace_module_identiner`

Replace given module's identifier to the new ones


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_replace_module_identiner">replace_module_identiner</a>(modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, old_names: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, new_names: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;
</code></pre>



<a name="0x2_move_module_replace_struct_identifier"></a>

## Function `replace_struct_identifier`

Replace given struct's identifier to the new ones


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_replace_struct_identifier">replace_struct_identifier</a>(modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, old_names: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, new_names: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;
</code></pre>



<a name="0x2_move_module_replace_constant_string"></a>

## Function `replace_constant_string`

Replace given string constant to the new ones


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_replace_constant_string">replace_constant_string</a>(modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, old_strings: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, new_strings: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;
</code></pre>



<a name="0x2_move_module_replace_constant_address"></a>

## Function `replace_constant_address`

Replace given address constant to the new ones


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_replace_constant_address">replace_constant_address</a>(modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, old_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;, new_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;
</code></pre>



<a name="0x2_move_module_replace_constant_u8"></a>

## Function `replace_constant_u8`

Replace given u8 constant to the new ones


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_replace_constant_u8">replace_constant_u8</a>(modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, old_u8s: <a href="">vector</a>&lt;u8&gt;, new_u8s: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;
</code></pre>



<a name="0x2_move_module_replace_constant_u64"></a>

## Function `replace_constant_u64`

Replace given u64 constant to the new ones


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_replace_constant_u64">replace_constant_u64</a>(modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, old_u64s: <a href="">vector</a>&lt;u64&gt;, new_u64s: <a href="">vector</a>&lt;u64&gt;): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;
</code></pre>



<a name="0x2_move_module_replace_constant_u256"></a>

## Function `replace_constant_u256`

Replace given u256 constant to the new ones


<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_replace_constant_u256">replace_constant_u256</a>(modules: <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;, old_u256s: <a href="">vector</a>&lt;u256&gt;, new_u256s: <a href="">vector</a>&lt;u256&gt;): <a href="">vector</a>&lt;<a href="move_module.md#0x2_move_module_MoveModule">move_module::MoveModule</a>&gt;
</code></pre>



<a name="0x2_move_module_module_id_from_name"></a>

## Function `module_id_from_name`



<pre><code><b>public</b> <b>fun</b> <a href="move_module.md#0x2_move_module_module_id_from_name">module_id_from_name</a>(<a href="account.md#0x2_account">account</a>: <b>address</b>, name: <a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x2_move_module_sort_and_verify_modules_inner"></a>

## Function `sort_and_verify_modules_inner`

Sort modules by dependency order and then verify.
Return
The first vector is the module names of all the modules.
The second vector is the module names of the modules with init function.
The third vector is the indices in input modules of each sorted modules.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_sort_and_verify_modules_inner">sort_and_verify_modules_inner</a>(modules: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, account_address: <b>address</b>): (<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, <a href="">vector</a>&lt;u64&gt;)
</code></pre>



<a name="0x2_move_module_request_init_functions"></a>

## Function `request_init_functions`

Request to call the init functions of the given modules
module_ids: ids of modules which have a init function


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_request_init_functions">request_init_functions</a>(module_ids: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;)
</code></pre>



<a name="0x2_move_module_replace_address_identifiers"></a>

## Function `replace_address_identifiers`

Native function to replace addresses identifier in module binary where the length of
<code>old_addresses</code> must equal to that of <code>new_addresses</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_address_identifiers">replace_address_identifiers</a>(bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, old_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;, new_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_move_module_replace_identifiers"></a>

## Function `replace_identifiers`

Native function to replace the name identifier <code>old_name</code> to <code>new_name</code> in module binary.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_identifiers">replace_identifiers</a>(bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, old_idents: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, new_idents: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_move_module_replace_addresses_constant"></a>

## Function `replace_addresses_constant`

Native function to replace constant addresses in module binary where the length of
<code>old_addresses</code> must equal to that of <code>new_addresses</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_addresses_constant">replace_addresses_constant</a>(bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, old_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;, new_addresses: <a href="">vector</a>&lt;<b>address</b>&gt;): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_move_module_replace_bytes_constant"></a>

## Function `replace_bytes_constant`

Native function to replace constant bytes in module binary where the length of
<code>old_bytes</code> must equal to that of <code>new_bytes</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_bytes_constant">replace_bytes_constant</a>(bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, old_bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, new_bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_move_module_replace_u8_constant"></a>

## Function `replace_u8_constant`

Native function to replace constant u8 in module binary where the length of
<code>old_u8s</code> must equal to that of <code>new_u8s</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_u8_constant">replace_u8_constant</a>(bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, old_u8s: <a href="">vector</a>&lt;u8&gt;, new_u8s: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_move_module_replace_u64_constant"></a>

## Function `replace_u64_constant`

Native function to replace constant u64 in module binary where the length of
<code>old_u64s</code> must equal to that of <code>new_u64s</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_u64_constant">replace_u64_constant</a>(bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, old_u64s: <a href="">vector</a>&lt;u64&gt;, new_u64s: <a href="">vector</a>&lt;u64&gt;): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_move_module_replace_u256_constant"></a>

## Function `replace_u256_constant`

Native function to replace constant u256 in module binary where the length of
<code>old_u256s</code> must equal to that of <code>new_u256s</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="move_module.md#0x2_move_module_replace_u256_constant">replace_u256_constant</a>(bytes: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, old_u256s: <a href="">vector</a>&lt;u256&gt;, new_u256s: <a href="">vector</a>&lt;u256&gt;): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>
