Feature: Rooch Bitcoin tests

    @serial
    Scenario: rooch_bitcoin test
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
    Scenario: btc_api test
      # prepare servers
      Given a bitcoind server for rooch_bitcoin_test
      Given a server for rooch_bitcoin_test

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
      Then cmd bitcoin-cli: "createrawtransaction [{\"txid\":\"{{$.listunspent[-1][0].txid}}\",\"vout\":{{$.listunspent[-1][0].vout}}}] {\"{{$.getnewaddress[-1]}}\":0.1}"
      Then cmd bitcoin-cli: "signrawtransactionwithwallet {{$.createrawtransaction[-1]}}"

      # Broadcast transaction using Rooch RPC
      Then cmd: "rpc request --method btc_broadcastTX --params '["{{$.signrawtransactionwithwallet[-1].hex}}"]' --json"

      # release servers
      Then stop the server
      Then stop the bitcoind server 
