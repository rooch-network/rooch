#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

REF="$1"
BTC_MAIN_RPC_URL="$2"
BTC_MAIN_RPC_PWD="$3"
OPENDA_GCP_MAINNET_BUCKET="$4"
OPENDA_GCP_MAINNET_CREDENTIAL="$5"

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
    --da "{\"da-backend\": {\"backends\": [{\"open-da\": {\"scheme\": \"gcs\", \"config\": {\"bucket\": \"$OPENDA_GCP_MAINNET_BUCKET\", \"credential\": \"$OPENDA_GCP_MAINNET_CREDENTIAL\"}}}]}}" \
    --traffic-burst-size 200 \
    --traffic-per-second 0.1 \
    --rocksdb-row-cache-size 17179869184 \
    --rocksdb-block-cache-size 17179869184
