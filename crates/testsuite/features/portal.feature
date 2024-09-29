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

      Then cmd: "object -t default::gas_market::RGasMarket"
      Then cmd: "object -t default::gas_faucet::RGasFaucet"
      Then cmd: "object -t default::admin::AdminCap"

      # submit the oracle price, require at least two record
      Then cmd: "move run --function rooch_framework::oracle::create_entry --args string:oracle1 --args string:oracle1_url --args string:oracle1_desc --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "event get-events-by-event-handle -t rooch_framework::oracle::NewOracleEvent --limit 1 -d true"
      Then cmd: "move run --function rooch_framework::oracle::submit_decimal_data --args object:{{$.event[-1].data[0].decoded_event_data.value.oracle_id}} --args string:BTCUSD --args u256:5805106000000 --args u8:8 --args string:1 --args object:{{$.event[-1].data[0].decoded_event_data.value.admin_id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function rooch_framework::oracle::submit_decimal_data --args object:{{$.event[-1].data[0].decoded_event_data.value.oracle_id}} --args string:BTCUSD --args u256:5805106000000 --args u8:8 --args string:1 --args object:{{$.event[-1].data[0].decoded_event_data.value.admin_id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::trusted_oracle::add_trusted_oracle --args object:{{$.event[-1].data[0].decoded_event_data.value.oracle_id}} --args object:{{$.object[2].data[0].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "move run --function rooch_framework::oracle::create_entry --args string:oracle2 --args string:oracle2_url --args string:oracle2_desc --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "event get-events-by-event-handle -t rooch_framework::oracle::NewOracleEvent --limit 1 -d true"
      Then cmd: "move run --function rooch_framework::oracle::submit_decimal_data --args object:{{$.event[-1].data[0].decoded_event_data.value.oracle_id}} --args string:BTCUSD --args u256:5805106000000 --args u8:8 --args string:2 --args object:{{$.event[-1].data[0].decoded_event_data.value.admin_id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function rooch_framework::oracle::submit_decimal_data --args object:{{$.event[-1].data[0].decoded_event_data.value.oracle_id}} --args string:BTCUSD --args u256:5805106000000 --args u8:8 --args string:2 --args object:{{$.event[-1].data[0].decoded_event_data.value.admin_id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::trusted_oracle::add_trusted_oracle --args object:{{$.event[-1].data[0].decoded_event_data.value.oracle_id}} --args object:{{$.object[2].data[0].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "move run --function rooch_framework::oracle::create_entry --args string:oracle3 --args string:oracle3_url --args string:oracle3_desc --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "event get-events-by-event-handle -t rooch_framework::oracle::NewOracleEvent --limit 1 -d true"
      Then cmd: "move run --function rooch_framework::oracle::submit_decimal_data --args object:{{$.event[-1].data[0].decoded_event_data.value.oracle_id}} --args string:BTCUSD --args u256:5805106000000 --args u8:8 --args string:3 --args object:{{$.event[-1].data[0].decoded_event_data.value.admin_id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function rooch_framework::oracle::submit_decimal_data --args object:{{$.event[-1].data[0].decoded_event_data.value.oracle_id}} --args string:BTCUSD --args u256:5805106000000 --args u8:8 --args string:3 --args object:{{$.event[-1].data[0].decoded_event_data.value.admin_id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move run --function default::trusted_oracle::add_trusted_oracle --args object:{{$.event[-1].data[0].decoded_event_data.value.oracle_id}} --args object:{{$.object[2].data[0].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # ensure the account0 RGas balance is 0
      Then cmd: "account balance -a {{$.account[2].account0.address}} --coin-type 0x3::gas_coin::RGas --json"
      Then assert: "{{$.account[-1].RGAS.balance}} == 0"

      # transfer 0.01 BTC to the gas market address
      Then cmd: "bitcoin transfer -s {{$.account[2].account0.bitcoin_address}} -t {{$.account[2].default.bitcoin_address}} -a 1000000"
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.account[2].account0.bitcoin_address}}"

      # schedule a task to consume the utxo event
      Then cmd: "task schedule --stop-on-checker-error --stop-on-runner-error --stop-after-executed-times 1 --checker-function default::gas_market::exists_new_events --checker-args object:{{$.object[0].data[0].id}} --checker-interval 1 --runner-function default::gas_market::consume_event --runner-args object:{{$.object[0].data[0].id}}"
      Then assert: "'{{$.task[-1]}}' not_contains error"

      # check the RGas balance of account0
      Then cmd: "account balance -a {{$.account[2].account0.address}} --coin-type 0x3::gas_coin::RGas --json"
      Then assert: "{{$.account[-1].RGAS.balance}} != 0"

      # start test gas faucet

      Then cmd: "move run --function default::gas_faucet::set_allow_repeat --args object:{{$.object[1].data[0].id}} --args bool:false --args object:{{$.object[2].data[0].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed" 

      Then cmd: "move run --function default::gas_faucet::set_require_utxo --args object:{{$.object[1].data[0].id}} --args bool:true --args object:{{$.object[2].data[0].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed" 

      # ensure the account1 RGas balance is 0
      Then cmd: "account balance -a {{$.account[2].account1.address}} --coin-type 0x3::gas_coin::RGas --json"
      Then assert: "{{$.account[-1].RGAS.balance}} == 0"

      # transfer 2 utxo to account1
      Then cmd: "bitcoin transfer -s {{$.account[2].account0.bitcoin_address}} -t {{$.account[2].account1.bitcoin_address}} -a 1000000" 
      Then cmd: "bitcoin transfer -s {{$.account[2].account0.bitcoin_address}} -t {{$.account[2].account1.bitcoin_address}} -a 1000000"
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.account[2].account0.bitcoin_address}}"
      Then sleep: "10"

      Then cmd: "object -o {{$.account[2].account1.bitcoin_address}} -t 0x4::utxo::UTXO"
      
      # the default address help the account1 to claim the airdrop
      Then cmd: "move run --function default::gas_faucet::claim --args object:{{$.object[1].data[0].id}} --args address:{{$.account[2].account1.address}} --args vector<object_id>:{{$.object[-1].data[0].id}},{{$.object[-1].data[1].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # check the RGas balance of account0
      Then cmd: "account balance -a {{$.account[2].account1.address}} --coin-type 0x3::gas_coin::RGas --json"
      Then assert: "{{$.account[-1].RGAS.balance}} != 0"

      # try claim again
      Then cmd: "move run --function default::gas_faucet::claim --args object:{{$.object[1].data[0].id}} --args address:{{$.account[2].account1.address}} --args vector<object_id>:{{$.object[-1].data[0].id}},{{$.object[-1].data[1].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} != executed"


      
