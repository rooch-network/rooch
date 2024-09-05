// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::cosmwasm_std {
    use std::string::String;
    use std::vector::{Self, Vec};
    use moveos_std::account::Account;
    use moveos_std::object::Object;

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

    public struct Env {
        block: BlockInfo,
        contract: ContractInfo,
        transaction: Option<TransactionInfo>,
    }

    public struct MessageInfo {
        sender: Addr,
        funds: Vec<Coin>,
    }

    // Response types
    public struct Attribute {
        key: String,
        value: String,
    }

    public struct Event {
        ty: String,
        attributes: Vec<Attribute>
    }

    public struct Response {
        messages: Vec<SubMsg>,
        attributes: Vec<Attribute>,
        events: Vec<Event>,
        data: Vec<u8>
    }

    public struct SubMsg {
        id: u64,
        msg: Vec<u8>,
        gas_limit: Option<u64>,
        reply_on: ReplyOn,
    }

    public struct Error {
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

    public fun set_data(response: &mut Response, data: Vec<u8>) {
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

    public fun new_sub_msg(id: u64, msg: Vec<u8>, gas_limit: Option<u64>, reply_on: u8): SubMsg {
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

    // Helper functions

    public fun addr_to_string(addr: &Addr): String {
        addr.address
    }

    public fun string_to_addr(s: String): Addr {
        Addr { address: s }
    }

}