
<a name="0xa_cosmwasm_std"></a>

# Module `0xa::cosmwasm_std`



-  [Struct `Coin`](#0xa_cosmwasm_std_Coin)
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
-  [Struct `MsgResponse`](#0xa_cosmwasm_std_MsgResponse)
-  [Struct `SubMsgResponse`](#0xa_cosmwasm_std_SubMsgResponse)
-  [Struct `SubMsgResult`](#0xa_cosmwasm_std_SubMsgResult)
-  [Struct `Reply`](#0xa_cosmwasm_std_Reply)
-  [Struct `ReplyOn`](#0xa_cosmwasm_std_ReplyOn)
-  [Struct `StdResult`](#0xa_cosmwasm_std_StdResult)
-  [Constants](#@Constants_0)
-  [Function `new_response`](#0xa_cosmwasm_std_new_response)
-  [Function `new_sub_msg_response`](#0xa_cosmwasm_std_new_sub_msg_response)
-  [Function `new_sub_msg_error`](#0xa_cosmwasm_std_new_sub_msg_error)
-  [Function `add_attribute`](#0xa_cosmwasm_std_add_attribute)
-  [Function `add_event`](#0xa_cosmwasm_std_add_event)
-  [Function `set_data`](#0xa_cosmwasm_std_set_data)
-  [Function `add_message`](#0xa_cosmwasm_std_add_message)
-  [Function `new_coin`](#0xa_cosmwasm_std_new_coin)
-  [Function `new_sub_msg`](#0xa_cosmwasm_std_new_sub_msg)
-  [Function `new_error`](#0xa_cosmwasm_std_new_error)
-  [Function `new_error_result`](#0xa_cosmwasm_std_new_error_result)
-  [Function `new_reply`](#0xa_cosmwasm_std_new_reply)
-  [Function `serialize_env`](#0xa_cosmwasm_std_serialize_env)
-  [Function `serialize_message_info`](#0xa_cosmwasm_std_serialize_message_info)
-  [Function `serialize_message`](#0xa_cosmwasm_std_serialize_message)
-  [Function `deserialize_stdresult`](#0xa_cosmwasm_std_deserialize_stdresult)
-  [Function `new_binary`](#0xa_cosmwasm_std_new_binary)
-  [Function `current_chain`](#0xa_cosmwasm_std_current_chain)
-  [Function `current_env`](#0xa_cosmwasm_std_current_env)
-  [Function `current_message_info`](#0xa_cosmwasm_std_current_message_info)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::base64</a>;
<b>use</b> <a href="">0x2::json</a>;
<b>use</b> <a href="">0x2::result</a>;
<b>use</b> <a href="">0x2::timestamp</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="">0x3::chain_id</a>;
</code></pre>



<a name="0xa_cosmwasm_std_Coin"></a>

## Struct `Coin`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Coin">Coin</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_BlockInfo"></a>

## Struct `BlockInfo`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_BlockInfo">BlockInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_TransactionInfo"></a>

## Struct `TransactionInfo`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_TransactionInfo">TransactionInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_ContractInfo"></a>

## Struct `ContractInfo`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_ContractInfo">ContractInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_Env"></a>

## Struct `Env`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">Env</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_MessageInfo"></a>

## Struct `MessageInfo`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_MessageInfo">MessageInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_Attribute"></a>

## Struct `Attribute`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Attribute">Attribute</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_Event"></a>

## Struct `Event`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Event">Event</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_Response"></a>

## Struct `Response`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">Response</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_SubMsg"></a>

## Struct `SubMsg`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_SubMsg">SubMsg</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_Error"></a>

## Struct `Error`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">Error</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_MsgResponse"></a>

## Struct `MsgResponse`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_MsgResponse">MsgResponse</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_SubMsgResponse"></a>

## Struct `SubMsgResponse`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_SubMsgResponse">SubMsgResponse</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_SubMsgResult"></a>

## Struct `SubMsgResult`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_SubMsgResult">SubMsgResult</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_Reply"></a>

## Struct `Reply`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_Reply">Reply</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_ReplyOn"></a>

## Struct `ReplyOn`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_ReplyOn">ReplyOn</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0xa_cosmwasm_std_StdResult"></a>

## Struct `StdResult`



<pre><code>#[data_struct]
<b>struct</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_StdResult">StdResult</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_cosmwasm_std_ErrorDeserialize"></a>

This error code is returned when a deserialization error occurs.


<pre><code><b>const</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_ErrorDeserialize">ErrorDeserialize</a>: u32 = 1;
</code></pre>



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



<a name="0xa_cosmwasm_std_new_sub_msg_response"></a>

## Function `new_sub_msg_response`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_new_sub_msg_response">new_sub_msg_response</a>(): <a href="cosmwasm_std.md#0xa_cosmwasm_std_SubMsgResult">cosmwasm_std::SubMsgResult</a>
</code></pre>



<a name="0xa_cosmwasm_std_new_sub_msg_error"></a>

## Function `new_sub_msg_error`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_new_sub_msg_error">new_sub_msg_error</a>(err: <a href="_String">string::String</a>): <a href="cosmwasm_std.md#0xa_cosmwasm_std_SubMsgResult">cosmwasm_std::SubMsgResult</a>
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



<a name="0xa_cosmwasm_std_new_reply"></a>

## Function `new_reply`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_new_reply">new_reply</a>(id: u64, payload: <a href="_String">string::String</a>, gas_used: u64, <a href="">result</a>: <a href="cosmwasm_std.md#0xa_cosmwasm_std_SubMsgResult">cosmwasm_std::SubMsgResult</a>): <a href="cosmwasm_std.md#0xa_cosmwasm_std_Reply">cosmwasm_std::Reply</a>
</code></pre>



<a name="0xa_cosmwasm_std_serialize_env"></a>

## Function `serialize_env`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_serialize_env">serialize_env</a>(env: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_cosmwasm_std_serialize_message_info"></a>

## Function `serialize_message_info`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_serialize_message_info">serialize_message_info</a>(info: &<a href="cosmwasm_std.md#0xa_cosmwasm_std_MessageInfo">cosmwasm_std::MessageInfo</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_cosmwasm_std_serialize_message"></a>

## Function `serialize_message`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_serialize_message">serialize_message</a>&lt;T: drop&gt;(msg: &T): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_cosmwasm_std_deserialize_stdresult"></a>

## Function `deserialize_stdresult`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_deserialize_stdresult">deserialize_stdresult</a>(raw: <a href="">vector</a>&lt;u8&gt;): <a href="_Result">result::Result</a>&lt;<a href="cosmwasm_std.md#0xa_cosmwasm_std_Response">cosmwasm_std::Response</a>, <a href="cosmwasm_std.md#0xa_cosmwasm_std_Error">cosmwasm_std::Error</a>&gt;
</code></pre>



<a name="0xa_cosmwasm_std_new_binary"></a>

## Function `new_binary`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_new_binary">new_binary</a>(data: <a href="">vector</a>&lt;u8&gt;): <a href="_String">string::String</a>
</code></pre>



<a name="0xa_cosmwasm_std_current_chain"></a>

## Function `current_chain`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_current_chain">current_chain</a>(): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0xa_cosmwasm_std_current_env"></a>

## Function `current_env`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_current_env">current_env</a>(): <a href="cosmwasm_std.md#0xa_cosmwasm_std_Env">cosmwasm_std::Env</a>
</code></pre>



<a name="0xa_cosmwasm_std_current_message_info"></a>

## Function `current_message_info`



<pre><code><b>public</b> <b>fun</b> <a href="cosmwasm_std.md#0xa_cosmwasm_std_current_message_info">current_message_info</a>(): <a href="cosmwasm_std.md#0xa_cosmwasm_std_MessageInfo">cosmwasm_std::MessageInfo</a>
</code></pre>
