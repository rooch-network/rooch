#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

BASE=$(git rev-parse --show-toplevel)

cd $BASE/crates/rooch-open-rpc

cargo run --package rooch-open-rpc-spec --bin rooch-open-rpc-spec -- record