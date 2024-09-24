Feature: Rooch Portal contract tests

   
    @serial
    Scenario: gas_market
      Given a bitcoind server for gas_market
      Given a server for gas_market

      Then cmd: "account list --json" 
      
      # mint utxos
      Then cmd bitcoin-cli: "generatetoaddress 101 {{$.account[-1].default.bitcoin_address}}"
      Then sleep: "10"
    
      # publish gas_market
      Then cmd: "move publish -p ../../infra/rooch-portal-v2/contract  --named-addresses gas_market=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      
