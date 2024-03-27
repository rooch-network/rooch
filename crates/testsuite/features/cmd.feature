Feature: Rooch CLI integration tests
    @serial
    Scenario: Init
      Then cmd: "init --skip-password"
      Then cmd: "env switch --alias local"

    @serial
    Scenario: ethereum rpc test
      Given a server for ethereum_rpc_test
      Then cmd: "rpc request --method eth_getBalance --params 0x1111111111111111111111111111111111111111"
      Then assert: "{{$.rpc[-1]}} == 0x56bc75e2d63100000"
      Then cmd: "rpc request --method eth_feeHistory --params '["0x5", "0x6524cad7", [10,20,30]]'"
      Then assert: "'{{$.rpc[-1]}}' contains baseFeePerGas"
      Then cmd: "rpc request --method net_version"
      Then assert: "'{{$.rpc[-1]}}' == '20230104'"
      Then stop the server

    @serial
    Scenario: rooch rpc test
      Given a server for rooch_rpc_test
      Then cmd: "rpc request --method rooch_getStates --params '["/resource/0x3/0x3::account_coin_store::CoinStores",{"decode":true}]'"
      Then assert: "{{$.rpc[-1][0].value_type}} == '0x3::account_coin_store::CoinStores'"
      Then cmd: "rpc request --method rooch_getStates --params '["/object/0x3",{"decode":true}]'"
      Then assert: "{{$.rpc[-1][0].value_type}} == '0x2::object::ObjectEntity<0x2::account::Account>'"
      Then cmd: "rpc request --method rooch_listStates --params '["/resource/0x3", null, null, {"decode":true}]"
      Then assert: "'{{$.rpc[-1]}}' contains '0x3::account_coin_store::CoinStores'"
      Then cmd: "rpc request --method rooch_getStates --params '["/object/0x711ab0301fd517b135b88f57e84f254c94758998a602596be8ae7ba56a0d14b3",{"decode":true}]'"
      Then assert: "{{$.rpc[-1][0].value_type}} == '0x2::object::ObjectEntity<0x3::timestamp::Timestamp>'"
      Then assert: "{{$.rpc[-1][0].decoded_value.value.value.value.milliseconds}} == 0"
      Then cmd: "rpc request --method rooch_getStates --params '["/object/0x3::timestamp::Timestamp",{"decode":true}]'"
      Then assert: "{{$.rpc[-1][0].value_type}} == '0x2::object::ObjectEntity<0x3::timestamp::Timestamp>'"
      Then stop the server 
    
    @serial
    Scenario: account
      Given a server for account

      Then cmd: "account create"
      Then cmd: "account list"
      #Then cmd: "account nullify --address 0xebf29d2aed4da3d2e13a32d71266a302fbfd5ceb3ff1f465c006fa207f1789ce"

      Then cmd: "rpc request --method rooch_getBalance --params '["{{$.address_mapping.default}}", "0x3::gas_coin::GasCoin"]'"
      Then assert: "'{{$.rpc[-1].coin_type}}' == '0x3::gas_coin::GasCoin'"
      Then assert: "'{{$.rpc[-1].balance}}' == '0'"

      # Get gas
      Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      
      # session key
      Then cmd: "session-key create  --scope 0x3::empty::empty"
      Then cmd: "move run --function 0x3::empty::empty  --session-key {{$.session-key[-1].authentication_key}}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # transaction
      Then cmd: "transaction get-transactions-by-order --cursor 0 --limit 1 --descending-order false"
      Then cmd: "transaction get-transactions-by-hash --hashes {{$.transaction[-1].data[0].execution_info.tx_hash}}"

      # account balance
      Then cmd: "account balance"
      Then cmd: "account balance --coin-type rooch_framework::gas_coin::GasCoin"
      Then cmd: "rpc request --method rooch_getBalance --params '["{{$.address_mapping.default}}", "0x3::gas_coin::GasCoin"]'"
      Then assert: "'{{$.rpc[-1].balance}}' != '0'"

      Then stop the server

    @serial
    Scenario: state
      Given a server for state
      Then cmd: "object --id 0x3" 
      Then cmd: "object --id 0x3::timestamp::Timestamp"
      Then cmd: "state --access-path /object/0x3::timestamp::Timestamp"
      Then assert: "{{$.state[-1][0].value_type}} == '0x2::object::ObjectEntity<0x3::timestamp::Timestamp>'"
      Then cmd: "state --access-path /object/0x3::chain_id::ChainID"
      Then assert: "{{$.state[-1][0].value_type}} == '0x2::object::ObjectEntity<0x3::chain_id::ChainID>'"
      Then assert: "{{$.state[-1][0].decoded_value.value.value.value.id}} == 20230104"
      Then cmd: "state --access-path /object/0x3::address_mapping::AddressMapping"
      Then assert: "{{$.state[-1][0].value_type}} == '0x2::object::ObjectEntity<0x3::address_mapping::AddressMapping>'"
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
    
    Then cmd: "rpc request --method rooch_queryTransactions --params '[{"tx_order_range":{"from_order":0,"to_order":2}}, null, "1", true]'"
    Then assert: "{{$.rpc[-1].data[0].sequence_info.tx_order}} == 1"
    Then assert: "{{$.rpc[-1].next_cursor}} == 1"
    Then assert: "{{$.rpc[-1].has_next_page}} == true"
    Then cmd: "rpc request --method rooch_queryTransactions --params '[{"tx_order_range":{"from_order":0,"to_order":2}}, "1", "1", true]'"
    Then assert: "{{$.rpc[-1].data[0].sequence_info.tx_order}} == 0"
    Then assert: "{{$.rpc[-1].next_cursor}} == 0"
    Then assert: "{{$.rpc[-1].has_next_page}} == false"
    Then cmd: "rpc request --method rooch_queryEvents --params '[{"tx_order_range":{"from_order":0, "to_order":2}}, null, "10", true]'"
    Then assert: "{{$.rpc[-1].data[0].indexer_event_id.tx_order}} == 1"
    Then assert: "{{$.rpc[-1].next_cursor.tx_order}} == 0"
    Then assert: "{{$.rpc[-1].has_next_page}} == false"

    # Sync states
    Then cmd: "rpc request --method rooch_queryGlobalStates --params '[{"object_type":"0x3::coin::CoinInfo"}, null, "10", true]'"
    Then assert: "{{$.rpc[-1].data[0].tx_order}} == 0"
    Then assert: "{{$.rpc[-1].data[0].object_type}} == 0x3::coin::CoinInfo"
    Then assert: "{{$.rpc[-1].has_next_page}} == false"

    Then cmd: "rpc request --method rooch_queryTableStates --params '[{"table_handle":"0x0"}, null, "10", true]'"
    Then assert: "{{$.rpc[-1].has_next_page}} == false"

