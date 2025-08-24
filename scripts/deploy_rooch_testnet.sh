#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

REF="$1"
BTC_TEST_RPC_URL="$2"
BTC_TEST_RPC_PWD="$3"
OPENDA_GCP_TESTNET_BUCKET="$4"
OPENDA_GCP_TESTNET_CREDENTIAL="$5"
TURBO_DA_TURING_ENDPOINT="$6"
TURBO_DA_TURING_API_KEY="$7"

# Validate full image references to avoid deploying non-rooch images
if [[ "$REF" == ghcr.io/* ]] && [[ "$REF" != ghcr.io/rooch-network/rooch* ]]; then
    echo "Error: full image reference must start with ghcr.io/rooch-network/rooch" >&2
    exit 1
fi
if [[ "$REF" == *"@"* ]] && [[ "$REF" != ghcr.io/rooch-network/rooch@* ]] && [[ "$REF" != sha256:* ]]; then
    echo "Error: only ghcr.io/rooch-network/rooch@<digest> or sha256:<digest> is allowed for digest references" >&2
    exit 1
fi
if [[ "$REF" == */*:* ]] && [[ "$REF" != ghcr.io/rooch-network/rooch:* ]]; then
    echo "Error: full image reference with tag must start with ghcr.io/rooch-network/rooch" >&2
    exit 1
fi

# Determine the docker image reference
if [[ "$REF" == ghcr.io/* ]]; then
    IMAGE="$REF"
elif [[ "$REF" == sha256:* ]]; then
    IMAGE="ghcr.io/rooch-network/rooch@$REF"
elif [[ "$REF" == *"@"* ]]; then
    IMAGE="$REF"
else
    IMAGE="ghcr.io/rooch-network/rooch:$REF"
fi

sleep 30
docker image prune -a -f
docker ps | grep rooch | grep -v faucet | awk '{print $1}' | xargs -r docker stop
docker ps -a | grep rooch | grep -v faucet | awk '{print $1}' | xargs -r docker rm -f
docker pull "$IMAGE"

# --btc-sync-block-interval 3 \
# --btc-rpc-url "$BTC_TEST_RPC_URL" \
# --btc-rpc-username rooch-test \
# --btc-rpc-password "$BTC_TEST_RPC_PWD" \

docker run -d --name rooch-testnet --restart unless-stopped -v /data:/root -p 6767:6767 -p 9184:9184 -e RUST_BACKTRACE=full  "$IMAGE" \
    server start -n test \
    --da "{\"da-min-block-to-submit\":9908, \"da-backend\":{\"backends\":[{\"open-da\":{\"scheme\":\"gcs\",\"config\":{\"bucket\":\"$OPENDA_GCP_TESTNET_BUCKET\",\"credential\":\"$OPENDA_GCP_TESTNET_CREDENTIAL\"}}},{\"open-da\":{\"scheme\":\"avail\",\"config\":{\"turbo_endpoint\":\"$TURBO_DA_TURING_ENDPOINT\",\"turbo_api_key\":\"$TURBO_DA_TURING_API_KEY\"}}}]}}" \
    --traffic-burst-size 200 \
    --traffic-per-second 0.1 \
    --rocksdb-row-cache-size 1073741824 \
    --rocksdb-block-cache-size 12884901888 \
    --pruner-enable