// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::cosmwasm_std {
    use std::vector;
    use std::string::String;
    use std::option::Option;

    use moveos_std::result::{Result, err};

    // Basic types
    struct Coin {
        denom: String,
        amount: u128,
    }

    struct Addr {
        address: String,
    }

    // Environment information
    struct BlockInfo {
        height: u64,
        time: u64,
        chain_id: String,
    }

    struct TransactionInfo {
        index: u64,
    }

    struct ContractInfo {
        address: Addr,
    }

    struct Env {
        block: BlockInfo,
        contract: ContractInfo,
        transaction: Option<TransactionInfo>,
    }

    struct MessageInfo {
        sender: Addr,
        funds: vector<Coin>,
    }

    // Response types
    struct Attribute {
        key: String,
        value: String,
    }

    struct Event {
        ty: String,
        attributes: vector<Attribute>
    }

    struct Response {
        messages: vector<SubMsg>,
        attributes: vector<Attribute>,
        events: vector<Event>,
        data: vector<u8>
    }

    struct SubMsg {
        id: u64,
        msg: vector<u8>,
        gas_limit: Option<u64>,
        reply_on: ReplyOn,
    }

    struct Error has store, copy, drop {
        code: u32,
        message: String,
    }

    // Enums
    struct ReplyOn {
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

    public fun new_addr(address: String): Addr {
        Addr { address }
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

    // Helper functions

    public fun addr_to_string(addr: &Addr): String {
        addr.address
    }

    public fun string_to_addr(s: String): Addr {
        Addr { address: s }
    }


    // Helper functions (these would need to be implemented)
    public fun serialize_env(_env: &Env): vector<u8> {
        // Implementation to serialize Env struct to bytes
        vector::empty<u8>() // Placeholder
    }

    public fun serialize_message_info(_info: &MessageInfo): vector<u8> {
        // Implementation to serialize MessageInfo struct to bytes
        vector::empty<u8>() // Placeholder
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

}