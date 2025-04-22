// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Implements NIP-01 event structure
module nostr::event {
    use std::vector;
    use std::string::{Self, String};
    use moveos_std::object::{Self, ObjectID};
    use moveos_std::hash;
    use moveos_std::hex;
    use moveos_std::timestamp;
    use moveos_std::event;
    use moveos_std::json;
    use moveos_std::string_utils;
    use rooch_framework::schnorr;
    use nostr::inner;

    // Kind of the event
    const EVENT_KIND_USER_METADATA: u16 = 0;

    // Output JSON filter in NIP-01
    const SPACE: u8 = 32; // 0x20, \ , [Space]

    // Content filter in NIP-01
    const LF: u8 = 10; // 0x0A, \n, Line Feed
    const DOUBLEQUOTE: u8 = 34; // 0x22, \", Double Quote
    const BACKSLASH: u8 = 92; // 0x5C, \\, Backslash
    const CR: u8 = 13; // 0x0D, \r, Carriage Return
    const TAB: u8 = 9; // 0x09, \t, Tab
    const BS: u8 = 8; // 0x08, \b, Backspace
    const FF: u8 = 12; // 0x0C, \f, Form Feed

    // Error codes starting from 1000
    const ErrorContentTooLarge: u64 = 1000;
    const ErrorMalformedId: u64 = 1001;
    const ErrorSignatureValidationFailure: u64 = 1002;
    const ErrorMalformedPublicKey: u64 = 1003;
    const ErrorUtf8Encoding: u64 = 1004;
    const ErrorMalformedSignature: u64 = 1005;
    const ErrorPreEventNotExist: u64 = 1006;

    #[data_struct]
    /// PreEvent
    struct PreEvent has key, copy, drop {
        id: vector<u8>, // 32-bytes lowercase hex-encoded sha256 of the serialized event data
        pubkey: vector<u8>, // 32-bytes lowercase hex-encoded public key of the event creator
        created_at: u64, // unix timestamp in seconds
        kind: u16, // integer between 0 and 65535
        tags: vector<vector<String>>, // arbitrary string
        content: String, // arbitrary string
    }

    #[data_struct]
    /// Event
    struct Event has key, copy, drop {
        id: vector<u8>, // 32-bytes lowercase hex-encoded sha256 of the serialized event data
        pubkey: vector<u8>, // 32-bytes lowercase hex-encoded public key of the event creator
        created_at: u64, // unix timestamp in seconds
        kind: u16, // integer between 0 and 65535
        tags: vector<vector<String>>, // arbitrary string
        content: String, // arbitrary string
        sig: vector<u8> // 64-bytes lowercase hex of the signature of the sha256 hash of the serialized event data, which is the same as the "id" field
    }

    #[data_struct]
    /// PreEvent create notification for Move events
    struct NostrPreEventCreatedEvent has copy, drop {
        id: ObjectID
    }

    #[data_struct]
    /// Event create notification for Move events
    struct NostrEventCreatedEvent has copy, drop {
        id: ObjectID
    }

    #[data_struct]
    /// UserMetadata field as stringified JSON object, when the Event kind is equal to 0
    struct UserMetadata has copy, drop {
        name: String,
        about: String,
        picture: String
    }

