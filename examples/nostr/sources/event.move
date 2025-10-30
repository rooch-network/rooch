// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Implements NIP-01 event structure
module nostr::event {
    use std::vector;
    use std::string::{Self, String};
    use std::option;
    use moveos_std::signer;
    use moveos_std::object::{Self, ObjectID};
    use moveos_std::hash;
    use moveos_std::hex;
    use moveos_std::timestamp;
    use moveos_std::event;
    use moveos_std::json;
    use moveos_std::string_utils;
    use rooch_framework::ecdsa_k1;
    use nostr::inner;

    // Kind of the event
    const EVENT_KIND_USER_METADATA: u16 = 0;

    // Error codes starting from 1000
    const ErrorContentTooLarge: u64 = 1000;
    const ErrorMalformedId: u64 = 1001;
    const ErrorSignatureValidationFailure: u64 = 1002;
    const ErrorMalformedPublicKey: u64 = 1003;
    const ErrorUtf8Encoding: u64 = 1004;
    const ErrorMalformedSignature: u64 = 1005;
    const ErrorPreEventNotExist: u64 = 1006;
    const ErrorInvalidUserMetadata: u64 = 1007;

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
    /// Event save notification for Move events
    struct NostrEventSavedEvent has copy, drop {
        id: ObjectID
    }

    #[data_struct]
    /// Plaintext Event save notification for Move events
    struct NostrPlaintextEventSavedEvent has copy, drop {
        id: ObjectID
    }

    #[data_struct]
    /// UserMetadata field as stringified JSON object, when the Event kind is equal to 0
    struct UserMetadata has copy, drop {
        name: String,
        about: String,
        picture: String
    }

    /// Serialize to byte arrays, which could be sha256 hashed and hex-encoded with lowercase to 32 byte arrays
    fun serialize(pubkey: String, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String): vector<u8> {
        let serialized = string::utf8(b"");
        let left_sb = string::utf8(b"[");
        let right_sb = string::utf8(b"]");
        let double_qm = string::utf8(b"\"");
        let coma = string::utf8(b",");

        // version 0, as described in NIP-01
        let version = 0;
        let version_str = string_utils::to_string_u8(version);
        string::append(&mut serialized, left_sb);
        string::append(&mut serialized, version_str);
        string::append(&mut serialized, coma);

        // pubkey
        assert!(string::length(&pubkey) == 64, ErrorMalformedPublicKey);
        string::append(&mut serialized, double_qm);
        string::append(&mut serialized, pubkey);
        string::append(&mut serialized, double_qm);
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
        string::append(&mut serialized, double_qm);
        string::append(&mut serialized, content);
        string::append(&mut serialized, double_qm);
        string::append(&mut serialized, right_sb);

        // get the serialized string bytes
        let serialized_bytes = string::into_bytes(serialized);

        // check UTF-8 encoding
        assert!(string::internal_check_utf8(&serialized_bytes), ErrorUtf8Encoding);

        serialized_bytes
    }

    /// Check signature with public key, id and signature for schnorr
    fun check_signature(id: vector<u8>, x_only_public_key: vector<u8>, signature: vector<u8>) {
        assert!(ecdsa_k1::verify(
            &signature,
            &x_only_public_key,
            &id,
            ecdsa_k1::sha256()
        ), ErrorSignatureValidationFailure);
    }

    /// Check the referenced user metadata from content with UserMetadata struct
    fun check_user_metadata(content: String) {
        // check the content integrity
        let content_json = json::to_json<String>(&content);
        // some bits are stripped for verification
        let dq = inner::doublequote();
        vector::remove_value(&mut content_json, &dq);
        vector::reverse(&mut content_json);
        vector::remove_value(&mut content_json, &dq);
        vector::reverse(&mut content_json);
        let bs = inner::backslash();
        while (vector::contains(&content_json, &bs)) {
            vector::remove_value(&mut content_json, &bs);
        };
        let user_metadata_option = json::from_json_option<UserMetadata>(content_json);
        assert!(option::is_some(&user_metadata_option), ErrorInvalidUserMetadata);
    }

