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

  ./rooch move publish -p ../../examples/counter --sender-account=0x14ec8212327103743151a16497032ebb9aa4274b9fbddc9bb02bd86f70f46bcf --rooch_examples=0x14ec8212327103743151a16497032ebb9aa4274b9fbddc9bb02bd86f70f46bcf
