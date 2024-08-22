Feature: Rooch CLI integration tests

    @serial
    Scenario: rooch rpc test
      Given a server for rooch_rpc_test
      Then cmd: "rpc request --method rooch_getStates --params '["/resource/0x3/0x3::account_coin_store::AutoAcceptCoins",{"decode":true}]' --json"
      #The object_type contians blank space, so, we should quote it
      Then assert: "'{{$.rpc[-1][0].object_type}}' == '0x2::object::DynamicField<0x1::string::String, 0x3::account_coin_store::AutoAcceptCoins>'"
      Then cmd: "rpc request --method rooch_getStates --params '["/object/0x3",{"decode":true}]' --json"
      Then assert: "{{$.rpc[-1][0].object_type}} == '0x2::account::Account'"
      Then cmd: "rpc request --method rooch_listStates --params '["/resource/0x3", null, null, {"decode":true}]' --json"
      Then assert: "'{{$.rpc[-1]}}' contains '0x3::account_coin_store::AutoAcceptCoins'"
      Then cmd: "rpc request --method rooch_getStates --params '["/object/0x4e8d2c243339c6e02f8b7dd34436a1b1eb541b0fe4d938f845f4dbb9d9f218a2",{"decode":true}]' --json"
      Then assert: "{{$.rpc[-1][0].object_type}} == '0x2::timestamp::Timestamp'"
      Then assert: "{{$.rpc[-1][0].decoded_value.value.milliseconds}} == 0"
      Then cmd: "rpc request --method rooch_getStates --params '["/object/0x2::timestamp::Timestamp",{"decode":true}]' --json"
      Then assert: "{{$.rpc[-1][0].object_type}} == '0x2::timestamp::Timestamp'"
      Then cmd: "rpc request --method rooch_getObjectStates --params '["0x4e8d2c243339c6e02f8b7dd34436a1b1eb541b0fe4d938f845f4dbb9d9f218a2", {"decode":false}]' --json"
      Then cmd: "rpc request --method rooch_getObjectStates --params '["0x4e8d2c243339c6e02f8b7dd34436a1b1eb541b0fe4d938f845f4dbb9d9f218a2", {"decode":true}]' --json"
      Then assert: "{{$.rpc[-1][0].object_type}} == '0x2::timestamp::Timestamp'"
      Then assert: "{{$.rpc[-1][0].value}} == {{$.rpc[-2][0].value}}"
      # ModuleStore is a named object, so we can directly use the struct tag as ObjectID arguments.
      # named_object_id(0x2::module_store::ModuleStore) == 0x2214495c6abca5dd5a2bf0f2a28a74541ff10c89818a1244af24c4874325ebdb
      # 0x3 is the rooch_framwork package address, the package's field key is the package address.
      Then cmd: "rpc request --method rooch_getFieldStates --params '["0x2::module_store::ModuleStore", ["0x3"], {"decode": true, "showDisplay": true}]' --json"
      Then assert: "{{$.rpc[-1][0].object_type}} == '0x2::module_store::Package'"
      Then cmd: "rpc request --method rooch_listFieldStates --params '["0x2214495c6abca5dd5a2bf0f2a28a74541ff10c89818a1244af24c4874325ebdb", null, "2", {"decode": false, "showDisplay": false}]' --json"
      Then assert: "{{$.rpc[-1].has_next_page}} == true"
      Then cmd: "rpc request --method rooch_getModuleABI --params '["0x2", "display"]'"
      Then assert: "{{$.rpc[-1].name}} == 'display'"
      Then stop the server 
    
    @serial
    Scenario: account
      Given a server for account

      Then cmd: "account create"
      Then cmd: "account list --json"
      Then cmd: "account export"
      Then cmd: "account export -a {{$.account[-1].account0.address}} --json"
      # use bitcoin_address
      Then cmd: "account nullify -a {{$.account[-2].account0.bitcoin_address}}"
      Then cmd: "account import -k {{$.account[-1].encoded_private_key}}"
      # use nostr_public_key
      Then cmd: "account nullify -a {{$.account[-2].account0.nostr_public_key}}"

      Then cmd: "rpc request --method rooch_getBalance --params '["{{$.address_mapping.default}}", "0x3::gas_coin::GasCoin"]' --json"
      Then assert: "'{{$.rpc[-1].coin_type}}' == '0x3::gas_coin::GasCoin'"
      Then assert: "'{{$.rpc[-1].balance}}' == '0'"

      # Get gas
      Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "rpc request --method rooch_getStates --params '["/object/0x4e8d2c243339c6e02f8b7dd34436a1b1eb541b0fe4d938f845f4dbb9d9f218a2",{"decode":true}]' --json"
      Then assert: "{{$.rpc[-1][0].object_type}} == '0x2::timestamp::Timestamp'"
      # ensure the tx_timestamp update the global timestamp
      Then assert: "{{$.rpc[-1][0].decoded_value.value.milliseconds}} != 0"

      # session key
      Then cmd: "session-key create  --app-name test --app-url https:://test.rooch.network --scope 0x3::empty::empty"
      Then cmd: "session-key list"
      Then assert: "'{{$.session-key[-1]}}' not_contains error"
      Then cmd: "move run --function 0x3::empty::empty  --session-key {{$.session-key[-1][0].name}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # transaction
      Then cmd: "transaction get-transactions-by-order --cursor 0 --limit 1 --descending-order false"
      Then cmd: "transaction get-transactions-by-hash --hashes {{$.transaction[-1].data[0].execution_info.tx_hash}}"
      Then cmd: "transaction build --function rooch_framework::empty::empty --json"

      # alias tx for transaction
      Then cmd: "tx get-transactions-by-order --cursor 1 --limit 2 --descending-order true"

      # account balance
      Then cmd: "account balance"
      Then cmd: "account balance --coin-type rooch_framework::gas_coin::GasCoin"
      Then cmd: "rpc request --method rooch_getBalance --params '["{{$.address_mapping.default}}", "0x3::gas_coin::GasCoin"]' --json"
      Then assert: "'{{$.rpc[-1].balance}}' != '0'"

      Then stop the server

    @serial
    Scenario: state
      Given a server for state
      Then cmd: "object -i 0x3" 
      Then cmd: "object -i 0x2::timestamp::Timestamp"
      Then cmd: "dynamic-field list-field-states --object-id 0x3"
      Then cmd: "dynamic-field get-field-states --object-id 0x3 --field-keys 0x064a9d6a507002868e9500d1a59e5b2760708a8d0bd64c78a55b9cc2cafdf6a0"
      Then assert: "{{$.dynamic-field[-2].data[0].state.id}} == {{$.dynamic-field[-1][0].id}}"
      Then cmd: "state --access-path /object/0x2::timestamp::Timestamp"
      Then assert: "{{$.state[-1][0].object_type}} == '0x2::timestamp::Timestamp'"
      Then cmd: "state --access-path /object/0x3::chain_id::ChainID"
      Then assert: "{{$.state[-1][0].object_type}} == '0x3::chain_id::ChainID'"
      Then assert: "{{$.state[-1][0].decoded_value.value.id}} == 4"
      Then cmd: "state --access-path /object/0x3::address_mapping::RoochToBitcoinAddressMapping"
      Then assert: "{{$.state[-1][0].object_type}} == '0x3::address_mapping::RoochToBitcoinAddressMapping'"
      Then cmd: "state --access-path /object/0x3::coin::CoinInfo<0x3::gas_coin::GasCoin>"
      Then assert: "{{$.state[-1][0].object_type}} == '0x3::coin::CoinInfo<0x3::gas_coin::GasCoin>'"
      Then stop the server

    @serial
    Scenario: event
    Given a server for event
    # event example and event prc
    Then cmd: "move publish -p ../../examples/event  --named-addresses rooch_examples=default --json"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    Then cmd: "move run --function default::event_test::emit_event  --args 10u64 --json"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    Then cmd: "move run --function default::event_test::emit_event  --args 11u64 --json"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    Then cmd: "move run --function default::event_test::emit_event  --args 11u64 --json"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    #cursor is None
    Then cmd: "event get-events-by-event-handle -t default::event_test::WithdrawEvent --limit 1  --descending-order false"
    Then assert: "{{$.event[-1].data[0].event_id.event_seq}} == 0"
    Then assert: "{{$.event[-1].next_cursor}} == 0"
    Then assert: "{{$.event[-1].has_next_page}} == true"
    Then cmd: "event get-events-by-event-handle -t default::event_test::WithdrawEvent --cursor 0 --limit 1  --descending-order false"
    Then assert: "{{$.event[-1].data[0].event_id.event_seq}} == 1"
    Then assert: "{{$.event[-1].next_cursor}} == 1"
    Then assert: "{{$.event[-1].has_next_page}} == true"
    Then cmd: "event get-events-by-event-handle -t default::event_test::WithdrawEvent --cursor 1 --limit 1  --descending-order false"
    Then assert: "{{$.event[-1].data[0].event_id.event_seq}} == 2"
    Then assert: "{{$.event[-1].has_next_page}} == false"
    Then stop the server

  @serial
  Scenario: indexer
    Given a server for indexer
    Then cmd: "move publish -p ../../examples/event  --named-addresses rooch_examples=default --json"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    Then cmd: "move run --function default::event_test::emit_event  --args 10u64 --json"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    Then cmd: "move run --function default::event_test::emit_event  --args 11u64 --json"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    
    # because the indexer is async update, so sleep 5 seconds to wait indexer update.
    Then sleep: "5"

    # genesis tx does not write indexer
    Then cmd: "rpc request --method rooch_queryTransactions --params '[{"tx_order_range":{"from_order":0,"to_order":2}}, null, "1", {"descending": true,"showDisplay":false}]' --json"
    Then assert: "{{$.rpc[-1].data[0].transaction.sequence_info.tx_order}} == 1"
    Then assert: "{{$.rpc[-1].next_cursor}} == 1"
    Then assert: "{{$.rpc[-1].has_next_page}} == true"
    Then cmd: "rpc request --method rooch_queryTransactions --params '[{"tx_order_range":{"from_order":0,"to_order":2}}, "1", "1", {"descending": true,"showDisplay":false}]' --json"
    Then assert: "{{$.rpc[-1].data[0].transaction.sequence_info.tx_order}} == 0"
    Then assert: "{{$.rpc[-1].next_cursor}} == 0"
    Then assert: "{{$.rpc[-1].has_next_page}} == false"
    Then cmd: "rpc request --method rooch_queryEvents --params '[{"tx_order_range":{"from_order":0, "to_order":2}}, null, "20", {"descending": true,"showDisplay":false}]' --json"
    Then assert: "{{$.rpc[-1].data[0].indexer_event_id.tx_order}} == 1"
    Then assert: "{{$.rpc[-1].next_cursor.tx_order}} == 0"
    Then assert: "{{$.rpc[-1].has_next_page}} == false"

    # Sync states
    Then cmd: "object -t 0x3::coin::CoinInfo --limit 10 -d"
    Then assert: "{{$.object[-1].data[0].tx_order}} == 1"
    Then assert: "{{$.object[-1].data[0].object_type}} == 0x3::coin::CoinInfo<0x3::gas_coin::GasCoin>"
    Then assert: "{{$.object[-1].has_next_page}} == false"

    Then cmd: "rpc request --method rooch_listFieldStates --params '["{{$.address_mapping.default}}", null, "10", {"descending": true,"showDisplay":false}]' --json"
    Then assert: "{{$.rpc[-1].has_next_page}} == false"

