// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Implements NIP-01 for handling client and relay messages.
module nostr::message {
    use std::string::{Self, String};
    use std::option::{Self, Option};
    use moveos_std::event;
    use moveos_std::string_utils;
    use moveos_std::signer;
    use moveos_std::json;
    use moveos_std::object::{Self, ObjectID};
    use nostr::event::Event;

    // types of client and relay messages
    const EVENT_KEY: vector<u8> = b"EVENT";

    // types of client messages
    const REQ_KEY: vector<u8> = b"REQ";
    const CLOSE_KEY: vector<u8> = b"CLOSE";

    // types of relay messages
    const OK_KEY: vector<u8> = b"OK";
    const EOSE_KEY: vector<u8> = b"EOSE";
    const CLOSED_KEY: vector<u8> = b"CLOSED";
    const NOTICE_KEY: vector<u8> = b"NOTICE";

    // Error codes from 1000 onward
    const ErrorEventMessage: u64 = 1000;

    #[data_struct]
    /// Message
    struct Message has key, copy, drop {
        event: Option<String>,
        req: Option<String>,
        close: Option<String>,
        ok: Option<String>,
        eose: Option<String>,
        closed: Option<String>,
        notice: Option<String>
    }

    #[data_struct]
    /// Event message create notification for Move events
    struct NostrEventMessageCreatedEvent has copy, drop {
        id: ObjectID
    }

    public fun event_key_string(): String {
        string::utf8(EVENT_KEY)
    }

    public fun req_key_string(): String {
        string::utf8(REQ_KEY)
    }

    public fun close_key_string(): String {
        string::utf8(CLOSE_KEY)
    }

    public fun ok_key_string(): String {
        string::utf8(OK_KEY)
    }

    public fun eose_key_string(): String {
        string::utf8(EOSE_KEY)
    }

    public fun closed_key_string(): String {
        string::utf8(CLOSED_KEY)
    }

    public fun notice_key_string(): String {
        string::utf8(NOTICE_KEY)
    }

    /// Parse an Event JSON to an Event message
    public fun parse_event_message(event_json: vector<u8>): String {
        let event_message = string::utf8(b"");
        let left_sb = string::utf8(b"[");
        let right_sb = string::utf8(b"]");
        let double_qm = string::utf8(b"\"");
        let coma = string::utf8(b",");
        let space = string::utf8(b" ");

        // check event json integrity
        // let _ = json::from_json<Event>(event_json);

        string::append(&mut event_message, left_sb);

        // event key
        let event_key = event_key_string();
        string::append(&mut event_message, double_qm);
        string::append(&mut event_message, event_key);
        string::append(&mut event_message, double_qm);

        string::append(&mut event_message, coma);
        string::append(&mut event_message, space);

        // event json
        let event_string = string::utf8(event_json);
        string::append(&mut event_message, event_string);

        string::append(&mut event_message, right_sb);

        event_message
    }

    public entry fun save_event_message_entry(signer: &signer, event_json: vector<u8>) {
        // parse the event json to event message
        let event_message = parse_event_message(event_json);

        // get the rooch address from the signer
        let rooch_address = signer::address_of(signer);

        // get message for event message
        let message = Message {
            event: option::some<String>(event_message),
            req: option::none<String>(),
            close: option::none<String>(),
            ok: option::none<String>(),
            eose: option::none<String>(),
            closed: option::none<String>(),
            notice: option::none<String>()
        };

        // move the message to the rooch address mapped to the nostr public key
        let message_object = object::new_account_named_object<Message>(rooch_address, message);

        // emit a move event nofitication
        let message_object_id = object::id(&message_object);
        let move_event = NostrEventMessageCreatedEvent {
            id: message_object_id
        };
        event::emit(move_event);

        // transfer message object to the rooch address
        object::transfer_extend(message_object, rooch_address);
    }

    fun is_event_message(event_message: String): bool {
        let event_key = event_key_string();
        string_utils::contains(&event_message, &event_key)
    }
}
