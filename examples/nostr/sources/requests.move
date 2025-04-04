// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Third party HTTP/HTTPS API request calls through oracle
module nostr::requests {
    use std::string::{Self, String};
    use moveos_std::json;

    // oracle address
    const ORACLE_ADDRESS: address = @0x694cbe655b126e9e6a997e86aaab39e538abf30a8c78669ce23a98740b47b65d;

    // Default gas allocation for notification callbacks 0.6 RGas
    const DEFAULT_NOTIFICATION_GAS: u256 = 60000000;
    const DEFAULT_ORACLE_FEE: u256 = 3200000000;

    // http/https requests related
    const NOSTR_ORACLE_URL: vector<u8> = b"https://api.getalby.com/nwc";
    const NOSTR_ORACLE_HEADERS: vector<u8> = b"{}";
    const NOSTR_ORACLE_METHOD: vector<u8> = b"POST";
    const NOSTR_PATH_PICK: vector<u8> = b".";

    // subpaths for the nostr oracle url
    const NIP47_PATH: vector<u8> = b"/nip47";
    const NIP47_INFO_PATH: vector<u8> = b"/nip47/info";
    const NIP47_NOTIFICATIONS_PATH: vector<u8> = b"/nip47/notifications";
    const PUBLISH_PATH: vector<u8> = b"/publish";
    const SUBSCRIPTIONS_PATH: vector<u8> = b"/subscriptions";
    const SUBSCRIPTIONS_ID_PATH: vector<u8> = b"/subscriptions/";

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

    public fun oracle_address(): address {
        ORACLE_ADDRESS
    }

    public fun default_notification_gas(): u256 {
        DEFAULT_NOTIFICATION_GAS
    }

    public fun default_oracle_fee(): u256 {
        DEFAULT_ORACLE_FEE
    }

    public fun nostr_oracle_url_string(): String {
        string::utf8(NOSTR_ORACLE_URL)
    }

    public fun nostr_oracle_headers_string(): String {
        string::utf8(NOSTR_ORACLE_HEADERS)
    }

    public fun nostr_oracle_method_string(): String {
        string::utf8(NOSTR_ORACLE_METHOD)
    }

    public fun nostr_path_pick_string(): String {
        string::utf8(NOSTR_PATH_PICK)
    }

    // TODO: NIP-47 subpaths

    public fun publish_path_string(): String {
        string::utf8(PUBLISH_PATH)
    }

    public fun subscriptions_path_string(): String {
        string::utf8(SUBSCRIPTIONS_PATH)
    }

    public fun subscriptions_id_path_string(): String {
        string::utf8(SUBSCRIPTIONS_ID_PATH)
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
