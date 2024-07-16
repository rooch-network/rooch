#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

REF="\$1"
BTC_TEST_RPC_URL="\$2"
BTC_TEST_RPC_PWD="\$3"
OPENDA_GCP_TESTNET_BUCKET="\$4"
OPENDA_GCP_TESTNET_CREDENTIAL="\$5"

sleep 30
docker image prune -a -f
docker ps | grep rooch | grep -v faucet | awk '{print \$1}' | xargs -r docker stop
docker ps -a | grep rooch | grep -v faucet | awk '{print \$1}' | xargs -r docker rm -f
docker pull "ghcr.io/rooch-network/rooch:\$REF"
docker run -d --name rooch --restart unless-stopped -v /data:/root -p 6767:6767 -p 9184:9184 "ghcr.io/rooch-network/rooch:\$REF" \
    server start -n test \
    --btc-rpc-url "\$BTC_TEST_RPC_URL" \
    --btc-rpc-username rooch-test \
    --btc-rpc-password "\$BTC_TEST_RPC_PWD" \
    --da '{"internal-da-server": {"servers": [{"open-da": {"scheme": "gcs", "config": {"bucket": "'\$OPENDA_GCP_TESTNET_BUCKET'", "credential": "'\$OPENDA_GCP_TESTNET_CREDENTIAL'"}}}]}}'
