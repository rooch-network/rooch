Feature: Rooch CLI bitseed tests

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
      Then cmd bitcoin-cli: "generatetoaddress 101 {{$.wallet[-1].addresses[0]}}"
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
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.wallet[-1].addresses[0]}}"
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
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.wallet[-1].addresses[0]}}"
      Then sleep: "10"

      # Sync bitseed
      Then cmd: "move run --function default::bitseed_runner::run"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Check deploy validity
      Then cmd: "move view --function 0xa::bitseed::view_validity --args string:{{$.deploy[-1].inscriptions[0].Id}} "
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      Then assert: "{{$.move[-1].return_values[0].decoded_value.value.vec[0].value.is_valid}} == true"

      # mint 
      Then cmd bitseed: "mint --fee-rate 1 --deploy-inscription-id {{$.deploy[-1].inscriptions[0].Id}} --user-input hello_bitseed" 
      Then assert: "'{{$.mint[-1]}}' not_contains error"

      # mine a block
      Then cmd ord: "wallet receive"
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.wallet[-1].addresses[0]}}"
      Then sleep: "10"

      # Sync bitseed
      Then cmd: "move run --function default::bitseed_runner::run"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Check mint bits validity
      Then cmd: "move view --function 0xa::bitseed::view_validity --args string:{{$.mint[-1].inscriptions[0].Id}} "
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      Then assert: "{{$.move[-1].return_values[0].decoded_value.value.vec[0].value.is_valid}} == true"
      
      # release servers
      Then stop the server
      Then stop the ord server 
      Then stop the bitcoind server