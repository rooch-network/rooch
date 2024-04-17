#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

RUST_LOG=debug cargo test -p testsuite --test integration -- --name "rooch bitseed test"
