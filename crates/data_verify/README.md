# Bitcoin data verify

## Setup

Refer to the scripts in the `scripts/bitcoin` directory to initialize the Bitcoin and Ord environments.

## Usage

### Bitcoin cli
`bitcoind -daemon  -conf=/Users/{user}/.bitcoin/bitcoin.conf`

### Export ord index
`ord --bitcoin-rpc-user roochuser --bitcoin-rpc-pass roochpass  index export --include-addresses --tsv ord.export.tsv`

### Rooch server cli
`rooch server start -n local --btc-rpc-url http://127.0.0.1:8332 --btc-rpc-username roochuser --btc-rpc-password roochpass --btc-start-block-height 767430 --btc-end-block-height 774697  --btc-network 1 --data-verify-mode true`

### Run Data verify
`cargo run`