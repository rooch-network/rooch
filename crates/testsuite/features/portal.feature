Feature: Rooch Portal contract tests

   
    @serial
    Scenario: gas_market
      Given a bitcoind server for gas_market
      Given a server for gas_market

      Then cmd: "account create --json"
      Then cmd: "account create --json"
      Then cmd: "account list --json" 
      
      # mint utxos
      Then cmd bitcoin-cli: "generatetoaddress 101 {{$.account[2].account0.bitcoin_address}}"
      Then sleep: "10"

      Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:1000000000000000 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    
      # publish gas_market via default address
      Then cmd: "move publish -p ../../infra/rooch-portal-v2/contract/gas_market  --named-addresses gas_market=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "event get-events-by-event-handle -t default::trusted_oracle::NewOracleEvent"

      # submit the oracle price, require at least two record
      Then cmd: "move run --function default::trusted_oracle::submit_data --args object:{{$.event[-1].data[0].decoded_event_data.value.oracle_id}} --args string:BTCUSD --args u256:5805106000000 --args u8:8 --args string:1 --args object:{{$.event[-1].data[0].decoded_event_data.value.admin_id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::trusted_oracle::submit_data --args object:{{$.event[-1].data[0].decoded_event_data.value.oracle_id}} --args string:BTCUSD --args u256:5805106000000 --args u8:8 --args string:1 --args object:{{$.event[-1].data[0].decoded_event_data.value.admin_id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "move run --function default::trusted_oracle::submit_data --args object:{{$.event[-1].data[1].decoded_event_data.value.oracle_id}} --args string:BTCUSD --args u256:5805106000000 --args u8:8 --args string:2 --args object:{{$.event[-1].data[1].decoded_event_data.value.admin_id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::trusted_oracle::submit_data --args object:{{$.event[-1].data[1].decoded_event_data.value.oracle_id}} --args string:BTCUSD --args u256:5805106000000 --args u8:8 --args string:2 --args object:{{$.event[-1].data[1].decoded_event_data.value.admin_id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "move run --function default::trusted_oracle::submit_data --args object:{{$.event[-1].data[2].decoded_event_data.value.oracle_id}} --args string:BTCUSD --args u256:5805106000000 --args u8:8 --args string:3 --args object:{{$.event[-1].data[2].decoded_event_data.value.admin_id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::trusted_oracle::submit_data --args object:{{$.event[-1].data[2].decoded_event_data.value.oracle_id}} --args string:BTCUSD --args u256:5805106000000 --args u8:8 --args string:3 --args object:{{$.event[-1].data[2].decoded_event_data.value.admin_id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # ensure the account0 RGas balance is 0
      Then cmd: "account balance -a {{$.account[2].account0.address}} --coin-type 0x3::gas_coin::RGas --json"
      Then assert: "{{$.account[-1].RGAS.balance}} == 0"

      # transfer 0.01 BTC to the gas market address
      Then cmd: "bitcoin transfer -s {{$.account[2].account0.bitcoin_address}} -t {{$.account[2].default.bitcoin_address}} -a 1000000"
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.account[2].account0.bitcoin_address}}"
      Then sleep: "10"

      Then cmd: "object -t default::gas_market::RGasMarket"
      # consume the utxo event
      Then cmd: "move run --function default::gas_market::consume_event --args object:{{$.object[-1].data[0].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # check the RGas balance of account0
      Then cmd: "account balance -a {{$.account[2].account0.address}} --coin-type 0x3::gas_coin::RGas --json"
      Then assert: "{{$.account[-1].RGAS.balance}} != 0"

      # ensure the account1 RGas balance is 0
      Then cmd: "account balance -a {{$.account[2].account1.address}} --coin-type 0x3::gas_coin::RGas --json"
      Then assert: "{{$.account[-1].RGAS.balance}} == 0"

      # transfer 2 utxo to account1
      Then cmd: "bitcoin transfer -s {{$.account[2].account0.bitcoin_address}} -t {{$.account[2].account1.bitcoin_address}} -a 1000000" 
      Then cmd: "bitcoin transfer -s {{$.account[2].account0.bitcoin_address}} -t {{$.account[2].account1.bitcoin_address}} -a 1000000"
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.account[2].account0.bitcoin_address}}"
      Then sleep: "10"

      Then cmd: "object -t default::gas_airdrop::RGasAirdrop"
      Then cmd: "object -o {{$.account[2].account1.bitcoin_address}} -t 0x4::utxo::UTXO"
      
      # the default address help the account1 to claim the airdrop
      Then cmd: "move run --function default::gas_airdrop::claim --args object:{{$.object[-2].data[0].id}} --args address:{{$.account[2].account1.address}} --args vector<object_id>:{{$.object[-1].data[0].id}},{{$.object[-1].data[1].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # check the RGas balance of account0
      Then cmd: "account balance -a {{$.account[2].account1.address}} --coin-type 0x3::gas_coin::RGas --json"
      Then assert: "{{$.account[-1].RGAS.balance}} != 0"

      # try claim again
      Then cmd: "move run --function default::gas_airdrop::claim --args object:{{$.object[-2].data[0].id}} --args address:{{$.account[2].account1.address}} --args vector<object_id>:{{$.object[-1].data[0].id}},{{$.object[-1].data[1].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} != executed"


      
