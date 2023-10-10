Feature: Rooch CLI integration tests
    @serial
    Scenario: Init
      Then cmd: "init"
      Then cmd: "env switch --alias local"


    @serial
    Scenario: account
      Given a server for account

      Then cmd: "object --id {default}"
      Then cmd: "account create"
      Then cmd: "account list"
      Then cmd: "account import --mnemonic-phrase "fiber tube acid imitate frost coffee choose crowd grass topple donkey submit""
      Then cmd: "account update --address 0xebf29d2aed4da3d2e13a32d71266a302fbfd5ceb3ff1f465c006fa207f1789ce --mnemonic-phrase "spike air embody solid upper grow mule slender shrimp suggest pride young""
      Then cmd: "account nullify --address 0xebf29d2aed4da3d2e13a32d71266a302fbfd5ceb3ff1f465c006fa207f1789ce"

      # session key
      Then cmd: "session-key create --sender-account {default} --scope 0x3::empty::empty"
      Then cmd: "move run --function 0x3::empty::empty --sender-account {default} --session-key {{$.session-key[-1].authentication_key}}"

      # transaction
      Then cmd: "transaction get-transactions-by-order --cursor 0 --limit 1"
      Then cmd: "transaction get-transactions-by-hash --hashes {{$.transaction[-1].data[0].execution_info.tx_hash}}"

      # event example
      Then cmd: "move publish -p ../../examples/event --sender-account {default} --named-addresses rooch_examples={default}"
      Then cmd: "move run --function {default}::event_test::emit_event --sender-account {default} --args 10u64"
      Then cmd: "event get-events-by-event-handle --event_handle_type {default}::event_test::WithdrawEvent --cursor 0 --limit 1"

      # account balance
      Then cmd: "move publish -p ../../examples/coins --sender-account {default} --named-addresses coins={default}"
      Then cmd: "move run --function {default}::fixed_supply_coin::faucet --sender-account {default}"
      Then cmd: "account balance"
      Then cmd: "account balance --coin-type {default}::fixed_supply_coin::FSC"

      Then stop the server

    @serial
    Scenario: kv store example
      Given a server for kv_store
      Then cmd: "move publish -p ../../examples/kv_store --sender-account {default} --named-addresses rooch_examples={default}"
      #FIXME how to pass args at here.
      #Then cmd: "move run --function {default}::kv_store::add_value --args 'b\"key1\"' 'b\"value1\"' --sender-account default"
      #Then cmd: "move view --function {default}::kv_store::get_value --args 'b\"key1\"' "
      #Then assert: "{{$.move[-1][0].move_value}} == "value1""
      #Then cmd: "state --access-path /resource/{default}/{default}::kv_store::KVStore
      #Then cmd: "state --access-path /table/{{$.move[-1][0].move_value.value.table.value.handle}}/key1"
      #Then assert: "{{$.move[-1][0].move_value}} == "value1""


      Then stop the server

    @serial
    Scenario: entry function example
      Given a server for entry_function

      Then cmd: "move publish -p ../../examples/entry_function_arguments/ --sender-account {default} --named-addresses rooch_examples={default}"
      Then cmd: "move run --function {default}::entry_function::emit_bool --args bool:true --sender-account {default}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function {default}::entry_function::emit_u8 --args u8:3 --sender-account {default}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function {default}::entry_function::emit_u8 --args 4u8 --sender-account {default}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function {default}::entry_function::emit_address --args address:0x3242 --sender-account {default}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function {default}::entry_function::emit_address --args @0x3242 --sender-account {default}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function {default}::entry_function::emit_object_id --args object_id:0x3134 --sender-account {default}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function {default}::entry_function::emit_string --args string:world --sender-account {default}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function {default}::entry_function::emit_vec_u8 --args "vector<u8>:2,3,4" --sender-account {default}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function {default}::entry_function::emit_vec_object_id --args "vector<address>:0x1324,0x41234,0x1234" --sender-account {default}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function {default}::entry_function::emit_mix --args 3u8 "vector<object_id>:0x2342,0x3132" --sender-account {default}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then stop the server

  @serial
  Scenario: publish through MoveAction and module upgrade
      Given a server for publish_through_move_action

      # The counter example
      Then cmd: "move publish -p ../../examples/counter --sender-account {default} --named-addresses rooch_examples={default} --by-move-action"
      Then cmd: "move view --function {default}::counter::value"
      Then assert: "{{$.move[-1].return_values[0].move_value}} == 0"
      Then cmd: "move run --function {default}::counter::increase --sender-account {default}"
      Then cmd: "move view --function {default}::counter::value"
      Then assert: "{{$.move[-1].return_values[0].move_value}} == 1"
      Then cmd: "resource --address {default} --resource {default}::counter::Counter"
      Then assert: "{{$.resource[-1].move_value.value.value}} == 1"

      # The entry_function_arguments example
      Then cmd: "move publish -p ../../examples/entry_function_arguments_old/ --sender-account {default} --named-addresses rooch_examples={default} --by-move-action"
      Then cmd: "move run --function {default}::entry_function::emit_mix --args 3u8 "vector<object_id>:0x2342,0x3132" --sender-account {default}"
      Then assert: ""{{$.move[-1]}}" contains FUNCTION_RESOLUTION_FAILURE"
      Then cmd: "move publish -p ../../examples/entry_function_arguments/ --sender-account {default} --named-addresses rooch_examples={default} --by-move-action"
      Then cmd: "move run --function {default}::entry_function::emit_mix --args 3u8 "vector<object_id>:0x2342,0x3132" --sender-account {default}"
      Then assert: "{{$.move[-1].output.status.type}} == executed"
      # check compatibility
      Then cmd: "move publish -p ../../examples/entry_function_arguments_old/ --sender-account {default} --named-addresses rooch_examples={default} --by-move-action"
      Then assert: ""{{$.move[-1]}}" contains MiscellaneousError"

      Then stop the server

  @serial
  Scenario: publish through Move entry function and module upgrade
      Given a server for publish_through_entry_function

      # The counter example
      Then cmd: "move publish -p ../../examples/counter --sender-account {default} --named-addresses rooch_examples={default}"
      Then cmd: "move view --function {default}::counter::value"
      Then assert: "{{$.move[-1].return_values[0].move_value}} == 0"
      Then cmd: "move run --function {default}::counter::increase --sender-account {default}"
      Then cmd: "move view --function {default}::counter::value"
      Then assert: "{{$.move[-1].return_values[0].move_value}} == 1"
      Then cmd: "resource --address {default} --resource {default}::counter::Counter"
      Then assert: "{{$.resource[-1].move_value.value.value}} == 1"

      # The entry_function_arguments example
      Then cmd: "move publish -p ../../examples/entry_function_arguments_old/ --sender-account {default} --named-addresses rooch_examples={default}"
      Then cmd: "move run --function {default}::entry_function::emit_mix --args 3u8 "vector<object_id>:0x2342,0x3132" --sender-account {default}"
      Then assert: ""{{$.move[-1]}}" contains FUNCTION_RESOLUTION_FAILURE"
      Then cmd: "move publish -p ../../examples/entry_function_arguments/ --sender-account {default} --named-addresses rooch_examples={default}"
      Then cmd: "move run --function {default}::entry_function::emit_mix --args 3u8 "vector<object_id>:0x2342,0x3132" --sender-account {default}"
      Then assert: "{{$.move[-1].output.status.type}} == executed"

      # check compatibility
      Then cmd: "move publish -p ../../examples/entry_function_arguments_old/ --sender-account {default} --named-addresses rooch_examples={default}"
      Then assert: ""{{$.move[-1]}}" contains MiscellaneousError"

      Then stop the server

 @serial
    Scenario: coins example
      Given a server for coins
      Then cmd: "account create"
      Then cmd: "move publish -p ../../examples/coins --sender-account {default} --named-addresses coins={default}"
      Then cmd: "move run --function {default}::fixed_supply_coin::faucet --sender-account {default}"
      #TODO change the argument `0x3` address to a user account
      Then cmd: "move run --function 0x3::coin::transfer_entry --type-args {default}::fixed_supply_coin::FSC --args address:0x3  --args 1u256 --sender-account {default}"

      Then stop the server

  @serial
    Scenario: rpc test
      Given a server for rpc
      Then cmd: "rpc request --method eth_getBalance --params \"0x1111111111111111111111111111111111111111\""
      Then assert: "{{$.result}}" equals "0x56bc75e2d63100000"

      Then stop the server
