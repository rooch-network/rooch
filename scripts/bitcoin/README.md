# Bitcoin local development environment

This directory contains scripts for setting up a local development environment for Bitcoin Core.

## Prerequisites
1. Install [Docker](https://docs.docker.com/install/) or [Podman](https://podman.io/docs/installation)
2. Install [ord](https://docs.ordinals.com/guides/wallet.html?highlight=install#installing-ord)

## Setup

1. Run `./node/run_local_node_docker.sh` or `./node/run_local_node_podman.sh` to start a local Bitcoin Core node
2. Run `source ./cmd_alias.sh` to set up aliases for running `bitcoin-cli` and `ord` commands

## Development on rooch

1. Run `rooch server start --btc-rpc-url http://127.0.0.1:18443 --btc-rpc-username roochuser --btc-rpc-password roochpass`
2. Run `rooch account list --json` to get the `bitcoin_address`
3. Run `bitcoin-cli generatetoaddress 101 <bitcoin_address>` to generate 101 blocks to the address
2. Run `rooch rpc request --method rooch_queryObjectStates --params '[{"object_type":"0x4::utxo::UTXO"},  null, "2", {"descending": true,"showDisplay":false}]'` to query the UTXO set
3. Run `rooch rpc request --method rooch_queryObjectStates --params '[{"object_type":"0x4::ord::Inscription"},  null, "2", {"descending": true,"showDisplay":false}]'` to query the Inscription set

## Usage

You can also configure the environment using the env script, use `./env -i`

1. Run `ord server` to start ord indexer server
2. Run `ord wallet create` to create a new ord wallet
3. Run `ord wallet receive` to get a new address to receive funds
4. Run `bitcoin-cli generatetoaddress 101 <address>` to generate 101 blocks to the address
5. Run `ord wallet balance` to check the balance of the wallet
6. Run `echo "{"p":"brc-20","op":"mint","tick":"Rooch","amt":"1"}">/tmp/hello.txt` to create a file
7. Run `ord wallet inscribe --fee-rate 1 --file /tmp/hello.txt --destination <address>` to inscribe the file to the blockchain
8. Run `bitcoin-cli generatetoaddress 1 <address>` to mine an inscription
9. Run `ord wallet inscriptions` to get the reveal transaction ID




### References
* [Bitcoin Core](https://bitcoincore.org/en/doc/25.0.0/)
* [ord testing](https://docs.ordinals.com/guides/testing.html): for testing ord inscriptions
