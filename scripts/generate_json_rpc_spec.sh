#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

BASE=$(git rev-parse --show-toplevel)

cd $BASE/crates/rooch-open-rpc

cargo -q run --example generate-json-rpc-spec -- record