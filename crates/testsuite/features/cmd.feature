Feature: Rooch CLI integration tests
    Scenario: Init
      Then cmd: "init"

    Scenario: move and account
      Given a server
      Then cmd: "move publish -p ../../examples/counter --sender-account 0x123 --named-addresses rooch_examples=0x123"
      Then cmd: "move run --function 0x123::counter::init --sender-account 0x123"
      Then cmd: "move view --function 0x123::counter::value"
      Then assert: "{{$.move[-1].data[0]}} == 0"
      Then cmd: "move run --function 0x123::counter::increase --sender-account 0x123"
      Then cmd: "move view --function 0x123::counter::value"
      Then assert: "{{$.move[-1].data[0]}} == 1"
      Then cmd: "object --id 0x123"
      Then cmd: "resource --address 0x123 --resource 0x123::counter::Counter"
      Then cmd: "account create"
      Then cmd: "account list"
      Then cmd: "account import "fiber tube acid imitate frost coffee choose crowd grass topple donkey submit""
