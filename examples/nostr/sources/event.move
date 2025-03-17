// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module nostr::event {
    use moveos_std::object::{Self, Object, ObjectID};

    const EVENT_DATA_IDENTIFIER: String = "e";
    const USER_DATA_IDENTIFIER: String = "p";
    const EVENTS_DATA_IDENTIFIER: String = "a";

    /// Event
    #[data_struct]
    struct Event has key, store {
        id: ObjectID, // id is the Rooch ObjectID with lowercase hex-encoded sha256 representation of 32-bytes Rooch address
        pubkey: vector<u8>,
        created_at: u64,
        kind: u64,
        tags: vector<vector<Tags>>,
        content: String,
        sig: vector<u8>
    }

    /// Tags
    #[data_struct]
    pub struct Tags {
        /// For referring to an event
        event_data: EventData,
        /// For another user
        user_data: UserData,
        /// For addressable or replaceable events
        events_data: EventsData,
        /// For other non-null strings
        non_null_string: NonNullString
    }

    /// EventData with `e` identifier
    #[data_struct]
    pub struct EventData {
        id: ObjectID,
        url: Option<String>,
        pubkey: Option<vector<u8>>
    }

    /// UserData with `p` identifier
    #[data_struct]
    pub struct UserData {
        pubkey: vector<u8>,
        url: Option<String>
    }

    /// EventsData with `a` identifier
    #[data_struct]
    pub struct EventsData {
        kind: u64,
        pubkey: vector<u8>,
        value: Option<String>,
        url: Option<String>
    }

    /// NonNullString with non-empty value
    #[data_struct]
    pub struct NonNullString {
        str: String
    }
}
