#[test_only]
module onchain_ai_chat::room_tests {
    use std::string;
    use std::signer;
    use moveos_std::account;
    use moveos_std::timestamp;
    use moveos_std::object;
    use onchain_ai_chat::room;

    // Test helpers
    #[test_only]
    /// Test helper to create different accounts for testing
    fun create_account_with_address(addr: address): signer {
        account::create_signer_for_testing(addr)
    }

    #[test_only]
    fun create_account(): signer {
        create_account_with_address(@0x42)
    }

    #[test]
    fun test_create_room() {
        let account = create_account();
        timestamp::update_global_time_for_test(1000); // 1 second in milliseconds

        let title = string::utf8(b"Test Room");
        // Specify room type as NORMAL
        let room_id = room::create_room(&account, title, true, room::room_type_normal());
        let room = object::borrow_object<room::Room>(room_id);
        
        let (room_title, is_public, creator, created_at, last_active, status, room_type) = room::get_room_info(room);
        assert!(room_title == title, 0);
        assert!(is_public == true, 1);
        assert!(creator == signer::address_of(&account), 2);
        assert!(created_at == 1000, 3); // Check milliseconds
        assert!(last_active == created_at, 4);
        assert!(status == 0, 5);
        assert!(room_type == room::room_type_normal(), 6);

        room::delete_room_for_testing(&account, room_id);
    }

    #[test]
    fun test_create_ai_room() {
        let account = create_account();
        timestamp::update_global_time_for_test(1000);

        let title = string::utf8(b"AI Room");
        let room_id = room::create_room(&account, title, true, room::room_type_ai());
        let room = object::borrow_object<room::Room>(room_id);
        
        let (room_title, is_public, creator, created_at, last_active, status, room_type) = room::get_room_info(room);
        assert!(room_title == title, 0);
        assert!(is_public == true, 1);
        assert!(creator == signer::address_of(&account), 2);
        assert!(created_at == 1000, 3);
        assert!(last_active == created_at, 4);
        assert!(status == 0, 5);
        assert!(room_type == room::room_type_ai(), 6);

        room::delete_room_for_testing(&account, room_id);
    }

    // #[test]
    // fun test_ai_room_message() {
    //     let account = create_account();
    //     timestamp::update_global_time_for_test(1000);

    //     let room_id = room::create_room(&account, string::utf8(b"AI Room"), true, room::room_type_ai());
        
    //     timestamp::update_global_time_for_test(2000);
    //     let message = string::utf8(b"Hello AI!");
    //     let room = object::borrow_mut_object_shared<room::Room>(room_id);
    //     room::send_message(&account, room, message);
        
    //     let room = object::borrow_object<room::Room>(room_id);
    //     let (_, _, _, _, last_active, _, _) = room::get_room_info(room);
    //     assert!(last_active == 2000, 0);

    //     // Get messages and verify types
    //     let messages = room::get_messages(room);
    //     assert!(vector::length(&messages) == 2, 1); // User message + AI response
    //     let user_message = vector::borrow(&messages, 0);
    //     assert!(room::get_message_type(user_message) == room::message_type_user(), 2);
    //     let ai_message = vector::borrow(&messages, 1);
    //     assert!(room::get_message_type(ai_message) == room::message_type_ai(), 3);

    //     room::delete_room_for_testing(&account, room_id);
    // }

    #[test]
    #[expected_failure(abort_code = room::ErrorInvalidRoomType)]
    fun test_invalid_room_type() {
        let account = create_account();
        let invalid_type: u8 = 99;
        let _room_id = room::create_room(
            &account,
            string::utf8(b"Invalid Room"),
            true,
            invalid_type
        );
    }

