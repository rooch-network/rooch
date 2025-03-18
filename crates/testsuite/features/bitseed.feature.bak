Feature: Rooch CLI bitseed tests

    @serial
    Scenario: rooch bitseed test
      Then cmd: "init --skip-password"
      Then cmd: "env switch --alias local"

      # prepare servers
      Given a bitcoind server for rooch_bitseed_test
      Given a server for rooch_bitseed_test

      # create rooch account
      Then cmd: "account list --json"

      # mint utxos
      Then cmd bitcoin-cli: "generatetoaddress 101 {{$.account[-1].default.bitcoin_address}}"
      Then sleep: "10" # wait rooch to sync  

      # generator
      Then cmd: "bitseed generator --fee-rate 5000 --name random --generator ../../generator/cpp/generator.wasm"
      Then assert: "'{{$.bitseed[-1]}}' not_contains error"

      # mine a block
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.account[-1].default.bitcoin_address}}"
      Then sleep: "10"

      # process bitseed inscription event
      Then cmd: "move run --function 0xa::inscribe_factory::process_bitseed_event --args u64:10 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Check mint generator validity
      Then cmd: "move view --function 0x4::ord::view_validity --args string:{{$.bitseed[-1].inscriptions[0].Id}} "
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      # For `.vec.value[0][1]`, the first index `0` means the first element of vec;
      # the second index `1` means the second field of `0x4::ord::MetaprotocolValidity`, that is `is_valid`.
      Then assert: "{{$.move[-1].return_values[0].decoded_value.value.vec.value[0][1]}} == true"

      # deploy
      Then cmd: "bitseed deploy --fee-rate 6000 --generator {{$.bitseed[-1].inscriptions[0].Id}} --tick bits --amount 210000000000 --deploy-args '{"height":{"type":"range","data":{"min":1,"max":1000}}}'"
      Then assert: "'{{$.bitseed[-1]}}' not_contains error"

      # mine a block
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.account[-1].default.bitcoin_address}}"
      Then sleep: "10"

      # Sync bitseed
      Then cmd: "move run --function 0xa::inscribe_factory::process_bitseed_event --args u64:10 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Check deploy validity
      Then cmd: "move view --function 0x4::ord::view_validity --args string:{{$.bitseed[-1].inscriptions[0].Id}} "
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      # For `.vec.value[0][1]`, the first index `0` means the first element of vec;
      # the second index `1` means the second field of `0x4::ord::MetaprotocolValidity`, that is `is_valid`.
      Then assert: "{{$.move[-1].return_values[0].decoded_value.value.vec.value[0][1]}} == true"

      # mint 1
      Then cmd: "bitseed mint --fee-rate 6000 --deploy-inscription-id {{$.bitseed[-1].inscriptions[0].Id}} --user-input test" 
      Then assert: "'{{$.bitseed[-1]}}' not_contains error"

      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.account[-1].default.bitcoin_address}}"
      Then sleep: "5"

      # mint 2
      Then cmd: "bitseed mint --fee-rate 7000 --deploy-inscription-id {{$.bitseed[-2].inscriptions[0].Id}} --user-input test2" 
      Then assert: "'{{$.bitseed[-1]}}' not_contains error"

      # mine a block
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.account[-1].default.bitcoin_address}}"
      Then sleep: "10"

      # Sync bitseed
      Then cmd: "move run --function 0xa::inscribe_factory::process_bitseed_event --args u64:10 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Check mint bits validity
      Then cmd: "move view --function 0x4::ord::view_validity --args string:{{$.bitseed[-1].inscriptions[0].Id}} "
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      # For `.vec.value[0][1]`, the first index `0` means the first element of vec;
      # the second index `1` means the second field of `0x4::ord::MetaprotocolValidity`, that is `is_valid`.
      Then assert: "{{$.move[-1].return_values[0].decoded_value.value.vec.value[0][1]}} == true"

      Then cmd: "move view --function 0x4::ord::view_validity --args string:{{$.bitseed[-2].inscriptions[0].Id}} "
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      # For `.vec.value[0][1]`, the first index `0` means the first element of vec;
      # the second index `1` means the second field of `0x4::ord::MetaprotocolValidity`, that is `is_valid`.
      Then assert: "{{$.move[-1].return_values[0].decoded_value.value.vec.value[0][1]}} == true"

      Then cmd: "bitseed merge --fee-rate 6000 --sft-inscription-ids {{$.bitseed[-1].inscriptions[0].Id}} --sft-inscription-ids {{$.bitseed[-2].inscriptions[0].Id}}" 
      Then assert: "'{{$.bitseed[-1]}}' not_contains error" 

      # mine a block
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.account[-1].default.bitcoin_address}}"
      Then sleep: "10"

      Then cmd: "move run --function 0xa::inscribe_factory::process_bitseed_event --args u64:10 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "move view --function 0x4::ord::view_validity --args string:{{$.bitseed[-1].inscriptions[0].Id}} "
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      # For `.vec.value[0][1]`, the first index `0` means the first element of vec;
      # the second index `1` means the second field of `0x4::ord::MetaprotocolValidity`, that is `is_valid`.
      Then assert: "{{$.move[-1].return_values[0].decoded_value.value.vec.value[0][1]}} == true"

      # release servers
      Then stop the server
      Then stop the bitcoind server

    @serial
    Scenario: rooch bitseed_on_rooch test
      Then cmd: "init --skip-password"
      Then cmd: "env switch --alias local"

      # prepare servers
      Given a bitcoind server for bitseed_on_rooch
      Given a server for bitseed_on_rooch

      Then cmd: "account list --json"

      # mint utxos
      Then cmd bitcoin-cli: "generatetoaddress 101 {{$.account[-1].default.bitcoin_address}}"
      Then sleep: "10" # wait ord sync and index 

      # Sync bitseed
      Then cmd: "move run --function 0xa::inscribe_factory::process_bitseed_event --args u64:10 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # deploy
      Then cmd: "bitseed deploy --fee-rate 5000 --factory 0x000000000000000000000000000000000000000000000000000000000000000a::mint_get_factory::MintGetFactory --tick test --amount 210000000000"
      Then assert: "'{{$.bitseed[-1]}}' not_contains error"

      # mine a block
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.account[-1].default.bitcoin_address}}"
      Then sleep: "10"

      # Sync bitseed
      Then cmd: "move run --function 0xa::inscribe_factory::process_bitseed_event --args u64:10 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Check deploy validity
      Then cmd: "move view --function 0x4::ord::view_validity --args string:{{$.bitseed[-1].inscriptions[0].Id}} "
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      # For `.vec.value[0][1]`, the first index `0` means the first element of vec;
      # the second index `1` means the second field of `0x4::ord::MetaprotocolValidity`, that is `is_valid`.
      Then assert: "{{$.move[-1].return_values[0].decoded_value.value.vec.value[0][1]}} == true"

      # mint on rooch
      Then cmd: "move run --function 0xa::mint_get_factory::mint --args string:bitseed --args string:test --json" 
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "object -t 0xa::bitseed::Bitseed -o {{$.account[-1].default.address}}"
      Then assert: "{{$.object[-1].data[0].owner}} == {{$.account[-1].default.address}}"

      # release servers
      Then stop the server
      Then stop the bitcoind server