#    Then cmd: "rpc request --method rooch_syncStates --params '[null, null, "2", false]'"
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
      Then cmd: "state --access-path /resource/{{$.address_mapping.default}}/{{$.address_mapping.default}}::kv_store::KVStore
      Then cmd: "state --access-path /table/{{$.state[-1][0].decoded_value.value.table.value.handle}}/key1"
      Then assert: "{{$.state[-1][0].decoded_value}} == "value1""


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
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move view --function default::counter::value"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 0"
      Then cmd: "move run --function default::counter::increase "
      Then cmd: "move view --function default::counter::value"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 1"
      Then cmd: "resource --address default --resource default::counter::Counter"
      Then assert: "{{$.resource[-1].decoded_value.value.value}} == 1"

      # The entry_function_arguments example
      Then cmd: "move publish -p ../../examples/entry_function_arguments_old/  --named-addresses rooch_examples=default --by-move-action"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::entry_function::emit_mix --args 3u8 --args "vector<object_id>:0x2342,0x3132" "
      Then assert: "'{{$.move[-1]}}' contains FUNCTION_RESOLUTION_FAILURE"
      Then cmd: "move publish -p ../../examples/entry_function_arguments/  --named-addresses rooch_examples=default --by-move-action"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      #Then cmd: "move run --function default::entry_function::emit_mix --args 3u8 --args "vector<object_id>:0x2342,0x3132" "
      #Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      # check compatibility
      Then cmd: "move publish -p ../../examples/entry_function_arguments_old/  --named-addresses rooch_examples=default --by-move-action"
      Then assert: "'{{$.move[-1].execution_info.status.type}}' == miscellaneouserror"

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
      Then cmd: "account create"
      Then cmd: "move publish -p ../../examples/coins  --named-addresses coins=default"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::fixed_supply_coin::faucet --args object:default::fixed_supply_coin::Treasury"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      
      Then cmd: "rpc request --method rooch_getBalance --params '["{{$.address_mapping.default}}", "{{$.address_mapping.default}}::fixed_supply_coin::FSC"]'"
      Then assert: "'{{$.rpc[-1].coin_type}}' == '{{$.address_mapping.default}}::fixed_supply_coin::FSC'"
      Then assert: "'{{$.rpc[-1].balance}}' != '0'"

      #TODO change the argument `0x3` address to a user account
      Then cmd: "move run --function rooch_framework::transfer::transfer_coin --type-args default::fixed_supply_coin::FSC --args address:0x3  --args 1u256 "
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then stop the server

  @serial
  Scenario: Issue a coin through module_template
    Given a server for issue_coin
    Then cmd: "move publish -p ../../examples/module_template/  --named-addresses rooch_examples=default"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    Then cmd: "move run --function default::coin_factory::issue_fixed_supply_coin --args string:my_coin  --args string:"My first coin" --args string:MyCoin --args 1010101u256 --args 8u8  "
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

    # check module `my_coin` is published, the name `my_coin` is the first arg in the last `move run` cmd.
    Then cmd: "move run --function default::my_coin::faucet --args object:default::my_coin::Treasury"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

    Then cmd: "rpc request --method rooch_getBalance --params '["{{$.address_mapping.default}}", "{{$.address_mapping.default}}::my_coin::MyCoin"]'"
    Then assert: "'{{$.rpc[-1].coin_type}}' == '{{$.address_mapping.default}}::my_coin::MyCoin'"
    Then assert: "'{{$.rpc[-1].balance}}' != '0'"
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

      Then cmd: "rpc request --method rooch_queryGlobalStates --params '[{"object_type":"{{$.address_mapping.default}}::child_object::Child"}, null, "10", true]'"
      Then assert: "{{$.rpc[-1].data[0].object_id}} == {{$.event[-1].data[0].decoded_event_data.value.id}}"

      Then cmd: "move run --function default::third_party_module_for_child_object::update_child_age --args object:{{$.event[-1].data[0].decoded_event_data.value.id}} --args u64:10"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "move view --function default::child_object::get_age --args object:{{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 10" 
       
      Then cmd: "move run --function default::third_party_module_for_child_object::remove_child_via_id --args object_id:{{$.event[-1].data[0].decoded_event_data.value.id}}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then stop the server
  