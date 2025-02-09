module onchain_ai_chat::room {
    use std::string::{Self, String};
    use std::vector;
    use moveos_std::table::{Self, Table};
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::timestamp;
    use moveos_std::signer;
    use moveos_std::hex;

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

    // Add message type constants
    const MESSAGE_TYPE_USER: u8 = 0;
    const MESSAGE_TYPE_AI: u8 = 1;

    // Public functions to expose constants
    public fun message_type_user(): u8 { MESSAGE_TYPE_USER }
    public fun message_type_ai(): u8 { MESSAGE_TYPE_AI }
    public fun room_type_normal(): u8 { ROOM_TYPE_NORMAL }
    public fun room_type_ai(): u8 { ROOM_TYPE_AI }

    /// Member structure to store member information
    struct Member has store, drop {
        address: address,
        nickname: String,
        joined_at: u64,
        last_active: u64,
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
        messages: Table<u64, Message>,
        message_counter: u64,
        created_at: u64,
        last_active: u64,
        status: u8,
        room_type: u8,  // normal or AI chat room
    }

    struct Message has store, copy, drop {
        sender: address,
        content: String,
        timestamp: u64,
        message_type: u8,  // distinguish between user messages and AI responses
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
            created_at: timestamp::now_seconds(),
            last_active: timestamp::now_seconds(),
            status: ROOM_STATUS_ACTIVE,
            room_type,
        };
        let room_obj = object::new(room);
        let room_id = object::id(&room_obj);
        object::to_shared(room_obj);
        room_id
    }

    /// Send a message and trigger AI response if needed
    public fun send_message(
        account: &signer,
        room: &mut Object<Room>,
        content: String,
    ) {
        let sender = signer::address_of(account);
        let room_mut = object::borrow_mut(room);
        let now = timestamp::now_seconds();
        
        if (room_mut.is_public) {
            // In public rooms, sending a message automatically makes you a member
            if (!table::contains(&room_mut.members, sender) && 
                !vector::contains(&room_mut.admins, &sender)) {
                let member = Member {
                    address: sender,
                    nickname: string::utf8(b""), // Default empty nickname
                    joined_at: now,
                    last_active: now,
                };
                table::add(&mut room_mut.members, sender, member);
            }
        } else {
            // In private rooms, only existing members can send messages
            assert!(
                table::contains(&room_mut.members, sender) || 
                vector::contains(&room_mut.admins, &sender),
                ErrorNotAuthorized
            );
        };
        
        assert!(room_mut.status == ROOM_STATUS_ACTIVE, ErrorRoomInactive);

        let message = Message {
            sender,
            content,
            timestamp: timestamp::now_seconds(),
            message_type: MESSAGE_TYPE_USER,
        };

        table::add(&mut room_mut.messages, room_mut.message_counter, message);
        room_mut.message_counter = room_mut.message_counter + 1;

        // If this is an AI room, generate AI response
        if (room_mut.room_type == ROOM_TYPE_AI) {
            add_ai_response(room_mut, content);
        };

        room_mut.last_active = timestamp::now_seconds();
    }

    /// Add AI response to the room (will be implemented by the framework)
    fun add_ai_response(room: &mut Room, _user_message: String){
        let response_message = string::utf8(b"AI response to your message: ");
        let ai_message = Message {
            sender: @0x1,
            content: response_message,
            timestamp: timestamp::now_seconds(),
            message_type: MESSAGE_TYPE_AI,
        };
        
        table::add(&mut room.messages, room.message_counter, ai_message);
        room.message_counter = room.message_counter + 1;
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

        let now = timestamp::now_seconds();
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
            let msg = table::borrow(&room_ref.messages, i);
            vector::push_back(&mut messages, *msg);
            i = i + 1;
        };
        messages
    }

    /// Get messages with pagination
    /// @param room - the room object
    /// @param start_index - starting message index
    /// @param limit - maximum number of messages to return
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
            let msg = table::borrow(&room_ref.messages, i);
            vector::push_back(&mut messages, *msg);
            i = i + 1;
        };
        messages
    }

    /// Get total message count in the room
    public fun get_message_count(room: &Object<Room>): u64 {
        let room_ref = object::borrow(room);
        room_ref.message_counter
    }

    /// Get message type
    public fun get_message_type(message: &Message): u8 {
        message.message_type
    }

    /// Get message content
    public fun get_message_content(message: &Message): String {
        message.content
    }

    /// Get message sender
    public fun get_message_sender(message: &Message): address {
        message.sender
    }

    /// Get message timestamp
    public fun get_message_timestamp(message: &Message): u64 {
        message.timestamp
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

    /// Delete a room, only creator can delete
    public fun delete_room(account: &signer, room: Object<Room>) {
        let room_ref = object::borrow(&room);
        assert!(room_ref.creator == signer::address_of(account), ErrorNotAuthorized);
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

    /// Send a message to a room - entry function
    public entry fun send_message_entry(
        account: &signer,
        room: &mut Object<Room>,
        content: String
    ) {
        send_message(account, room, content);
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

    /// Delete a room - entry function
    public entry fun delete_room_entry(
        account: &signer,
        room_id: ObjectID 
    ) {
        let room = object::take_object_extend<Room>(room_id);
        delete_room(account, room);
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

    #[test_only]
    /// Test helper function to delete a room
    public fun delete_room_for_testing(account: &signer, room_id: ObjectID) {
        let room = object::take_object_extend<Room>(room_id);
        delete_room(account, room);
    }
}