
<a name="0xa_cosmwasm_vm"></a>

# Module `0xa::cosmwasm_vm`



-  [Resource `Instance`](#0xa_cosmwasm_vm_Instance)
-  [Function `code_checksum`](#0xa_cosmwasm_vm_code_checksum)
-  [Function `store`](#0xa_cosmwasm_vm_store)
-  [Function `from_code`](#0xa_cosmwasm_vm_from_code)
-  [Function `call_instantiate`](#0xa_cosmwasm_vm_call_instantiate)
-  [Function `call_execute`](#0xa_cosmwasm_vm_call_execute)
-  [Function `call_query`](#0xa_cosmwasm_vm_call_query)
-  [Function `call_migrate`](#0xa_cosmwasm_vm_call_migrate)
-  [Function `call_reply`](#0xa_cosmwasm_vm_call_reply)
-  [Function `call_sudo`](#0xa_cosmwasm_vm_call_sudo)
-  [Function `destroy_instance`](#0xa_cosmwasm_vm_destroy_instance)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::features</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::result</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std">0xa::cosmwasm_std</a>;
</code></pre>



<a name="0xa_cosmwasm_vm_Instance"></a>

## Resource `Instance`



<pre><code><b>struct</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">Instance</a> <b>has</b> store, key
</code></pre>



<a name="0xa_cosmwasm_vm_code_checksum"></a>

## Function `code_checksum`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_code_checksum">code_checksum</a>(instance: &<a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_store"></a>

## Function `store`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_store">store</a>(instance: &<a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>): &<a href="_Table">table::Table</a>&lt;<a href="_String">string::String</a>, <a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_from_code"></a>

## Function `from_code`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_from_code">from_code</a>(code: <a href="">vector</a>&lt;u8&gt;): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_call_instantiate"></a>

## Function `call_instantiate`



<pre><code>#[data_struct(#[T])]
<b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_call_instantiate">call_instantiate</a>&lt;T: drop&gt;(instance: &<b>mut</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>, info: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_MessageInfo">cosmwasm_std::MessageInfo</a>, msg: &T): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_call_execute"></a>

## Function `call_execute`



<pre><code>#[data_struct(#[T])]
<b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_call_execute">call_execute</a>&lt;T: drop&gt;(instance: &<b>mut</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>, info: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_MessageInfo">cosmwasm_std::MessageInfo</a>, msg: &T): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_call_query"></a>

## Function `call_query`



<pre><code>#[data_struct(#[T])]
<b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_call_query">call_query</a>&lt;T: drop&gt;(instance: &<a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>, msg: &T): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_call_migrate"></a>

## Function `call_migrate`



<pre><code>#[data_struct(#[T])]
<b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_call_migrate">call_migrate</a>&lt;T: drop&gt;(instance: &<b>mut</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>, msg: &T): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_call_reply"></a>

## Function `call_reply`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_call_reply">call_reply</a>(instance: &<b>mut</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>, reply: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Reply">cosmwasm_std::Reply</a>): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_call_sudo"></a>

## Function `call_sudo`



<pre><code>#[data_struct(#[T])]
<b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_call_sudo">call_sudo</a>&lt;T: drop&gt;(instance: &<b>mut</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>, env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>, msg: &T): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_vm_destroy_instance"></a>

## Function `destroy_instance`

Destroys an Instance and releases associated resources.


<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_destroy_instance">destroy_instance</a>(instance: <a href="cosmwasm_vm.md#0xa_cosmwasm_vm_Instance">cosmwasm_vm::Instance</a>): <a href="_Option">option::Option</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>
