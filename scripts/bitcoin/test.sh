#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

set -e

while getopts "huia" opt; do
  case $opt in
    h)
      cat <<EOF
Usage:
    test <flags>
Flags:
    -h   Print this help
    -u   Run unit test
    -b   Run bitcoin integration test
    -s   Run bitseed integration test
    -a   Run all test
EOF
      exit 1
      ;;
    u)
      UNIT_TEST=1
      ;;
    b)
      INT_TEST=1
      ;;
    s)
      BITSEED_INT_TEST=1
      ;;
    a)
      UNIT_TEST=1
      INT_TEST=1
      BITSEED_INT_TEST=1
      ;;
  esac
done

if [ ! -z "$UNIT_TEST" ]; then
  cargo run --bin rooch move test -p frameworks/bitcoin-move bitseed
fi

if [ ! -z "$INT_TEST" ]; then
  cargo test -p testsuite --test integration -- --name "rooch bitcoin test"
fi

if [ ! -z "$BITSEED_INT_TEST" ]; then
  cargo test -p testsuite --test integration -- --name "rooch bitseed test"
fi