#    Then cmd: "rpc request --method rooch_syncStates --params '[null, null, "2", false]' --json"
#    Then assert: "{{$.rpc[-1].data[0].tx_order}} == 0"
#    Then assert: "{{$.rpc[-1].next_cursor.state_index}} == 1"
#    Then assert: "{{$.rpc[-1].has_next_page}} == true"

    Then stop the server

  @serial
    Scenario: kv_store example
      Given a server for kv_store
      Then cmd: "move publish -p ../../examples/kv_store  --named-addresses rooch_examples=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::kv_store::add_value --args string:key1 --args string:value1 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move view --function default::kv_store::get_value --args string:key1"
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} == value1"
      #the access-path argument do not support named address yet, so, we use `{{$.address_mapping.default}}` template var to repleace it.
      Then cmd: "state --access-path /resource/{{$.address_mapping.default}}/{{$.address_mapping.default}}::kv_store::KVStore"
      Then cmd: "state --access-path /fields/{{$.state[-1][0].decoded_value.value.value.value.table.value.handle.value.id}}/key1"
      Then assert: "{{$.state[-1][0].decoded_value.value.value}} == value1"


      Then stop the server

    @serial
    Scenario: entry_function example
      Given a server for entry_function

      Then cmd: "move publish -p ../../examples/entry_function_arguments/  --named-addresses rooch_examples=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_bool --args bool:true  --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_u8 --args u8:3 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_u8 --args 4u8  --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_address --args address:0x3242  --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_address --args @0x3242  --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_object_id --args object_id:0x3134  --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_string --args string:world  --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_vec_u8 --args "vector<u8>:2,3,4"  --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_vec_object_id --args "vector<object_id>:0x1324,0x41234,0x1234"  --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_mix --args 3u8 --args "vector<object_id>:0x2342,0x3132"  --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_object --args "object:default::entry_function::TestStruct"  --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_object_mut --args "object:default::entry_function::TestStruct"  --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then stop the server

  @serial
  Scenario: publish through MoveAction and module upgrade
      Given a server for publish_through_move_action

      # The counter example
      Then cmd: "move publish -p ../../examples/counter  --named-addresses rooch_examples=default --by-move-action --json"
      Then assert: "'{{$.move[-1]}}' contains INVALID_MODULE_PUBLISHER"

      Then stop the server

  @serial
  Scenario: publish_through_entry_function publish through Move entry function and module upgrade
      Given a server for publish_through_entry_function

      # The counter example
      Then cmd: "move publish -p ../../examples/counter  --named-addresses rooch_examples=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move view --function default::counter::value"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 0"
      Then cmd: "move run --function default::counter::increase  --json"
      Then cmd: "move view --function default::counter::value"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 1"
      Then cmd: "resource --address default --resource default::counter::Counter"
      Then assert: "{{$.resource[-1].decoded_value.value.value.value.value}} == 1"

      # The entry_function_arguments example
      Then cmd: "move publish -p ../../examples/entry_function_arguments_old/  --named-addresses rooch_examples=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_mix --args 3u8 --args "vector<object_id>:0x2342,0x3132"  --json"
      Then assert: "'{{$.move[-1]}}' contains FUNCTION_RESOLUTION_FAILURE"
      Then cmd: "move publish -p ../../examples/entry_function_arguments/  --named-addresses rooch_examples=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_mix --args 3u8 --args "vector<object_id>:0x2342,0x3132"  --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # check compatibility
      Then cmd: "move publish -p ../../examples/entry_function_arguments_old/  --named-addresses rooch_examples=default --json"
      Then assert: "'{{$.move[-1].execution_info.status.type}}' == 'moveabort'"

      Then stop the server

  @serial
  Scenario: coins example
      Given a server for coins
      Then cmd: "account create --json"
      Then cmd: "move publish -p ../../examples/coins  --named-addresses coins=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::fixed_supply_coin::faucet --args object:default::fixed_supply_coin::Treasury --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      
      Then cmd: "account list --json"

      Then cmd: "rpc request --method rooch_getBalance --params '["{{$.account[-1].default.bitcoin_address}}", "{{$.account[-1].default.hex_address}}::fixed_supply_coin::FSC"]' --json"
      Then assert: "'{{$.rpc[-1].coin_type}}' == '{{$.address_mapping.default}}::fixed_supply_coin::FSC'"
      Then assert: "'{{$.rpc[-1].balance}}' != '0'"

      Then cmd: "rpc request --method rooch_getBalance --params '["{{$.account[-1].default.nostr_public_key}}", "{{$.account[-1].default.hex_address}}::fixed_supply_coin::FSC"]' --json"
      Then assert: "'{{$.rpc[-1].coin_type}}' == '{{$.address_mapping.default}}::fixed_supply_coin::FSC'"
      Then assert: "'{{$.rpc[-1].balance}}' != '0'"

      Then cmd: "account transfer --coin-type default::fixed_supply_coin::FSC --to {{$.account[-1].account0.bitcoin_address}} --amount 1"

      Then assert: "{{$.account[-1].execution_info.status.type}} == executed"
      Then stop the server

  @serial
  Scenario: Issue a coin through module_template
    Given a server for issue_coin
    Then cmd: "move publish -p ../../examples/module_template/  --named-addresses rooch_examples=default --json"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    
    #TODO: uncomment this once move_module::binding_module_address is ready
    #Then cmd: "move run --function default::coin_factory::issue_fixed_supply_coin --args string:my_coin  --args string:"My first coin" --args string:MyCoin --args 1010101u256 --args 8u8   --json"
    #Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

    # check module `my_coin` is published, the name `my_coin` is the first arg in the last `move run` cmd.
    #Then cmd: "move run --function default::my_coin::faucet --args object:default::my_coin::Treasury --json"
    #Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

    #Then cmd: "rpc request --method rooch_getBalance --params '["{{$.address_mapping.default}}", "{{$.address_mapping.default}}::my_coin::MyCoin"]' --json"
    #Then assert: "'{{$.rpc[-1].coin_type}}' == '{{$.address_mapping.default}}::my_coin::MyCoin'"
    #Then assert: "'{{$.rpc[-1].balance}}' != '0'"
    Then stop the server
  
  @serial
  Scenario: basic_object example
      Given a server for basic_object
      Then cmd: "account create"
      Then cmd: "account list --json"
      Then cmd: "move publish -p ../../examples/basic_object  --named-addresses basic_object=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      
      #object pub transfer
      Then cmd: "move run --function default::third_party_module::create_and_pub_transfer --args u64:1 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "event get-events-by-event-handle -t default::pub_transfer::NewPubEvent"
      Then cmd: "move run --function 0x3::transfer::transfer_object --type-args default::pub_transfer::Pub --args address:{{$.account[-1].account0.hex_address}} --args object:{{$.event[-1].data[0].decoded_event_data.value.id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      #TODO FIXME the indexer do not update the owner after object transfer.
      Then sleep: "2"
      Then cmd: "object -t default::pub_transfer::Pub"
      Then assert: "{{$.object[-1].data[0].owner}} == {{$.account[-1].account0.address}}"
      
      #child object
      Then cmd: "move run --function default::third_party_module_for_child_object::create_child --args string:alice --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      
      Then cmd: "event get-events-by-event-handle -t default::child_object::NewChildEvent"
      Then cmd: "state --access-path /object/{{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.state[-1][0].decoded_value.value.name}} == alice"

      Then cmd: "move run --function default::third_party_module_for_child_object::update_child_name --args object:{{$.event[-1].data[0].decoded_event_data.value.id}} --args string:bob --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      
      Then cmd: "state --access-path /object/{{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.state[-1][0].decoded_value.value.name}} == bob"

      # because the indexer is async update, so sleep 2 seconds to wait indexer update.
      Then sleep: "2"

      Then cmd: "object -t {{$.address_mapping.default}}::child_object::Child --limit 10 -d"
      Then assert: "{{$.object[-1].data[0].id}} == {{$.event[-1].data[0].decoded_event_data.value.id}}"

      Then cmd: "move run --function default::third_party_module_for_child_object::update_child_age --args object:{{$.event[-1].data[0].decoded_event_data.value.id}} --args u64:10 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "move view --function default::child_object::get_age --args object:{{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 10" 
       
      Then cmd: "move run --function default::third_party_module_for_child_object::remove_child --args object:{{$.event[-1].data[0].decoded_event_data.value.id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then stop the server

  @serial
  Scenario: object display example
      Given a server for object_display
      Then cmd: "account create"
      Then cmd: "move publish -p ../../examples/display  --named-addresses display=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "move run --function default::display::create_object --sender default --args 'string:test_object' --args 'address:default' --args 'string:test object description' --json"
      
      Then cmd: "event get-events-by-event-handle -t default::display::NewObjectEvent"
      Then cmd: "state --access-path /object/{{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.state[-1][0].object_type}} == '{{$.address_mapping.default}}::display::ObjectType'"

      Then cmd: "rpc request --method rooch_getStates --params '["/object/{{$.event[-1].data[0].decoded_event_data.value.id}}", {"decode": false, "showDisplay": true}]' --json"
      Then assert: "{{$.rpc[-1][0].display_fields.fields.name}}  == test_object"

      # because the indexer is async update, so sleep 2 seconds to wait indexer update.
      Then sleep: "2"

      Then cmd: "rpc request --method rooch_queryObjectStates --params '[{"object_type":"{{$.address_mapping.default}}::display::ObjectType"}, null, "10", {"descending": false,"showDisplay":true}]' --json"
      Then assert: "{{$.rpc[-1].data[0].id}} == {{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.rpc[-1].data[0].display_fields.fields.name}} == test_object"

      Then cmd: "rpc request --method rooch_getObjectStates --params '["{{$.event[-1].data[0].decoded_event_data.value.id}}", {"decode": false, "showDisplay": true}]' --json"
      Then assert: "{{$.rpc[-1][0].display_fields.fields.name}} == test_object"

      
      Then stop the server
    
    @serial
    Scenario: wasm test
      # prepare servers
      Given a server for wasm_test

      # publish wasm execution
      Then cmd: "move publish -p ../../examples/wasm_execution  --named-addresses rooch_examples=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # test wasm trap
      Then cmd: "move run --function default::wasm_execution::run_trap --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # test wasm forever
      Then cmd: "move run --function default::wasm_execution::run_infinite_loop --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # test wasm alloc
      Then cmd: "move run --function default::wasm_execution::run_alloc --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # run wasm cpp generator
      Then cmd: "move run --function default::wasm_execution::run_generator_cpp --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # run wasm rust generator
      Then cmd: "move run --function default::wasm_execution::run_generator_rust --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # release servers
      Then stop the server

  @serial
    Scenario: view_function_loop example
      Given a server for view_function_loop
      Then cmd: "account create"
      Then cmd: "move publish -p ../../examples/view_function_loop  --named-addresses rooch_examples=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move view --function default::out_of_gas_loop::out_of_gas"
      Then assert: "{{$.move[-1].vm_status.ExecutionFailure.status_code}} == 4002"
