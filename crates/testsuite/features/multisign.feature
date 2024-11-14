Feature: Rooch CLI multisign integration tests

   
    @serial
    Scenario: multisign_account
      Given a bitcoind server for multisign_account
      Given a server for multisign_account

      Then cmd: "account create"
      Then cmd: "account create"
      Then cmd: "account create"
      Then cmd: "account list --json"

      #create multisign account
      Then cmd: "account create-multisign -t 2 -p {{$.account[-1].account0.public_key}} -p {{$.account[-1].account1.public_key}} -p {{$.account[-1].account2.public_key}} --json"
      Then assert: "'{{$.account[-1]}}' not_contains error"

      # Prepare funds
      # mint btc
      Then cmd bitcoin-cli: "generatetoaddress 101 {{$.account[-1].multisign_bitcoin_address}}"
      Then sleep: "10" # wait for the transaction to be confirmed

      # l1 transaction
      Then cmd: "bitcoin build-tx --sender {{$.account[-1].multisign_bitcoin_address}} -o {{$.account[-2].account0.bitcoin_address}}:100000000"
      Then assert: "'{{$.bitcoin[-1]}}' not_contains error"
      Then cmd: "bitcoin sign-tx -s {{$.account[-1].participants[0].participant_address}}   {{$.bitcoin[-1].path}} -y"
      Then assert: "'{{$.bitcoin[-1]}}' not_contains error"
      Then cmd: "bitcoin sign-tx -s {{$.account[-1].participants[2].participant_address}}   {{$.bitcoin[-1].path}} -y"
      Then assert: "'{{$.bitcoin[-1]}}' not_contains error"
      Then cmd: "bitcoin broadcast-tx {{$.bitcoin[-1].path}}"
      Then assert: "'{{$.bitcoin[-1]}}' not_contains error"

      Then cmd bitcoin-cli: "generatetoaddress 1 {{$.account[-1].multisign_bitcoin_address}}"
      Then sleep: "10" # wait for the transaction to be confirmed

      Then cmd: "account balance -a {{$.account[-2].account0.address}} --json"
      Then assert: "{{$.account[-1].BTC.balance}} == 100000000"

      #transfer some gas to multisign account
      Then cmd: "account transfer --to {{$.account[-2].multisign_address}} --amount 10000000000 --coin-type rooch_framework::gas_coin::RGas --json"
      Then assert: "{{$.account[-1].execution_info.status.type}} == executed"

      # l2 transaction
      Then cmd: "tx build --sender {{$.account[-3].multisign_address}}  --function rooch_framework::empty::empty --json"
      Then assert: "'{{$.tx[-1]}}' not_contains error"
      Then cmd: "tx sign {{$.tx[-1].path}} -s {{$.account[-3].participants[0].participant_address}}  --json -y"
      Then assert: "'{{$.tx[-1]}}' not_contains error"
      Then cmd: "tx sign {{$.tx[-1].path}} -s {{$.account[-3].participants[1].participant_address}}  --json -y"
      Then assert: "'{{$.tx[-1]}}' not_contains error"
      Then cmd: "tx submit {{$.tx[-1].path}} --json"
      Then assert: "{{$.tx[-1].execution_info.status.type}} == executed"


      Then stop the server
      Then stop the bitcoind server 