    /// Create an Event id
    fun create_event_id(pubkey: String, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String): vector<u8> {
        // serialize input to bytes for an Event id
        let serialized = serialize(pubkey, created_at, kind, tags, content);

        // hash with sha256
        let id = hash::sha2_256(serialized);

        // verify the length of the hex bytes to 32 bytes (64 characters)
        assert!(vector::length(&hex::encode(id)) == 64, ErrorMalformedId);

        id
    }

    /// Create a pre Event
    public fun create_pre_event(x_only_public_key: String, kind: u16, tags: vector<vector<String>>, content: String) {
        assert!(string::length(&content) <= 1000, ErrorContentTooLarge);

        // get now timestamp by seconds
        let created_at = timestamp::now_seconds();

        // create event id
        let id = create_event_id(x_only_public_key, created_at, kind, tags, content);

        // get the hex decoded public key bytes
        let pubkey = hex::decode(&string::into_bytes(x_only_public_key));

        // derive a rooch address
        let rooch_address = inner::derive_rooch_address(pubkey);

        // handle a range of different kinds of a pre Event
        if (kind == EVENT_KIND_USER_METADATA) {
            check_user_metadata(content);
            // clear past user metadata pre events from the user with the same rooch address from the public key
            let pre_event_object_id = object::account_named_object_id<PreEvent>(rooch_address);
            if (object::exists_object_with_type<PreEvent>(pre_event_object_id)) {
                let pre_event_object = object::take_object_extend<PreEvent>(pre_event_object_id);
                let pre_event = object::remove(pre_event_object);
                drop_pre_event(pre_event);
            };
        };

        // save the pre event to the rooch address mapped to the public key
        let pre_event = PreEvent {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content,
        };
        let pre_event_object = object::new_account_named_object<PreEvent>(rooch_address, pre_event);

        // emit a move event notification
        let pre_event_object_id = object::id(&pre_event_object);
        let move_pre_event = NostrPreEventCreatedEvent {
            id: pre_event_object_id
        };
        event::emit(move_pre_event);

        // transfer pre event object to the rooch address
        object::transfer_extend(pre_event_object, rooch_address);
    }

    /// Entry function to create a pre Event
    public entry fun create_pre_event_entry(x_only_public_key: String, kind: u16, tags: vector<vector<String>>, content: String) {
        create_pre_event(x_only_public_key, kind, tags, content);
    }

    /// Create an Event
    public fun create_event(signer: &signer, signature: String) {
        // get the signer's rooch address
        let rooch_address = signer::address_of(signer);

        // get the pre event object id from the address
        let pre_event_object_id = object::account_named_object_id<PreEvent>(rooch_address);

        // check the pre event object id if it exists
        assert!(object::exists_object_with_type<PreEvent>(pre_event_object_id), ErrorPreEventNotExist);

        // take the pre event object from the object store
        let pre_event_object = object::borrow_object<PreEvent>(pre_event_object_id);

        // borrow the pre event
        let pre_event = object::borrow<PreEvent>(pre_event_object);

        // flatten the elements
        let (id, pubkey, created_at, kind, tags, content) = unpack_pre_event(*pre_event);

        // decode signature with hex
        let sig = hex::decode(&string::into_bytes(signature));

        // check the signature
        check_signature(id, pubkey, sig);

        // handle a range of different kinds of an Event
        if (kind == EVENT_KIND_USER_METADATA) {
            check_user_metadata(content);
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
            sig
        };
        let event_object = object::new_account_named_object<Event>(rooch_address, event);

        // emit a move event notification
        let event_object_id = object::id(&event_object);
        let move_event = NostrEventCreatedEvent {
            id: event_object_id
        };
        event::emit(move_event);

        // transfer event object to the rooch address
        object::transfer_extend(event_object, rooch_address);
    }

    /// Entry function to create an Event
    public entry fun create_event_entry(signer: &signer, signature: String) {
        create_event(signer, signature);
    }

