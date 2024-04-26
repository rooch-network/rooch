#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

set -e

cargo run --bin rooch move test -p frameworks/bitcoin-move
cargo test -p testsuite --test integration -- --name "rooch bitcoin test"
