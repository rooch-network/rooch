#!/bin/bash

container_name="bitcoind"

ord() {
  command ord --regtest --rpc-url http://127.0.0.1:18443 --bitcoin-rpc-user roochuser --bitcoin-rpc-pass roochpass "$@"
}

bitcoin-cli() {
  command docker exec -it bitcoind bitcoin-cli -regtest "$@"
}

getBitcoinNode() {
  container_id=$(docker ps --filter "name=${container_name}" --format "{{.ID}}")
  echo "$container_id"
}

init() {
  container_id=$(getBitcoinNode)
  if [ -n "$container_id" ]; then
    echo "Bitcoin node is already running."
  else
    echo "Starting Bitcoin node..."
    ./node/run_local_node_docker.sh

  sleep 1

  attempt=1
  max_attempts=30
  while [ $attempt -le $max_attempts ]; do
    if docker inspect -f '{{.State.Running}}' $container_name 2>/dev/null | grep -q "true"; then
      echo "Container $container_name is running."
      break
    else
      echo "Attempt $attempt: Waiting for $container_name to start..."
      sleep 1
      ((attempt++))
    fi
  done
  fi

  # Step 1: Create a new ord wallet
  ord wallet create

  # Step 2: Get a new address to receive funds
  address=$(ord wallet receive | jq -r '.address')
  echo "You bitcoin address $address"

  # Step 3: Generate 101 blocks to the address
  bitcoin-cli generatetoaddress 101 $address > /dev/null 2>&1

  # Step 4: Check the balance of the wallet
  ord wallet balance

  # Step 5: Create a file with specific content
  echo '{"p":"brc-20","op":"mint","tick":"Rooch","amt":"1"}' > /tmp/hello.txt

  # Step 6: Inscribe the file to the blockchain
  ord wallet inscribe --fee-rate 1 --file /tmp/hello.txt --destination "${address}" > /dev/null 2>&1

  # Step 7: Mine an inscription with 1 block
  bitcoin-cli generatetoaddress 1 $address > /dev/null 2>&1

  echo "You inscriptions"
  # Step 8: Get the reveal transaction ID
  ord wallet inscriptions

  # Step 9: start rooch node
  cargo run --package rooch --bin rooch server start --btc-rpc-url http://127.0.0.1:18443 --btc-rpc-username roochuser --btc-rpc-password roochpass --btc-start-block-height 0 --btc-network 4

}

clean() {
  # clean ord index
  indexPath=$(ord index info 2>/dev/null | jq -r '.index_path')
  if [ -n "$indexPath" ]; then
    rm "$indexPath"
  fi

  # clean bitcoin docker
  container_id=$(getBitcoinNode)
  if [ -n "$container_id" ]; then
    docker rm -f $container_id
    sleep 2
  fi

  # clean bitcoin data
  rm -rf ~/.bitcoin

  # clean rooch data
  cargo run --package rooch --bin rooch server clean -n local
}

reset() {
  clean &
  wait

  init
}

while getopts "hicr" opt; do
  case $opt in
    h)
      cat <<EOF
Usage:
    check_pr <flags>
Flags:
    -h   Print this help
    -i   init bitcoin env
    -c   clean bitcoin env
    -r   reset bitcoin env
EOF
      exit 1
      ;;
    i)
      INIT=1
      ;;
    c)
      CLEAN=1
      ;;
    r)
      RESET=1
      ;;
  esac
done

if [ ! -z "$INIT" ]; then
  init
fi

if [ ! -z "$CLEAN" ]; then
  clean
fi

if [ ! -z "$RESET" ]; then
  reset
fi