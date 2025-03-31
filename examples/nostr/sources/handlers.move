// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Implements NIP-01 for handling client requests.
/// There's no relay as handlers deal with all client requests and redirect them to third party applications using oracle http/https proxies.
module nostr::handlers {
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::simple_multimap::{Self, SimpleMultiMap};
    use nostr::event::Event;
    use nostr::inner::Tags;

    // types of messages
    const EVENT_KEY: vector<u8> = b"EVENT";
    const REQ_KEY: vector<u8> = b"REQ";
    const CLOSE_KEY: vector<u8> = b"CLOSE";

    // Error codes from 1000 onward
    const ErrorEventRequest: u64 = 1000;

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
        let event_key = event_key_string();
        let event_json = simple_map::borrow(event_request, &event_key);
        let event = json::from_json<Event>(event_json);
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
