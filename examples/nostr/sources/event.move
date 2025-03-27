// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Implements NIP-01 event structure
module nostr::event {
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::event;
    use moveos_std::bcs;
    use std::vector;
    use rooch_framework::ecdsa_k1;
    use std::string::{Self, String};
    use moveos_std::timestamp;

    // Identifier for the tag of the event
    const EVENT_TAG: String = "e";
    const USER_TAG: String = "p";
    const ADDRESSABLE_REPLACEABLE_TAG: String = "a";

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
    const ErrorMalformedPublicKey: u64 = 1000;
    const ErrorIdIsEmpty: u64 = 1001;
    const ErrorMalformedId: u64 = 1002;
    const ErrorMalformedSignature: u64 = 1003;
    const ErrorSignatureValidationFailure: u64 = 1004;
    const ErrorContentTooLarge: u64 = 1005;

    /// Event
    #[data_struct]
    struct Event has key, store {
        id: ObjectID, // 32-bytes lowercase hex-encoded sha256 of the serialized event data
        pubkey: vector<u8>, // 32-bytes lowercase hex-encoded public key of the event creator
        created_at: u64, // unix timestamp in seconds
        kind: u16, // integer between 0 and 65535
        tags: vector<vector<Tags>>, // an array of arrays of Tags from non-null arbitrary string
        content: vector<u8>, // arbitrary string
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

    /// Tags
    #[data_struct]
    struct Tags {
        /// For referring to an event
        event: Option<EventTag>,
        /// For another user
        user: Option<UserTag>,
        /// For addressable or replaceable events
        addressable_replaceable: Option<AddressableReplaceableTag>,
        /// For other non-null strings
        non_null_string: Option<NonNullStringTag>
    }

    /// EventTag with `e` identifier
    #[data_struct]
    struct EventTag {
        id: ObjectID,
        url: Option<String>,
        pubkey: Option<vector<u8>>
    }

    /// UserTag with `p` identifier
    #[data_struct]
    struct UserTag {
        pubkey: vector<u8>,
        url: Option<String>
    }

    /// AddressableReplaceableTag with `a` identifier
    #[data_struct]
    struct AddressableReplaceableTag {
        kind: u16,
        pubkey: vector<u8>,
        value: Option<String>,
        url: Option<String>
    }

    /// NonNullStringTag with non-empty value
    #[data_struct]
    struct NonNullStringTag {
        str: String
    }

    /// Serialize to byte arrays, which could be sha256 hashed and be converted to ObjectID for the Event.id
    public fun serialize(public_key: &vector<u8>, kind: &u16, tags: &vector<vector<Tags>>, content: &vector<u8>): vector<u8> {
        let serialized = vector::empty<u8>();

        // version 0, as described in NIP-01
        let version = vector::singleton<u8>(0);
        vector::push_back(&mut serialized, version);

        // public key
        assert!(vector::length(public_key) == 32, ErrorMalformedPublicKey);
        vector::push_back(&mut serialized, public_key);

        // creation time
        let created_at = timestamp::now_seconds();
        let created_at_bytes = bcs::to_bytes(created_at);
        vector::push_back(&mut serialized, created_at_bytes);

        // kind
        let kind_bytes = bcs::to_bytes(kind);
        vector::push_back(&mut serialized, kind_bytes);

        // tags
        let tags_bytes = bcs::to_bytes(tags);
        vector::push_back(&mut serialized, tags_bytes);

        // content
        while (vector::contains(content, LF)) {
            vector::remove_value(content, LF);
        }
        while (vector::contains(content, DOUBLEQUOTE)) {
            vector::remove_value(content, DOUBLEQUOTE);
        }
        while (vector::contains(content, BACKSLASH)) {
            vector::remove_value(content, BACKSLASH);
        }
        while (vector::contains(content, CR)) {
            vector::remove_value(content, CR);
        }
        while (vector::contains(content, TAB)) {
            vector::remove_value(content, TAB);
        }
        while (vector::contains(content, BS)) {
            vector::remove_value(content, BS);
        }
        while (vector::contains(content, FF)) {
            vector::remove_value(content, FF);
        }
        vector::push_back(&mut serialized, content);
        serialized
    }

    /// Check signature of the event whether it is valid for the id of the event
    public fun check_signature(event: Event) {
        // public key
        assert!(vector::length(event.pubkey) == 32, ErrorMalformedPublicKey);
        // id
        assert!(object::has_parent(&event.id), ErrorIdIsEmpty);
        let id_bytes = vector::empty<u8>();
        let i = 0;
        while (i < vector::length(&id.path)) {
            let addr = *vector::borrow(&id.path, i);
            let addr_bytes = bcs::to_bytes(&addr);
            vector::append(&mut id_bytes, addr_bytes);
            i = i + 1;
        };
        assert!(vector::length(id_bytes) == 32, ErrorMalformedId);
        // signature
        assert!(vector::length(event.sig) == 64, ErrorMalformedSignature);
        // TODO: with schnorr verify
        assert!(ecdsa_k1::verify(
            &event.sig,
            &event.pubkey,
            &id_bytes,
            ecdsa_k1::sha256()
        ), ErrorSignatureValidationFailure);
    }

    /// Create an Event
    public fun create_event(owner: &signer, public_key: &vector<u8>, kind: &u16, tags: &String, content_str: &String) {
        assert!(!string::length(content_str) > 1000, ErrorContentTooLarge);
        let content = bcs::to_bytes(content_str);
        
        // serialize input to bytes for an Event id
        let serialized = serialize(public_key, kind, tags, &content); // TODO: tags.
        
    }
}
