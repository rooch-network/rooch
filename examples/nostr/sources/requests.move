// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Third party HTTP/HTTPS API request calls
module nostr::requests {
    use std::string::{Self, String};
    use moveos_std::json;

    // http/https responses related
    const MAX_RESPONSE_LENGTH: u64 = 65536;

    // callback
    const NOTIFY_CALLBACK: vector<u8> = b"callback::publish_response";

    #[data_struct]
    struct EventRequest has copy, drop {
        id: String,
        pubkey: String,
        created_at: u64,
        kind: u16,
        tags: vector<vector<String>>,
        content: String,
        sig: String,
    }

    #[data_struct]
    struct EventRequestWithRelays has copy, drop {
        id: String,
        pubkey: String,
        created_at: u64,
        kind: u16,
        tags: vector<vector<String>>,
        content: String,
        sig: String,
        relays: vector<String>
    }

    public fun max_response_length(): u64 {
        MAX_RESPONSE_LENGTH
    }

    public fun notify_callback_string(): String {
        string::utf8(NOTIFY_CALLBACK)
    }

    /// call https://nostrhttp.com/event with POST request and event request body
    public fun new_event_request(id: String, pubkey: String, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String, sig: String): EventRequest {
        let event_request = EventRequest {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content,
            sig
        };
        event_request
    }

    /// call https://nostrhttp.com/event with POST request and event and relays request body
    public fun new_event_request_with_relays(id: String, pubkey: String, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String, sig: String, relays: vector<String>): EventRequestWithRelays {
        let event_request_with_relays = EventRequestWithRelays {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content,
            sig,
            relays
        };
        event_request_with_relays
    }

    public fun event_request_json(event_request: &EventRequest): vector<u8> {
        json::to_json(event_request)
    }

    public fun event_request_with_relays_json(event_request_with_relays: &EventRequestWithRelays): vector<u8> {
        json::to_json(event_request_with_relays)
    }
}
