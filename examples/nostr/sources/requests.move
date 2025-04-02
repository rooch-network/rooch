// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Third party HTTP/HTTPS API request calls through oracle
module nostr::requests {
    use moveos_std::json;
    use nostr::event::{Self, Event};
    use verity::oracles;

    // oracles related
    const ORACLE_ADDRESS: address = @0x694cbe655b126e9e6a997e86aaab39e538abf30a8c78669ce23a98740b47b65d;

    // Default gas allocation for notification callbacks 0.6 RGas
    const DEFAULT_NOTIFICATION_GAS: u256 = 60000000;
    const DEFAULT_ORACLE_FEE: u256 = 3200000000;

    // http/https requests related
    const NOSTR_PROXY_ORACLE_URL: vector<u8> = b"https://nostrhttp.com";
    const NOSTR_RELAY_ORACLE_URL: vector<u8> = b"https://api.nostr.watch/v1/nip";
    const NOSTR_ORACLE_HEADERS: vector<u8> = b"{}";
    const NOSTR_ORACLE_METHOD: vector<u8> = b"POST";
    const NOSTR_PATH_PICK: vector<u8> = b".";

    // subpaths for the nostr proxy oracle url
    const STATUS_PATH: vector<u8> = b"status";
    const EVENT_PATH: vector<u8> = b"event";
    const FEED_PATH: vector<u8> = b"feed";

    // subpaths for the nostr relay orcale url
    const NIP_VERSION: u8 = 1;

    // http/https responses related
    const MAX_RESPONSE_LENGTH: u64 = 65536;

    #[data_struct]
    struct EventRequest {
        id: String,
        pubkey: String,
        created_at: u64,
        kind: u16,
        tags: vector<vector<String>>,
        content: String,
        sig: String,
    }

    #[data_struct]
    struct EventRequestWithRelays {
        id: String,
        pubkey: String,
        created_at: u64,
        kind: u16,
        tags: vector<vector<String>>,
        content: String,
        sig: String,
        relays: vector<String>
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
        }
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
        }
        event_request_with_relays
    }

    public fun event_request_json(event_request: &EventRequest): vector<u8> {
        json::to_json(event_request)
    }

    public fun event_request_with_relays_json(event_request_with_relays: &EventRequestWithRelays): vector<u8> {
        json::to_json(event_request_with_relays)
    }
}
