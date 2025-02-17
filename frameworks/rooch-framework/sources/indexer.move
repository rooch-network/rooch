// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::indexer {
    use std::signer;
    use std::string::String;
    use rooch_framework::core_addresses;
    use rooch_framework::onchain_config;
    use moveos_std::object;
    use moveos_std::object::{ObjectID};
    use moveos_std::event;

    struct FieldIndexerTablePlaceholder has key {
        _placeholder: bool,
    }

    struct FieldIndexerData has copy, store, drop {
        // id: ObjectID,
        /// Describes how to parse a value from a Struct, using a json path.
        path: String,
        /// use for expandsion
        ext: String
    }

    struct AddFieldIndexerEvent has copy, drop {
        id: ObjectID,
        path: String,
        ext: String
    }

    fun init() {
        let field_indexer_id = object::named_object_id<FieldIndexerTablePlaceholder>();
        if(!object::exists_object(field_indexer_id)){
            let field_indexer_obj = object::new_named_object(FieldIndexerTablePlaceholder{
                _placeholder: false
            });
            object::transfer_extend(field_indexer_obj, @rooch_framework);
        };
    }

    public entry fun add_field_indexer_entry(account: &signer, id: ObjectID, path: String, ext: String) {
        add_field_indexer(account, id, path, ext);
    }

    public fun add_field_indexer(account: &signer, id: ObjectID, path: String, ext: String) {
        let account_addr = signer::address_of(account);
        if(!core_addresses::is_rooch_genesis_address(account_addr)) {
            onchain_config::ensure_admin(account);
        };

        let field_indexer_id = object::named_object_id<FieldIndexerTablePlaceholder>();
        let field_indexer = object::borrow_mut_object_extend<FieldIndexerTablePlaceholder>(field_indexer_id);
        object::add_field(field_indexer, id, FieldIndexerData {
            path,
            ext
        });

        event::emit(
            AddFieldIndexerEvent {
                id,
                path,
                ext,
            }
        );
    }

    #[test_only]
    use std::string;

    #[test_only]
    struct TestStruct has store, copy, drop {
        id: u64,
    }

    #[test]
    fun test_indexer(){
        let admin_account = moveos_std::signer::module_signer<FieldIndexerTablePlaceholder>();
        rooch_framework::genesis::init_for_test();
        init();

        let id = object::named_object_id<TestStruct>();
        add_field_indexer(&admin_account, id, string::utf8(b"test"), string::utf8(b"ext"));
        let field_indexer_id = object::named_object_id<FieldIndexerTablePlaceholder>();
        let field_indexer_obj = object::borrow_object<FieldIndexerTablePlaceholder>(field_indexer_id);
        assert!(object::contains_field(field_indexer_obj, id), 1001);
    }
}