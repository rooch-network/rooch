#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

CUCUMBER_FILTER="bitcoin-move" cargo test -p testsuite --test integration
