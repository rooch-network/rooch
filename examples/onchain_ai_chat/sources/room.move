module onchain_ai_chat::room {
    use std::string::String;
    use std::vector;
    use moveos_std::table::{Self, Table};
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::timestamp;
    use moveos_std::signer;

    // Error codes
    const ErrorRoomNotFound: u64 = 1;
    const ErrorRoomAlreadyExists: u64 = 2;
    const ErrorNotAuthorized: u64 = 3;
    const ErrorRoomInactive: u64 = 4;
    const ErrorMaxMembersReached: u64 = 5;
    const ErrorInvalidRoomName: u64 = 6;

    /// Room status constants
    const ROOM_STATUS_ACTIVE: u8 = 0;
    const ROOM_STATUS_CLOSED: u8 = 1;
    const ROOM_STATUS_BANNED: u8 = 2;

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
        members: vector<address>,
        messages: Table<u64, Message>,  // Changed from Object<Table> to Table
        message_counter: u64,
        created_at: u64,
        last_active: u64,
        status: u8,
    }

    struct Message has store, drop {
        sender: address,
        content: String,
        timestamp: u64,
    }

    /// Initialize a new room and make it shared
    public fun create_room(
        account: &signer,
        title: String,
        is_public: bool
    ): ObjectID {
        let creator = signer::address_of(account);
        let room = Room {
            title,
            is_public,
            creator,
            admins: vector::singleton(creator),
            members: vector::empty(),
            messages: table::new(),
            message_counter: 0,
            created_at: timestamp::now_seconds(),
            last_active: timestamp::now_seconds(),
            status: ROOM_STATUS_ACTIVE,
        };
        let room_obj = object::new(room);
        let room_id = object::id(&room_obj);
        object::to_shared(room_obj);
        room_id
    }

    /// Send a message to the room
    /// For public rooms, sender will be automatically added as a member if not already
    /// For private rooms, only existing members can send messages
    public fun send_message(
        account: &signer,
        room: &mut Object<Room>,
        content: String
    ) {
        let sender = signer::address_of(account);
        let room_mut = object::borrow_mut(room);
        
        if (room_mut.is_public) {
            // In public rooms, sending a message automatically makes you a member
            if (!vector::contains(&room_mut.members, &sender) && 
                !vector::contains(&room_mut.admins, &sender)) {
                vector::push_back(&mut room_mut.members, sender);
            }
        } else {
            // In private rooms, only existing members can send messages
            assert!(
                vector::contains(&room_mut.members, &sender) || 
                vector::contains(&room_mut.admins, &sender),
                ErrorNotAuthorized
            );
        };
        
        assert!(room_mut.status == ROOM_STATUS_ACTIVE, ErrorRoomInactive);

        let message = Message {
            sender,
            content,
            timestamp: timestamp::now_seconds(),
        };

        table::add(&mut room_mut.messages, room_mut.message_counter, message);
        room_mut.message_counter = room_mut.message_counter + 1;
        room_mut.last_active = timestamp::now_seconds();
    }

    /// Add member to private room
    public fun add_member(
        account: &signer,
        room: &mut Object<Room>,
        member: address
    ) {
        let sender = signer::address_of(account);
        let room_mut = object::borrow_mut(room);
        
        // Check if sender is admin
        assert!(vector::contains(&room_mut.admins, &sender), ErrorNotAuthorized);
        
        // Check if room is active
        assert!(room_mut.status == ROOM_STATUS_ACTIVE, ErrorRoomInactive);
        
        if (!vector::contains(&room_mut.members, &member)) {
            vector::push_back(&mut room_mut.members, member);
        };
    }

    /// Get room info
    public fun get_room_info(room: &Object<Room>): (String, bool, address, u64, u64, u8) {
        let room_ref = object::borrow(room);
        (
            room_ref.title,
            room_ref.is_public,
            room_ref.creator,
            room_ref.created_at,
            room_ref.last_active,
            room_ref.status
        )
    }

    /// Check if address is member of room
    public fun is_member(room: &Object<Room>, addr: address): bool {
        let room_ref = object::borrow(room);
        vector::contains(&room_ref.members, &addr) || 
        vector::contains(&room_ref.admins, &addr)
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
            members: _,
            messages,
            message_counter: _,
            created_at: _,
            last_active: _,
            status: _,
        } = object::remove(room);
        table::drop(messages);
    }

    /// Create a new room - entry function
    public entry fun create_room_entry(
        account: &signer,
        title: String,
        is_public: bool
    ) {
        let _room_id = create_room(account, title, is_public);
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
        add_member(account, room, member);
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