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
        // Set initial timestamp
        timestamp::update_global_time_for_test_secs(1);

        let title = string::utf8(b"Test Room");
        let room_id = room::create_room(&account, title, true);
        let room = object::borrow_object<room::Room>(room_id);
        
        let (room_title, is_public, creator, created_at, last_active, status) = room::get_room_info(room);
        assert!(room_title == title, 0);
        assert!(is_public == true, 1);
        assert!(creator == signer::address_of(&account), 2);
        assert!(created_at == 1, 3);
        assert!(last_active == created_at, 4);
        assert!(status == 0, 5);

        room::delete_room_for_testing(&account, room_id);
    }

    #[test]
    fun test_send_message() {
        let account = create_account();
        // Set initial timestamp
        timestamp::update_global_time_for_test_secs(1);

        let room_id = room::create_room(&account, string::utf8(b"Test Room"), true);
        
        timestamp::update_global_time_for_test_secs(2);
        let message = string::utf8(b"Hello, World!");
        let room = object::borrow_mut_object_shared<room::Room>(room_id);
        room::send_message(&account, room, message);
        
        let room = object::borrow_object<room::Room>(room_id);
        let (_, _, _, _, last_active, _) = room::get_room_info(room);
        assert!(last_active == 2, 0);

        room::delete_room_for_testing(&account, room_id);
    }

    #[test]
    fun test_private_room_member_management() {
        let admin = create_account_with_address(@0x42);
        let member = create_account_with_address(@0x43);
        timestamp::update_global_time_for_test_secs(1);
        
        let room_id = room::create_room(&admin, string::utf8(b"Private Room"), false);
        let room = object::borrow_object<room::Room>(room_id);
        
        // Initially member should not be in the room
        assert!(!room::is_member(room, signer::address_of(&member)), 0);
        
        // Add member to room
        let room = object::borrow_mut_object_shared<room::Room>(room_id);
        room::add_member(&admin, room, signer::address_of(&member));
        
        // Now member should be in the room
        let room = object::borrow_object<room::Room>(room_id);
        assert!(room::is_member(room, signer::address_of(&member)), 1);
        
        timestamp::update_global_time_for_test_secs(2);
        let message = string::utf8(b"Member message");
        let room = object::borrow_mut_object_shared<room::Room>(room_id);
        room::send_message(&member, room, message);

        room::delete_room_for_testing(&admin, room_id);
    }

    #[test]
    #[expected_failure(abort_code = room::ErrorNotAuthorized)]
    fun test_unauthorized_message() {
        let admin = create_account_with_address(@0x42);
        let other = create_account_with_address(@0x44);
        timestamp::update_global_time_for_test_secs(1);
        
        let room_id = room::create_room(&admin, string::utf8(b"Private Room"), false);
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
        
        let room_id = room::create_room(&admin, string::utf8(b"Private Room"), false);
        let room = object::borrow_mut_object_shared<room::Room>(room_id);
        room::add_member(&other, room, signer::address_of(&new_member));

        room::delete_room_for_testing(&admin, room_id);
    }
}