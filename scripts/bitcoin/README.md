# Bitcoin local development environment

This directory contains scripts for setting up a local development environment for Bitcoin Core.

## Prerequisites
1. Install [Docker](https://docs.docker.com/install/) or [Podman](https://podman.io/docs/installation)
2. Install [ord](https://docs.ordinals.com/guides/wallet.html?highlight=install#installing-ord)

## Setup

1. Run `./node/run_local_node_docker.sh` or `./node/run_local_node_podman.sh` to start a local Bitcoin Core node
2. Run `source ./cmd_alias_docker.sh` or `source ./cmd_alias_podman.sh` to set up aliases for running `bitcoin-cli` and `ord` commands

## Development on rooch

1. Run `rooch server start -n local --btc-sync-block-interval 1 --btc-rpc-url http://127.0.0.1:18443 --btc-rpc-username roochuser --btc-rpc-password roochpass`
2. Run `rooch account list --json` to get the active account `bitcoin_address`
3. Run `bitcoin-cli generatetoaddress 101 <bitcoin_address>` to generate 101 blocks to the address
4. Run `rooch rpc request --method btc_queryUTXOs --params '["all",  null, "2", true]'` to query the UTXO set
5. Run `rooch rpc request --method btc_queryInscriptions --params '["all",  null, "2", true]'` to query the Inscription set
6. Run `rooch account balance` show the balance of active account(Include BTC)

## Usage

You can also configure the environment using the env script, use `./env.sh -i`

1. Run `ord server` to start ord indexer server
2. Run `ord wallet create` to create a new ord wallet
3. Run `ord wallet receive` to get a new address to receive funds
4. Run `bitcoin-cli generatetoaddress 101 <address>` to generate 101 blocks to the address
5. Run `ord wallet balance` to check the balance of the wallet
6. Run `echo "{"p":"brc-20","op":"mint","tick":"Rooch","amt":"1"}">/tmp/hello.txt` to create a file
7. Run `ord wallet inscribe --fee-rate 1 --file /tmp/hello.txt --destination <address>` to inscribe the file to the blockchain
8. Run `bitcoin-cli generatetoaddress 1 <address>` to mine an inscription
9. Run `ord wallet inscriptions` to get the reveal transaction ID

## Bitseed

```bash
rooch bitseed generator --name random --generator generator/cpp/generator.wasm
rooch bitseed deploy --fee-rate 5000 --generator $the_inscription_from_pre_step --tick bits --amount 210000000000 --deploy-args '{"height":{"type":"range","data":{"min":1,"max":1000}}}'
rooch bitseed mint --fee-rate 5000 --deploy-inscription-id $the_inscription_from_pre_step --user-input hello_bitseed
rooch bitseed split --fee-rate 5000 --sft-inscription-id $the_inscription_from_pre_step --amounts 500 --amounts 300
rooch bitseed merge --fee-rate 5000 --sft-inscription-ids $the_inscription_from_pre_step_0 --sft-inscription-ids $the_inscription_from_pre_step_1 --sft-inscription-ids $the_inscription_from_pre_step_2
rooch bitseed view --sft-inscription-id $the_inscription_from_pre_step
```

## More integration test examples

* [bitcoin](../../crates/testsuite/features/bitcoin.feature)  
* [ord](../../crates/testsuite/features/ord.feature)  
* [bitseed](../../crates/testsuite/features/bitseed.feature)  


### References
* [Bitcoin Core](https://bitcoincore.org/en/doc/25.0.0/)
* [ord testing](https://docs.ordinals.com/guides/testing.html): for testing ord inscriptions
