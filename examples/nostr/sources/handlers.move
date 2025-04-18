// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Implements NIP-01 for handling client messages.
/// There's no relay as handlers deal with all client messages and redirect them to third party applications using http/https proxies.
module nostr::handlers {
    use std::string::{Self, String};
    use std::option;
    use std::u256;
    use std::vector;
    use moveos_std::bcs;
    use moveos_std::json;
    use moveos_std::object::{Self, ObjectID};
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::simple_multimap::{Self, SimpleMultiMap};
    use moveos_std::account;
    use rooch_framework::account_coin_store;
    use rooch_framework::gas_coin::RGas;
    use nostr::event::{Self, Event};
    use nostr::inner::{Self, Tags};
    use nostr::requests;

    // types of messages
    const EVENT_KEY: vector<u8> = b"EVENT";
    const REQ_KEY: vector<u8> = b"REQ";
    const CLOSE_KEY: vector<u8> = b"CLOSE";

    // Error codes from 1000 onward
    const ErrorEventMessage: u64 = 1000;
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

    /// Publish a signed event to relays
    public fun publish_event(event_message: SimpleMap<String, String>) {
        assert!(is_event_message(&event_message), ErrorEventMessage);

        // parse the event message to event struct
        let event_key = event_key_string();
        let event_json = simple_map::borrow(&event_message, &event_key);
        let event_json_str = bcs::to_bytes(event_json);
        let event = json::from_json<Event>(event_json_str);

        // TODO: check if the event is in the Move's state and whether signer (address) owns the event
        let public_key = event::pubkey(&event);
        let rooch_address = inner::derive_rooch_address(public_key);
        let event_object_id = object::account_named_object_id<Event>(rooch_address);
        let event_object = object::borrow_object<Event>(event_object_id);
        let event_from_store = object::borrow(event_object);
        // TODO: error messages
        assert!(event::id(&event) == event::id(event_from_store), ErrorInvalidPublicKeyOwner);
        assert!(event::pubkey(&event) == event::pubkey(event_from_store), ErrorInvalidPublicKeyOwner);
        assert!(event::created_at(&event) == event::created_at(event_from_store), ErrorInvalidPublicKeyOwner);
        assert!(event::kind(&event) == event::kind(event_from_store), ErrorInvalidPublicKeyOwner);
        assert!(event::tags(&event) == event::tags(event_from_store), ErrorInvalidPublicKeyOwner);
        assert!(event::content(&event) == event::content(event_from_store), ErrorInvalidPublicKeyOwner);
        assert!(event::sig(&event) == event::sig(event_from_store), ErrorInvalidPublicKeyOwner);

        // create a http/https event request
        // TODO: use params from user config for user defined relays
        let (id, pubkey, created_at, kind, tags, content, sig) = event::unpack_event(event);
        let id_str = string::utf8(id);
        let pubkey_str = string::utf8(pubkey);
        let sig_str = string::utf8(sig);
        let event_request = requests::new_event_request(id_str, pubkey_str, created_at, kind, tags, content, sig_str);
        let event_request_json = requests::event_request_json(&event_request);
    }

    /// TODO: NIP-01: index the a single alphabet letter tag with the first value returned to be used with tag filter from the client
    fun index_tags(_tags: vector<Tags>): SimpleMultiMap<String, String> {
        // create a simple multi map for the single-letter english alphabet letters of tag index
        let alphabet = simple_multimap::new<String, String>();
        alphabet
    }

    fun is_event_message(event_message: &SimpleMap<String, String>): bool {
        let event_key = event_key_string();
        simple_map::contains_key(event_message, &event_key)
    }
}
