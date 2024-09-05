
<a name="0xa_cosmwasm_std"></a>

# Module `0xa::cosmwasm_std`



-  [Struct `Coin`](#0xa_cosmwasm_std_Coin)
-  [Struct `Addr`](#0xa_cosmwasm_std_Addr)
-  [Struct `BlockInfo`](#0xa_cosmwasm_std_BlockInfo)
-  [Struct `TransactionInfo`](#0xa_cosmwasm_std_TransactionInfo)
-  [Struct `ContractInfo`](#0xa_cosmwasm_std_ContractInfo)
-  [Struct `Env`](#0xa_cosmwasm_std_Env)
-  [Struct `MessageInfo`](#0xa_cosmwasm_std_MessageInfo)
-  [Struct `Attribute`](#0xa_cosmwasm_std_Attribute)
-  [Struct `Event`](#0xa_cosmwasm_std_Event)
-  [Struct `Response`](#0xa_cosmwasm_std_Response)
-  [Struct `SubMsg`](#0xa_cosmwasm_std_SubMsg)
-  [Struct `Error`](#0xa_cosmwasm_std_Error)
-  [Struct `ReplyOn`](#0xa_cosmwasm_std_ReplyOn)
-  [Constants](#@Constants_0)
-  [Function `new_response`](#0xa_cosmwasm_std_new_response)
-  [Function `add_attribute`](#0xa_cosmwasm_std_add_attribute)
-  [Function `add_event`](#0xa_cosmwasm_std_add_event)
-  [Function `set_data`](#0xa_cosmwasm_std_set_data)
-  [Function `add_message`](#0xa_cosmwasm_std_add_message)
-  [Function `new_coin`](#0xa_cosmwasm_std_new_coin)
-  [Function `new_addr`](#0xa_cosmwasm_std_new_addr)
-  [Function `new_sub_msg`](#0xa_cosmwasm_std_new_sub_msg)
-  [Function `new_error`](#0xa_cosmwasm_std_new_error)
-  [Function `new_error_result`](#0xa_cosmwasm_std_new_error_result)
-  [Function `addr_to_string`](#0xa_cosmwasm_std_addr_to_string)
-  [Function `string_to_addr`](#0xa_cosmwasm_std_string_to_addr)
-  [Function `serialize_env`](#0xa_cosmwasm_std_serialize_env)
-  [Function `serialize_message_info`](#0xa_cosmwasm_std_serialize_message_info)
-  [Function `deserialize_response`](#0xa_cosmwasm_std_deserialize_response)
-  [Function `deserialize_error`](#0xa_cosmwasm_std_deserialize_error)
-  [Function `error_code_to_string`](#0xa_cosmwasm_std_error_code_to_string)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::result</a>;
</code></pre>



<a name="0xa_cosmwasm_std_Coin"></a>

## Struct `Coin`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Coin">Coin</a>
</code></pre>



<a name="0xa_cosmwasm_std_Addr"></a>

## Struct `Addr`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Addr">Addr</a>
</code></pre>



<a name="0xa_cosmwasm_std_BlockInfo"></a>

## Struct `BlockInfo`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_BlockInfo">BlockInfo</a>
</code></pre>



<a name="0xa_cosmwasm_std_TransactionInfo"></a>

## Struct `TransactionInfo`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_TransactionInfo">TransactionInfo</a>
</code></pre>



<a name="0xa_cosmwasm_std_ContractInfo"></a>

## Struct `ContractInfo`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_ContractInfo">ContractInfo</a>
</code></pre>



<a name="0xa_cosmwasm_std_Env"></a>

## Struct `Env`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">Env</a>
</code></pre>



<a name="0xa_cosmwasm_std_MessageInfo"></a>

## Struct `MessageInfo`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_MessageInfo">MessageInfo</a>
</code></pre>



<a name="0xa_cosmwasm_std_Attribute"></a>

## Struct `Attribute`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Attribute">Attribute</a>
</code></pre>



<a name="0xa_cosmwasm_std_Event"></a>

## Struct `Event`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Event">Event</a>
</code></pre>



<a name="0xa_cosmwasm_std_Response"></a>

## Struct `Response`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">Response</a>
</code></pre>



<a name="0xa_cosmwasm_std_SubMsg"></a>

## Struct `SubMsg`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_SubMsg">SubMsg</a>
</code></pre>



<a name="0xa_cosmwasm_std_Error"></a>

## Struct `Error`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">Error</a>
</code></pre>



<a name="0xa_cosmwasm_std_ReplyOn"></a>

## Struct `ReplyOn`



<pre><code><b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_ReplyOn">ReplyOn</a>
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_cosmwasm_std_REPLY_ALWAYS"></a>



<pre><code><b>const</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_REPLY_ALWAYS">REPLY_ALWAYS</a>: u8 = 3;
</code></pre>



<a name="0xa_cosmwasm_std_REPLY_ON_ERROR"></a>



<pre><code><b>const</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_REPLY_ON_ERROR">REPLY_ON_ERROR</a>: u8 = 2;
</code></pre>



<a name="0xa_cosmwasm_std_REPLY_ON_SUCCESS"></a>



<pre><code><b>const</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_REPLY_ON_SUCCESS">REPLY_ON_SUCCESS</a>: u8 = 1;
</code></pre>



<a name="0xa_cosmwasm_std_new_response"></a>

## Function `new_response`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_new_response">new_response</a>(): <a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>
</code></pre>



<a name="0xa_cosmwasm_std_add_attribute"></a>

## Function `add_attribute`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_add_attribute">add_attribute</a>(response: &<b>mut</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, key: <a href="_String">string::String</a>, value: <a href="_String">string::String</a>)
</code></pre>



<a name="0xa_cosmwasm_std_add_event"></a>

## Function `add_event`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_add_event">add_event</a>(response: &<b>mut</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="">event</a>: <a href="cosmwasm_std.md#0xa_cosmwasm_std_Event">cosmwasm_std::Event</a>)
</code></pre>



<a name="0xa_cosmwasm_std_set_data"></a>

## Function `set_data`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_set_data">set_data</a>(response: &<b>mut</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, data: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0xa_cosmwasm_std_add_message"></a>

## Function `add_message`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_add_message">add_message</a>(response: &<b>mut</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, msg: <a href="cosmwasm_std.md#0xa_cosmwasm_std_SubMsg">cosmwasm_std::SubMsg</a>)
</code></pre>



<a name="0xa_cosmwasm_std_new_coin"></a>

## Function `new_coin`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_new_coin">new_coin</a>(denom: <a href="_String">string::String</a>, amount: u128): <a href="cosmwasm_std.md#0xa_cosmwasm_std_Coin">cosmwasm_std::Coin</a>
</code></pre>



<a name="0xa_cosmwasm_std_new_addr"></a>

## Function `new_addr`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_new_addr">new_addr</a>(<b>address</b>: <a href="_String">string::String</a>): <a href="cosmwasm_std.md#0xa_cosmwasm_std_Addr">cosmwasm_std::Addr</a>
</code></pre>



<a name="0xa_cosmwasm_std_new_sub_msg"></a>

## Function `new_sub_msg`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_new_sub_msg">new_sub_msg</a>(id: u64, msg: <a href="">vector</a>&lt;u8&gt;, gas_limit: <a href="_Option">option::Option</a>&lt;u64&gt;, reply_on: u8): <a href="cosmwasm_std.md#0xa_cosmwasm_std_SubMsg">cosmwasm_std::SubMsg</a>
</code></pre>



<a name="0xa_cosmwasm_std_new_error"></a>

## Function `new_error`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_new_error">new_error</a>(code: u32, message: <a href="_String">string::String</a>): <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>
</code></pre>



<a name="0xa_cosmwasm_std_new_error_result"></a>

## Function `new_error_result`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_new_error_result">new_error_result</a>&lt;T&gt;(code: u32, message: <a href="_String">string::String</a>): <a href="_Result">result::Result</a>&lt;T, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_std_addr_to_string"></a>

## Function `addr_to_string`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_addr_to_string">addr_to_string</a>(addr: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Addr">cosmwasm_std::Addr</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0xa_cosmwasm_std_string_to_addr"></a>

## Function `string_to_addr`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_string_to_addr">string_to_addr</a>(s: <a href="_String">string::String</a>): <a href="cosmwasm_std.md#0xa_cosmwasm_std_Addr">cosmwasm_std::Addr</a>
</code></pre>



<a name="0xa_cosmwasm_std_serialize_env"></a>

## Function `serialize_env`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_serialize_env">serialize_env</a>(_env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_cosmwasm_std_serialize_message_info"></a>

## Function `serialize_message_info`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_serialize_message_info">serialize_message_info</a>(_info: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_MessageInfo">cosmwasm_std::MessageInfo</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_cosmwasm_std_deserialize_response"></a>

## Function `deserialize_response`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_deserialize_response">deserialize_response</a>(raw: <a href="">vector</a>&lt;u8&gt;): <a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>
</code></pre>



<a name="0xa_cosmwasm_std_deserialize_error"></a>

## Function `deserialize_error`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_deserialize_error">deserialize_error</a>(raw: <a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0xa_cosmwasm_std_error_code_to_string"></a>

## Function `error_code_to_string`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_error_code_to_string">error_code_to_string</a>(_code: u64): <a href="_String">string::String</a>
</code></pre>