    /// TODO: serialize matching the standard
    /// Serialize to byte arrays, which could be sha256 hashed and hex-encoded with lowercase to 32 byte arrays
    fun serialize(pubkey: String, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String): vector<u8> {
        let serialized = string::utf8(b"");
        let left = string::utf8(b"[");
        let right = string::utf8(b"]");
        let coma = string::utf8(b",");

        // version 0, as described in NIP-01
        let version = 0;
        let version_str = string_utils::to_string_u8(version);
        string::append(&mut serialized, left);
        string::append(&mut serialized, version_str);
        string::append(&mut serialized, coma);

        // pubkey
        assert!(string::length(&pubkey) == 64, ErrorMalformedPublicKey);
        string::append(&mut serialized, pubkey);
        string::append(&mut serialized, coma);

        // created_at
        let created_at_str = string_utils::to_string_u64(created_at);
        string::append(&mut serialized, created_at_str);
        string::append(&mut serialized, coma);

        // kind
        let kind_str = string_utils::to_string_u16(kind);
        string::append(&mut serialized, kind_str);
        string::append(&mut serialized, coma);

        // tags
        let tags_str = string::utf8(json::to_json(&tags));
        string::append(&mut serialized, tags_str);
        string::append(&mut serialized, coma);

        // content
        let content_json = json::to_json(&content);
        // escape some characters of the content field
        while (vector::contains(&content_json, &LF)) {
            vector::remove_value(&mut content_json, &LF);
        };
        while (vector::contains(&content_json, &DOUBLEQUOTE)) {
            vector::remove_value(&mut content_json, &DOUBLEQUOTE);
        };
        while (vector::contains(&content_json, &BACKSLASH)) {
            vector::remove_value(&mut content_json, &BACKSLASH);
        };
        while (vector::contains(&content_json, &CR)) {
            vector::remove_value(&mut content_json, &CR);
        };
        while (vector::contains(&content_json, &TAB)) {
            vector::remove_value(&mut content_json, &TAB);
        };
        while (vector::contains(&content_json, &BS)) {
            vector::remove_value(&mut content_json, &BS);
        };
        while (vector::contains(&content_json, &FF)) {
            vector::remove_value(&mut content_json, &FF);
        };
        let content_str = string::utf8(content_json);
        string::append(&mut serialized, content_str);
        string::append(&mut serialized, right);

        // get the serialized string bytes
        let serialized_bytes = string::into_bytes(serialized);

        // remove whitespace and line breaks from output JSON
        while (vector::contains(&serialized_bytes, &SPACE)) {
            vector::remove_value(&mut serialized_bytes, &SPACE);
        };
        while (vector::contains(&serialized_bytes, &LF)) {
            vector::remove_value(&mut serialized_bytes, &LF);
        };

        // check UTF-8 encoding
        assert!(string::internal_check_utf8(&serialized_bytes), ErrorUtf8Encoding);

        serialized_bytes
    }

    /// Check signature with public key, id and signature for schnorr
    fun check_signature(id: vector<u8>, public_key: vector<u8>, signature: vector<u8>) {
        assert!(schnorr::verify(
            &signature,
            &public_key,
            &id,
        ), ErrorSignatureValidationFailure);
    }

    /// Create an Event id
    fun create_event_id(pubkey: String, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String): vector<u8> {
        // serialize input to bytes for an Event id
        let serialized = serialize(pubkey, created_at, kind, tags, content);

        // hash with sha256
        let hashed_data = hash::sha2_256(serialized);

        // encode with lowercase hex
        let id = hex::encode(hashed_data);

        // verify the length of the hex bytes to 32 bytes (64 characters)
        assert!(vector::length(&id) == 64, ErrorMalformedId);

        id
    }

    /// Create a pre Event
    public fun create_pre_event(public_key: vector<u8>, kind: u16, tags: vector<vector<String>>, content: String) {
        assert!(string::length(&content) <= 1000, ErrorContentTooLarge);
        let pubkey = string::utf8(public_key);

        // get now timestamp by seconds
        let created_at = timestamp::now_seconds();

        // create event id
        let id = create_event_id(pubkey, created_at, kind, tags, content);

        // derive a rooch address
        let rooch_address = inner::derive_rooch_address(public_key);

        // save the pre event to the rooch address mapped to the public key
        let pre_event = PreEvent {
            id,
            pubkey: public_key,
            created_at,
            kind,
            tags,
            content,
        };
        let pre_event_object = object::new_account_named_object<PreEvent>(rooch_address, pre_event);

        // emit a move event nofitication
        let pre_event_object_id = object::id(&pre_event_object);
        let move_pre_event = NostrPreEventCreatedEvent {
            id: pre_event_object_id
        };
        event::emit(move_pre_event);

        // transfer pre event object to the rooch address
        object::transfer_extend(pre_event_object, rooch_address);
    }

