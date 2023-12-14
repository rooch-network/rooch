#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

alias ord="ord --regtest --rpc-url http://127.0.0.1:18443 --bitcoin-rpc-user roochuser --bitcoin-rpc-pass roochpass"
alias bitcoin-cli="docker exec -it bitcoind bitcoin-cli -regtest"