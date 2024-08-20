#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

alias ord="ord --regtest --bitcoin-rpc-url http://127.0.0.1:18443 --bitcoin-rpc-username roochuser --bitcoin-rpc-password roochpass"
alias bitcoin-cli="podman exec -it bitcoind bitcoin-cli -regtest"

