Feature: Rooch Bitcoin ord tests

    @serial
    Scenario: rooch_bitcoin ord burn test
      # prepare servers
      Given a bitcoind server for rooch_ord_test
      Given a ord server for rooch_ord_test
      Given a server for rooch_ord_test

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
      
      # create a inscription
      Then cmd ord bash: "echo "{"p":"brc-20","op":"mint","tick":"Rooch","amt":"1"}">/tmp/hello.txt"
      Then cmd ord: "wallet inscribe --fee-rate 1 --file /tmp/hello.txt --destination {{$.wallet[-2].addresses[0]}}"

      # mine a block
      Then cmd ord: "wallet receive"
      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.wallet[-1].addresses[0]}}"
      Then sleep: "10"

      # get a inscription
      Then cmd ord: "wallet inscriptions"

      # burn latest inscription
      Then cmd ord: "wallet burn --fee-rate 1 --postage 1000sat {{$.wallet[-1][0].inscription}}"
      Then assert: "'{{$.wallet[-1]}}' not_contains error"

      # Check inscription burned
      Then cmd: "move view --function 0x4::ord::view_inscription_burned --args string:{{$.wallet[-2][0].inscription}} "
      Then assert: "{{$.move[-1].vm_status}} == Executed"
      Then assert: "{{$.move[-1].return_values[0].decoded_value.value.vec[0].value}} == true"

      # release servers
      Then stop the server
      Then stop the ord server 
      Then stop the bitcoind server