
<a name="0x2_tx_meta"></a>

# Module `0x2::tx_meta`



-  [Struct `TxMeta`](#0x2_tx_meta_TxMeta)
-  [Struct `FunctionCallMeta`](#0x2_tx_meta_FunctionCallMeta)
-  [Constants](#@Constants_0)
-  [Function `move_action_script_type`](#0x2_tx_meta_move_action_script_type)
-  [Function `move_action_function_type`](#0x2_tx_meta_move_action_function_type)
-  [Function `move_action_module_bundle_type`](#0x2_tx_meta_move_action_module_bundle_type)
-  [Function `action_type`](#0x2_tx_meta_action_type)
-  [Function `is_script_call`](#0x2_tx_meta_is_script_call)
-  [Function `is_function_call`](#0x2_tx_meta_is_function_call)
-  [Function `is_module_publish`](#0x2_tx_meta_is_module_publish)
-  [Function `function_meta`](#0x2_tx_meta_function_meta)
-  [Function `function_meta_module_address`](#0x2_tx_meta_function_meta_module_address)
-  [Function `function_meta_module_name`](#0x2_tx_meta_function_meta_module_name)
-  [Function `function_meta_function_name`](#0x2_tx_meta_function_meta_function_name)


<pre><code><b>use</b> <a href="">0x1::ascii</a>;
<b>use</b> <a href="">0x1::option</a>;
</code></pre>



<a name="0x2_tx_meta_TxMeta"></a>

## Struct `TxMeta`

The transaction Meta data
We can not define MoveAction in Move, so we define a simple meta data struct to represent it


<pre><code><b>struct</b> <a href="tx_meta.md#0x2_tx_meta_TxMeta">TxMeta</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>action_type: u8</code>
</dt>
<dd>
 The MoveAction type of the current transaction
</dd>
<dt>
<code>function_meta: <a href="_Option">option::Option</a>&lt;<a href="tx_meta.md#0x2_tx_meta_FunctionCallMeta">tx_meta::FunctionCallMeta</a>&gt;</code>
</dt>
<dd>
 If the action type is MoveActionFunctionType, this field is the function call meta data
</dd>
</dl>


</details>

<a name="0x2_tx_meta_FunctionCallMeta"></a>

## Struct `FunctionCallMeta`

The FunctionCall Meta data


<pre><code><b>struct</b> <a href="tx_meta.md#0x2_tx_meta_FunctionCallMeta">FunctionCallMeta</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>module_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>module_name: <a href="_String">ascii::String</a></code>
</dt>
<dd>

</dd>
<dt>
<code>function_name: <a href="_String">ascii::String</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x2_tx_meta_MoveActionFunctionType"></a>



<pre><code><b>const</b> <a href="tx_meta.md#0x2_tx_meta_MoveActionFunctionType">MoveActionFunctionType</a>: u8 = 1;
</code></pre>



<a name="0x2_tx_meta_MoveActionModuleBundleType"></a>



<pre><code><b>const</b> <a href="tx_meta.md#0x2_tx_meta_MoveActionModuleBundleType">MoveActionModuleBundleType</a>: u8 = 2;
</code></pre>



<a name="0x2_tx_meta_MoveActionScriptType"></a>



<pre><code><b>const</b> <a href="tx_meta.md#0x2_tx_meta_MoveActionScriptType">MoveActionScriptType</a>: u8 = 0;
</code></pre>



<a name="0x2_tx_meta_move_action_script_type"></a>

## Function `move_action_script_type`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_move_action_script_type">move_action_script_type</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_move_action_script_type">move_action_script_type</a>(): u8 { <a href="tx_meta.md#0x2_tx_meta_MoveActionScriptType">MoveActionScriptType</a> }
</code></pre>



</details>

<a name="0x2_tx_meta_move_action_function_type"></a>

## Function `move_action_function_type`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_move_action_function_type">move_action_function_type</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_move_action_function_type">move_action_function_type</a>(): u8 { <a href="tx_meta.md#0x2_tx_meta_MoveActionFunctionType">MoveActionFunctionType</a> }
</code></pre>



</details>

<a name="0x2_tx_meta_move_action_module_bundle_type"></a>

## Function `move_action_module_bundle_type`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_move_action_module_bundle_type">move_action_module_bundle_type</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_move_action_module_bundle_type">move_action_module_bundle_type</a>(): u8 { <a href="tx_meta.md#0x2_tx_meta_MoveActionModuleBundleType">MoveActionModuleBundleType</a> }
</code></pre>



</details>

<a name="0x2_tx_meta_action_type"></a>

## Function `action_type`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_action_type">action_type</a>(self: &<a href="tx_meta.md#0x2_tx_meta_TxMeta">tx_meta::TxMeta</a>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_action_type">action_type</a>(self: &<a href="tx_meta.md#0x2_tx_meta_TxMeta">TxMeta</a>): u8 {
    self.action_type
}
</code></pre>



</details>

<a name="0x2_tx_meta_is_script_call"></a>

## Function `is_script_call`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_is_script_call">is_script_call</a>(self: &<a href="tx_meta.md#0x2_tx_meta_TxMeta">tx_meta::TxMeta</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_is_script_call">is_script_call</a>(self: &<a href="tx_meta.md#0x2_tx_meta_TxMeta">TxMeta</a>): bool {
    self.action_type == <a href="tx_meta.md#0x2_tx_meta_MoveActionScriptType">MoveActionScriptType</a>
}
</code></pre>



</details>

<a name="0x2_tx_meta_is_function_call"></a>

## Function `is_function_call`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_is_function_call">is_function_call</a>(self: &<a href="tx_meta.md#0x2_tx_meta_TxMeta">tx_meta::TxMeta</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_is_function_call">is_function_call</a>(self: &<a href="tx_meta.md#0x2_tx_meta_TxMeta">TxMeta</a>): bool {
    self.action_type == <a href="tx_meta.md#0x2_tx_meta_MoveActionFunctionType">MoveActionFunctionType</a>
}
</code></pre>



</details>

<a name="0x2_tx_meta_is_module_publish"></a>

## Function `is_module_publish`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_is_module_publish">is_module_publish</a>(self: &<a href="tx_meta.md#0x2_tx_meta_TxMeta">tx_meta::TxMeta</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_is_module_publish">is_module_publish</a>(self: &<a href="tx_meta.md#0x2_tx_meta_TxMeta">TxMeta</a>): bool {
    self.action_type == <a href="tx_meta.md#0x2_tx_meta_MoveActionModuleBundleType">MoveActionModuleBundleType</a>
}
</code></pre>



</details>

<a name="0x2_tx_meta_function_meta"></a>

## Function `function_meta`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_function_meta">function_meta</a>(self: &<a href="tx_meta.md#0x2_tx_meta_TxMeta">tx_meta::TxMeta</a>): <a href="_Option">option::Option</a>&lt;<a href="tx_meta.md#0x2_tx_meta_FunctionCallMeta">tx_meta::FunctionCallMeta</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_function_meta">function_meta</a>(self: &<a href="tx_meta.md#0x2_tx_meta_TxMeta">TxMeta</a>): Option&lt;<a href="tx_meta.md#0x2_tx_meta_FunctionCallMeta">FunctionCallMeta</a>&gt; {
    *&self.function_meta
}
</code></pre>



</details>

<a name="0x2_tx_meta_function_meta_module_address"></a>

## Function `function_meta_module_address`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_function_meta_module_address">function_meta_module_address</a>(function_meta: &<a href="tx_meta.md#0x2_tx_meta_FunctionCallMeta">tx_meta::FunctionCallMeta</a>): &<b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_function_meta_module_address">function_meta_module_address</a>(function_meta: &<a href="tx_meta.md#0x2_tx_meta_FunctionCallMeta">FunctionCallMeta</a>): &<b>address</b> {
    &function_meta.module_address
}
</code></pre>



</details>

<a name="0x2_tx_meta_function_meta_module_name"></a>

## Function `function_meta_module_name`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_function_meta_module_name">function_meta_module_name</a>(function_meta: &<a href="tx_meta.md#0x2_tx_meta_FunctionCallMeta">tx_meta::FunctionCallMeta</a>): &<a href="_String">ascii::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_function_meta_module_name">function_meta_module_name</a>(function_meta: &<a href="tx_meta.md#0x2_tx_meta_FunctionCallMeta">FunctionCallMeta</a>): &std::ascii::String {
    &function_meta.module_name
}
</code></pre>



</details>

<a name="0x2_tx_meta_function_meta_function_name"></a>

## Function `function_meta_function_name`



<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_function_meta_function_name">function_meta_function_name</a>(function_meta: &<a href="tx_meta.md#0x2_tx_meta_FunctionCallMeta">tx_meta::FunctionCallMeta</a>): &<a href="_String">ascii::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="tx_meta.md#0x2_tx_meta_function_meta_function_name">function_meta_function_name</a>(function_meta: &<a href="tx_meta.md#0x2_tx_meta_FunctionCallMeta">FunctionCallMeta</a>): &std::ascii::String {
    &function_meta.function_name
}
</code></pre>



</details>
