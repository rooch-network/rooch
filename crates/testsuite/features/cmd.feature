Feature: Rooch CLI integration tests
    Scenario: Init
      Then cmd: "init"

    Scenario: account
      Given the server

      Then cmd: "object --id {default}"
      Then cmd: "account create"
      Then cmd: "account list"
      Then cmd: "account import "fiber tube acid imitate frost coffee choose crowd grass topple donkey submit""
    
      # TODO split Scenario for every example
      # counter example
      Then cmd: "move publish -p ../../examples/counter --sender-account {default} --named-addresses rooch_examples={default}"
      Then cmd: "move run --function {default}::counter::init_entry --sender-account {default}"
      Then cmd: "move view --function {default}::counter::value"
      Then assert: "{{$.move[-1][0].move_value}} == 0"
      Then cmd: "move run --function {default}::counter::increase --sender-account {default}"
      Then cmd: "move view --function {default}::counter::value"
      Then assert: "{{$.move[-1][0].move_value}} == 1"
      Then cmd: "resource --address {default} --resource {default}::counter::Counter"

      Then cmd: "transaction get-by-hash --hash {{$.account[0].execution_info.tx_hash}}"
      Then cmd: "transaction get-by-index --start 0 --limit 10"

      # event example
      Then cmd: "move publish -p ../../examples/event_example --sender-account {default} --named-addresses rooch_examples={default}"
      Then cmd: "move run --function {default}::event_test::emit_event --sender-account {default} --args 10u64"
      Then cmd: "event get-events-by-event-handle --event_handle_type {default}::event_test::WithdrawEvent --cursor 0 --limit 1"

      # kv store example
      Then cmd: "move publish -p ../../examples/kv_store --sender-account {default} --named-addresses rooch_examples={default}"
      #FIXME how to pass args at here.
      #Then cmd: "move run --function {default}::kv_store::add_value --args 'b\"key1\"' 'b\"value1\"' --sender-account default"
      #Then cmd: "move view --function {default}::kv_store::get_value --args 'b\"key1\"' "
      #Then assert: "{{$.move[-1][0].move_value}} == "value1""
      #Then cmd: "state --access-path /resource/{default}/{default}::kv_store::KVStore
      #Then cmd: "state --access-path /table/{{$.move[-1][0].move_value.value.table.value.handle}}/key1"
      #Then assert: "{{$.move[-1][0].move_value}} == "value1""
