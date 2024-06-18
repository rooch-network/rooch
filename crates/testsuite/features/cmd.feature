Feature: Rooch CLI integration tests
    @serial
    Scenario: Init
      Then cmd: "init --skip-password"
      Then cmd: "env switch --alias local"
 
    @serial
    Scenario: rooch rpc test
      Given a server for rooch_rpc_test
      Then cmd: "rpc request --method rooch_getStates --params '["/resource/0x3/0x3::account_coin_store::AutoAcceptCoins",{"decode":true}]' --json"
      Then assert: "{{$.rpc[-1][0].value_type}} == '0x3::account_coin_store::AutoAcceptCoins'"
      Then cmd: "rpc request --method rooch_getStates --params '["/object/0x3",{"decode":true}]' --json"
      Then assert: "{{$.rpc[-1][0].value_type}} == '0x2::object::ObjectEntity<0x2::account::Account>'"
      Then cmd: "rpc request --method rooch_listStates --params '["/resource/0x3", null, null, {"decode":true}]' --json"
      Then assert: "'{{$.rpc[-1]}}' contains '0x3::account_coin_store::AutoAcceptCoins'"
      Then cmd: "rpc request --method rooch_getStates --params '["/object/0x5921974509dbe44ab84328a625f4a6580a5f89dff3e4e2dec448cb2b1c7f5b9",{"decode":true}]' --json"
      Then assert: "{{$.rpc[-1][0].value_type}} == '0x2::object::ObjectEntity<0x2::object::Timestamp>'"
      Then assert: "{{$.rpc[-1][0].decoded_value.value.value.value.milliseconds}} == 0"
      Then cmd: "rpc request --method rooch_getStates --params '["/object/0x2::object::Timestamp",{"decode":true}]' --json"
      Then assert: "{{$.rpc[-1][0].value_type}} == '0x2::object::ObjectEntity<0x2::object::Timestamp>'"
      Then cmd: "rpc request --method rooch_getObjectStates --params '["0x5921974509dbe44ab84328a625f4a6580a5f89dff3e4e2dec448cb2b1c7f5b9", {"decode":false}]' --json"
      Then cmd: "rpc request --method rooch_getObjectStates --params '["0x5921974509dbe44ab84328a625f4a6580a5f89dff3e4e2dec448cb2b1c7f5b9", {"decode":true}]' --json"
      Then assert: "{{$.rpc[-1][0].object_type}} == '0x2::object::Timestamp'"
      Then assert: "{{$.rpc[-1][0].value}} == {{$.rpc[-2][0].value}}"
      Then cmd: "rpc request --method rooch_getFieldStates --params '["0x2214495c6abca5dd5a2bf0f2a28a74541ff10c89818a1244af24c4874325ebdb", ["0x41022214495c6abca5dd5a2bf0f2a28a74541ff10c89818a1244af24c4874325ebdb8238d4e7553801ebf92b4311e16bbeb26eec676fd5bcbb31dcc59610148d90c8070000000000000000000000000000000000000000000000000000000000000002066f626a656374084f626a656374494400"], {"decode": true, "showDisplay": true}]' --json"
      Then assert: "{{$.rpc[-1][0].value_type}} == '0x2::object::ObjectEntity<0x2::module_store::Package>'"
      Then cmd: "rpc request --method rooch_listFieldStates --params '["0x2214495c6abca5dd5a2bf0f2a28a74541ff10c89818a1244af24c4874325ebdb", null, "2", {"decode": false, "showDisplay": false}]' --json"
      Then assert: "{{$.rpc[-1].has_next_page}} == true"
      Then stop the server 
    
    @serial
    Scenario: account
      Given a server for account

      Then cmd: "account create"
      Then cmd: "account list --json"
      # use bitcoin_address
      Then cmd: "account nullify -a {{$.account[-1].account0.bitcoin_address}}"

      Then cmd: "rpc request --method rooch_getBalance --params '["{{$.address_mapping.default}}", "0x3::gas_coin::GasCoin"]' --json"
      Then assert: "'{{$.rpc[-1].coin_type}}' == '0x3::gas_coin::GasCoin'"
      Then assert: "'{{$.rpc[-1].balance}}' == '0'"

      # Get gas
      Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "rpc request --method rooch_getStates --params '["/object/0x5921974509dbe44ab84328a625f4a6580a5f89dff3e4e2dec448cb2b1c7f5b9",{"decode":true}]' --json"
      Then assert: "{{$.rpc[-1][0].value_type}} == '0x2::object::ObjectEntity<0x2::object::Timestamp>'"
      # ensure the tx_timestamp update the global timestamp
      Then assert: "{{$.rpc[-1][0].decoded_value.value.value.value.milliseconds}} != 0"

      # session key
      Then cmd: "session-key create  --app-name test --app-url https:://test.rooch.network --scope 0x3::empty::empty"
      Then cmd: "move run --function 0x3::empty::empty  --session-key {{$.session-key[-1].authentication_key}}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # transaction
      Then cmd: "transaction get-transactions-by-order --cursor 0 --limit 1 --descending-order false"
      Then cmd: "transaction get-transactions-by-hash --hashes {{$.transaction[-1].data[0].execution_info.tx_hash}}"

      # account balance
      Then cmd: "account balance"
      Then cmd: "account balance --coin-type rooch_framework::gas_coin::GasCoin"
      Then cmd: "rpc request --method rooch_getBalance --params '["{{$.address_mapping.default}}", "0x3::gas_coin::GasCoin"]' --json"
      Then assert: "'{{$.rpc[-1].balance}}' != '0'"

      Then stop the server

    @serial
    Scenario: state
      Given a server for state
      Then cmd: "object --id 0x3" 
      Then cmd: "object --id 0x2::object::Timestamp"
      Then cmd: "state --access-path /object/0x2::object::Timestamp"
      Then assert: "{{$.state[-1][0].value_type}} == '0x2::object::ObjectEntity<0x2::object::Timestamp>'"
      Then cmd: "state --access-path /object/0x3::chain_id::ChainID"
      Then assert: "{{$.state[-1][0].value_type}} == '0x2::object::ObjectEntity<0x3::chain_id::ChainID>'"
      Then assert: "{{$.state[-1][0].decoded_value.value.value.value.id}} == 4"
      Then cmd: "state --access-path /object/0x3::address_mapping::RoochToBitcoinAddressMapping"
      Then assert: "{{$.state[-1][0].value_type}} == '0x2::object::ObjectEntity<0x3::address_mapping::RoochToBitcoinAddressMapping>'"
      Then cmd: "state --access-path /object/0x3::coin::CoinInfo<0x3::gas_coin::GasCoin>"
      Then assert: "{{$.state[-1][0].value_type}} == '0x2::object::ObjectEntity<0x3::coin::CoinInfo<0x3::gas_coin::GasCoin>>'"
      Then stop the server

    @serial
    Scenario: event
    Given a server for event
    # event example and event prc
    Then cmd: "move publish -p ../../examples/event  --named-addresses rooch_examples=default"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    Then cmd: "move run --function default::event_test::emit_event  --args 10u64"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    Then cmd: "move run --function default::event_test::emit_event  --args 11u64"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    Then cmd: "move run --function default::event_test::emit_event  --args 11u64"
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
    Then cmd: "move publish -p ../../examples/event  --named-addresses rooch_examples=default"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    Then cmd: "move run --function default::event_test::emit_event  --args 10u64"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    Then cmd: "move run --function default::event_test::emit_event  --args 11u64"
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
    Then cmd: "rpc request --method rooch_queryObjectStates --params '[{"object_type":"0x3::coin::CoinInfo"}, null, "10", {"descending": true,"showDisplay":false}]' --json"
    Then assert: "{{$.rpc[-1].data[0].tx_order}} == 1"
    Then assert: "{{$.rpc[-1].data[0].object_type}} == 0x3::coin::CoinInfo<0x3::gas_coin::GasCoin>"
    Then assert: "{{$.rpc[-1].has_next_page}} == false"

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
      Then cmd: "move publish -p ../../examples/kv_store  --named-addresses rooch_examples=default"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::kv_store::add_value --args string:key1 --args string:value1"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move view --function default::kv_store::get_value --args string:key1"
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} == value1"
      #the access-path argument do not support named address yet, so, we use `{{$.address_mapping.default}}` template var to repleace it.
      Then cmd: "state --access-path /resource/{{$.address_mapping.default}}/{{$.address_mapping.default}}::kv_store::KVStore"
      Then cmd: "state --access-path /fields/{{$.state[-1][0].decoded_value.value.table.value.handle.value.id}}/key1"
      Then assert: "{{$.state[-1][0].decoded_value}} == value1"


      Then stop the server

    @serial
    Scenario: entry_function example
      Given a server for entry_function

      Then cmd: "move publish -p ../../examples/entry_function_arguments/  --named-addresses rooch_examples=default"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_bool --args bool:true "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_u8 --args u8:3 "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_u8 --args 4u8 "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_address --args address:0x3242 "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_address --args @0x3242 "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_object_id --args object_id:0x3134 "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_string --args string:world "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_vec_u8 --args "vector<u8>:2,3,4" "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_vec_object_id --args "vector<object_id>:0x1324,0x41234,0x1234" "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_mix --args 3u8 --args "vector<object_id>:0x2342,0x3132" "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_object --args "object:default::entry_function::TestStruct" "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_object_mut --args "object:default::entry_function::TestStruct" "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then stop the server

  @serial
  Scenario: publish through MoveAction and module upgrade
      Given a server for publish_through_move_action

      # The counter example
      Then cmd: "move publish -p ../../examples/counter  --named-addresses rooch_examples=default --by-move-action"
      Then assert: "'{{$.move[-1]}}' contains INVALID_MODULE_PUBLISHER"

      Then stop the server

  @serial
  Scenario: publish through Move entry function and module upgrade
      Given a server for publish_through_entry_function

      # The counter example
      Then cmd: "move publish -p ../../examples/counter  --named-addresses rooch_examples=default"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move view --function default::counter::value"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 0"
      Then cmd: "move run --function default::counter::increase "
      Then cmd: "move view --function default::counter::value"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 1"
      Then cmd: "resource --address default --resource default::counter::Counter"
      Then assert: "{{$.resource[-1].decoded_value.value.value}} == 1"

      # The entry_function_arguments example
      Then cmd: "move publish -p ../../examples/entry_function_arguments_old/  --named-addresses rooch_examples=default"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_mix --args 3u8 --args "vector<object_id>:0x2342,0x3132" "
      Then assert: "'{{$.move[-1]}}' contains FUNCTION_RESOLUTION_FAILURE"
      Then cmd: "move publish -p ../../examples/entry_function_arguments/  --named-addresses rooch_examples=default"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_mix --args 3u8 --args "vector<object_id>:0x2342,0x3132" "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # check compatibility
      Then cmd: "move publish -p ../../examples/entry_function_arguments_old/  --named-addresses rooch_examples=default"
      Then assert: "'{{$.move[-1].execution_info.status.type}}' == 'moveabort'"

      Then stop the server

  @serial
  Scenario: coins example
      Given a server for coins
      Then cmd: "account create --json"
      Then cmd: "move publish -p ../../examples/coins  --named-addresses coins=default"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::fixed_supply_coin::faucet --args object:default::fixed_supply_coin::Treasury"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      
      Then cmd: "account list --json"
      Then cmd: "rpc request --method rooch_getBalance --params '["{{$.account[-1].default.bitcoin_address}}", "{{$.account[-1].default.hex_address}}::fixed_supply_coin::FSC"]' --json"
      Then assert: "'{{$.rpc[-1].coin_type}}' == '{{$.address_mapping.default}}::fixed_supply_coin::FSC'"
      Then assert: "'{{$.rpc[-1].balance}}' != '0'"

      Then cmd: "account transfer --coin-type default::fixed_supply_coin::FSC --to {{$.account[-1].account0.bitcoin_address}} --amount 1"
      Then assert: "{{$.account[-1].execution_info.status.type}} == executed"
      Then stop the server

  @serial
  Scenario: Issue a coin through module_template
    Given a server for issue_coin
    Then cmd: "move publish -p ../../examples/module_template/  --named-addresses rooch_examples=default"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    
    #TODO: uncomment this once move_module::binding_module_address is ready
    #Then cmd: "move run --function default::coin_factory::issue_fixed_supply_coin --args string:my_coin  --args string:"My first coin" --args string:MyCoin --args 1010101u256 --args 8u8  "
    #Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

    # check module `my_coin` is published, the name `my_coin` is the first arg in the last `move run` cmd.
    #Then cmd: "move run --function default::my_coin::faucet --args object:default::my_coin::Treasury"
    #Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

    #Then cmd: "rpc request --method rooch_getBalance --params '["{{$.address_mapping.default}}", "{{$.address_mapping.default}}::my_coin::MyCoin"]' --json"
    #Then assert: "'{{$.rpc[-1].coin_type}}' == '{{$.address_mapping.default}}::my_coin::MyCoin'"
    #Then assert: "'{{$.rpc[-1].balance}}' != '0'"
    Then stop the server
  
  @serial
  Scenario: basic_object example
      Given a server for basic_object
      Then cmd: "account create"
      Then cmd: "move publish -p ../../examples/basic_object  --named-addresses basic_object=default"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::third_party_module_for_child_object::create_child --args string:alice"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      
      Then cmd: "event get-events-by-event-handle -t default::child_object::NewChildEvent"
      Then cmd: "state --access-path /object/{{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.state[-1][0].decoded_value.value.value.value.name}} == alice"

      Then cmd: "move run --function default::third_party_module_for_child_object::update_child_name --args object:{{$.event[-1].data[0].decoded_event_data.value.id}} --args string:bob"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      
      Then cmd: "state --access-path /object/{{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.state[-1][0].decoded_value.value.value.value.name}} == bob"

      # because the indexer is async update, so sleep 2 seconds to wait indexer update.
      Then sleep: "2"

      Then cmd: "rpc request --method rooch_queryObjectStates --params '[{"object_type":"{{$.address_mapping.default}}::child_object::Child"}, null, "10", {"descending": true,"showDisplay":false}]' --json"
      Then assert: "{{$.rpc[-1].data[0].object_id}} == {{$.event[-1].data[0].decoded_event_data.value.id}}"

      Then cmd: "move run --function default::third_party_module_for_child_object::update_child_age --args object:{{$.event[-1].data[0].decoded_event_data.value.id}} --args u64:10"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "move view --function default::child_object::get_age --args object:{{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 10" 
       
      Then cmd: "move run --function default::third_party_module_for_child_object::remove_child_via_id --args object_id:{{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then stop the server

  @serial
  Scenario: object display example
      Given a server for object_display
      Then cmd: "account create"
      Then cmd: "move publish -p ../../examples/display  --named-addresses display=default"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "move run --function default::display::create_object --sender default --args 'string:test_object' --args 'address:default' --args 'string:test object description'"
      
      Then cmd: "event get-events-by-event-handle -t default::display::NewObjectEvent"
      Then cmd: "state --access-path /object/{{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.state[-1][0].decoded_value.value.value.type}} == '{{$.address_mapping.default}}::display::ObjectType'"

      Then cmd: "rpc request --method rooch_getStates --params '["/object/{{$.event[-1].data[0].decoded_event_data.value.id}}", {"decode": false, "showDisplay": true}]' --json"
      Then assert: "{{$.rpc[-1][0].display_fields.fields.name}}  == test_object"

      # because the indexer is async update, so sleep 2 seconds to wait indexer update.
      Then sleep: "2"

      Then cmd: "rpc request --method rooch_queryObjectStates --params '[{"object_type":"{{$.address_mapping.default}}::display::ObjectType"}, null, "10", {"descending": false,"showDisplay":true}]' --json"
      Then assert: "{{$.rpc[-1].data[0].object_id}} == {{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.rpc[-1].data[0].display_fields.fields.name}} == test_object"

      Then cmd: "rpc request --method rooch_getObjectStates --params '["{{$.event[-1].data[0].decoded_event_data.value.id}}", {"decode": false, "showDisplay": true}]' --json"
      Then assert: "{{$.rpc[-1][0].display_fields.fields.name}} == test_object"

      
      Then stop the server
    
    @serial
    Scenario: wasm test
      # prepare servers
      Given a server for wasm_test

      # publish wasm execution
      Then cmd: "move publish -p ../../examples/wasm_execution  --named-addresses rooch_examples=default"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # test wasm trap
      Then cmd: "move run --function default::wasm_execution::run_trap"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # test wasm forever
      Then cmd: "move run --function default::wasm_execution::run_infinite_loop"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # test wasm alloc
      Then cmd: "move run --function default::wasm_execution::run_alloc"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # run wasm cpp generator
      #Then cmd: "move run --function default::wasm_execution::run_generator_cpp"
      #Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # run wasm rust generator
      #Then cmd: "move run --function default::wasm_execution::run_generator_rust"
      #Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # release servers
      Then stop the server

    @serial
    Scenario: rooch_bitcoin test
      # prepare servers
      Given a bitcoind server for rooch_bitcoin_test
      Given a server for rooch_bitcoin_test

      Then cmd: "account list --json" 
      
      # mint utxos
      Then cmd bitcoin-cli: "generatetoaddress 101 {{$.account[-1].default.bitcoin_address}}"
      Then sleep: "10" # wait rooch sync and index

      # query utxos
      Then cmd: "rpc request --method rooch_queryObjectStates --params '[{"object_type_with_owner":{"object_type":"0x4::utxo::UTXO","owner":"{{$.account[-1].default.bitcoin_address}}"}},null, null, null]' --json"
      Then assert: "{{$.rpc[-1].data[0].owner}} == {{$.account[-1].default.address}}"

      # release servers
      Then stop the server
      Then stop the bitcoind server 

    @serial
    Scenario: rooch bitseed test
      Then cmd: "init --skip-password"
      Then cmd: "env switch --alias local"

      # prepare servers
      Given a bitcoind server for rooch_bitseed_test
      Given a ord server for rooch_bitseed_test
      Given a server for rooch_bitseed_test

      # create rooch account
      Then cmd: "account create"
      Then cmd: "account list --json"

      # init wallet
      Then cmd ord: "wallet create"
      Then cmd ord: "wallet receive"

      # mint utxos
      Then cmd bitcoin-cli: "generatetoaddress 101 {{$.wallet[-1].address}}"
      Then sleep: "10" # wait ord sync and index
      Then cmd ord: "wallet balance"
      Then assert: "{{$.wallet[-1].total}} == 5000000000"

      # publish bitseed runner
      Then cmd: "move publish -p ../../examples/bitseed_runner  --named-addresses rooch_examples=default"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # generator
      Then cmd bitseed: "generator --fee-rate 1 --name random --generator /app/test-data/generator.wasm"
      Then assert: "'{{$.generator[-1]}}' not_contains error"

      # mine a block
      Then cmd ord: "wallet receive"
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.wallet[-1].address}}"
      Then sleep: "10"

      # Sync bitseed
      Then cmd: "move run --function default::bitseed_runner::run"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Check mint generator validity
      Then cmd: "move view --function 0xa::bitseed::view_validity --args string:{{$.generator[-1].inscriptions[0].Id}} "
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      Then assert: "{{$.move[-1].return_values[0].decoded_value.value.vec[0].value.is_valid}} == true"

      # deploy
      Then cmd bitseed: "deploy --fee-rate 1 --generator {{$.generator[-1].inscriptions[0].Id}} --tick bits --amount 210000000000 --deploy-args {"height":{"type":"range","data":{"min":1,"max":1000}}}"
      Then assert: "'{{$.deploy[-1]}}' not_contains error"

      # mine a block
      Then cmd ord: "wallet receive"
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.wallet[-1].address}}"
      Then sleep: "10"

      # Sync bitseed
      Then cmd: "move run --function default::bitseed_runner::run"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Check mint deploy validity
      Then cmd: "move view --function 0xa::bitseed::view_validity --args string:{{$.deploy[-1].inscriptions[0].Id}} "
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      Then assert: "{{$.move[-1].return_values[0].decoded_value.value.vec[0].value.is_valid}} == true"

      # mint 
      Then cmd bitseed: "mint --fee-rate 1 --deploy-inscription-id {{$.deploy[-1].inscriptions[0].Id}} --user-input hello_bitseed" 
      Then assert: "'{{$.mint[-1]}}' not_contains error"

      # mine a block
      Then cmd ord: "wallet receive"
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.wallet[-1].address}}"
      Then sleep: "10"

      # Sync bitseed
      Then cmd: "move run --function default::bitseed_runner::run"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Check mint bits validity
      Then cmd: "move view --function 0xa::bitseed::view_validity --args string:{{$.deploy[-1].inscriptions[0].Id}} "
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      Then assert: "{{$.move[-1].return_values[0].decoded_value.value.vec[0].value.is_valid}} == true"
      
      # release servers
      Then stop the server
      Then stop the ord server 
      Then stop the bitcoind server 