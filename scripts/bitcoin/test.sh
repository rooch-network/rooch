#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

cargo test -p testsuite --test integration -- --name "rooch bitcoin test"
