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

    // Name of the tag of the event
    const EVENT_TAG_KEY: String = "e";
    const USER_TAG_KEY: String = "p";
    const ADDRESSABLE_REPLACEABLE_TAG_KEY: String = "a";

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
    const ErrorMalformedOtherEventId: u64 = 1003;
    const ErrorMalformedPublicKey: u64 = 1004;
    const ErrorEmptyTags: u64 = 1005;
    const ErrorUtf8Encoding: u64 = 1006;

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

    /// EventData for the internal data struct
    #[data_struct]
    struct EventData has key {
        id: vector<u8>, // 32-bytes lowercase hex-encoded sha256 of the serialized event data
        pubkey: vector<u8>, // 32-bytes lowercase hex-encoded public key of the event creator
        created_at: u64, // unix timestamp in seconds
        kind: u16, // integer between 0 and 65535
        tags: vector<Tags>, // an array of Tags from non-null arbitrary string
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

    /// EventTag with `e` name
    #[data_struct]
    struct EventTag {
        id: vector<u8>,
        url: Option<String>,
        pubkey: Option<vector<u8>>
    }

    /// UserTag with `p` name
    #[data_struct]
    struct UserTag {
        pubkey: vector<u8>,
        url: Option<String>
    }

    /// AddressableReplaceableTag with `a` name
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
        while (vector::contains(content_bytes, LF)) {
            vector::remove_value(content_bytes, LF);
        }
        while (vector::contains(content_bytes, DOUBLEQUOTE)) {
            vector::remove_value(content_bytes, DOUBLEQUOTE);
        }
        while (vector::contains(content_bytes, BACKSLASH)) {
            vector::remove_value(content_bytes, BACKSLASH);
        }
        while (vector::contains(content_bytes, CR)) {
            vector::remove_value(content_bytes, CR);
        }
        while (vector::contains(content_bytes, TAB)) {
            vector::remove_value(content_bytes, TAB);
        }
        while (vector::contains(content_bytes, BS)) {
            vector::remove_value(content_bytes, BS);
        }
        while (vector::contains(content_bytes, FF)) {
            vector::remove_value(content_bytes, FF);
        }
        vector::push_back(&mut serialized, content_bytes);

        // remove whitespace and line breaks from output JSON
        while (vector::contains(serialized, SPACE)) {
            vector::remove_value(serialized, SPACE);
        }
        while (vector::contains(serialized, LF)) {
            vector::remove_value(serialized, LF);
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

    /// build string tags to inner struct tags
    fun build_tags(tags_str: vector<vector<String>>): vector<Tags> {
        // init tags list
        let tags_list = vector::empty<Tags>();
        // perform build
        let i = 0;
        let tags_str_len = vector::length(tags_str);
        while (i < tags_str_len) {
            let tag_str_list = vector::borrow(&tags_str, i);
            let o = 0;
            let tag_str_list_len = vector::length(tag_str_list);
            while (o < tag_str_list_len) {
                let tag_str = vector::borrow(&tag_str_list, o);
                // take special circumstance for the NIP-01 defined keys
                if (o == 0) {
                    let tag_value_index = o + 1;
                    let tag_value = vector::borrow(&tag_str_list, tag_value_index);
                    if (tag_str == EVENT_TAG_KEY) {
                        // get the id of another Event
                        let id = bcs::to_bytes(&tag_value);
                        assert!(vector::length(&id) == 32, ErrorMalformedOtherEventId);
                        // get the url of recommended relay if it exists
                        let url_option = option::none<String>();
                        if (tag_str_list_len == 3) {
                            let index = o + 2;
                            let str = vector::borrow(&tag_str_list, index);
                            option::fill<String>(url_option, str);
                        }
                        // get the author's public key if it exists
                        let pubkey_option = option::none<vector<u8>>();
                        if (tag_str_list_len == 4) {
                            let index = o + 3;
                            let str = vector::borrow(&tag_str_list, index);
                            option::fill<String>(pubkey_option, str);
                        }
                        let event_tag = EventTag {
                            id,
                            url: url_option,
                            pubkey: pubkey_option
                        }
                        let tags = Tags {
                            event: option::some(event_tag),
                            user: option::none<UserTag>(),
                            addressable_replaceable: option::none<AddressableReplaceableTag>(),
                            non_null_string: option::none<NonNullStringTag>()
                        }
                        vector::push_back(&mut tags_list, tags);
                    }
                    // TODO: USER_TAG_KEY
                    if (tag_str == USER_TAG_KEY) {

                    }
                    // TODO: ADDRESSABLE_REPLACEABLE_TAG_KEY
                    if (tag_str == ADDRESSABLE_REPLACEABLE_TAG_KEY) {

                    }
                }
                // TODO: proceed with normal arbitrary strings
            }
        }
    }

    /// NIP-01: index the a single alphabet letter tag with the first value returned to be used with tag filter from the client
    fun index_single_alphabet_letter_tag() {

    }
}
