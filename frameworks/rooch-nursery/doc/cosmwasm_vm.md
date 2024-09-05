
<a name="0xa_cosmwasm_vm"></a>

# Module `0xa::cosmwasm_vm`



-  [Struct `Instance`](#0xa_cosmwasm_vm_Instance)
-  [Function `from_code`](#0xa_cosmwasm_vm_from_code)
-  [Function `call_instantiate`](#0xa_cosmwasm_vm_call_instantiate)
-  [Function `call_execute`](#0xa_cosmwasm_vm_call_execute)
-  [Function `call_query`](#0xa_cosmwasm_vm_call_query)
-  [Function `call_migrate`](#0xa_cosmwasm_vm_call_migrate)
-  [Function `call_reply`](#0xa_cosmwasm_vm_call_reply)
-  [Function `call_sudo`](#0xa_cosmwasm_vm_call_sudo)
-  [Function `destroy_instance`](#0xa_cosmwasm_vm_destroy_instance)
-  [Function `from_slice`](#0xa_cosmwasm_vm_from_slice)
-  [Function `to_vec`](#0xa_cosmwasm_vm_to_vec)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::features</a>;
<b>use</b> <a href="">0x2::result</a>;
<b>use</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std">0xa::cosmwasm_std</a>;
</code></pre>



<a name="0xa_cosmwasm_vm_Instance"></a>

## Struct `Instance`



<pre><code><b>struct</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">Instance</a>
</code></pre>



<a name="0xa_cosmwasm_vm_from_code"></a>

## Function `from_code`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_from_code">from_code</a>(code: <a href="">vector</a>&lt;u8&gt;): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_call_instantiate"></a>

## Function `call_instantiate`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_call_instantiate">call_instantiate</a>(instance: &<a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>, info: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_MessageInfo">cosmwasm_std::MessageInfo</a>, msg: <a href="">vector</a>&lt;u8&gt;): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_call_execute"></a>

## Function `call_execute`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_call_execute">call_execute</a>(instance: &<a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>, info: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_MessageInfo">cosmwasm_std::MessageInfo</a>, msg: <a href="">vector</a>&lt;u8&gt;): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_call_query"></a>

## Function `call_query`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_call_query">call_query</a>(instance: &<a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>, msg: <a href="">vector</a>&lt;u8&gt;): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_call_migrate"></a>

## Function `call_migrate`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_call_migrate">call_migrate</a>(instance: &<a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>, msg: <a href="">vector</a>&lt;u8&gt;): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_call_reply"></a>

## Function `call_reply`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_call_reply">call_reply</a>(instance: &<a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>, msg: <a href="">vector</a>&lt;u8&gt;): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_call_sudo"></a>

## Function `call_sudo`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_call_sudo">call_sudo</a>(instance: &<a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>, msg: <a href="">vector</a>&lt;u8&gt;): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_destroy_instance"></a>

## Function `destroy_instance`

Destroys an Instance and releases associated resources.


<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_destroy_instance">destroy_instance</a>(instance: <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>): <a href="_Option">option::Option</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_from_slice"></a>

## Function `from_slice`

Deserialize a slice of bytes into the given type T.
This function mimics the behavior of cosmwasm_vm::from_slice.


<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_from_slice">from_slice</a>&lt;T&gt;(_data: <a href="">vector</a>&lt;u8&gt;): <a href="_Result">result::Result</a>&lt;T, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_to_vec"></a>

## Function `to_vec`

Serialize the given data to a vector of bytes.
This function mimics the behavior of cosmwasm_vm::to_vec.


<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_to_vec">to_vec</a>&lt;T&gt;(_data: &T): <a href="_Result">result::Result</a>&lt;<a href="">vector</a>&lt;u8&gt;, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>
