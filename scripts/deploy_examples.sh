#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

for dir in examples/*/;
  do dir=${dir%*/};
    name_addr=$(basename $dir);
    echo $name_addr
    rooch move build -p "$dir" --named-addresses rooch_examples=default,$name_addr=default;
    rooch move publish --named-addresses rooch_examples=default,$name_addr=default -p examples/$name_addr/;
  done