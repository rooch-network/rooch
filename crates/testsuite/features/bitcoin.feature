Feature: Rooch Bitcoin tests

    @serial
    Scenario: rooch bitcoin test
      # prepare servers
      Given a bitcoind server for rooch_bitcoin_test
      Given a server for rooch_bitcoin_test

      Then cmd: "account list --json" 
      
      # mint utxos
      Then cmd bitcoin-cli: "generatetoaddress 101 {{$.account[-1].default.bitcoin_address}}"
      Then sleep: "10" # wait rooch sync and index

      # query utxos
      Then cmd: "object -t 0x4::utxo::UTXO -o {{$.account[-1].default.bitcoin_address}}"
      Then assert: "{{$.object[-1].data[0].owner}} == {{$.account[-1].default.address}}"

      # release servers
      Then stop the server
      Then stop the bitcoind server 

    @serial
    Scenario: rooch bitcoin reorg test
      # prepare servers
      Given a bitcoind server for bitcoin_reorg_test
      Given a server for bitcoin_reorg_test
      
      # Wait genesis block synced
      Then sleep: "10" # wait rooch sync and index

      # Update the reorg pending block count
      Then cmd: "move run --function 0x4::pending_block::update_reorg_block_count_for_local --args u64:1 --json"
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

    @serial
    Scenario: rooch bitcoin api test
      Then cmd: "init --skip-password"
      Then cmd: "env switch --alias local"

      # prepare servers
      Given a bitcoind server for rooch_bitcoin_api_test
      Given a server for rooch_bitcoin_api_test

      # Create and load a wallet
      Then cmd bitcoin-cli: "createwallet \"test_wallet\""
      Then cmd bitcoin-cli: "loadwallet \"test_wallet\""

      # Prepare funds
      Then cmd bitcoin-cli: "getnewaddress"
      
      # mint utxos
      Then cmd bitcoin-cli: "generatetoaddress 101 {{$.getnewaddress[-1]}}"
      Then sleep: "10" # wait rooch sync and index

      # Get UTXO for transaction input
      Then cmd bitcoin-cli: "listunspent 1 9999999 [\"{{$.getnewaddress[-1]}}\"] true"

      # Create a Bitcoin transaction
      Then cmd bitcoin-cli: "createrawtransaction [{\"txid\":\"{{$.listunspent[-1][0].txid}}\",\"vout\":{{$.listunspent[-1][0].vout}}}] {\"{{$.getnewaddress[-1]}}\":49.999}"
      Then cmd bitcoin-cli: "signrawtransactionwithwallet {{$.createrawtransaction[-1]}}"

      # Broadcast transaction using Rooch RPC
      Then cmd: "rpc request --method btc_broadcastTX --params '["{{$.signrawtransactionwithwallet[-1].hex}}", 0.1, 0.1]' --json"

      # Verify transaction broadcast
      Then cmd bitcoin-cli: "getrawmempool"
      Then assert: "{{$.getrawmempool[-1][0]}} == {{$.rpc[-1]}}"
      
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.getnewaddress[-1]}}"
      Then sleep: "10" # wait for the transaction to be confirmed
      
      Then cmd bitcoin-cli: "gettransaction {{$.rpc[-1]}}"
      Then assert: "{{$.gettransaction[-1].confirmations}} == 1"

      # release servers
      Then stop the server
      Then stop the bitcoind server 
