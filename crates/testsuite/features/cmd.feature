Feature: Rooch CLI integration tests
    
  Scenario: Publish modules and interact with them
    Given a server
    Then cmd: "move publish -p ../../examples/counter --sender-account 0x123 --named-addresses rooch_examples=0x123"
    Then cmd: "move run --function 0x123::counter::init --sender-account 0x123"
    Then cmd: "move view --function 0x123::counter::value"
    Then cmd: "move run --function 0x123::counter::increase --sender-account 0x123"
    Then cmd: "move view --function 0x123::counter::value"
    Then cmd: "object --id 0x123"
    Then cmd: "resource --address 0x123 --resource 0x123::counter::Counter"

# TODO: add more cli tests, account, init, etc.