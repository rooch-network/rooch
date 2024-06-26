Feature: Rooch Bitcoin Reorg tests

    @serial
    Scenario: rooch bitcoin reorg test
      # prepare servers
      Given a bitcoind server for bitcoin_reorg_test
      Given a server for bitcoin_reorg_test
      
      # Wait genesis block synced
      Then sleep: "10" # wait rooch sync and index

      # Update the reorg pending block count
      Then cmd: "move run --function 0x4::pending_block::update_reorg_block_count_for_local --args u64:1"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "account list --json" 
      
      # mint utxos
      Then cmd bitcoin-cli: "generatetoaddress 2 {{$.account[-1].default.bitcoin_address}}"
      Then sleep: "10" # wait rooch sync and index
      
      Then cmd bitcoin-cli: "getbestblockhash"
      # invalid the latest
      Then cmd bitcoin-cli: "invalidateblock {{$.getbestblockhash[-1]}}"
      # generate a new block
      Then cmd bitcoin-cli: "generatetoaddress 2 {{$.account[-1].default.bitcoin_address}}"
       
      Then sleep: "10" # wait rooch sync and index

      Then cmd: "event get-events-by-event-handle -t 0x4::pending_block::ReorgEvent --descending-order true"
      Then assert: "{{$.event[-1].data[0].decoded_event_data.value.success}} == true"
      
      
      # release servers
      Then stop the server
      Then stop the bitcoind server 