#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

REF="$1"
BTC_MAIN_RPC_URL="$2"
BTC_MAIN_RPC_PWD="$3"

sleep 30
docker image prune -a -f
docker ps | grep rooch | grep -v faucet | awk '{print $1}' | xargs -r docker stop
docker ps -a | grep rooch | grep -v faucet | awk '{print $1}' | xargs -r docker rm -f
docker pull "ghcr.io/rooch-network/rooch:$REF"
docker run -d --name rooch-mainnet --restart unless-stopped -v /data:/root -p 6767:6767 -p 9184:9184 -e RUST_BACKTRACE=full  "ghcr.io/rooch-network/rooch:$REF" \
    server start -n main \
    --btc-sync-block-interval 20 \
    --btc-rpc-url "$BTC_MAIN_RPC_URL" \
    --btc-rpc-username rooch-main \
    --btc-rpc-password "$BTC_MAIN_RPC_PWD" \
    --traffic-burst-size 100000 \
    --traffic-per-second 10000
