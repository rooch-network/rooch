module vector_object::vector_object {
    use moveos_std::object;
    use moveos_std::object::{Object, ObjectID};
    use std::vector;
    use std::signer;
    use std::debug;
    use std::string::String;
    use rooch_framework::coin;
    use rooch_framework::gas_coin::RGas;
    use rooch_framework::coin::Coin;

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

    entry public fun transfer_vector_object_nested(account: &signer, mock_object_list: vector<vector<Object<MockObject>>>) {
        debug::print(&mock_object_list);

        while(vector::length(&mock_object_list) > 0) {
            let inner_list = vector::pop_back(&mut mock_object_list);
            while(vector::length(&inner_list) > 0) {
                let mock_object_arg = vector::pop_back(&mut inner_list);
                object::transfer(mock_object_arg, signer::address_of(account));
            };
            vector::destroy_empty(inner_list);
        };

        vector::destroy_empty(mock_object_list);
    }

    entry public fun vector_nested(_account: &signer, nested_vector: vector<vector<u64>>) {
        while(vector::length(&nested_vector) > 0) {
            let inner_list = vector::pop_back(&mut nested_vector);
            while(vector::length(&inner_list) > 0) {
                let arg = vector::pop_back(&mut inner_list);
                debug::print(&arg);
            }
        }
    }

    entry public fun vector_string_argument(_account: &signer, string_argument: vector<String>) {
        debug::print(&string_argument);
    }

    entry public fun vector_object_id_argument(_account: &signer, string_argument: vector<ObjectID>) {
        debug::print(&string_argument);
    }

    entry public fun string_argument(_account: &signer, string_argument: String) {
        debug::print(&string_argument);
    }
}