    /// Entry function to create a pre Event
    public entry fun create_pre_event_entry(public_key: vector<u8>, kind: u16, tags: vector<vector<String>>, content: String) {
        create_pre_event(public_key, kind, tags, content);
    }

    /// Create an Event
    public fun create_event(public_key: vector<u8>, signature: vector<u8>) {
        // derive a rooch address
        let rooch_address = inner::derive_rooch_address(public_key);

        // get the pre event object id from the address
        let pre_event_object_id = object::account_named_object_id<PreEvent>(rooch_address);

        // check the pre event object id if it exists
        assert!(object::exists_object_with_type<PreEvent>(pre_event_object_id), ErrorPreEventNotExist);

        // take the pre event object from the object store
        let pre_event_object = object::borrow_object<PreEvent>(pre_event_object_id);

        // borrow the pre event
        let pre_event = object::borrow<PreEvent>(pre_event_object);

        // flatten the elements
        let PreEvent { id, pubkey, created_at, kind, tags, content } = *pre_event;

        // check the signature
        check_signature(id, pubkey, signature);

        // handle a range of different kinds of an Event
        if (kind == EVENT_KIND_USER_METADATA) {
            // check the content integrity
            let content_bytes = string::bytes(&content);
            let _ = json::from_json<UserMetadata>(*content_bytes);
            // clear past user metadata events from the user with the same rooch address from the public key
            let event_object_id = object::account_named_object_id<Event>(rooch_address);
            if (object::exists_object_with_type<Event>(event_object_id)) {
                let event_object = object::take_object_extend<Event>(event_object_id);
                let event = object::remove(event_object);
                drop_event(event);
            };
        };

        // save the event to the rooch address mapped to the public key
        let event = Event {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content,
            sig: signature
        };
        let event_object = object::new_account_named_object<Event>(rooch_address, event);

        // emit a move event nofitication
        let event_object_id = object::id(&event_object);
        let move_event = NostrEventCreatedEvent {
            id: event_object_id
        };
        event::emit(move_event);

        // transfer event object to the rooch address
        object::transfer_extend(event_object, rooch_address);
    }

    /// Entry function to create an Event
    public entry fun create_event_entry(public_key: vector<u8>, signature: vector<u8>) {
        create_event(public_key, signature);
    }

    /// Save an Event
    public fun save_event(public_key: vector<u8>, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String, signature: vector<u8>): vector<u8> {
        let pubkey = string::utf8(public_key);

        // check signature length
        assert!(vector::length(&signature) == 128, ErrorMalformedSignature);

        // check public key length
        assert!(vector::length(&public_key) == 64, ErrorMalformedPublicKey);

        // create event id
        let id = create_event_id(pubkey, created_at, kind, tags, content);

        // check the signature
        check_signature(id, public_key, signature);

        // derive a rooch address
        let rooch_address = inner::derive_rooch_address(public_key);

        // handle a range of different kinds of an Event
        if (kind == EVENT_KIND_USER_METADATA) {
            // check the content integrity
            let content_bytes = string::bytes(&content);
            let _ = json::from_json<UserMetadata>(*content_bytes);
            // clear past user metadata events from the user with the same rooch address from the public key
            let event_object_id = object::account_named_object_id<Event>(rooch_address);
            if (object::exists_object_with_type<Event>(event_object_id)) {
                let event_object = object::take_object_extend<Event>(event_object_id);
                let event = object::remove(event_object);
                drop_event(event);
            };
        };

        // save the event to the rooch address mapped to the public key
        let event = Event {
            id,
            pubkey: public_key,
            created_at,
            kind,
            tags,
            content,
            sig: signature
        };
        let event_object = object::new_account_named_object<Event>(rooch_address, event);

        // emit a move event nofitication
        let event_object_id = object::id(&event_object);
        let move_event = NostrEventCreatedEvent {
            id: event_object_id
        };
        event::emit(move_event);

        // transfer event object to the rooch address
        object::transfer_extend(event_object, rooch_address);

        // return the event object as JSON
        let event_json = json::to_json<Event>(&event);
        event_json
    }

