// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Implements NIP-01 event structure
module nostr::event {
    use std::vector;
    use std::string::{Self, String};
    use std::option;
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::event;
    use moveos_std::bcs;
    use moveos_std::hash;
    use moveos_std::hex;
    use moveos_std::timestamp;
    use moveos_std::event;
    use rooch_framework::bitcoin_address;
    use rooch_framework::schnorr;

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
    const ErrorEmptyTags: u64 = 1004;
    const ErrorUtf8Encoding: u64 = 1005;

    /// Event
    #[data_struct]
    struct Event has key {
        id: vector<u8>, // 32-bytes lowercase hex-encoded sha256 of the serialized event data
        pubkey: vector<u8>, // 32-bytes lowercase hex-encoded public key of the event creator
        created_at: u64, // unix timestamp in seconds
        kind: u16, // integer between 0 and 65535
        tags: vector<vector<String>>, // arbitrary string
        content: String, // arbitrary string
        sig: vector<u8> // 64-bytes lowercase hex of the signature of the sha256 hash of the serialized event data, which is the same as the "id" field
    }

    /// Event create notification for Move events
    #[data_struct]
    struct MoveEventNotification has store {
        id: ObjectID
    }

    /// UserMetadata field as stringified JSON object, when the Event kind is equal to 0
    #[data_struct]
    struct UserMetadata has key {
        name: String,
        about: String,
        picture: String
    }

    /// Serialize to byte arrays, which could be sha256 hashed and hex-encoded with lowercase to 32 byte arrays
    public fun serialize(pubkey: &String, created_at: &u64, kind: &u16, tags: &vector<vector<String>>, content: &String): vector<u8> {
        let serialized = vector::empty<u8>();

        // version 0, as described in NIP-01
        let version = vector::singleton<u8>(0);
        vector::push_back(&mut serialized, version);

        // pubkey
        let pubkey_bytes = vector::to_bytes(pubkey);
        assert!(vector::length(pubkey_bytes) == 32, ErrorMalformedPublicKey);
        vector::push_back(&mut serialized, pubkey_bytes);

        // created_at
        let created_at_bytes = bcs::to_bytes(created_at);
        vector::push_back(&mut serialized, created_at_bytes);

        // kind
        let kind_bytes = bcs::to_bytes(kind);
        vector::push_back(&mut serialized, kind_bytes);

        // tags
        assert!(!vector::is_empty<vector<vector<String>>>(tags), ErrorEmptyTags);
        let tags_bytes = bcs::to_bytes(tags);
        vector::push_back(&mut serialized, tags_bytes);

        // content, escape some characters
        let content_bytes = bcs::to_bytes(content);
        while (vector::contains(&content_bytes, &LF)) {
            vector::remove_value(&mut content_bytes, &LF);
        }
        while (vector::contains(&content_bytes, &DOUBLEQUOTE)) {
            vector::remove_value(&mut content_bytes, &DOUBLEQUOTE);
        }
        while (vector::contains(&content_bytes, &BACKSLASH)) {
            vector::remove_value(&mut content_bytes, &BACKSLASH);
        }
        while (vector::contains(&content_bytes, &CR)) {
            vector::remove_value(&mut content_bytes, &CR);
        }
        while (vector::contains(&content_bytes, &TAB)) {
            vector::remove_value(&mut content_bytes, &TAB);
        }
        while (vector::contains(&content_bytes, &BS)) {
            vector::remove_value(&mut content_bytes, &BS);
        }
        while (vector::contains(&content_bytes, &FF)) {
            vector::remove_value(&mut content_bytes, &FF);
        }
        vector::push_back(&mut serialized, content_bytes);

        // remove whitespace and line breaks from output JSON
        while (vector::contains(&serialized, &SPACE)) {
            vector::remove_value(&mut serialized, &SPACE);
        }
        while (vector::contains(&serialized, &LF)) {
            vector::remove_value(&mut serialized, &LF);
        }

        // check UTF-8 encoding
        assert!(string::internal_check_utf8(&serialized), ErrorUtf8Encoding);
        serialized
    }

    /// Check signature with public key, id and signature for schnorr
    public fun check_signature(public_key: &vector<u8>, id: &vector<u8>, signature: &vector<u8>) {
        assert!(schnorr::verify(
            signature,
            public_key,
            id,
            schnorr::sha256()
        ), ErrorSignatureValidationFailure);
    }

    /// Create an Event
    public fun create_event(public_key: &vector<u8>, kind: &u16, tags: &vector<vector<String>>, content: &String, signature: &vector<u8>): Event {
        assert!(!string::length(content) > 1000, ErrorContentTooLarge);
        let pubkey = string::utf8(*public_key);

        // get now timestamp by seconds
        let created_at = timestamp::now_seconds();

        // serialize input to bytes for an Event id
        let serialized = serialize(&pubkey, &created_at, kind, tags, content);

        // hash with sha256
        let hashed_data = hash::sha2_256(serialized);

        // encode with lowercase hex
        let id = hex::encode(hashed_data);

        // verify the length of the hex bytes to 32 bytes
        assert!(vector::length(&id) == 32, ErrorMalformedId);

        // check the signature
        check_signature(public_key, &id, signature);

        // derive a bitcoin taproot address from the public key
        let bitcoin_taproot_address = derive_bitcoin_taproot_address_from_pubkey(public_key);

        // derive a rooch address from the bitcoin taproot address
        let rooch_address = to_rooch_address(&bitcoin_taproot_address);

        // handle a range of different kinds of an Event
        if (kind == EVENT_KIND_USER_METADATA) {
            // clear past user metadata events from the user with the same rooch address from the public key
            let event_object_id = object::account_named_object_id<Event>(rooch_address);
            let event_object = object::take_object_extend<Event>(event_object_id);
            let event = object::remove(event_object);
            drop_event(event);
        }

        // save the event to the rooch address mapped to the public key
        let event = Event {
            id,
            pubkey: *public_key,
            created_at,
            kind: *kind,
            tags: *tags,
            content,
            sig: *signature
        }
        let event_object = object::new_account_named_object(rooch_address, event);
        object::transfer_extend(event_object, rooch_address);
        // emit a move event nofitication
        let event_object_id = object::id(&event_object);
        let move_event = MoveEventNotification {
            id: event_object_id
        }
        event::emit(move_event)
    }

    /// drop an event
    fun drop_event(event: Event) {
        let Event {id: _, pubkey: _, created_at: _, kind: _, tags: _, content: _, sig: _} = event;
    }
}
