# Bitcoin local development environment

This directory contains scripts for setting up a local development environment for Bitcoin Core.

## Prerequisites
1. Install [Docker](https://docs.docker.com/install/) or [Podman](https://podman.io/docs/installation)
2. Install [ord](https://docs.ordinals.com/guides/inscriptions.html#installing-ord)

## Setup

1. Run `./run_local_node_docker.sh` or `./run_local_node_podman.sh` to start a local Bitcoin Core node
2. Run `source ./cmd_alias.sh` to set up aliases for running `bitcoin-cli` and `ord` commands

## Development on rooch

1. Run `rooch server start --btc-rpc-url http://127.0.0.1:18443 --btc-rpc-username roochuser --btc-rpc-password roochpass --btc-start-block-height 0`
2. Run `rooch rpc request --method rooch_queryGlobalStates --params '[{"object_type":"0x4::utxo::UTXO"},null, "2", true]'` to query the UTXO set
3. Run `rooch rpc request --method rooch_queryGlobalStates --params '[{"object_type":"0x4::ord::Inscription"},null, "2", true]'` to query the Inscription set

## Usage

1. Run `ord wallet create` to create a new ord wallet
2. Run `ord wallet receive` to get a new address to receive funds
3. Run `bitcoin-cli generatetoaddress 101 <address>` to generate 101 blocks to the address
4. Run `ord wallet balance` to check the balance of the wallet
5. Run `echo "Hello rooch">/tmp/hello.txt` to create a file
6. Run `ord wallet inscribe --fee-rate 1 --file /tmp/hello.txt` to inscribe the file to the blockchain
7. Run `bitcoin-cli generatetoaddress 1 <address>` to generate a block to the address

### References
* [Bitcoin Core](https://bitcoincore.org/en/doc/25.0.0/)
* [ord testing](https://docs.ordinals.com/guides/testing.html): for testing ord inscriptions