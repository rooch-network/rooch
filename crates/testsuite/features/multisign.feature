Feature: Rooch CLI multisign integration tests

   
    @serial
    Scenario: multisign_account
      Given a server for multisign_account

      Then cmd: "account create"
      Then cmd: "account create"
      Then cmd: "account create"
      Then cmd: "account list --json"

      #create multisign account
      Then cmd: "account create-multisign -t 2 -p {{$.account[-1].account0.public_key}} -p {{$.account[-1].account1.public_key}} -p {{$.account[-1].account2.public_key}} --json"
      Then assert: "'{{$.account[-1]}}' not_contains error"

      #transfer some gas to multisign account
      Then cmd: "account transfer --to {{$.account[-1].multisign_address}} --amount 10000000000 --coin-type rooch_framework::gas_coin::GasCoin"
      Then assert: "{{$.account[-1].execution_info.status.type}} == executed"  

      # transaction
      Then cmd: "tx build --sender {{$.account[-2].multisign_address}}  --function rooch_framework::empty::empty --json"
      Then assert: "'{{$.tx[-1]}}' not_contains error"
      Then cmd: "tx sign {{$.tx[-1].path}} --json"
      Then assert: "'{{$.tx[-1]}}' not_contains error"
      Then cmd: "tx submit {{$.tx[-1].path}} --json"
      Then assert: "{{$.tx[-1].execution_info.status.type}} == executed"


      Then stop the server
