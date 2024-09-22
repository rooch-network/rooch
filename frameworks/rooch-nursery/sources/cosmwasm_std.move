// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::cosmwasm_std {
    use std::vector;
    use std::string::String;
    use std::option::Option;

    use moveos_std::result::{Result, err};
    use moveos_std::json;
    use moveos_std::timestamp;
    use moveos_std::tx_context;

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
        time: u64,
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
    struct Reply has store, copy, drop {
        id: u64,
        payload: vector<u8>,
        gas_used: u64,
        //result: SubMsgResult, TOOD support SubMsgResult
    }

    // Enums
    #[data_struct]
    struct ReplyOn has store, copy, drop {
        value: u8,
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

    public fun new_reply(id: u64, payload: vector<u8>, gas_used: u64): Reply {
        Reply {
            id: id,
            payload: payload,
            gas_used: gas_used,
        }
    }

    // Helper functions

    // Helper functions (these would need to be implemented)
    public fun serialize_env(env: &Env): vector<u8> {
        json::to_json(env)
    }

    public fun serialize_message_info(info: &MessageInfo): vector<u8> {
        json::to_json(info)
    }

    public fun deserialize_response(raw: vector<u8>): Response {
        // Implementation to deserialize bytes to Response struct
        Response { 
            messages: vector::empty(), 
            attributes: vector::empty(), 
            events: vector::empty(), 
            data: raw 
        } // Placeholder
    }

    public fun deserialize_error(raw: vector<u8>): String {
        // Implementation to deserialize bytes to Error string
        std::string::utf8(raw) // Placeholder
    }

    public fun error_code_to_string(_code: u64): String {
        // Implementation to convert error code to string
        std::string::utf8(vector::empty()) // Placeholder
    }

    public fun current_env(): Env {
        let sender = tx_context::sender();
        let sequence_number = tx_context::sequence_number();
        let tx_hash = tx_context::tx_hash();

        Env {
            block: BlockInfo {
                height: sequence_number,
                time: timestamp::now_milliseconds(),
                chain_id: std::string::utf8(b"rooch"),
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