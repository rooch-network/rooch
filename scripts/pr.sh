#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

# A script to check whether a local commit related to Move repo is ready for a PR.

set -e
CARGO_HOME=${CARGO_HOME:-~/.cargo}

function install_cargo_machete {
  if ! command -v cargo-machete &>/dev/null; then
    cargo install cargo-machete --locked --version 0.7.0
  fi
}

function install_cargo_nextest {
  if ! command -v cargo-nextest &>/dev/null; then
    cargo install cargo-nextest --locked
  fi
}

install_cargo_machete
install_cargo_nextest

# Run only tests which would also be run on CI
export ENV_TEST_ON_CI=1

while getopts "hcxtdgmea" opt; do
  case $opt in
    h)
      cat <<EOF
Usage:
    check_pr <flags>
Flags:
    -h   Print this help
    -c   Check the core prover crates using cargo fmt/clippy.
         This is the default if no flags are provided.
    -x   Like -c, but adds more crates (specifically all which depend
         on move-model)
    -t   In addition to fmt/clippy, run cargo test
    -d   Run documentation generation, abi generation, etc. for move-stdlib
         and other tested frameworks.
    -g   Run the Move git checks script (whitespace check). This works
         only for committed clients.
    -m   Run the Move unit and verification tests.
    -e   Run the examples tests.
    -a   Run all of the above
EOF
      exit 1
      ;;
    c)
      CHECK=1
      ;;
    x)
      CHECK=1
      CHECK_MORE=1
      ;;
    d)
      GEN_ARTIFACTS=1
      ;;
    g)
      GIT_CHECKS=1
      ;;
    t)
      CHECK=1
      ALSO_TEST=1
      ;;
    m)
      MOVE_TESTS=1
      ;;
    e)
      EXAMPLES_TESTS=1
      ;;
    a)
      CHECK=1
      CHECK_MORE=1
      GEN_ARTIFACTS=1
      GIT_CHECKS=1
      ALSO_TEST=1
      MOVE_TESTS=1
      MOVE_E2E_TESTS=1
      EXAMPLES_TESTS=1
      ;;
  esac
done

MOVE_TEST_CRATES="\
  frameworks/move-stdlib\
  frameworks/moveos-stdlib\
  frameworks/rooch-framework\
  frameworks/bitcoin-move\
  frameworks/rooch-nursery\
"

if [ ! -z "$CHECK" ]; then
  cargo machete
  cargo fmt -- --check
  cargo clippy --workspace --all-targets --all-features --tests --benches -- -D warnings
fi

if [ ! -z "$ALSO_TEST" ]; then
    export RUST_BACKTRACE=1
    cargo nextest run --workspace --all-features --exclude rooch-framework-tests --exclude rooch-integration-test-runner -v
    cargo test -p rooch-framework-tests -p rooch-integration-test-runner
    cargo test --release -p rooch-framework-tests bitcoin_test
    RUST_LOG=warn cargo test -p testsuite --test integration
fi

if [ ! -z "$MOVE_TESTS" ]; then
  for crate in $MOVE_TEST_CRATES; do
    echo "*************** [check-pr] Move tests $crate"
    (
      cargo run --bin rooch move test -p $crate
    )
  done
fi

if [ ! -z "$EXAMPLES_TESTS" ]; then
  for dir in examples/*/; 
  do dir=${dir%*/};  
    name_addr=$(basename $dir); 
    cargo run --bin rooch move build -p "$dir" --named-addresses rooch_examples=default,$name_addr=default;
    cargo run --bin rooch move test -p "$dir"; 
  done
fi
