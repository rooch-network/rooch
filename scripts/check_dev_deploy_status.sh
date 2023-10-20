#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

KEYWORD="rooch"

# get the container id
CONTAINER_ID=$(docker ps -a | grep $KEYWORD | awk '{print $1}')

if [ -z "$CONTAINER_ID" ]; then
    echo "No container found related to the keyword $KEYWORD"
    exit 1
fi

# get container status
STATUS=$(docker inspect --format '{{.State.Status}}' $CONTAINER_ID)

if [ "$STATUS" != "running" ]; then
    echo "Container $CONTAINER_ID is not running，trying to clean data and restart"
    echo "Start cleaning the data."
    rooch server clean -n dev
    rm -rf ~/.rooch
    rooch init -m "$DEV_MNEMONIC_PHRASE"
    docker start $CONTAINER_ID
    if [ $? -eq 0 ]; then
        echo "Container $CONTAINER_ID Successfully restarted."
        echo "Redeploy the examples"
        for dir in /root/rooch/examples/*/; do
            dir=${dir%*/}
            name_addr=$(basename $dir)
            echo $name_addr
            docker run -v /root:/root $CONTAINER_ID move build -p "$dir" --named-addresses rooch_examples=default,$name_addr=default
            docker run -v /root:/root $CONTAINER_ID move publish -p "$dir" --named-addresses rooch_examples=default,$name_addr=default
        done
    else
        echo "Container $CONTAINER_ID Startup failed, please check the reason."
    fi
else
    echo "Container $CONTAINER_ID is running"
fi
