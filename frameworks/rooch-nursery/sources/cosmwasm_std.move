// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::cosmwasm_std {
    use std::vector;
    use std::string::{Self, String};
    use std::option::{Self, Option};

    use moveos_std::base64;
    use moveos_std::json;
    use moveos_std::timestamp;
    use moveos_std::tx_context;
    use moveos_std::result::{Result, err, ok};

    use rooch_framework::chain_id;

    // Error codes

    /// This error code is returned when a deserialization error occurs.
    const ErrorDeserialize: u32 = 1;

    // Basic types
    #[data_struct]
    struct Coin has store, copy, drop {
        denom: String,
        amount: u128,
    }

    // Environment information
    #[data_struct]
    struct BlockInfo has store, copy, drop {
        height: u64,
        time: u128,
        chain_id: String,
    }

    #[data_struct]
    struct TransactionInfo has store, copy, drop {
        index: u64,
    }

    #[data_struct]
    struct ContractInfo has store, copy, drop {
        address: address,
    }

    #[data_struct]
    struct Env has store, copy, drop {
        block: BlockInfo,
        contract: ContractInfo,
        transaction: Option<TransactionInfo>,
    }

    #[data_struct]
    struct MessageInfo has store, copy, drop {
        sender: address,
        funds: vector<Coin>,
    }

    // Response types
    #[data_struct]
    struct Attribute has store, copy, drop {
        key: String,
        value: String,
    }

    #[data_struct]
    struct Event has store, copy, drop {
        ty: String,
        attributes: vector<Attribute>
    }

    #[data_struct]
    struct Response has store, copy, drop {
        messages: vector<SubMsg>,
        attributes: vector<Attribute>,
        events: vector<Event>,
        data: vector<u8>
    }

    #[data_struct]
    struct SubMsg has store, copy, drop {
        id: u64,
        msg: vector<u8>,
        gas_limit: Option<u64>,
        reply_on: ReplyOn,
    }

    #[data_struct]
    struct Error has store, copy, drop {
        code: u32,
        message: String,
    }

    #[data_struct]
    struct MsgResponse has store, copy, drop {
        type_url: String,
        value: vector<u8>,
    }

    #[data_struct]
    struct SubMsgResponse has store, copy, drop {
        events: vector<Event>,
        msg_responses: vector<MsgResponse>,
    }

    #[data_struct]
    struct SubMsgResult has store, copy, drop {
        ok: Option<SubMsgResponse>,
        err: Option<String>,
    }

    #[data_struct]
    struct Reply has store, copy, drop {
        id: u64,
        payload: String,
        gas_used: u64,
        result: SubMsgResult,
    }

    // Enums
    #[data_struct]
    struct ReplyOn has store, copy, drop {
        value: u8,
    }

    #[data_struct]
    struct StdResult has copy, drop {
        ok: Option<Response>,
        error: Option<String>,
    }

    // Constants for ReplyOn
    const REPLY_ON_SUCCESS: u8 = 1;
    const REPLY_ON_ERROR: u8 = 2;
    const REPLY_ALWAYS: u8 = 3;

    // Functions
    public fun new_response(): Response {
        Response {
            messages: vector::empty(),
            attributes: vector::empty(),
            events: vector::empty(),
            data: vector::empty(),
        }
    }

    public fun new_sub_msg_response(): SubMsgResult {
        SubMsgResult{
            ok: option::some(
                SubMsgResponse {
                    events: vector::empty(),
                    msg_responses: vector::empty(),
                }
            ),
            err: option::none(),
        }
    }

    public fun new_sub_msg_error(err: String): SubMsgResult {
        SubMsgResult{
            ok: option::none(),
            err: option::some(err),
        }
    }

    public fun add_attribute(response: &mut Response, key: String, value: String) {
        vector::push_back(&mut response.attributes, Attribute { key, value });
    }

    public fun add_event(response: &mut Response, event: Event) {
        vector::push_back(&mut response.events, event);
    }

    public fun set_data(response: &mut Response, data: vector<u8>) {
        response.data = data;
    }

    public fun add_message(response: &mut Response, msg: SubMsg) {
        vector::push_back(&mut response.messages, msg);
    }

    public fun new_coin(denom: String, amount: u128): Coin {
        Coin { denom, amount }
    }

    public fun new_sub_msg(id: u64, msg: vector<u8>, gas_limit: Option<u64>, reply_on: u8): SubMsg {
        SubMsg {
            id,
            msg,
            gas_limit,
            reply_on: ReplyOn { value: reply_on },
        }
    }

    public fun new_error(code: u32, message: String): Error {
        Error { code, message }
    }

    public fun new_error_result<T>(code: u32, message: String): Result<T, Error> {
        err(new_error(code, message))
    }

    public fun new_reply(id: u64, payload: String, gas_used: u64, result: SubMsgResult): Reply {
        Reply {
            id: id,
            payload: payload,
            gas_used: gas_used,
            result: result,
        }
    }

    // Helper functions
    public fun serialize_env(env: &Env): vector<u8> {
        json::to_json(env)
    }

    public fun serialize_message_info(info: &MessageInfo): vector<u8> {
        json::to_json(info)
    }

    public fun serialize_message<T: drop>(msg: &T): vector<u8> {
        json::to_json(msg)
    }

    public fun deserialize_stdresult(raw: vector<u8>): Result<Response, Error> {
        let result_option = json::from_json_option<StdResult>(raw);
        if (option::is_none(&result_option)) {
            return new_error_result(ErrorDeserialize, string::utf8(b"deserialize_response_error"))
        };

        let std_result = option::extract(&mut result_option);
        if (option::is_some(&std_result.ok)) {
            ok(option::extract(&mut std_result.ok))
        } else {
            err(new_error(1, option::extract(&mut std_result.error)))
        }
    }

    public fun new_binary(data: vector<u8>): String {
        let encode_bytes = base64::encode(&data);
        string::utf8(encode_bytes)
    }

    public fun current_chain(): vector<u8> {
        if (chain_id::is_main()) {
            b"rooch_main"
        } else if (chain_id::is_test()) {
            b"rooch_test"
        } else if (chain_id::is_dev()) {
            b"rooch_dev"
        } else {
            b"rooch_local"
        }
    }

    public fun current_env(): Env {
        let sender = tx_context::sender();
        let sequence_number = tx_context::sequence_number();

        Env {
            block: BlockInfo {
                height: sequence_number,
                time: (timestamp::now_milliseconds() as u128) * 1000000u128, // nanos
                chain_id: std::string::utf8(current_chain()),
            },
            contract: ContractInfo {
                address: sender,
            },
            transaction: std::option::some(TransactionInfo {
                index: 0, 
            }),
        }
    }

    public fun current_message_info(): MessageInfo {
        let sender = tx_context::sender();
        
        MessageInfo {
            sender: sender,
            funds: vector::empty(), 
        }
    }
}