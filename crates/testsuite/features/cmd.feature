Feature: Rooch CLI integration tests
    Scenario: Init
      Then cmd: "init"

    Scenario: move and account
      Given a server
      Then cmd: "move publish -p ../../examples/counter --sender-account {default} --named-addresses rooch_examples={default}"
      Then cmd: "move run --function {default}::counter::init --sender-account {default}"
      Then cmd: "move view --function {default}::counter::value"
      Then assert: "{{$.move[-1][0]}} == 0"
      Then cmd: "move run --function {default}::counter::increase --sender-account {default}"
      Then cmd: "move view --function {default}::counter::value"
      Then assert: "{{$.move[-1][0]}} == 1"
      Then cmd: "account object --id {default}"
      Then cmd: "account resource --address {default} --resource {default}::counter::Counter"
      Then cmd: "account create"
      Then cmd: "account list"
      Then cmd: "account import "fiber tube acid imitate frost coffee choose crowd grass topple donkey submit""