    /// Entry function to save an Event
    public entry fun save_event_entry(public_key: vector<u8>, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String, signature: vector<u8>) {
        let _event_json = save_event(public_key, created_at, kind, tags, content, signature);
    }

    /// Save an Event with plaintext. Do not check integrity of created_at, kind, tags and content with id.
    public fun save_event_plaintext(id: vector<u8>, public_key: vector<u8>, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String, signature: vector<u8>): vector<u8> {
        // check id length
        assert!(vector::length(&id) == 64, ErrorMalformedId);

        // check public key length
        assert!(vector::length(&public_key) == 64, ErrorMalformedPublicKey);

        // check signature length
        assert!(vector::length(&signature) == 128, ErrorMalformedSignature);

        // check the signature
        check_signature(id, public_key, signature);

        // derive a rooch address
        let rooch_address = inner::derive_rooch_address(public_key);

        // handle a range of different kinds of an Event
        if (kind == EVENT_KIND_USER_METADATA) {
            // check the content integrity
            let content_bytes = string::bytes(&content);
            let _ = json::from_json<UserMetadata>(*content_bytes);
            // clear past user metadata events from the user with the same rooch address from the public key
            let event_object_id = object::account_named_object_id<Event>(rooch_address);
            if (object::exists_object_with_type<Event>(event_object_id)) {
                let event_object = object::take_object_extend<Event>(event_object_id);
                let event = object::remove(event_object);
                drop_event(event);
            };
        };

        // save the event to the rooch address mapped to the public key
        let event = Event {
            id,
            pubkey: public_key,
            created_at,
            kind,
            tags,
            content,
            sig: signature
        };
        let event_object = object::new_account_named_object<Event>(rooch_address, event);

        // emit a move event nofitication
        let event_object_id = object::id(&event_object);
        let move_event = NostrEventCreatedEvent {
            id: event_object_id
        };
        event::emit(move_event);

        // transfer event object to the rooch address
        object::transfer_extend(event_object, rooch_address);

        // return the event object as JSON
        let event_json = json::to_json<Event>(&event);
        event_json
    }

    /// Entry function to save an Event with plaintext
    public entry fun save_event_plaintext_entry(id: vector<u8>, public_key: vector<u8>, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String, signature: vector<u8>) {
        let _event_json = save_event_plaintext(id, public_key, created_at, kind, tags, content, signature);
    }

    /// drop an event
    fun drop_event(event: Event) {
        let Event {id: _, pubkey: _, created_at: _, kind: _, tags: _, content: _, sig: _} = event;
    }

    public fun unpack_event(event: Event): (vector<u8>, vector<u8>, u64, u16, vector<vector<String>>, String, vector<u8>) {
        let Event { id, pubkey, created_at, kind, tags, content, sig } = event;
        (id, pubkey, created_at, kind, tags, content, sig)
    }

    /// getter functions

    public fun id(event: &Event): vector<u8> {
        event.id
    }

    public fun pubkey(event: &Event): vector<u8> {
        event.pubkey
    }

    public fun created_at(event: &Event): u64 {
        event.created_at
    }

    public fun kind(event: &Event): u16 {
        event.kind
    }

    public fun tags(event: &Event): vector<vector<String>> {
        event.tags
    }

    public fun content(event: &Event): String {
        event.content
    }

    public fun sig(event: &Event): vector<u8> {
        event.sig
    }
}
