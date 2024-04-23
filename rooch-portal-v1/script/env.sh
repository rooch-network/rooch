#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

default_address=0xb7ac336861ff431cf867400a1eaa9708b5666954b5268486e6bd2e948aab1a42
target_address=address:0x676c71c28aa7be1733a5d8c1b4e0e00f2927bd0e5acd592f37fbde99496aa366

getDefaultAddress() {
  container_id=$()
  echo "$container_id"
}

rooch() {
  command cargo run --package rooch --bin rooch "$@"
}

reset () {
  lsof -ti:50051 | xargs kill

  rooch server clean

  nohup cargo run --package rooch --bin rooch server start &

  sleep 10
}
# https://github.com/rooch-network/rooch/issues/1599

dep_coin() {
  # dep token
  rooch move publish -p ../../examples/coins --named-addresses coins=default

  # faucet
  rooch move run --function default::fixed_supply_coin::faucet --args object:default::fixed_supply_coin::Treasury

  # transfer
  rooch move run --function rooch_framework::transfer::transfer_coin --type-args default::fixed_supply_coin::FSC --args ${target_address}  --args 1u256
}

dep_nft() {
  # dep nft
  rooch move publish -p ../../examples/nft --named-addresses nft=default

  # create collection
  rooch move run --function default::collection::create_collection_entry --args string:test --args string:https://i.seadn.io/s/raw/files/d0f989ab16333bbf348fc74f0d4a6d8d.png --args address:default --args string:testtest --args u64:1000

  # get collection id
  collection_object_id=$(rooch event get-events-by-event-handle -t default::collection::CreateCollectionEvent | jq -r '.data[0].decoded_event_data.value.object_id')

  # mint nft
  rooch move run --function default::nft::mint_entry --args object:${collection_object_id} --args string:testg

  # find nft
  nft_obj_id=$(rooch rpc request --method rooch_queryGlobalStates --params '[{"object_type":"'"${default_address}"'::nft::NFT"}, null, "10", true]' | jq -r '.data[0].object_id')

  # transfer nft
  rooch move run --function rooch_framework::transfer::transfer_object --type-args default::nft::NFT --args ${target_address} --args object:${nft_obj_id}
}

dep_mint() {
  # dep mint
  rooch move publish -p ../../examples/btc_holder_farmer --named-addresses btc_holder_farmer=default
}

# utxo ref /scripts/bitcoin/env.sh

while getopts "hncrm" opt; do
  case $opt in
    h)
      cat <<EOF
Usage:
    check_pr <flags>
Flags:
    -h   Print this help
    -n   dep_nft
    -c   dep_coin
    -r   reset
    -m   Mint
EOF
      exit 1
      ;;
    n)
      NFT=1
      ;;
    c)
      COIN=1
      ;;
    r)
      RESET=1
      ;;
    m)
      MINT=1
      ;;
  esac
done

if [ ! -z "$NFT" ]; then
  dep_nft
fi

if [ ! -z "$COIN" ]; then
  dep_coin
fi

if [ ! -z "$RESET" ]; then
  reset
fi

if [ ! -z "$MINT" ]; then
  dep_mint
fi