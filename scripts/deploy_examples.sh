#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

for dir in /root/rooch/examples/*/;
  do dir=${dir%*/};
    name_addr=$(basename $dir);
    echo $name_addr
    docker run -v /root:/root ghcr.io/rooch-network/rooch:main move build -p "$dir" --named-addresses rooch_examples=default,$name_addr=default;
    docker run -v /root:/root ghcr.io/rooch-network/rooch:main move publish -p "$dir" --named-addresses rooch_examples=default,$name_addr=default ;
  done
