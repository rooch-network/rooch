// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Implements NIP-01 for handling client requests.
/// There's no relay as handlers deal with all client requests and redirect them to third party applications using oracle http/https proxies.
module nostr::handlers {
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::simple_multimap::{Self, SimpleMultiMap};
    use nostr::event::Event;
    use nostr::inner::{Self, Tags};
    use verity::oracles;

    // oracles related
    const ORACLE_ADDRESS: address = @0x694cbe655b126e9e6a997e86aaab39e538abf30a8c78669ce23a98740b47b65d;

    /// Default gas allocation for notification callbacks 0.6 RGas
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

    // types of messages
    const EVENT_KEY: vector<u8> = b"EVENT";
    const REQ_KEY: vector<u8> = b"REQ";
    const CLOSE_KEY: vector<u8> = b"CLOSE";

    // Error codes from 1000 onward
    const ErrorEventRequest: u64 = 1000;
    const ErrorInvalidPublicKeyOwner: u64 = 1001;

    public fun event_key_string(): String {
        string::utf8(EVENT_KEY)
    }

    public fun req_key_string(): String {
        string::utf8(REQ_KEY)
    }

    public fun close_key_string(): String {
        string::utf8(CLOSE_KEY)
    }

    /// Publish an event to supported NIP relays using an external oracle
    public entry fun publish_event(event_request: &SimpleMap<String, String>) {
        assert!(is_event_request(event_request), ErrorEventRequest);
        // parse the event request to event struct
        let event_key = event_key_string();
        let event_json = simple_map::borrow(event_request, &event_key);
        let event = json::from_json<Event>(event_json);
        // check if the event is in the Move's state and whether signer (address) owns the event
        let event_object_id = object::id<Event>(&event);
        let event_object = object::borrow_object<Event>(event_object_id);
        let object_owner = object::owner<Event>(event_object);
        let rooch_address = inner::derive_rooch_address(&event.pubkey);
        assert!(rooch_address == object_owner, ErrorInvalidPublicKeyOwner);
        // TODO: proxy the event to the oracle
    }

    /// TODO: NIP-01: index the a single alphabet letter tag with the first value returned to be used with tag filter from the client
    fun index_tags(tags: vector<Tags>): SimpleMultiMap<String, String> {
        // create a simple multi map for the single-letter english alphabet letters of tag index
        let alphabet = simple_multimap::new<String, String>();
    }

    fun is_event_request(event_request: &SimpleMap<String, String>): bool {
        let event_key = event_key_string();
        simple_map::contains_key(event_request, &event_key)
    }
}
