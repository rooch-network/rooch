module onchain_ai_chat::room {
    use std::string::{Self, String};
    use std::vector;
    use std::option::{Self, Option};
    use moveos_std::table::{Self, Table};
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::timestamp;
    use moveos_std::signer;
    use moveos_std::hex;
    use onchain_ai_chat::ai_service;
    use onchain_ai_chat::message::{Self, Message};

    friend onchain_ai_chat::ai_callback;

    // Error codes
    const ErrorRoomNotFound: u64 = 1;
    const ErrorRoomAlreadyExists: u64 = 2;
    const ErrorNotAuthorized: u64 = 3;
    const ErrorRoomInactive: u64 = 4;
    const ErrorMaxMembersReached: u64 = 5;
    const ErrorInvalidRoomName: u64 = 6;
    const ErrorInvalidRoomType: u64 = 7;

    /// Room status constants
    const ROOM_STATUS_ACTIVE: u8 = 0;
    const ROOM_STATUS_CLOSED: u8 = 1;
    const ROOM_STATUS_BANNED: u8 = 2;

    // Add room type constants
    const ROOM_TYPE_NORMAL: u8 = 0;
    const ROOM_TYPE_AI: u8 = 1;

    // Public functions to expose constants
    public fun room_type_normal(): u8 { ROOM_TYPE_NORMAL }
    public fun room_type_ai(): u8 { ROOM_TYPE_AI }

    /// Member structure to store member information
    struct Member has store, drop {
        address: address,
        nickname: String,
        joined_at: u64,    // Now in milliseconds
        last_active: u64,  // Now in milliseconds
    }

    /// Room structure for chat functionality
    /// Note on privacy:
    /// - All messages in the room are visible on-chain, regardless of room privacy settings
    /// - is_public: true  => Anyone can join the room automatically when sending their first message
    /// - is_public: false => Only admins can add members, and only members can send messages
    struct Room has key {
        title: String,
        is_public: bool,
        creator: address,
        admins: vector<address>,
        members: Table<address, Member>,  // Changed from vector to Table
        messages: Table<u64, ObjectID>,  // Changed from Table<u64, Message> to Table<u64, ObjectID>
        message_counter: u64,
        created_at: u64,    // Now in milliseconds
        last_active: u64,   // Now in milliseconds
        status: u8,
        room_type: u8,  // normal or AI chat room
    }

    /// Initialize a new room with room type
    public fun create_room(
        account: &signer,
        title: String,
        is_public: bool,
        room_type: u8,
    ): ObjectID {
        assert!(
            room_type == ROOM_TYPE_NORMAL || room_type == ROOM_TYPE_AI,
            ErrorInvalidRoomType
        );
        
        let creator = signer::address_of(account);
        let room = Room {
            title,
            is_public,
            creator,
            admins: vector::singleton(creator),
            members: table::new(),  // Initialize empty table
            messages: table::new(),
            message_counter: 0,
            created_at: timestamp::now_milliseconds(),
            last_active: timestamp::now_milliseconds(),
            status: ROOM_STATUS_ACTIVE,
            room_type,
        };
        let room_obj = object::new(room);
        let room_id = object::id(&room_obj);
        object::to_shared(room_obj);
        room_id
    }

    /// Add message to room - use message_counter as id
    fun add_message(room: &mut Room, sender: address, content: String, message_type: u8) {
        let msg_id = message::new_message(
            room.message_counter,
            sender,
            content,
            message_type
        );
        table::add(&mut room.messages, room.message_counter, msg_id);
        room.message_counter = room.message_counter + 1;
    }

    /// Send a message and trigger AI response if needed
    public fun send_message(
        account: &signer,
        room_obj: &mut Object<Room>,
        content: String,
    ) {
        let sender = signer::address_of(account);
        let now = timestamp::now_milliseconds();
        
        let room = object::borrow_mut(room_obj);

        if (room.is_public) {
            // In public rooms, sending a message automatically makes you a member
            if (!table::contains(&room.members, sender) && 
                !vector::contains(&room.admins, &sender)) {
                let member = Member {
                    address: sender,
                    nickname: generate_default_nickname(sender), // Use default nickname generator
                    joined_at: now,
                    last_active: now,
                };
                table::add(&mut room.members, sender, member);
            }
        } else {
            // In private rooms, only existing members can send messages
            assert!(
                table::contains(&room.members, sender) || 
                vector::contains(&room.admins, &sender),
                ErrorNotAuthorized
            );
        };
        
        assert!(room.status == ROOM_STATUS_ACTIVE, ErrorRoomInactive);

        add_message(room, sender, content, message::type_user());

        room.last_active = timestamp::now_milliseconds();
    }

    /// Add AI response to the room (will be implemented by the framework)
    public(friend) fun add_ai_response(room: &mut Room, response_message: String){
        add_message(room, @onchain_ai_chat, response_message, message::type_ai());
    }

    /// Generate default nickname from address
    fun generate_default_nickname(addr: address): String {
        let addr_bytes = std::bcs::to_bytes(&addr);
        let prefix = vector::empty<u8>();
        // Copy first 4 bytes
        let i = 0;
        while (i < 4 && i < vector::length(&addr_bytes)) {
            vector::push_back(&mut prefix, *vector::borrow(&addr_bytes, i));
            i = i + 1;
        };
        
        let nickname = b"user_0x";
        vector::append(&mut nickname, hex::encode(prefix));
        string::utf8(nickname)
    }

    /// Add member to private room with nickname
    public fun add_member(
        account: &signer,
        room: &mut Object<Room>,
        member_addr: address,
        nickname: String,
    ) {
        let sender = signer::address_of(account);
        let room_mut = object::borrow_mut(room);
        
        // Check if sender is admin
        assert!(vector::contains(&room_mut.admins, &sender), ErrorNotAuthorized);
        
        // Check if room is active
        assert!(room_mut.status == ROOM_STATUS_ACTIVE, ErrorRoomInactive);
        
        // Check if member already exists
        assert!(!table::contains(&room_mut.members, member_addr), ErrorRoomAlreadyExists);

        let now = timestamp::now_milliseconds();
        let member = Member {
            address: member_addr,
            nickname,
            joined_at: now,
            last_active: now,
        };
        
        table::add(&mut room_mut.members, member_addr, member);
    }

    /// Get room information
    public fun get_room_info(room: &Object<Room>): (String, bool, address, u64, u64, u8, u8) {
        let room_ref = object::borrow(room);
        (
            room_ref.title,
            room_ref.is_public,
            room_ref.creator,
            room_ref.created_at,
            room_ref.last_active,
            room_ref.status,
            room_ref.room_type
        )
    }

    /// Get all messages in the room
    public fun get_messages(room: &Object<Room>): vector<Message> {
        let room_ref = object::borrow(room);
        let messages = vector::empty<Message>();
        let i = 0;
        while (i < room_ref.message_counter) {
            let msg_id = table::borrow(&room_ref.messages, i);
            let msg_obj = object::borrow_object<Message>(*msg_id);
            vector::push_back(&mut messages, *object::borrow(msg_obj));
            i = i + 1;
        };
        messages
    }

    /// Get messages with pagination
    public fun get_messages_paginated(
        room: &Object<Room>, 
        start_index: u64,
        limit: u64
    ): vector<Message> {
        let room_ref = object::borrow(room);
        let messages = vector::empty<Message>();
        
        // Check if start_index is valid
        if (start_index >= room_ref.message_counter) {
            return messages
        };
        
        // Calculate end index
        let end_index = if (start_index + limit > room_ref.message_counter) {
            room_ref.message_counter
        } else {
            start_index + limit
        };
        
        let i = start_index;
        while (i < end_index) {
            let msg_id = table::borrow(&room_ref.messages, i);
            let msg_obj = object::borrow_object<Message>(*msg_id);
            vector::push_back(&mut messages, *object::borrow(msg_obj));
            i = i + 1;
        };
        messages
    }

    /// Get total message count in the room
    public fun get_message_count(room: &Object<Room>): u64 {
        let room_ref = object::borrow(room);
        room_ref.message_counter
    }

    /// Get last N messages from the room
    public fun get_last_messages(room_obj: &Object<Room>, limit: u64): vector<Message> {
        let room = object::borrow(room_obj);
        let messages = vector::empty();
        let start = if (room.message_counter > limit) {
            room.message_counter - limit
        } else {
            0
        };
        
        let i = start;
        while (i < room.message_counter) {
            let msg_id = table::borrow(&room.messages, i);
            let msg_obj = object::borrow_object<Message>(*msg_id);
            vector::push_back(&mut messages, *object::borrow(msg_obj));
            i = i + 1;
        };
        messages
    }

    /// Check if address is member of room
    public fun is_member(room: &Object<Room>, addr: address): bool {
        let room_ref = object::borrow(room);
        table::contains(&room_ref.members, addr) || 
        vector::contains(&room_ref.admins, &addr)
    }

    /// Get member info
    public fun get_member_info(room: &Object<Room>, addr: address): (String, u64, u64) {
        let room_ref = object::borrow(room);
        assert!(table::contains(&room_ref.members, addr), ErrorNotAuthorized);
        let member = table::borrow(&room_ref.members, addr);
        (
            member.nickname,
            member.joined_at,
            member.last_active
        )
    }

    /// Create a new room - entry function
    public entry fun create_room_entry(
        account: &signer,
        title: String,
        is_public: bool
    ) {
        let _room_id = create_room(account, title, is_public, ROOM_TYPE_NORMAL);
    }

    /// Create a new AI room - entry function
    public entry fun create_ai_room_entry(
        account: &signer,
        title: String,
        is_public: bool,
    ) {
        let _room_id = create_room(account, title, is_public, ROOM_TYPE_AI);
    }

    /// Send a message and trigger AI response if needed
    public entry fun send_message_entry(
        account: &signer,
        room_obj: &mut Object<Room>,
        content: String
    ) {
        let room_id = object::id(room_obj);
        let is_ai_room = object::borrow(room_obj).room_type == ROOM_TYPE_AI;
        
        // If it's an AI room, request AI response
        if (is_ai_room) {
            //TODO make the number of messages to fetch configurable
            let message_limit: u64 = 10;
            let prev_messages = get_last_messages(room_obj, message_limit);
            ai_service::request_ai_response(
                account,
                room_id,
                content,
                prev_messages
            );
        };
        send_message(account, room_obj, content);
    }

    /// Add a member to a private room - entry function
    public entry fun add_member_entry(
        account: &signer,
        room: &mut Object<Room>,
        member: address
    ) {
        let nickname = generate_default_nickname(member);
        add_member(account, room, member, nickname);
    }

    /// Change room status (active/closed/banned) - entry function
    public entry fun change_room_status_entry(
        account: &signer,
        room: &mut Object<Room>,
        new_status: u8
    ) {
        let sender = signer::address_of(account);
        let room_mut = object::borrow_mut(room);
        assert!(room_mut.creator == sender, ErrorNotAuthorized);
        assert!(
            new_status == ROOM_STATUS_ACTIVE || 
            new_status == ROOM_STATUS_CLOSED || 
            new_status == ROOM_STATUS_BANNED,
            ErrorInvalidRoomName
        );
        room_mut.status = new_status;
    }

    /// Create a AI room and send the first message in one transaction
    public entry fun create_ai_room_with_message_entry(
        account: &signer,
        title: String,
        is_public: bool,
        first_message: String,
    ) {
        
        // Create the room
        let room_id = create_room(account, title, is_public, ROOM_TYPE_AI);
        
        // Get room object and send message
        let room_obj = object::borrow_mut_object_shared<Room>(room_id);
        
        let prev_messages = vector::empty();
        ai_service::request_ai_response(
            account,
            room_id,
            first_message,
            prev_messages
        );
        send_message(account, room_obj, first_message);
    }

    /// Update room title
    public(friend) fun update_room_title(room_obj: &mut Room, new_title: String) {
        room_obj.title = new_title;
    }

    /// Parse title from AI response - first line starting with #
    public fun parse_title_from_response(content: &String): Option<String> {
        let first_line_end = string::index_of(content, &string::utf8(b"\n"));
        // If no newline found (index_of returns length if not found)
        if (first_line_end == string::length(content)) {
            return option::none()
        };
        
        let first_line = string::sub_string(content, 0, first_line_end);
        
        if (starts_with(string::bytes(&first_line), &b"# ")) {
            // Remove "# " prefix, using first_line_end for length
            option::some(string::sub_string(&first_line, 2, first_line_end))
        } else {
            option::none()
        }
    }

    //TODO migrate to std::string::starts_with
    fun starts_with(haystack: &vector<u8>, needle: &vector<u8>): bool {
        let haystack_len = vector::length(haystack);
        let needle_len = vector::length(needle);

        if (needle_len > haystack_len) {
            return false
        };

        let i = 0;
        while (i < needle_len) {
            if (vector::borrow(haystack, i) != vector::borrow(needle, i)) {
                return false
            };
            i = i + 1;
        };

        true
    }

    #[test]
    fun test_parse_title_from_response() {
        let content = string::utf8(b"# Understanding Move\nMove is a programming language...");
        let title_opt = parse_title_from_response(&content);
        assert!(option::is_some(&title_opt), 1);
        assert!(option::borrow(&title_opt) == &string::utf8(b"Understanding Move"), 2);

        // Test without newline
        let content = string::utf8(b"# No newline");
        let title_opt = parse_title_from_response(&content);
        assert!(option::is_none(&title_opt), 3);

        // Test without title
        let content = string::utf8(b"This is not a title\nSecond line");
        let title_opt = parse_title_from_response(&content);
        assert!(option::is_none(&title_opt), 4);
    }

    #[test]
    fun test_send_message() {
        use moveos_std::account;
        
        let alice = account::create_signer_for_testing(@0x42);
        timestamp::update_global_time_for_test(1000);

        let room_id = create_room(&alice, string::utf8(b"Test Room"), true, room_type_normal());
        
        timestamp::update_global_time_for_test(2000);
        let message = string::utf8(b"Hello, World!");
        let room = object::borrow_mut_object_shared<Room>(room_id);
        send_message(&alice, room, message);
        
        let room = object::borrow_object<Room>(room_id);
        let messages = get_messages(room);
        assert!(vector::length(&messages) == 1, 0);
        
        let msg = vector::borrow(&messages, 0);
        assert!(message::get_sender(msg) == signer::address_of(&alice), 1);
        assert!(message::get_content(msg) == message, 2);

        // Check message ownership
        let msg_id = *table::borrow(&object::borrow(room).messages, 0);
        let msg_obj = object::borrow_object<Message>(msg_id);
        assert!(object::owner(msg_obj) == signer::address_of(&alice), 3);

        delete_room_for_testing(room_id);
    }

    #[test_only]
    /// Test helper function to delete a room, only available in test mode
    fun force_delete_room(room: Object<Room>) {
        let Room { 
            title: _,
            is_public: _,
            creator: _,
            admins: _,
            members,
            messages,
            message_counter: _,
            created_at: _,
            last_active: _,
            status: _,
            room_type: _,
        } = object::remove(room);
        
        table::drop(members);
        table::drop(messages);
    }

    #[test_only]
    /// Public test helper function to delete a room
    public fun delete_room_for_testing(room_id: ObjectID) {
        let room = object::take_object_extend<Room>(room_id);
        force_delete_room(room);
    }

}