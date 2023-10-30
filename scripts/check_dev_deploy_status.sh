#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <param1> [param2 ...]"
    exit 1
fi

KEYWORD="rooch"
IMAGE_TAG="$1"
DEV_MNEMONIC_PHRASE="$2"

# get the container id
CONTAINER_ID=$(docker ps -a | grep $KEYWORD | awk '{print $1}')

if [ -z "$CONTAINER_ID" ]; then
    echo "No container found related to the keyword $KEYWORD"
    exit 1
fi

sleep 30

# get container status
STATUS=$(docker inspect --format "{{.State.Status}}" $CONTAINER_ID)

if [ "$STATUS" != "running" ]; then
    echo "!!! Container $CONTAINER_ID is not running, trying to clean data and restart"
    echo "Start cleaning the data with image_tag: $IMAGE_TAG"
    docker run --rm -v "/root:/root" ghcr.io/rooch-network/rooch:$IMAGE_TAG server clean -n dev
    rm -rf ~/.rooch
    docker run --rm -v "/root:/root" ghcr.io/rooch-network/rooch:$IMAGE_TAG init -m "$(echo $DEV_MNEMONIC_PHRASE)" --skip-password
    docker start $CONTAINER_ID
    if [ $? -eq 0 ]; then
        echo "Container $CONTAINER_ID Successfully restarted."
        echo "Redeploy the examples"
        for dir in /root/rooch/examples/*/; do
            dir=${dir%*/}
            name_addr=$(basename $dir)
            echo $name_addr
            docker run --rm -v "/root:/root" ghcr.io/rooch-network/rooch:$IMAGE_TAG move build -p "$dir" --named-addresses rooch_examples=default,$name_addr=default
            docker run --rm -v "/root:/root" ghcr.io/rooch-network/rooch:$IMAGE_TAG move publish -p "$dir" --named-addresses rooch_examples=default,$name_addr=default
        done
    else
        echo "Container $CONTAINER_ID Startup failed, please check the reason."
    fi
else
    echo "Container $CONTAINER_ID is running"
fi
