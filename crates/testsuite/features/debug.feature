Feature: Rooch CLI integration tests
    @serial
    Scenario: Init
      Then cmd: "init --skip-password"
      Then cmd: "env switch --alias local"
 
    @serial
    Scenario: rooch bitcoin test
      Given a bitcoind server for rooch_bitcoin_test
      Given a ord server for rooch_bitcoin_test
      Given a server for rooch_bitcoin_test
      Then stop the server
      Then stop the ord server 
      Then stop the bitcoind server 