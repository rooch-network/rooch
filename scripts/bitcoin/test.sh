#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

set -e

while getopts "huwbosa" opt; do
  case $opt in
    h)
      cat <<EOF
Usage:
    test <flags>
Flags:
    -h   Print this help
    -u   Run unit test
    -w   Run wasm integration test
    -b   Run bitcoin integration test
    -o   Run ord integration test
    -s   Run bitseed integration test
    -a   Run all test
EOF
      exit 1
      ;;
    u)
      UNIT_TEST=1
      ;;
    w)
      WASM_INT_TEST=1
      ;;
    b)
      UNIT_TEST=1
      BITCOIN_INT_TEST=1
      ;;
    o)
      UNIT_TEST=0
      ORD_INT_TEST=1
      ;;
    s)
      UNIT_TEST=1
      BITSEED_INT_TEST=1
      ;;
    a)
      UNIT_TEST=1
      WASM_INT_TEST=1
      BITCOIN_INT_TEST=1
      BITSEED_INT_TEST=1
      ;;
  esac
done

export CARGO_BUILD_JOBS=8
export RUST_LOG=debug 
export RUST_BACKTRACE=1

if [ ! -z "$UNIT_TEST" ]; then
  cargo run --bin rooch move test -p frameworks/rooch-nursery cosmwasm_vm 
fi

if [ ! -z "$WASM_INT_TEST" ]; then
  cargo test -p testsuite --test integration -- --name "cosmwasm-vm test"
fi

if [ ! -z "$BITCOIN_INT_TEST" ]; then
  cargo test -p testsuite --test integration -- --name "rooch bitcoin api test"
fi

if [ ! -z "$ORD_INT_TEST" ]; then
  cargo test -p testsuite --test integration -- --name "rooch_bitcoin ord burn test"
fi

if [ ! -z "$BITSEED_INT_TEST" ]; then
  cargo test -p testsuite --test integration -- --name "rooch bitseed test"
fi
