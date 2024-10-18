module vector_object::vector_object {
    use moveos_std::object;
    use moveos_std::object::Object;
    use std::vector;
    use std::signer;
    use std::debug;

    struct MockObject has key,store,drop,copy {value: u64}

    entry public fun create_mock_object_to_sender(account: &signer) {
        let user_object = object::new(MockObject{value: 123});
        let object_id = object::id(&user_object);
        debug::print(&object_id);
        object::transfer(user_object, signer::address_of(account));
    }

    entry public fun create_mock_object_to_user(_account: &signer, user_address: address) {
        let user_object = object::new(MockObject{value: 123});
        let object_id = object::id(&user_object);
        debug::print(&object_id);
        object::transfer(user_object, user_address);
    }

    entry public fun create_named_mock_object_to_sender(account: &signer) {
        let user_object = object::new_named_object(MockObject{value: 123});
        let object_id = object::id(&user_object);
        debug::print(&object_id);
        object::transfer(user_object, signer::address_of(account));
    }

    entry public fun create_named_mock_object_to_user(_account: &signer, user_address: address) {
        let user_object = object::new_named_object(MockObject{value: 123});
        let object_id = object::id(&user_object);
        debug::print(&object_id);
        object::transfer(user_object, user_address);
    }

    entry public fun transfer_vector_object(account: &signer, mock_object_list: vector<Object<MockObject>>) {
        debug::print(&mock_object_list);

        while(vector::length(&mock_object_list) > 0) {
            let mock_object_arg = vector::pop_back(&mut mock_object_list);
            object::transfer(mock_object_arg, signer::address_of(account));
        };

        vector::destroy_empty(mock_object_list);
    }
}