    /// Save an Event
    public fun save_event(x_only_public_key: String, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String, signature: String) {
        // check signature length
        assert!(string::length(&signature) == 128, ErrorMalformedSignature);

        // check public key length
        assert!(string::length(&x_only_public_key) == 64, ErrorMalformedPublicKey);

        // create event id
        let id = create_event_id(x_only_public_key, created_at, kind, tags, content);

        // get the hex decoded public key bytes
        let pubkey = hex::decode(&string::into_bytes(x_only_public_key));

        // get the hex decoded signature bytes
        let sig = hex::decode(&string::into_bytes(signature));

        // check the signature
        check_signature(id, pubkey, sig);

        // derive a rooch address
        let rooch_address = inner::derive_rooch_address(pubkey);

        // handle a range of different kinds of an Event
        if (kind == EVENT_KIND_USER_METADATA) {
            check_user_metadata(content);
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
            sig
        };
        let event_object = object::new_account_named_object<Event>(rooch_address, event);

        // emit a move event notification
        let event_object_id = object::id(&event_object);
        let move_event = NostrEventSavedEvent {
            id: event_object_id
        };
        event::emit(move_event);

        // transfer event object to the rooch address
        object::transfer_extend(event_object, rooch_address);
    }

    /// Entry function to save an Event
    public entry fun save_event_entry(x_only_public_key: String, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String, signature: String) {
        save_event(x_only_public_key, created_at, kind, tags, content, signature);
    }

    /// Save an Event with plaintext. Do not check integrity of created_at, kind, tags and content with id.
    public fun save_event_plaintext(id_encoded: String, x_only_public_key: String, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String, signature: String) {
        // check id length
        assert!(string::length(&id_encoded) == 64, ErrorMalformedId);

        // check public key length
        assert!(string::length(&x_only_public_key) == 64, ErrorMalformedPublicKey);

        // check signature length
        assert!(string::length(&signature) == 128, ErrorMalformedSignature);

        // get the hex decoded id bytes
        let id = hex::decode(&string::into_bytes(id_encoded));

        // get the hex decoded public key bytes
        let pubkey = hex::decode(&string::into_bytes(x_only_public_key));

        // get the hex decoded signature bytes
        let sig = hex::decode(&string::into_bytes(signature));

        // check the signature
        check_signature(id, pubkey, sig);

        // derive a rooch address
        let rooch_address = inner::derive_rooch_address(pubkey);

        // handle a range of different kinds of an Event
        if (kind == EVENT_KIND_USER_METADATA) {
            check_user_metadata(content);
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
            sig
        };
        let event_object = object::new_account_named_object<Event>(rooch_address, event);

        // emit a move event notification
        let event_object_id = object::id(&event_object);
        let move_event = NostrPlaintextEventSavedEvent {
            id: event_object_id
        };
        event::emit(move_event);

        // transfer event object to the rooch address
        object::transfer_extend(event_object, rooch_address);
    }

    /// Entry function to save an Event with plaintext
    public entry fun save_event_plaintext_entry(id: String, x_only_public_key: String, created_at: u64, kind: u16, tags: vector<vector<String>>, content: String, signature: String) {
        save_event_plaintext(id, x_only_public_key, created_at, kind, tags, content, signature);
    }

    /// drop a pre event
    fun drop_pre_event(pre_event: PreEvent) {
        let PreEvent {id: _, pubkey: _, created_at: _, kind: _, tags: _, content: _} = pre_event;
    }

    /// drop an event
    fun drop_event(event: Event) {
        let Event {id: _, pubkey: _, created_at: _, kind: _, tags: _, content: _, sig: _} = event;
    }

    public fun unpack_pre_event(pre_event: PreEvent): (vector<u8>, vector<u8>, u64, u16, vector<vector<String>>, String) {
        let PreEvent { id, pubkey, created_at, kind, tags, content } = pre_event;
        (id, pubkey, created_at, kind, tags, content)
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