    // Update all other test cases to include room_type parameter
    #[test]
    fun test_send_message() {
        let account = create_account();
        timestamp::update_global_time_for_test(1000);

        let room_id = room::create_room(&account, string::utf8(b"Test Room"), true, room::room_type_normal());
        
        timestamp::update_global_time_for_test(2000);
        let message = string::utf8(b"Hello, World!");
        let room = object::borrow_mut_object_shared<room::Room>(room_id);
        room::send_message(&account, room, message);
        
        let room = object::borrow_object<room::Room>(room_id);
        let (_, _, _, _, last_active, _, _) = room::get_room_info(room);
        assert!(last_active == 2000, 0);

        room::delete_room_for_testing(&account, room_id);
    }

    #[test]
    fun test_private_room_member_management() {
        let admin = create_account_with_address(@0x42);
        let member = create_account_with_address(@0x43);
        timestamp::update_global_time_for_test(1000);
        
        let room_id = room::create_room(&admin, string::utf8(b"Private Room"), false, room::room_type_normal());
        let room = object::borrow_object<room::Room>(room_id);
        
        // Initially member should not be in the room
        assert!(!room::is_member(room, signer::address_of(&member)), 0);
        
        // Add member to room with nickname
        let room = object::borrow_mut_object_shared<room::Room>(room_id);
        let nickname = string::utf8(b"Test Member");
        room::add_member(&admin, room, signer::address_of(&member), nickname);
        
        // Now member should be in the room
        let room = object::borrow_object<room::Room>(room_id);
        assert!(room::is_member(room, signer::address_of(&member)), 1);
        
        // Verify member info
        let (member_nickname, joined_at, last_active) = room::get_member_info(room, signer::address_of(&member));
        assert!(member_nickname == nickname, 2);
        assert!(joined_at == 1000, 3);
        assert!(last_active == joined_at, 4);
        
        // Test message sending
        timestamp::update_global_time_for_test(2000);
        let message = string::utf8(b"Member message");
        let room = object::borrow_mut_object_shared<room::Room>(room_id);
        room::send_message(&member, room, message);

        room::delete_room_for_testing(&admin, room_id);
    }

    #[test]
    fun test_public_room_auto_join() {
        let admin = create_account_with_address(@0x42);
        let user = create_account_with_address(@0x43);
        timestamp::update_global_time_for_test_secs(1);
        
        let room_id = room::create_room(&admin, string::utf8(b"Public Room"), true, room::room_type_normal());
        let room = object::borrow_object<room::Room>(room_id);
        
        // Initially user should not be in the room
        assert!(!room::is_member(room, signer::address_of(&user)), 0);
        
        // Send message to auto-join
        let room = object::borrow_mut_object_shared<room::Room>(room_id);
        room::send_message(&user, room, string::utf8(b"Auto join message"));
        
        // Now user should be in the room
        let room = object::borrow_object<room::Room>(room_id);
        assert!(room::is_member(room, signer::address_of(&user)), 1);

        room::delete_room_for_testing(&admin, room_id);
    }

    #[test]
    #[expected_failure(abort_code = room::ErrorNotAuthorized)]
    fun test_unauthorized_message() {
        let admin = create_account_with_address(@0x42);
        let other = create_account_with_address(@0x44);
        timestamp::update_global_time_for_test_secs(1);
        
        let room_id = room::create_room(&admin, string::utf8(b"Private Room"), false, room::room_type_normal());
        let room = object::borrow_mut_object_shared<room::Room>(room_id);
        room::send_message(&other, room, string::utf8(b"Unauthorized message"));

        room::delete_room_for_testing(&admin, room_id);
    }

    #[test]
    #[expected_failure(abort_code = room::ErrorNotAuthorized)]
    fun test_unauthorized_member_add() {
        let admin = create_account_with_address(@0x42);
        let other = create_account_with_address(@0x45);
        let new_member = create_account_with_address(@0x46);
        timestamp::update_global_time_for_test_secs(1);
        
        let room_id = room::create_room(&admin, string::utf8(b"Private Room"), false, room::room_type_normal());
        let room = object::borrow_mut_object_shared<room::Room>(room_id);
        room::add_member(&other, room, signer::address_of(&new_member), string::utf8(b"Unauthorized"));

        room::delete_room_for_testing(&admin, room_id);
    }
}